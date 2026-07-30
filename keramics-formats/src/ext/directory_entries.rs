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

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::path_component::PathComponent;

use super::directory_entry::ExtDirectoryEntry;

/// Extended File System (ext) directory entries.
pub struct ExtDirectoryEntries {
    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Entries.
    entries: BTreeMap<ByteString, ExtDirectoryEntry>,

    /// Names in order of insert.
    names: Vec<ByteString>,

    /// Value to indicate the directory entries were read.
    pub is_read: bool,
}

impl ExtDirectoryEntries {
    /// Creates new directory entries.
    pub fn new(encoding: &CharacterEncoding) -> Self {
        Self {
            encoding: encoding.clone(),
            entries: BTreeMap::new(),
            names: Vec::new(),
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(
        &self,
        entry_index: usize,
    ) -> Option<(&ByteString, &ExtDirectoryEntry)> {
        match self.names.get(entry_index) {
            Some(name) => self.entries.get_key_value(name),
            None => None,
        }
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

    /// Inserts a directory entry.
    pub fn insert_entry(
        &mut self,
        name: ByteString,
        entry: ExtDirectoryEntry,
    ) -> Option<ExtDirectoryEntry> {
        if !self.entries.contains_key(&name) {
            self.names.push(name.clone());
        }
        self.entries.insert(name, entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::ext::directory_tree::ExtDirectoryTree;

    fn get_directory_entries() -> Result<ExtDirectoryEntries, ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x02, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00, 0x38, 0x00, 0x09, 0x01, 0x74, 0x65,
            0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let mut directory_tree: ExtDirectoryTree =
            ExtDirectoryTree::new(&CharacterEncoding::Utf8, 256);

        let mut directory_entries: ExtDirectoryEntries =
            ExtDirectoryEntries::new(&CharacterEncoding::Utf8);
        directory_tree.read_inline_data(&test_data, &mut directory_entries)?;
        directory_entries.is_read = true;

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
}
