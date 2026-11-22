/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::indexed_hash_map::IndexedHashMap;

use super::block_tree_branch_entry::XfsBlockTreeBranchEntry;
use super::block_tree_branch_header::XfsBlockTreeBranchHeader;
use super::block_tree_leaf_header::XfsBlockTreeLeafHeader;
use super::directory_entry::XfsDirectoryEntry;
use super::directory_tree_leaf_entry::XfsDirectoryTreeLeafEntry;
use super::directory_tree_value::XfsDirectoryTreeValue;
use super::file_system_block::XfsFileSystemBlock;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) directory tree.
pub struct XfsDirectoryTree {
    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Allocation group size.
    allocation_group_size: u32,

    /// Block size.
    block_size: u32,

    /// Block number bit shift.
    block_number_bit_shift: u64,

    /// Relative block number bit mask.
    relative_block_number_bit_mask: u64,
}

impl XfsDirectoryTree {
    /// Creates a new directory tree.
    pub fn new(
        character_encoding: &CharacterEncoding,
        allocation_group_size: u32,
        number_of_relative_block_number_bits: u32,
        block_size: u32,
    ) -> Self {
        Self {
            character_encoding: character_encoding.clone(),
            allocation_group_size,
            block_size,
            block_number_bit_shift: number_of_relative_block_number_bits as u64,
            relative_block_number_bit_mask: (1 << (number_of_relative_block_number_bits as u64))
                - 1,
        }
    }

    /// Reads the directory entries.
    pub fn read_entries(
        &self,
        data_stream: &DataStreamReference,
        extents: &[XfsPackedExtent],
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut read_block_numbers: HashSet<u32> = HashSet::new();

        match self.read_entries_from_node(data_stream, 0, extents, entries, &mut read_block_numbers)
        {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read directory from root node: 0"
                );
                Err(error)
            }
        }
    }

    /// Reads the directory entries from a directory tree branch node.
    pub fn read_entries_from_branch_node(
        &self,
        file_system_block: &XfsFileSystemBlock,
        data_stream: &DataStreamReference,
        extents: &[XfsPackedExtent],
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<(), ErrorTrace> {
        if file_system_block.signature != 0xfebe {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported file system block signature: 0x{:04x}",
                file_system_block.signature
            )));
        }
        let data_size: usize = file_system_block.data.len();

        if data_size < 16 {
            return Err(keramics_core::error_trace_new!(
                "Invalid branch header size value out of bounds"
            ));
        }
        keramics_core::debug_trace_structure!(XfsBlockTreeBranchHeader::debug_read_data(
            &file_system_block.data[12..16]
        ));
        let mut branch_header: XfsBlockTreeBranchHeader = XfsBlockTreeBranchHeader::new();

        match branch_header.read_data(&file_system_block.data[12..16]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block tree branch header"
                );
                return Err(error);
            }
        }
        let entries_data_end_offset: usize = 16 + ((branch_header.number_of_entries as usize) * 8);

        if entries_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of entries value out of bounds"
            ));
        }
        let mut data_offset: usize = 16;

        for entry_index in 0..branch_header.number_of_entries {
            keramics_core::debug_trace_structure!(XfsBlockTreeBranchEntry::debug_read_data(
                &file_system_block.data[data_offset..]
            ));
            let mut entry: XfsBlockTreeBranchEntry = XfsBlockTreeBranchEntry::new();

            match entry.read_data(&file_system_block.data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory tree branch entry: {}",
                            entry_index
                        ),
                    );
                    return Err(error);
                }
            }
            data_offset += 8;

            match self.read_entries_from_node(
                data_stream,
                entry.block_number,
                extents,
                entries,
                read_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory from root node: {}",
                            entry.block_number
                        ),
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads the directory entries from a directory tree leaf node.
    pub fn read_entries_from_leaf_node(
        &self,
        file_system_block: &XfsFileSystemBlock,
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        if file_system_block.signature != 0xfeeb {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported file system block signature: 0x{:04x}",
                file_system_block.signature
            )));
        }
        let data_size: usize = file_system_block.data.len();

        if data_size < 32 {
            return Err(keramics_core::error_trace_new!(
                "Invalid leaf header size value out of bounds"
            ));
        }
        keramics_core::debug_trace_structure!(XfsBlockTreeLeafHeader::debug_read_data(
            &file_system_block.data[12..32]
        ));
        let mut leaf_header: XfsBlockTreeLeafHeader = XfsBlockTreeLeafHeader::new();

        match leaf_header.read_data(&file_system_block.data[12..32]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block tree leaf header"
                );
                return Err(error);
            }
        }
        let entries_data_end_offset: usize = 32 + ((leaf_header.number_of_entries as usize) * 8);

        if entries_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of entries value out of bounds"
            ));
        }
        let mut data_offset: usize = 32;

        for entry_index in 0..leaf_header.number_of_entries {
            keramics_core::debug_trace_structure!(XfsDirectoryTreeLeafEntry::debug_read_data(
                &file_system_block.data[data_offset..]
            ));
            let mut entry: XfsDirectoryTreeLeafEntry = XfsDirectoryTreeLeafEntry::new();

            match entry.read_data(&file_system_block.data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read directory tree leaf entry: {}", entry_index),
                    );
                    return Err(error);
                }
            }
            data_offset += 8;

            let value_offset: usize = entry.value_offset as usize;

            if value_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - value offset value out of bounds",
                    entry_index
                )));
            }
            let value_end_offset: usize = value_offset + 8;

            if value_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - value size value out of bounds",
                    entry_index
                )));
            }
            let mut value: XfsDirectoryTreeValue = XfsDirectoryTreeValue::new();

            match value.read_data(&file_system_block.data[value_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory tree leaf entry: {} remote value",
                            entry_index
                        ),
                    );
                    return Err(error);
                }
            }
            let name_end_offset: usize = value_end_offset + (entry.name_size as usize);

            if entry.name_size == 0 || name_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - name size value out of bounds",
                    entry_index
                )));
            }
            let mut name: ByteString = ByteString::new_with_encoding(&self.character_encoding);
            name.read_data(&file_system_block.data[value_end_offset..name_end_offset]);

            if name != "." && name != ".." {
                entries.insert(name, XfsDirectoryEntry::new(value.inode_number, 0));
            }
        }
        Ok(())
    }

    /// Reads the directory entries from a directory tree node.
    pub fn read_entries_from_node(
        &self,
        data_stream: &DataStreamReference,
        logical_block_number: u32,
        extents: &[XfsPackedExtent],
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<(), ErrorTrace> {
        if read_block_numbers.contains(&logical_block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Directory tree node: {} already read",
                logical_block_number
            )));
        }
        let extent_index: usize = match extents.binary_search_by(|extent| {
            let extent_end_block_number: u64 =
                extent.logical_block_number + (extent.number_of_blocks as u64);

            if (logical_block_number as u64) >= extent_end_block_number {
                Ordering::Less
            } else if (logical_block_number as u64) < extent.logical_block_number {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for node: {}",
                    logical_block_number
                )));
            }
        };
        let extent: &XfsPackedExtent = match extents.get(extent_index) {
            Some(extent) => extent,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve extent: {} for node: {}",
                    extent_index, logical_block_number
                )));
            }
        };
        let allocation_group_index: u64 =
            extent.physical_block_number >> self.block_number_bit_shift;
        let allocation_group_block_number: u64 =
            allocation_group_index * (self.allocation_group_size as u64);
        let relative_block_number: u64 =
            extent.physical_block_number & self.relative_block_number_bit_mask;
        let extent_block_number: u64 = (logical_block_number as u64) - extent.logical_block_number;
        let physical_block_number: u64 = relative_block_number + extent_block_number;

        let block_offset: u64 =
            (allocation_group_block_number + physical_block_number) * (self.block_size as u64);

        let mut file_system_block: XfsFileSystemBlock = XfsFileSystemBlock::new();

        match file_system_block.read_at_position(
            data_stream,
            self.block_size,
            SeekFrom::Start(block_offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read allocation group: {} file system block: {}",
                        allocation_group_index, physical_block_number
                    )
                );
                return Err(error);
            }
        }
        read_block_numbers.insert(logical_block_number);

        if file_system_block.signature == 0xfeeb {
            match self.read_entries_from_leaf_node(&file_system_block, entries) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory from directory tree leaf node: {}",
                            logical_block_number
                        ),
                    );
                    return Err(error);
                }
            }
        } else if file_system_block.signature == 0xfebe {
            match self.read_entries_from_branch_node(
                &file_system_block,
                data_stream,
                extents,
                entries,
                read_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory from directory tree branch node: {}",
                            logical_block_number
                        ),
                    );
                    return Err(error);
                }
            }
        } else {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported allocation group: {} file system block: {} signature: 0x{:04x}",
                allocation_group_index, physical_block_number, file_system_block.signature
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::xfs::enums::XfsExtentType;

    fn build_directory_leaf_block(block_size: usize) -> Vec<u8> {
        let mut block: Vec<u8> = vec![0; block_size];
        // File system block signature 0xfeeb at offset 8.
        block[8..10].copy_from_slice(&0xfeebu16.to_be_bytes());
        // Leaf header at offset 12..32: number_of_entries = 3 (".", "..", "entry1").
        block[12..14].copy_from_slice(&3u16.to_be_bytes());

        // Entry 0 at offset 32: "." at value_offset 128.
        block[36..38].copy_from_slice(&128u16.to_be_bytes());
        block[38] = 1;
        block[128..136].copy_from_slice(&10u64.to_be_bytes());
        block[136] = b'.';

        // Entry 1 at offset 40: ".." at value_offset 144.
        block[44..46].copy_from_slice(&144u16.to_be_bytes());
        block[46] = 2;
        block[144..152].copy_from_slice(&11u64.to_be_bytes());
        block[152..154].copy_from_slice(b"..");

        // Entry 2 at offset 48: "entry1" at value_offset 160.
        block[52..54].copy_from_slice(&160u16.to_be_bytes());
        block[54] = 6;
        block[160..168].copy_from_slice(&200u64.to_be_bytes());
        block[168..174].copy_from_slice(b"entry1");

        block
    }

    #[test]
    fn test_read_entries_branch_and_leaf() -> Result<(), ErrorTrace> {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 3 * block_size];

        // Block 0: branch node (0xfebe) pointing to logical block 1.
        let branch: &mut [u8] = &mut image[0..block_size];
        branch[8..10].copy_from_slice(&0xfebeu16.to_be_bytes());
        branch[12..14].copy_from_slice(&1u16.to_be_bytes());
        branch[20..24].copy_from_slice(&1u32.to_be_bytes());

        // Block 1: leaf node (0xfeeb).
        let leaf: Vec<u8> = build_directory_leaf_block(block_size);
        image[block_size..2 * block_size].copy_from_slice(&leaf);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let directory_tree: XfsDirectoryTree =
            XfsDirectoryTree::new(&CharacterEncoding::Utf8, 16, 4, block_size as u32);
        let extents: [XfsPackedExtent; 1] = [XfsPackedExtent {
            number_of_blocks: 2,
            physical_block_number: 0,
            logical_block_number: 0,
            extent_type: XfsExtentType::InFile,
        }];
        let mut entries: IndexedHashMap<ByteString, XfsDirectoryEntry> = IndexedHashMap::new();
        directory_tree.read_entries(&data_stream, &extents, &mut entries)?;

        assert_eq!(entries.len(), 1);
        let entry: &XfsDirectoryEntry = entries
            .get_value_by_key(&ByteString::from("entry1"))
            .unwrap();
        assert_eq!(entry.inode_number, 200);

        Ok(())
    }

    #[test]
    fn test_read_entries_with_errors() {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 3 * block_size];
        // Block 0: unsupported signature 0x1234.
        image[8..10].copy_from_slice(&0x1234u16.to_be_bytes());
        // Block 1: branch pointing back to block 1 (cycle).
        image[block_size + 8..block_size + 10].copy_from_slice(&0xfebeu16.to_be_bytes());
        image[block_size + 12..block_size + 14].copy_from_slice(&1u16.to_be_bytes());
        image[block_size + 20..block_size + 24].copy_from_slice(&1u32.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let directory_tree: XfsDirectoryTree =
            XfsDirectoryTree::new(&CharacterEncoding::Utf8, 16, 4, block_size as u32);
        let mut entries: IndexedHashMap<ByteString, XfsDirectoryEntry> = IndexedHashMap::new();

        // Test missing extent for root block 0.
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries(&data_stream, &[], &mut entries);
        assert!(result.is_err());

        // Test unsupported signature at block 0.
        let extents: [XfsPackedExtent; 1] = [XfsPackedExtent {
            number_of_blocks: 2,
            physical_block_number: 0,
            logical_block_number: 0,
            extent_type: XfsExtentType::InFile,
        }];
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries(&data_stream, &extents, &mut entries);
        assert!(result.is_err());

        // Test branch node cycle detection at block 1.
        let mut read_blocks: HashSet<u32> = HashSet::new();
        let result: Result<(), ErrorTrace> = directory_tree.read_entries_from_node(
            &data_stream,
            1,
            &extents,
            &mut entries,
            &mut read_blocks,
        );
        assert!(result.is_err());

        // Test direct branch node errors: wrong signature, short data, entries out of bounds.
        let mut block: XfsFileSystemBlock = XfsFileSystemBlock::new();
        block.signature = 0x0000;
        let mut read_blocks: HashSet<u32> = HashSet::new();
        let result: Result<(), ErrorTrace> = directory_tree.read_entries_from_branch_node(
            &block,
            &data_stream,
            &extents,
            &mut entries,
            &mut read_blocks,
        );
        assert!(result.is_err());

        block.signature = 0xfebe;
        block.data = vec![0; 10];
        let result: Result<(), ErrorTrace> = directory_tree.read_entries_from_branch_node(
            &block,
            &data_stream,
            &extents,
            &mut entries,
            &mut read_blocks,
        );
        assert!(result.is_err());

        block.data = vec![0; 20];
        block.data[12..14].copy_from_slice(&10u16.to_be_bytes());
        let result: Result<(), ErrorTrace> = directory_tree.read_entries_from_branch_node(
            &block,
            &data_stream,
            &extents,
            &mut entries,
            &mut read_blocks,
        );
        assert!(result.is_err());

        // Test direct leaf node errors: wrong signature, short data, entries OOB,
        // value offset OOB, value size OOB, zero/OOB name size.
        let mut leaf_block: XfsFileSystemBlock = XfsFileSystemBlock::new();
        leaf_block.signature = 0x0000;
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());

        leaf_block.signature = 0xfeeb;
        leaf_block.data = vec![0; 20];
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());

        leaf_block.data = vec![0; 40];
        leaf_block.data[12..14].copy_from_slice(&10u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());

        // Value offset out of bounds.
        leaf_block.data = vec![0; 64];
        leaf_block.data[12..14].copy_from_slice(&1u16.to_be_bytes());
        leaf_block.data[36..38].copy_from_slice(&100u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());

        // Value end offset out of bounds.
        leaf_block.data[36..38].copy_from_slice(&60u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());

        // Zero name size.
        leaf_block.data[36..38].copy_from_slice(&48u16.to_be_bytes());
        leaf_block.data[38] = 0;
        let result: Result<(), ErrorTrace> =
            directory_tree.read_entries_from_leaf_node(&leaf_block, &mut entries);
        assert!(result.is_err());
    }
}
