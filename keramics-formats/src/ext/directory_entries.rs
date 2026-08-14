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

use keramics_types::ByteString;

use crate::types::IndexedHashMap;

use super::directory_entry::ExtDirectoryEntry;

/// Extended File System (ext) directory entries.
pub type ExtDirectoryEntries = IndexedHashMap<ByteString, ExtDirectoryEntry>;

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::ErrorTrace;
    use keramics_encodings::CharacterEncoding;

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

        let mut directory_entries: ExtDirectoryEntries = ExtDirectoryEntries::new();
        directory_tree.read_inline_data(&test_data, &mut directory_entries)?;

        Ok(directory_entries)
    }

    #[test]
    fn test_get_key_value_by_index() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_key_value_by_index(0);
        assert!(entry.is_some());

        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_key_value_by_index(99);
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_key_value_by_key() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        let name: ByteString = ByteString::from("testfile1");
        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_key_value_by_key(&name);
        assert!(entry.is_some());

        let name: ByteString = ByteString::from("bogus");
        let entry: Option<(&ByteString, &ExtDirectoryEntry)> =
            test_struct.get_key_value_by_key(&name);
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_len() -> Result<(), ErrorTrace> {
        let test_struct: ExtDirectoryEntries = get_directory_entries()?;

        assert_eq!(test_struct.len(), 1);

        Ok(())
    }
}
