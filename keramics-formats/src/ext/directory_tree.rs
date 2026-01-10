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

use std::collections::BTreeMap;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::{ByteString, bytes_to_u32_le};

use super::block_range::{ExtBlockRange, ExtBlockRangeType};
use super::directory_entry::ExtDirectoryEntry;

/// Extended File System directory.
pub struct ExtDirectoryTree {
    /// Character encoding.
    encoding: CharacterEncoding,

    /// Block size.
    block_size: u32,
}

impl ExtDirectoryTree {
    /// Creates a new directory tree.
    pub fn new(encoding: &CharacterEncoding, block_size: u32) -> Self {
        Self {
            encoding: encoding.clone(),
            block_size,
        }
    }

    /// Reads the directory tree from block data.
    pub fn read_block_data(
        &mut self,
        data_stream: &DataStreamReference,
        block_ranges: &[ExtBlockRange],
        entries: &mut BTreeMap<ByteString, ExtDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        for block_range in block_ranges.iter() {
            if block_range.range_type != ExtBlockRangeType::InFile {
                break;
            }
            let mut block_offset: u64 =
                block_range.physical_block_number * (self.block_size as u64);

            for _ in 0..block_range.number_of_blocks as usize {
                match self.read_node_at_position(
                    data_stream,
                    SeekFrom::Start(block_offset),
                    entries,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read directory tree node at offset: {} (0x{:08x})",
                                block_offset, block_offset
                            )
                        );
                        return Err(error);
                    }
                }
                block_offset += self.block_size as u64;
            }
        }
        Ok(())
    }

    /// Reads the directory tree from inline data.
    pub fn read_inline_data(
        &mut self,
        data: &[u8],
        entries: &mut BTreeMap<ByteString, ExtDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        keramics_core::debug_trace_data!("ExtDirectoryTreeInline", 0, &data, data_size);

        let parent_inode_number: u32 = bytes_to_u32_le!(data, 0);

        keramics_core::debug_trace_structure!(format!(
            concat!(
                "ExtDirectoryTreeInline {{\n",
                "    parent_inode_number: {},\n",
                "}}\n\n"
            ),
            parent_inode_number
        ));

        self.read_node_data(&data, 4, data_size, entries)
    }

    /// Reads the directory tree node from a buffer.
    fn read_node_data(
        &mut self,
        data: &[u8],
        mut data_offset: usize,
        data_size: usize,
        entries: &mut BTreeMap<ByteString, ExtDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        while data_offset < data_size {
            keramics_core::debug_trace_structure!(ExtDirectoryEntry::debug_read_data(
                &data[data_offset..]
            ));

            let mut entry: ExtDirectoryEntry = ExtDirectoryEntry::new();

            match entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read directory entry");
                    return Err(error);
                }
            }
            if entry.size == 0 {
                break;
            }
            if entry.size < 8 || (entry.size as usize) > data_size - data_offset {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid directory entry size: {} value out of bounds",
                    entry.size
                )));
            }
            let name: ByteString = match entry.read_name(&data[data_offset..], &self.encoding) {
                Ok(name) => name,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read directory entry name"
                    );
                    return Err(error);
                }
            };
            data_offset += entry.size as usize;

            // TODO: print trailing data

            // Ignore inode number 0
            if entry.inode_number == 0 {
                continue;
            }
            // Ignore "." and ".."
            if name == "." || name == ".." {
                continue;
            }
            entries.insert(name, entry);
        }
        Ok(())
    }

    /// Reads the directory tree node from a specific position in a data stream.
    fn read_node_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
        entries: &mut BTreeMap<ByteString, ExtDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; self.block_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("ExtDirectoryTreeNode", offset, &data, self.block_size);

        self.read_node_data(&data, 0, self.block_size as usize, entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x02, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x01, 0x02, 0x2e, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x0c, 0x00, 0x02, 0x02, 0x2e, 0x2e, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x0a, 0x02, 0x6c, 0x6f, 0x73, 0x74, 0x2b, 0x66, 0x6f, 0x75, 0x6e, 0x64,
            0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x14, 0x00, 0x09, 0x01, 0x65, 0x6d, 0x70, 0x74,
            0x79, 0x66, 0x69, 0x6c, 0x65, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x10, 0x00,
            0x08, 0x02, 0x74, 0x65, 0x73, 0x74, 0x64, 0x69, 0x72, 0x31, 0x0e, 0x00, 0x00, 0x00,
            0x18, 0x00, 0x0e, 0x01, 0x66, 0x69, 0x6c, 0x65, 0x5f, 0x68, 0x61, 0x72, 0x64, 0x6c,
            0x69, 0x6e, 0x6b, 0x31, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x12, 0x07,
            0x66, 0x69, 0x6c, 0x65, 0x5f, 0x73, 0x79, 0x6d, 0x62, 0x6f, 0x6c, 0x69, 0x63, 0x6c,
            0x69, 0x6e, 0x6b, 0x31, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x20, 0x00, 0x17, 0x07,
            0x64, 0x69, 0x72, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x79, 0x5f, 0x73, 0x79, 0x6d, 0x62,
            0x6f, 0x6c, 0x69, 0x63, 0x6c, 0x69, 0x6e, 0x6b, 0x31, 0x00, 0x12, 0x00, 0x00, 0x00,
            0x18, 0x00, 0x0e, 0x01, 0x6e, 0x66, 0x63, 0x5f, 0x74, 0xc3, 0xa9, 0x73, 0x74, 0x66,
            0x69, 0x6c, 0xc3, 0xa8, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x18, 0x00, 0x10, 0x01,
            0x6e, 0x66, 0x64, 0x5f, 0x74, 0x65, 0xcc, 0x81, 0x73, 0x74, 0x66, 0x69, 0x6c, 0x65,
            0xcc, 0x80, 0x14, 0x00, 0x00, 0x00, 0x10, 0x00, 0x06, 0x01, 0x6e, 0x66, 0x64, 0x5f,
            0xc2, 0xbe, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x0a, 0x01, 0x6e, 0x66,
            0x6b, 0x64, 0x5f, 0x33, 0xe2, 0x81, 0x84, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_block_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = ExtDirectoryTree::new(&CharacterEncoding::Utf8, 256);

        let block_ranges: Vec<ExtBlockRange> = vec![ExtBlockRange {
            logical_block_number: 0,
            physical_block_number: 0,
            number_of_blocks: 1,
            range_type: ExtBlockRangeType::InFile,
        }];
        let mut entries: BTreeMap<ByteString, ExtDirectoryEntry> = BTreeMap::new();
        test_struct.read_block_data(&data_stream, &block_ranges, &mut entries)?;

        assert_eq!(entries.len(), 10);

        Ok(())
    }

    #[test]
    fn test_read_inline_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x38, 0x00, 0x09, 0x01, 0x74, 0x65,
            0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let mut test_struct = ExtDirectoryTree::new(&CharacterEncoding::Utf8, 256);

        let mut entries: BTreeMap<ByteString, ExtDirectoryEntry> = BTreeMap::new();
        test_struct.read_inline_data(&test_data, &mut entries)?;

        assert_eq!(entries.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_node_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtDirectoryTree::new(&CharacterEncoding::Utf8, 256);

        let mut entries: BTreeMap<ByteString, ExtDirectoryEntry> = BTreeMap::new();
        test_struct.read_node_data(&test_data, 0, 256, &mut entries)?;

        assert_eq!(entries.len(), 10);

        Ok(())
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = ExtDirectoryTree::new(&CharacterEncoding::Utf8, 256);

        let mut entries: BTreeMap<ByteString, ExtDirectoryEntry> = BTreeMap::new();
        test_struct.read_node_at_position(&data_stream, SeekFrom::Start(0), &mut entries)?;

        assert_eq!(entries.len(), 10);

        Ok(())
    }
}
