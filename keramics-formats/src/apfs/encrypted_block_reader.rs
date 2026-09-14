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

use std::cmp::{Ordering, min};
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::BlockReader;

use super::encryption_context::ApfsEncryptionContext;
use super::extent::ApfsExtent;

/// Apple File System (APFS) encrypted block reader.
pub struct ApfsEncryptedBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Encryption context.
    encryption_context: Arc<ApfsEncryptionContext>,

    /// Block size.
    block_size: u32,

    /// Extents.
    extents: Vec<ApfsExtent>,

    /// The size.
    size: u64,
}

impl ApfsEncryptedBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        encryption_context: &Arc<ApfsEncryptionContext>,
        block_size: u32,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            encryption_context: encryption_context.clone(),
            block_size,
            extents: Vec::new(),
            size,
        }
    }

    /// Opens a block reader.
    pub(super) fn open(&mut self, extents: Vec<ApfsExtent>) -> Result<(), ErrorTrace> {
        self.extents = extents;

        Ok(())
    }
}

impl BlockReader for ApfsEncryptedBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the extents.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            let extent_end_offset: u64 = extent.logical_offset + extent.size;

            if current_offset >= extent_end_offset {
                Ordering::Less
            } else if current_offset < extent.logical_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let extent: &ApfsExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {} for offset: {} (0x{:08x})",
                        extent_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - extent.logical_offset;
            let range_remainder_size: u64 = extent.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            // An extent with physical block number 0 is sparse.
            if extent.physical_block_number == 0 {
                data[data_offset..data_end_offset].fill(0);
            } else {
                let range_physical_offset: u64 =
                    (extent.physical_block_number as u64) * (self.block_size as u64);

                let mut block_offset: u64 = range_relative_offset;
                while data_offset < data_end_offset {
                    let block_start_offset: u64 =
                        (block_offset / (self.block_size as u64)) * (self.block_size as u64);
                    let block_data_offset: usize = (block_offset - block_start_offset) as usize;
                    let block_physical_offset: u64 = range_physical_offset + block_start_offset;

                    let mut encrypted_data: Vec<u8> = vec![0; self.block_size as usize];

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.data_stream,
                        &mut encrypted_data,
                        SeekFrom::Start(block_physical_offset)
                    );
                    let mut block_data: Vec<u8> = vec![0; self.block_size as usize];

                    match self.encryption_context.decrypt_block(
                        block_physical_offset,
                        &encrypted_data,
                        &mut block_data,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to decrypt block: {} (0x{:08x})",
                                    block_physical_offset, block_physical_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                    let block_read_size: usize = min(
                        data_end_offset - data_offset,
                        (self.block_size as usize) - block_data_offset,
                    );
                    let block_data_end_offset: usize = block_data_offset + block_read_size;

                    data[data_offset..data_offset + block_read_size]
                        .copy_from_slice(&block_data[block_data_offset..block_data_end_offset]);
                    data_offset += block_read_size;
                    block_offset += block_read_size as u64;
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            extent_index += 1;
        }
        Ok(data_offset)
    }
}
