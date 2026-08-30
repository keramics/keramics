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

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterEncoding;
use keramics_types::{ByteString, bytes_to_u32_be, bytes_to_u64_be};

use crate::indexed_hash_map::IndexedHashMap;

use super::directory_entry::XfsDirectoryEntry;
use super::directory_table_entry_v1::XfsDirectoryTableEntryV1;
use super::directory_table_entry_v2::XfsDirectoryTableEntryV2;
use super::directory_table_header_v1::XfsDirectoryTableHeaderV1;
use super::directory_table_header_v2_32bit::XfsDirectoryTableHeader32bitV2;
use super::directory_table_header_v2_64bit::XfsDirectoryTableHeader64bitV2;

/// X File System (XFS) directory table.
pub struct XfsDirectoryTable {
    /// Character encoding.
    character_encoding: CharacterEncoding,
}

impl XfsDirectoryTable {
    /// Creates a new directory table.
    pub fn new(character_encoding: &CharacterEncoding) -> Self {
        Self {
            character_encoding: character_encoding.clone(),
        }
    }

    /// Reads the directory table from a buffer.
    pub fn read_data(
        &mut self,
        has_directory_v2: bool,
        has_file_type: bool,
        data: &[u8],
        entries: &mut IndexedHashMap<ByteString, XfsDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        keramics_core::debug_trace_data!("XfsDirectoryTable", 0, data, data_size);

        if data_size < 2 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let number_of_entries_32bit: u8 = if has_directory_v2 { data[0] } else { 0 };
        let number_of_entries_64bit: u8 = if has_directory_v2 { data[1] } else { 0 };

        if number_of_entries_32bit != 0 && number_of_entries_64bit != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported header"));
        }
        let is_64bit: bool = number_of_entries_64bit != 0;

        let header_size: usize = if !has_directory_v2 {
            9
        } else if is_64bit {
            10
        } else {
            6
        };
        if header_size > data_size {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let parent_inode_number: u64;
        let number_of_entries: u8;

        if !has_directory_v2 {
            keramics_core::debug_trace_structure!(XfsDirectoryTableHeaderV1::debug_read_data(
                &data[0..header_size]
            ));
            let mut header: XfsDirectoryTableHeaderV1 = XfsDirectoryTableHeaderV1::new();

            match header.read_data(&data[0..header_size]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read version 1 header");
                    return Err(error);
                }
            }
            parent_inode_number = header.parent_inode_number;
            number_of_entries = header.number_of_entries;
        } else if is_64bit {
            keramics_core::debug_trace_structure!(XfsDirectoryTableHeader64bitV2::debug_read_data(
                &data[0..header_size]
            ));
            let mut header: XfsDirectoryTableHeader64bitV2 = XfsDirectoryTableHeader64bitV2::new();

            match header.read_data(&data[0..header_size]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read 64-bit version 2 header"
                    );
                    return Err(error);
                }
            }
            parent_inode_number = header.parent_inode_number;
            number_of_entries = number_of_entries_64bit;
        } else {
            keramics_core::debug_trace_structure!(XfsDirectoryTableHeader32bitV2::debug_read_data(
                &data[0..header_size]
            ));
            let mut header: XfsDirectoryTableHeader32bitV2 = XfsDirectoryTableHeader32bitV2::new();

            match header.read_data(&data[0..header_size]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read 32-bit version 2 header"
                    );
                    return Err(error);
                }
            }
            parent_inode_number = header.parent_inode_number as u64;
            number_of_entries = number_of_entries_32bit;
        }
        let mut data_offset: usize = header_size;

        for entry_index in 0..number_of_entries {
            let name_size_offset: usize = data_offset + if !has_directory_v2 { 8 } else { 0 };

            if name_size_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - name offset value out of bounds",
                    entry_index
                )));
            }
            let name_size: usize = data[name_size_offset] as usize;
            let name_offset: usize = name_size_offset + if !has_directory_v2 { 1 } else { 3 };
            let name_end_offset: usize = name_offset + name_size;

            let entry_size: usize = name_size
                + if !has_directory_v2 {
                    9
                } else {
                    3 + if has_file_type { 1 } else { 0 } + if is_64bit { 8 } else { 4 }
                };
            let data_end_offset: usize = data_offset + entry_size;

            if data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} size value out of bounds",
                    entry_index
                )));
            }
            if !has_directory_v2 {
                keramics_core::debug_trace_structure!(XfsDirectoryTableEntryV1::debug_read_data(
                    &data[data_offset..data_end_offset]
                ));
                // TODO: debug trace name
            } else {
                keramics_core::debug_trace_structure!(XfsDirectoryTableEntryV2::debug_read_data(
                    &data[data_offset..data_end_offset]
                ));
                // TODO: debug trace name and file type
            }
            let inode_number: u64 = if !has_directory_v2 {
                bytes_to_u64_be!(data, data_offset)
            } else if is_64bit {
                let inode_offset: usize = name_end_offset + if has_file_type { 1 } else { 0 };
                bytes_to_u64_be!(data, inode_offset)
            } else {
                let inode_offset: usize = name_end_offset + if has_file_type { 1 } else { 0 };
                bytes_to_u32_be!(data, inode_offset) as u64
            };
            let mut name: ByteString = ByteString::new_with_encoding(&self.character_encoding);
            name.read_data(&data[name_offset..name_end_offset]);

            entries.insert(
                name,
                XfsDirectoryEntry::new(inode_number, parent_inode_number),
            );
            data_offset = data_end_offset;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x09, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x09, 0x00, 0x60, 0x65, 0x6d, 0x70, 0x74, 0x79,
            0x66, 0x69, 0x6c, 0x65, 0x01, 0x00, 0x00, 0x3f, 0x03, 0x08, 0x00, 0x78, 0x74, 0x65,
            0x73, 0x74, 0x64, 0x69, 0x72, 0x31, 0x02, 0x00, 0x00, 0x3f, 0x04, 0x0e, 0x00, 0x90,
            0x66, 0x69, 0x6c, 0x65, 0x5f, 0x68, 0x61, 0x72, 0x64, 0x6c, 0x69, 0x6e, 0x6b, 0x31,
            0x01, 0x00, 0x00, 0x3f, 0x05, 0x12, 0x00, 0xb0, 0x66, 0x69, 0x6c, 0x65, 0x5f, 0x73,
            0x79, 0x6d, 0x62, 0x6f, 0x6c, 0x69, 0x63, 0x6c, 0x69, 0x6e, 0x6b, 0x31, 0x07, 0x00,
            0x00, 0x3f, 0x07, 0x17, 0x00, 0xd0, 0x64, 0x69, 0x72, 0x65, 0x63, 0x74, 0x6f, 0x72,
            0x79, 0x5f, 0x73, 0x79, 0x6d, 0x62, 0x6f, 0x6c, 0x69, 0x63, 0x6c, 0x69, 0x6e, 0x6b,
            0x31, 0x07, 0x00, 0x00, 0x3f, 0x08, 0x0e, 0x00, 0xf8, 0x6e, 0x66, 0x63, 0x5f, 0x74,
            0xc3, 0xa9, 0x73, 0x74, 0x66, 0x69, 0x6c, 0xc3, 0xa8, 0x01, 0x00, 0x00, 0x3f, 0x09,
            0x10, 0x01, 0x18, 0x6e, 0x66, 0x64, 0x5f, 0x74, 0x65, 0xcc, 0x81, 0x73, 0x74, 0x66,
            0x69, 0x6c, 0x65, 0xcc, 0x80, 0x01, 0x00, 0x00, 0x3f, 0x0a, 0x06, 0x01, 0x38, 0x6e,
            0x66, 0x64, 0x5f, 0xc2, 0xbe, 0x01, 0x00, 0x00, 0x3f, 0x0b, 0x0a, 0x01, 0x50, 0x6e,
            0x66, 0x6b, 0x64, 0x5f, 0x33, 0xe2, 0x81, 0x84, 0x34, 0x01, 0x00, 0x00, 0x3f, 0x0c,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsDirectoryTable::new(&CharacterEncoding::Utf8);
        let mut entries: IndexedHashMap<ByteString, XfsDirectoryEntry> = IndexedHashMap::new();
        test_struct.read_data(true, true, &test_data, &mut entries)?;

        assert_eq!(entries.len(), 9);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = XfsDirectoryTable::new(&CharacterEncoding::Utf8);

        let test_data: Vec<u8> = get_test_data();
        let mut entries: IndexedHashMap<ByteString, XfsDirectoryEntry> = IndexedHashMap::new();
        let result = test_struct.read_data(true, true, &test_data[0..15], &mut entries);
        assert!(result.is_err());
    }
}
