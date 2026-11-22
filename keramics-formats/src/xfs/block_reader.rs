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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::BlockReader;

use super::enums::XfsExtentType;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) block reader.
pub struct XfsBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block size.
    block_size: u32,

    /// Relative block number bit mask.
    relative_block_number_bit_mask: u64,

    /// Extents.
    extents: Vec<XfsPackedExtent>,

    /// The size.
    size: u64,
}

impl XfsBlockReader {
    /// Creates a new block reader.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        number_of_relative_block_number_bits: u32,
        block_size: u32,
        extents: &[XfsPackedExtent],
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_size,
            relative_block_number_bit_mask: (1 << (number_of_relative_block_number_bits as u64))
                - 1,
            extents: extents.to_vec(),
            size,
        }
    }
}

impl BlockReader for XfsBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the extents.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let block_number: u64 = current_offset / (self.block_size as u64);

        let mut extent_index: usize = match self.extents.binary_search_by(|extent| {
            let extent_end_block_number: u64 =
                extent.logical_block_number + (extent.number_of_blocks as u64);

            if block_number >= extent_end_block_number {
                Ordering::Less
            } else if block_number < extent.logical_block_number {
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
            let extent: &XfsPackedExtent = match self.extents.get(extent_index) {
                Some(extent) => extent,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve extent: {} for offset: {} (0x{:08x})",
                        extent_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 =
                current_offset - (extent.logical_block_number * (self.block_size as u64));
            let range_remainder_size: u64 = ((extent.number_of_blocks as u64)
                * (self.block_size as u64))
                - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            match &extent.extent_type {
                XfsExtentType::InFile => {
                    let physical_block_number: u64 =
                        extent.physical_block_number & self.relative_block_number_bit_mask;

                    let range_physical_offset: u64 =
                        physical_block_number * (self.block_size as u64);

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_physical_offset + range_relative_offset)
                    );
                }
                XfsExtentType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            extent_index += 1;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    #[test]
    fn test_get_size() {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let block_reader: XfsBlockReader = XfsBlockReader::new(&data_stream, 4, 512, &[], 1024);

        assert_eq!(block_reader.get_size(), 1024);
    }

    #[test]
    fn test_read_data_from_blocks() -> Result<(), ErrorTrace> {
        let mut backing: Vec<u8> = vec![0; 24];
        backing[0..8].copy_from_slice(b"AAAAAAAA");
        backing[8..16].copy_from_slice(b"BBBBBBBB");
        backing[16..24].copy_from_slice(b"CCCCCCCC");

        let data_stream: DataStreamReference = open_fake_data_stream(&backing);
        let extents: [XfsPackedExtent; 3] = [
            XfsPackedExtent {
                number_of_blocks: 1,
                physical_block_number: 0,
                logical_block_number: 0,
                extent_type: XfsExtentType::InFile,
            },
            XfsPackedExtent {
                number_of_blocks: 1,
                physical_block_number: 0,
                logical_block_number: 1,
                extent_type: XfsExtentType::Sparse,
            },
            XfsPackedExtent {
                number_of_blocks: 1,
                physical_block_number: 2,
                logical_block_number: 2,
                extent_type: XfsExtentType::InFile,
            },
        ];
        let mut block_reader: XfsBlockReader =
            XfsBlockReader::new(&data_stream, 4, 8, &extents, 20);

        // Test reading across an in-file extent, a sparse extent, and a second in-file extent.
        let mut data: [u8; 16] = [0xff; 16];
        let read_count: usize = block_reader.read_data_from_blocks(&mut data, 4)?;
        assert_eq!(read_count, 16);
        assert_eq!(&data, b"AAAA\x00\x00\x00\x00\x00\x00\x00\x00CCCC");

        // Test stopping when current_offset reaches total size.
        let mut tail_data: [u8; 8] = [0xff; 8];
        let read_count: usize = block_reader.read_data_from_blocks(&mut tail_data, 16)?;
        assert_eq!(read_count, 8);
        assert_eq!(&tail_data, b"CCCCCCCC");

        let mut at_end: [u8; 4] = [0; 4];
        let read_count: usize = block_reader.read_data_from_blocks(&mut at_end, 20)?;
        assert_eq!(read_count, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_with_errors() {
        let data_stream: DataStreamReference = open_fake_data_stream(b"AAAAAAAA");
        let extents: [XfsPackedExtent; 1] = [XfsPackedExtent {
            number_of_blocks: 1,
            physical_block_number: 0,
            logical_block_number: 2,
            extent_type: XfsExtentType::InFile,
        }];
        let mut block_reader: XfsBlockReader =
            XfsBlockReader::new(&data_stream, 4, 8, &extents, 64);

        // Test with offset before first extent.
        let mut data: [u8; 4] = [0; 4];
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 0);
        assert!(result.is_err());

        // Test with offset after last extent.
        let result: Result<usize, ErrorTrace> = block_reader.read_data_from_blocks(&mut data, 32);
        assert!(result.is_err());

        // Test reading past the end of extents while still below logical size.
        let mut wide_data: [u8; 16] = [0; 16];
        let result: Result<usize, ErrorTrace> =
            block_reader.read_data_from_blocks(&mut wide_data, 16);
        assert!(result.is_err());
    }
}
