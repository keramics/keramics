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

use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::indexed_hash_map::IndexedHashMap;

use super::directory_entry::XfsDirectoryEntry;
use super::directory_list_element::XfsDirectoryListElement;
use super::enums::XfsExtentType;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) directory list.
pub struct XfsDirectoryList {
    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Allocation group size.
    allocation_group_size: u32,

    /// Block size.
    block_size: u32,

    /// Directory block size.
    directory_block_size: u32,

    /// Block number bit shift.
    block_number_bit_shift: u64,

    /// Relative block number bit mask.
    relative_block_number_bit_mask: u64,
}

impl XfsDirectoryList {
    /// Creates a new directory list.
    pub fn new(
        character_encoding: &CharacterEncoding,
        allocation_group_size: u32,
        number_of_relative_block_number_bits: u32,
        block_size: u32,
        directory_block_size: u32,
    ) -> Self {
        Self {
            character_encoding: character_encoding.clone(),
            allocation_group_size,
            block_size,
            directory_block_size,
            block_number_bit_shift: number_of_relative_block_number_bits as u64,
            relative_block_number_bit_mask: (1 << (number_of_relative_block_number_bits as u64))
                - 1,
        }
    }

    /// Reads the directory entries.
    pub fn read_entries(
        &self,
        has_file_type: bool,
        data_stream: &DataStreamReference,
        data_size: u64,
        extents: &[XfsPackedExtent],
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut read_block_numbers: HashSet<u64> = HashSet::new();
        let mut extent_offset: u64 = 0;

        let blocks_per_directory_block: u32 = self.directory_block_size / self.block_size;

        for extent in extents.iter() {
            let logical_offset: u64 = extent.logical_block_number * (self.block_size as u64);

            if logical_offset > 0x800000000 {
                break;
            }
            let extent_size: u64 = (extent.number_of_blocks as u64) * (self.block_size as u64);

            if extent.extent_type == XfsExtentType::Sparse {
                extent_offset += extent_size;
                continue;
            }
            let allocation_group_index: u64 =
                extent.physical_block_number >> self.block_number_bit_shift;
            let allocation_group_block_number: u64 =
                allocation_group_index * (self.allocation_group_size as u64);
            let mut physical_block_number: u64 =
                extent.physical_block_number & self.relative_block_number_bit_mask;

            let extent_end_offset: u64 = extent_offset + extent_size;

            while extent_offset < extent_end_offset {
                if extent_offset >= data_size {
                    break;
                }
                if (self.directory_block_size as u64) > extent_end_offset - extent_offset {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported directory list element spanning multiple extents"
                    ));
                }
                match self.read_entries_from_element(
                    has_file_type,
                    data_stream,
                    allocation_group_block_number,
                    physical_block_number,
                    entries,
                    &mut read_block_numbers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read entries from directory list element: {}",
                                physical_block_number
                            )
                        );
                        return Err(error);
                    }
                }
                physical_block_number += blocks_per_directory_block as u64;
                extent_offset += self.directory_block_size as u64;
            }
        }
        Ok(())
    }

    /// Reads the directory entries from a directory list element.
    pub fn read_entries_from_element(
        &self,
        has_file_type: bool,
        data_stream: &DataStreamReference,
        allocation_group_block_number: u64,
        physical_block_number: u64,
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
        read_block_numbers: &mut HashSet<u64>,
    ) -> Result<(), ErrorTrace> {
        if read_block_numbers.contains(&physical_block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Directory list element: {} already read",
                physical_block_number
            )));
        }
        let directory_list_element: XfsDirectoryListElement =
            XfsDirectoryListElement::new(&self.character_encoding);

        let physical_offset: u64 =
            (allocation_group_block_number + physical_block_number) * (self.block_size as u64);

        match directory_list_element.read_at_position(
            has_file_type,
            data_stream,
            self.directory_block_size,
            SeekFrom::Start(physical_offset),
            entries,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read directory list element: {}",
                        physical_block_number
                    )
                );
                return Err(error);
            }
        }
        read_block_numbers.insert(physical_block_number);

        Ok(())
    }
}
