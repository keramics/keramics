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

use std::io;
use std::io::{Read, Seek};

use keramics_core::{DataStream, DataStreamReference};

use crate::block_tree::BlockTree;

use super::block_range::{ExtBlockRange, ExtBlockRangeType};

/// Extended File System block stream.
pub struct ExtBlockStream {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Block size.
    block_size: u32,

    /// Block tree.
    block_tree: BlockTree<ExtBlockRange>,

    /// The current offset.
    current_offset: u64,

    /// The size.
    size: u64,
}

impl ExtBlockStream {
    /// Creates a new block stream.
    pub(super) fn new(block_size: u32, size: u64) -> Self {
        Self {
            data_stream: None,
            block_size: block_size,
            block_tree: BlockTree::<ExtBlockRange>::new(0, 0, 0),
            current_offset: 0,
            size: size,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(
        &mut self,
        data_stream: &DataStreamReference,
        number_of_blocks: u64,
        block_ranges: &Vec<ExtBlockRange>,
    ) -> io::Result<()> {
        let block_tree_data_size: u64 = number_of_blocks * (self.block_size as u64);
        self.block_tree =
            BlockTree::<ExtBlockRange>::new(block_tree_data_size, 0, self.block_size as u64);
        for block_range in block_ranges.iter() {
            let logical_offset: u64 = block_range.logical_block_number * (self.block_size as u64);
            let range_size: u64 = block_range.number_of_blocks * (self.block_size as u64);
            match self
                .block_tree
                .insert_value(logical_offset, range_size, block_range.clone())
            {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            };
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads media data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> io::Result<usize> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = self.current_offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_range: &ExtBlockRange = match self.block_tree.get_value(current_offset) {
                Some(value) => value,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Missing block range for offset: {}", current_offset),
                    ));
                }
            };
            let range_logical_offset: u64 =
                block_range.logical_block_number * (self.block_size as u64);
            let range_size: u64 = block_range.number_of_blocks * (self.block_size as u64);

            let range_relative_offset: u64 = current_offset - range_logical_offset;
            let range_remainder_size: u64 = range_size - range_relative_offset;

            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;
            let range_read_count: usize = match block_range.range_type {
                ExtBlockRangeType::InFile => match self.data_stream.as_ref() {
                    Some(data_stream) => match data_stream.write() {
                        Ok(mut data_stream) => {
                            let range_physical_offset: u64 =
                                block_range.physical_block_number * (self.block_size as u64);
                            data_stream.read_at_position(
                                &mut data[data_offset..data_end_offset],
                                io::SeekFrom::Start(range_physical_offset + range_relative_offset),
                            )?
                        }
                        Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                    },
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing data stream",
                        ))
                    }
                },
                ExtBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            current_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }
}

impl Read for ExtBlockStream {
    /// Reads data.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = self.read_data_from_blocks(&mut buf[..read_size])?;

        self.current_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for ExtBlockStream {
    /// Sets the current position of the data.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.current_offset = match pos {
            io::SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            io::SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            io::SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

impl DataStream for ExtBlockStream {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.size)
    }
}

// TODO: add tests
