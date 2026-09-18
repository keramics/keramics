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

use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::range_descriptor::VolsnapRangeDescriptor;
use super::store_block::VolsnapStoreBlock;

/// Volume Shadow Snapshot (volsnap) store range list.
pub struct VolsnapStoreRangeList {
    /// Range descriptors.
    range_descriptors: Vec<VolsnapRangeDescriptor>,
}

impl VolsnapStoreRangeList {
    /// Creates a new range list.
    pub fn new() -> Self {
        Self {
            range_descriptors: Vec::new(),
        }
    }

    /// Reads the range list from a specific position in a data stream.
    pub fn read_at_offset(
        &mut self,
        data_stream: &DataStreamReference,
        mut offset: u64,
    ) -> Result<(), ErrorTrace> {
        let mut read_store_blocks: HashSet<u64> = HashSet::new();

        loop {
            if read_store_blocks.contains(&offset) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Store block at offset: {} (0x{:08x}) already read",
                    offset, offset
                )));
            }
            let mut store_block: VolsnapStoreBlock = VolsnapStoreBlock::new();

            match store_block.read_at_position(data_stream, SeekFrom::Start(offset)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read store range list block at offset: {} (0x{:08x})",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            }
            read_store_blocks.insert(offset);

            if store_block.block_type != 5 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported store range list block - unsupported block type",
                ));
            }
            if store_block.current_block_offset != offset {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported store range list block - current block offset value out of bounds",
                ));
            }
            // Ignore the last 8 bytes to align the range descriptor size with the block size.
            let data_size: usize = store_block.data.len() - 8;

            for data_offset in (128..data_size).step_by(24) {
                let data_end_offset: usize = data_offset + 24;

                if store_block.data[data_offset..data_end_offset] == [0; 24] {
                    break;
                }
                keramics_core::debug_trace_data_and_structure!(
                    "VolsnapRangeDescriptor",
                    offset + (data_offset as u64),
                    &store_block.data[data_offset..data_end_offset],
                    24,
                    VolsnapRangeDescriptor::debug_read_data(&store_block.data[data_offset..])
                );
                let mut range_descriptor: VolsnapRangeDescriptor = VolsnapRangeDescriptor::new();

                match range_descriptor.read_data(&store_block.data[data_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read range descriptor"
                        );
                        return Err(error);
                    }
                }
                self.range_descriptors.push(range_descriptor);
            }
            if store_block.next_block_offset == 0 {
                break;
            }
            if !store_block.next_block_offset.is_multiple_of(16384) {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported store range list block - next block offset value not a multiple of 16384",
                ));
            }
            offset = store_block.next_block_offset;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream_with_offset;

    fn get_test_data() -> Vec<u8> {
        let mut test_data: Vec<u8> = vec![0; 16384];
        test_data[0..256].copy_from_slice(&[
            0x6b, 0x87, 0x08, 0x38, 0x76, 0xc1, 0x48, 0x4e, 0xb7, 0xae, 0x04, 0x04, 0x6e, 0x6c,
            0xc7, 0x52, 0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xa9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xc0, 0xa9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        test_data
    }

    #[test]
    fn test_read_at_offset() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference =
            open_fake_data_stream_with_offset(&test_data, 0x04a98000);

        let mut test_struct = VolsnapStoreRangeList::new();
        test_struct.read_at_offset(&data_stream, 0x04a98000)?;

        assert_eq!(test_struct.range_descriptors.len(), 1);

        Ok(())
    }
}
