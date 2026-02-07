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

use keramics_core::{DataStreamReference, ErrorTrace};

use super::block_range::HfsBlockRange;
use super::extent_descriptor::HfsExtentDescriptor;
use super::extents_overflow_file::HfsExtentsOverflowFile;
use super::fork_descriptor::HfsForkDescriptor;

/// Hierarchical File System (HFS) block ranges.
pub struct HfsBlockRanges {
    /// Number of blocks.
    pub number_of_blocks: u32,

    /// Ranges.
    pub ranges: Vec<HfsBlockRange>,
}

impl HfsBlockRanges {
    /// Creates new block ranges.
    pub fn new() -> Self {
        Self {
            number_of_blocks: 0,
            ranges: Vec::new(),
        }
    }

    /// Reads the block ranges from extents.
    pub fn read_extents(
        &mut self,
        data_area_block_number: u16,
        extents: &Vec<HfsExtentDescriptor>,
    ) -> Result<(), ErrorTrace> {
        let mut logical_block_number: u32 = 0;

        for extent_descriptor in extents.iter() {
            if extent_descriptor.block_number > u32::MAX - (data_area_block_number as u32) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid block number: {} value out of bounds",
                    extent_descriptor.block_number
                )));
            }
            if extent_descriptor.number_of_blocks > u32::MAX - logical_block_number {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid number of blocks: {} value out of bounds",
                    extent_descriptor.number_of_blocks
                )));
            }
            let physical_block_number: u32 =
                (data_area_block_number as u32) + extent_descriptor.block_number;

            let block_range: HfsBlockRange = HfsBlockRange::new(
                logical_block_number,
                physical_block_number,
                extent_descriptor.number_of_blocks,
            );
            self.ranges.push(block_range);

            logical_block_number += extent_descriptor.number_of_blocks;
        }
        self.number_of_blocks = logical_block_number;

        Ok(())
    }

    /// Reads the block ranges from a fork descriptor and overflow extents.
    pub fn read_fork_descriptor(
        &mut self,
        data_area_block_number: u16,
        identifier: u32,
        fork_descriptor: &HfsForkDescriptor,
        data_stream: &DataStreamReference,
        extents_overflow_file: &HfsExtentsOverflowFile,
    ) -> Result<(), ErrorTrace> {
        match self.read_extents(data_area_block_number, &fork_descriptor.extents) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine block ranges from fork descriptor"
                );
                return Err(error);
            }
        }
        if self.number_of_blocks < fork_descriptor.number_of_blocks {
            let mut overflow_extents: Vec<HfsExtentDescriptor> = Vec::new();

            match extents_overflow_file.get_extents_by_identifier(
                data_stream,
                identifier,
                &mut overflow_extents,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve overflow extents of file entry: {}",
                            identifier
                        )
                    );
                    return Err(error);
                }
            }
            match self.read_extents(data_area_block_number, &overflow_extents) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to determine block ranges from overflow extents"
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}
