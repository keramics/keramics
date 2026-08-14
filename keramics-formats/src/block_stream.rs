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

use std::cmp::min;
use std::io::SeekFrom;

use keramics_core::{DataStream, ErrorTrace};

use super::traits::BlockReader;

/// Block stream.
pub struct BlockStream<T: BlockReader> {
    /// The block reader.
    block_reader: T,

    /// The current offset.
    current_offset: u64,
}

impl<T: BlockReader> BlockStream<T> {
    /// Creates a new block stream.
    pub(super) fn new(block_reader: T) -> Self {
        Self {
            block_reader,
            current_offset: 0,
        }
    }
}

impl<T: BlockReader + Send + Sync> DataStream for BlockStream<T> {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.block_reader.get_size())
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        let size: u64 = self.block_reader.get_size();

        if self.current_offset >= size {
            return Ok(0);
        }
        let remaining_size: u64 = size - self.current_offset;
        let read_size: usize = min(buf.len(), remaining_size as usize);

        let read_count: usize = match self
            .block_reader
            .read_data_from_blocks(&mut buf[..read_size], self.current_offset)
        {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                return Err(error);
            }
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::End(relative_offset) => {
                let size: u64 = self.block_reader.get_size();

                match size.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}
