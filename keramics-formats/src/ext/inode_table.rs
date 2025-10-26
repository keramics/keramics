/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License. You may
 * obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations
 * under the License.
 */

use std::io::SeekFrom;

use keramics_checksums::ReversedCrc32Context;
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;

use crate::block_tree::BlockTree;

use super::features::ExtFeatures;
use super::group_descriptor::ExtGroupDescriptor;
use super::inode::ExtInode;

/// Extended File System (ext) inode table.
pub struct ExtInodeTable {
    /// Mediator.
    mediator: MediatorReference,

    /// Format version.
    format_version: u8,

    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Metadata checksum seed.
    metadata_checksum_seed: Option<u32>,

    /// Block size.
    pub block_size: u32,

    /// Inode size.
    inode_size: u16,

    /// Number of inodes per block group.
    number_of_inodes_per_block_group: u32,

    /// Block tree.
    block_tree: BlockTree<ExtGroupDescriptor>,
}

impl ExtInodeTable {
    /// Creates a new inode table.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            format_version: 2,
            encoding: CharacterEncoding::Utf8,
            metadata_checksum_seed: None,
            block_size: 0,
            inode_size: 0,
            number_of_inodes_per_block_group: 0,
            block_tree: BlockTree::<ExtGroupDescriptor>::new(0, 0, 0),
        }
    }

    /// Initializes the inode table.
    pub fn initialize(
        &mut self,
        features: &ExtFeatures,
        block_size: u32,
        inode_size: u16,
        number_of_inodes_per_block_group: u32,
        group_descriptors: &mut Vec<ExtGroupDescriptor>,
    ) -> Result<(), ErrorTrace> {
        self.format_version = features.get_format_version();
        self.metadata_checksum_seed = features.get_metadata_checksum_seed();
        self.block_size = block_size;
        self.inode_size = inode_size;
        self.number_of_inodes_per_block_group = number_of_inodes_per_block_group;

        let group_size: u64 = (number_of_inodes_per_block_group as u64) * (inode_size as u64);
        let block_tree_size: u64 = (group_descriptors.len() as u64) * group_size;
        self.block_tree = BlockTree::<ExtGroupDescriptor>::new(
            block_tree_size,
            number_of_inodes_per_block_group as u64,
            inode_size as u64,
        );
        let mut inode_table_offset: u64 = 0;
        for group_descriptor in group_descriptors.drain(0..) {
            match self
                .block_tree
                .insert_value(inode_table_offset, group_size, group_descriptor)
            {
                Ok(_) => {}
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
            inode_table_offset += group_size;
        }
        Ok(())
    }

    /// Retrieves a specific inode.
    pub fn get_inode(
        &self,
        data_stream: &DataStreamReference,
        inode_number: u32,
    ) -> Result<ExtInode, ErrorTrace> {
        let inode_table_offset: u64 = ((inode_number - 1) as u64) * (self.inode_size as u64);

        let group_descriptor: &ExtGroupDescriptor =
            match self.block_tree.get_value(inode_table_offset) {
                Some(value) => value,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing group descriptor for inode: {}",
                        inode_number
                    )));
                }
            };
        let inode_group_index: u32 = (inode_number - 1) % self.number_of_inodes_per_block_group;

        let mut inode_data_offset: u64 = (inode_group_index as u64) * (self.inode_size as u64);
        inode_data_offset += group_descriptor.inode_table_block_number * (self.block_size as u64);

        let inode_size: usize = self.inode_size as usize;
        let mut data: Vec<u8> = vec![0; inode_size];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(inode_data_offset)
        );
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "ExtInode data of size: {} at offset: {} (0x{:08x})\n",
                self.inode_size, inode_data_offset, inode_data_offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        let mut inode: ExtInode = ExtInode::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(inode.debug_read_data(self.format_version, &data));
        }
        match inode.read_data(self.format_version, &data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read inode");
                return Err(error);
            }
        }
        match self.metadata_checksum_seed {
            Some(checksum_seed) => {
                let mut crc32_context: ReversedCrc32Context =
                    ReversedCrc32Context::new(0x82f63b78, checksum_seed);

                let inode_number_data: [u8; 4] = inode_number.to_le_bytes();
                crc32_context.update(&inode_number_data);
                crc32_context.update(&data[100..104]);
                crc32_context.update(&data[0..124]);
                crc32_context.update(&[0; 2]);
                crc32_context.update(&data[126..128]);

                if inode_size > 128 {
                    crc32_context.update(&data[128..130]);
                    crc32_context.update(&[0; 2]);
                    crc32_context.update(&data[132..inode_size]);
                }
                let mut calculated_checksum: u32 = crc32_context.finalize();
                calculated_checksum = 0xffffffff - calculated_checksum;

                if inode_size <= 128 {
                    calculated_checksum &= 0x0000ffff;
                }
                if inode.checksum != 0 && (inode.checksum as u32) != calculated_checksum {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Mismatch between stored: 0x{:04x} and calculated: 0x{:04x} checksums",
                        inode.checksum, calculated_checksum
                    )));
                }
            }
            None => {}
        };
        match inode.read_data_reference(self.format_version, data_stream, self.block_size) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read inode data reference");
                return Err(error);
            }
        }
        Ok(inode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests.
}
