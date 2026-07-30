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
use std::sync::{Arc, RwLock};

use keramics_compression::{LzfseContext, Lznt1Context, ZlibContext};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use super::enums::DecmpfsCompressionMethod;

use crate::lru_cache::LruCache;

/// Apple File System Compression (decmpfs) data stream.
pub struct DecmpfsDataStream {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Compression method.
    compression_method: DecmpfsCompressionMethod,

    /// The compressed size.
    compressed_size: u64,

    /// The current offset.
    current_offset: u64,

    /// The size.
    size: u64,

    /// The block offsets.
    block_offsets: Vec<u64>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,
}

impl DecmpfsDataStream {
    const BLOCK_SIZE: usize = 65536;

    /// Creates a new compressed stream.
    pub(crate) fn new(compression_method: DecmpfsCompressionMethod) -> Self {
        Self {
            data_stream: None,
            compression_method,
            compressed_size: 0,
            current_offset: 0,
            size: 0,
            block_offsets: Vec::new(),
            block_cache: LruCache::new(8),
        }
    }

    /// Opens a block stream.
    pub(crate) fn open(
        &mut self,
        data_stream: &DataStreamReference,
        size: u64,
    ) -> Result<(), ErrorTrace> {
        self.compressed_size = match data_stream.write() {
            Ok(mut data_stream) => match data_stream.get_size() {
                Ok(size) => size,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve size");
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
        self.data_stream = Some(data_stream.clone());
        self.size = size;

        Ok(())
    }

    /// Reads media data based on the compressed blocks.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        todo!();
    }
}

impl DataStream for DecmpfsDataStream {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = match self.read_data_from_blocks(&mut buf[..read_size]) {
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
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset value out of bounds"
                    ));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;
}
