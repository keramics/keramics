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

use std::io::SeekFrom;
use std::path::PathBuf;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

/// Data stream of a specific range within another data stream.
pub struct RangeDataStream {
    /// The (base) data stream.
    data_stream: DataStreamReference,

    /// The current offset.
    current_offset: u64,

    /// The offset of the range.
    range_offset: u64,

    /// The size of the range.
    range_size: u64,
}

impl RangeDataStream {
    /// Creates a new data stream.
    pub fn new(data_stream: DataStreamReference, range_offset: u64) -> Self {
        Self {
            data_stream,
            current_offset: 0,
            range_offset,
            range_size: 0,
        }
    }

    /// Opens a data stream.
    pub fn open(&mut self) -> Result<(), ErrorTrace> {
        let data_stream_size: u64 = match self.data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => size,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to determine data stream size"
                    );
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        if self.range_offset >= data_stream_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid range offset value exceeds data stream size"
            ));
        }
        self.range_size = data_stream_size - self.range_offset;

        Ok(())
    }
}

impl DataStream for RangeDataStream {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.range_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.range_size {
            return Ok(0);
        }
        let remaining_size: u64 = self.range_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = keramics_core::data_stream_read_at_position!(
            &self.data_stream,
            &mut buf[0..read_size],
            SeekFrom::Start(self.range_offset + self.current_offset)
        );
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
                match self.range_size.checked_add_signed(relative_offset) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for get_offset.
    // TODO: add tests for get_size.
    // TODO: add tests for open.
    // TODO: add tests for read.
    // TODO: add tests for seek.
}
