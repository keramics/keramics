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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;
use crate::traits::BlockReader;

use super::block_map::{VdiBlockMap, VdiBlockMapEntry};
use super::block_range::{VdiBlockRange, VdiBlockRangeType};

/// Virtual Disk Image (VDI) block reader.
pub struct VdiBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Data (blocks) area offset.
    data_offset: u64,

    /// Block size.
    block_size: u64,

    /// Block map.
    block_map: VdiBlockMap,

    /// Block tree.
    block_tree: BlockTree<VdiBlockRange>,

    /// Parent data stream.
    parent_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl VdiBlockReader {
    /// Creates a block reader.
    pub fn new(
        data_stream: &DataStreamReference,
        block_map_offset: u64,
        data_offset: u64,
        block_size: u64,
        number_of_blocks: u32,
        parent_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            data_offset,
            block_size,
            block_map: VdiBlockMap::new(block_map_offset, number_of_blocks),
            block_tree: BlockTree::<VdiBlockRange>::new(size, 0, block_size),
            parent_data_stream,
            size,
        }
    }

    /// Reads a specific block map entry and fills the block tree.
    fn read_block_map_entry(&mut self, block_number: u64) -> Result<(), ErrorTrace> {
        let entry: VdiBlockMapEntry = match self
            .block_map
            .read_entry(&self.data_stream, block_number as u32)
        {
            Ok(entry) => entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read block map entry");
                return Err(error);
            }
        };
        let media_offset: u64 = block_number * self.block_size;

        let block_range: VdiBlockRange = match entry.block_number {
            0xffffffff => {
                if self.parent_data_stream.is_some() {
                    VdiBlockRange::new(
                        media_offset,
                        0,
                        self.block_size,
                        VdiBlockRangeType::InParent,
                    )
                } else {
                    VdiBlockRange::new(media_offset, 0, self.block_size, VdiBlockRangeType::Sparse)
                }
            }
            0xfffffffe => {
                VdiBlockRange::new(media_offset, 0, self.block_size, VdiBlockRangeType::Sparse)
            }
            physical_block_number => VdiBlockRange::new(
                media_offset,
                self.data_offset + (physical_block_number as u64 * self.block_size),
                self.block_size,
                VdiBlockRangeType::InFile,
            ),
        };
        match self
            .block_tree
            .insert_value(media_offset, self.block_size, block_range)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to insert block range into block tree"
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

impl BlockReader for VdiBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the block map.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let block_number: u64 = current_offset / self.block_size;

            let mut result: Result<Option<&VdiBlockRange>, ErrorTrace> =
                self.block_tree.get_value(current_offset);

            if result == Ok(None) {
                match self.read_block_map_entry(block_number) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to read block map entry: {}", block_number)
                        );
                        return Err(error);
                    }
                }
                result = self.block_tree.get_value(current_offset);
            }
            let block_range: &VdiBlockRange = match result {
                Ok(Some(block_range)) => block_range,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {} (0x{:08x})",
                        current_offset, current_offset
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                            current_offset, current_offset,
                        )
                    );
                    return Err(error);
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.media_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            let range_read_count: usize = match block_range.range_type {
                VdiBlockRangeType::InFile => {
                    let physical_offset: u64 = block_range.data_offset + range_relative_offset;

                    keramics_core::data_stream_read_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(physical_offset)
                    )
                }
                VdiBlockRangeType::InParent => match &self.parent_data_stream {
                    Some(data_stream) => {
                        keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(current_offset)
                        )
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing parent data stream"
                        ));
                    }
                },
                VdiBlockRangeType::Sparse => {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_block_reader() -> Result<VdiBlockReader, ErrorTrace> {
        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        Ok(VdiBlockReader::new(
            &data_stream,
            0x200,
            0x400,
            1048576,
            4,
            None,
            4194304,
        ))
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let block_reader: VdiBlockReader = get_block_reader()?;

        let size: u64 = block_reader.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks() -> Result<(), ErrorTrace> {
        let mut block_reader: VdiBlockReader = get_block_reader()?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_reader.read_data_from_blocks(&mut data, 0)?;
        assert_eq!(read_size, 512);

        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut expected_data: Vec<u8> = vec![0; 512];

        keramics_core::data_stream_read_at_position!(
            &file_data_stream,
            &mut expected_data,
            SeekFrom::Start(0x400)
        );
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_reader: VdiBlockReader = get_block_reader()?;

        let result: Result<usize, ErrorTrace> =
            block_reader.read_data_from_blocks(&mut [0; 512], 4194304);
        assert_eq!(result?, 0);

        Ok(())
    }
}
