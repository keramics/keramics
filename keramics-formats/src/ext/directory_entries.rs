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

use std::collections::BTreeMap;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::path_component::PathComponent;

use super::block_range::ExtBlockRange;
use super::directory_entry::ExtDirectoryEntry;
use super::directory_tree::ExtDirectoryTree;

/// Extended File System (ext) directory entries.
pub struct ExtDirectoryEntries {
    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Entries.
    entries: BTreeMap<ByteString, ExtDirectoryEntry>,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl ExtDirectoryEntries {
    /// Creates new directory entries.
    pub fn new(encoding: &CharacterEncoding) -> Self {
        Self {
            encoding: encoding.clone(),
            entries: BTreeMap::new(),
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(
        &self,
        entry_index: usize,
    ) -> Option<(&ByteString, &ExtDirectoryEntry)> {
        self.entries.iter().nth(entry_index)
    }

    /// Retrieves a specific directory entry by name.
    pub fn get_entry_by_name(
        &self,
        name: &PathComponent,
    ) -> Result<Option<(&ByteString, &ExtDirectoryEntry)>, ErrorTrace> {
        let lookup_name: ByteString = match name.to_byte_string(&self.encoding) {
            Ok(byte_string) => byte_string,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to convert path component to byte string"
                );
                return Err(error);
            }
        };
        Ok(self.entries.get_key_value(&lookup_name))
    }

    /// Retrieves the number of entries.
    pub fn get_number_of_entries(&self) -> usize {
        self.entries.len()
    }

    /// Determines if the directory entries were read.
    pub fn is_read(&self) -> bool {
        return self.is_read;
    }

    /// Reads the directory entries from block data.
    pub fn read_block_data(
        &mut self,
        data_stream: &DataStreamReference,
        block_size: u32,
        block_ranges: &Vec<ExtBlockRange>,
    ) -> Result<(), ErrorTrace> {
        let mut directory_tree: ExtDirectoryTree =
            ExtDirectoryTree::new(&self.encoding, block_size);

        match directory_tree.read_block_data(data_stream, block_ranges, &mut self.entries) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read directory tree from block data"
                );
                return Err(error);
            }
        }
        self.is_read = true;

        Ok(())
    }

    /// Reads the directory entries from inline data.
    pub fn read_inline_data(&mut self, data: &[u8], block_size: u32) -> Result<(), ErrorTrace> {
        let mut directory_tree: ExtDirectoryTree =
            ExtDirectoryTree::new(&self.encoding, block_size);

        match directory_tree.read_inline_data(data, &mut self.entries) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read directory tree from inline data"
                );
                return Err(error);
            }
        }
        self.is_read = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::ext::block_range::ExtBlockRangeType;

    fn get_test_data_inline() -> Vec<u8> {
        vec![
            0x02, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x38, 0x00, 0x09, 0x01, 0x74, 0x65,
            0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    fn get_directory_entries() -> Result<ExtDirectoryEntries, ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_inline();

        let mut directory_entries: ExtDirectoryEntries =
            ExtDirectoryEntries::new(&CharacterEncoding::Utf8);
        directory_entries.read_inline_data(&test_data, 256)?;

        Ok(directory_entries)
    }

    #[test]
    fn test_get_entry_by_index() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        let entry: Option<(&ByteString, &ExtDirectoryEntry)> = test_struct.get_entry_by_index(0);
        assert!(entry.is_some());

        let entry: Option<(&ByteString, &ExtDirectoryEntry)> = test_struct.get_entry_by_index(99);
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_entry_by_name() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        let name: PathComponent = PathComponent::ByteString(ByteString::from("testfile1"));
        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_entry_by_name(&name)?;
        assert!(entry.is_some());

        let name: PathComponent = PathComponent::ByteString(ByteString::from("bogus"));
        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_entry_by_name(&name)?;
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_entries() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        assert_eq!(test_struct.get_number_of_entries(), 1);

        Ok(())
    }

    #[test]
    fn test_read_block_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
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

        let mut test_struct: ExtDirectoryEntries =
            ExtDirectoryEntries::new(&CharacterEncoding::Utf8);

        assert_eq!(test_struct.entries.len(), 0);
        assert_eq!(test_struct.is_read, false);

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let block_ranges: &Vec<ExtBlockRange> = &vec![ExtBlockRange {
            logical_block_number: 0,
            physical_block_number: 0,
            number_of_blocks: 1,
            range_type: ExtBlockRangeType::InFile,
        }];
        test_struct.read_block_data(&data_stream, 256, &block_ranges)?;

        assert_eq!(test_struct.entries.len(), 10);
        assert_eq!(test_struct.is_read, true);

        Ok(())
    }

    #[test]
    fn test_read_inline_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_inline();

        let mut test_struct: ExtDirectoryEntries =
            ExtDirectoryEntries::new(&CharacterEncoding::Utf8);

        assert_eq!(test_struct.entries.len(), 0);
        assert_eq!(test_struct.is_read, false);

        test_struct.read_inline_data(&test_data, 256)?;

        assert_eq!(test_struct.entries.len(), 1);
        assert_eq!(test_struct.is_read, true);

        Ok(())
    }
}
