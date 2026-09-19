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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;
use crate::traits::BlockReader;

use super::block_descriptor::VolsnapBlockDescriptor;

/// Volume Shadow Snapshot (volsnap) block reader.
pub struct VolsnapBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Block tree.
    block_tree: Arc<BlockTree<VolsnapBlockDescriptor>>,

    /// Size.
    size: u64,
}

impl VolsnapBlockReader {
    /// Creates a block reader.
    pub fn new(
        data_stream: &DataStreamReference,
        block_size: u32,
        block_tree: Arc<BlockTree<VolsnapBlockDescriptor>>,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            block_tree: block_tree.clone(),
            size,
        }
    }
}

impl BlockReader for VolsnapBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_number: u64 = current_offset / (self.block_size as u64);
            let range_read_count: usize = 0;

            match self.block_tree.get_value(current_offset) {
                Ok(Some(block_descriptor)) => {
                    // TODO: add check for availability of next store
                    if block_descriptor.is_forwarder() {
                        // TODO: read data from next store
                    } else {
                        // TODO: read data from volume
                    }
                }
                Ok(None) => {
                    // TODO: add check for availability of next store
                    // TODO: read data from next store

                    // TODO check for reverse block descriptor
                    // TODO check current bitmap
                    // if not in reverse block list and in current and previous bitmap
                    // fill the buffer with 0-byte values

                    // TODO: read data from volume
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block descriptor for offset: {} (0x{:08x})",
                            current_offset, current_offset,
                        )
                    );
                    return Err(error);
                }
            }
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            current_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }
}
