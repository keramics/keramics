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

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;

use super::block_range::{ExtBlockRange, ExtBlockRangeType};

/// Extended File System (ext) block stream.
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
    ) -> Result<(), ErrorTrace> {
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
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads media data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
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
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {}",
                        current_offset
                    )));
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
                ExtBlockRangeType::InFile => {
                    let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                        Some(data_stream) => data_stream,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing data stream"));
                        }
                    };
                    let range_physical_offset: u64 =
                        block_range.physical_block_number * (self.block_size as u64);

                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_physical_offset + range_relative_offset)
                    );
                    read_count
                }
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

impl DataStream for ExtBlockStream {
    /// Retrieves the size of the data.
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
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_block_stream() -> Result<ExtBlockStream, ErrorTrace> {
        let mut block_stream = ExtBlockStream::new(1024, 11358);

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ext/ext2.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let block_ranges: Vec<ExtBlockRange> = vec![
            ExtBlockRange {
                logical_block_number: 0,
                physical_block_number: 3073,
                number_of_blocks: 12,
                range_type: ExtBlockRangeType::InFile,
            },
            ExtBlockRange {
                logical_block_number: 12,
                physical_block_number: 0,
                number_of_blocks: 14,
                range_type: ExtBlockRangeType::Sparse,
            },
        ];
        block_stream.open(&data_stream, 26, &block_ranges)?;

        Ok(block_stream)
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut block_stream = ExtBlockStream::new(1024, 11358);

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ext/ext2.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let block_ranges: Vec<ExtBlockRange> = vec![
            ExtBlockRange {
                logical_block_number: 0,
                physical_block_number: 3073,
                number_of_blocks: 12,
                range_type: ExtBlockRangeType::InFile,
            },
            ExtBlockRange {
                logical_block_number: 12,
                physical_block_number: 0,
                number_of_blocks: 14,
                range_type: ExtBlockRangeType::Sparse,
            },
        ];
        block_stream.open(&data_stream, 26, &block_ranges)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;

        let size: u64 = block_stream.get_size()?;
        assert_eq!(size, 11358);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, block_stream.size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;

        let offset = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = block_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, block_stream.size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x59, 0x6f, 0x75, 0x22, 0x20, 0x28, 0x6f,
            0x72, 0x20, 0x22, 0x59, 0x6f, 0x75, 0x72, 0x22, 0x29, 0x20, 0x73, 0x68, 0x61, 0x6c,
            0x6c, 0x20, 0x6d, 0x65, 0x61, 0x6e, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x64, 0x69,
            0x76, 0x69, 0x64, 0x75, 0x61, 0x6c, 0x20, 0x6f, 0x72, 0x20, 0x4c, 0x65, 0x67, 0x61,
            0x6c, 0x20, 0x45, 0x6e, 0x74, 0x69, 0x74, 0x79, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 0x69, 0x73, 0x69, 0x6e, 0x67, 0x20, 0x70, 0x65,
            0x72, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x73, 0x20, 0x67, 0x72, 0x61, 0x6e,
            0x74, 0x65, 0x64, 0x20, 0x62, 0x79, 0x20, 0x74, 0x68, 0x69, 0x73, 0x20, 0x4c, 0x69,
            0x63, 0x65, 0x6e, 0x73, 0x65, 0x2e, 0x0a, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x22, 0x53, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x22, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x20,
            0x73, 0x68, 0x61, 0x6c, 0x6c, 0x20, 0x6d, 0x65, 0x61, 0x6e, 0x20, 0x74, 0x68, 0x65,
            0x20, 0x70, 0x72, 0x65, 0x66, 0x65, 0x72, 0x72, 0x65, 0x64, 0x20, 0x66, 0x6f, 0x72,
            0x6d, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x6d, 0x61, 0x6b, 0x69, 0x6e, 0x67, 0x20, 0x6d,
            0x6f, 0x64, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x2c, 0x0a,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x69, 0x6e, 0x63, 0x6c, 0x75, 0x64, 0x69, 0x6e,
            0x67, 0x20, 0x62, 0x75, 0x74, 0x20, 0x6e, 0x6f, 0x74, 0x20, 0x6c, 0x69, 0x6d, 0x69,
            0x74, 0x65, 0x64, 0x20, 0x74, 0x6f, 0x20, 0x73, 0x6f, 0x66, 0x74, 0x77, 0x61, 0x72,
            0x65, 0x20, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x20, 0x63, 0x6f, 0x64, 0x65, 0x2c,
            0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e,
            0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x2c,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x75, 0x72, 0x61,
            0x74, 0x69, 0x6f, 0x6e, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x73, 0x2e, 0x0a, 0x0a, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x4f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x22, 0x20,
            0x66, 0x6f, 0x72, 0x6d, 0x20, 0x73, 0x68, 0x61, 0x6c, 0x6c, 0x20, 0x6d, 0x65, 0x61,
            0x6e, 0x20, 0x61, 0x6e, 0x79, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x20, 0x72, 0x65, 0x73,
            0x75, 0x6c, 0x74, 0x69, 0x6e, 0x67, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x6d, 0x65,
            0x63, 0x68, 0x61, 0x6e, 0x69, 0x63, 0x61, 0x6c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x69, 0x6f,
            0x6e, 0x20, 0x6f, 0x72, 0x20, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x6c, 0x61, 0x74, 0x69,
            0x6f, 0x6e, 0x20, 0x6f, 0x66, 0x20, 0x61, 0x20, 0x53, 0x6f, 0x75, 0x72, 0x63, 0x65,
            0x20, 0x66, 0x6f, 0x72, 0x6d, 0x2c, 0x20, 0x69, 0x6e, 0x63, 0x6c, 0x75, 0x64, 0x69,
            0x6e, 0x67, 0x20, 0x62, 0x75, 0x74, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x6e,
            0x6f, 0x74, 0x20, 0x6c, 0x69, 0x6d, 0x69, 0x74, 0x65, 0x64, 0x20, 0x74, 0x6f, 0x20,
            0x63, 0x6f, 0x6d, 0x70, 0x69, 0x6c, 0x65, 0x64, 0x20, 0x6f, 0x62, 0x6a, 0x65, 0x63,
            0x74, 0x20, 0x63, 0x6f, 0x64, 0x65, 0x2c, 0x20, 0x67, 0x65, 0x6e, 0x65, 0x72, 0x61,
            0x74, 0x65, 0x64, 0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x61, 0x74,
            0x69, 0x6f, 0x6e, 0x2c, 0x0a, 0x20, 0x20, 0x20,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: ExtBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
