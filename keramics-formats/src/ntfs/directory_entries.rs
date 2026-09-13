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

use std::collections::HashMap;

use keramics_core::ErrorTrace;

use super::constants::*;
use super::directory_entry::NtfsDirectoryEntry;
use super::file_name::NtfsFileName;

/// New Technologies File System (NTFS) directory entries.
pub struct NtfsDirectoryEntries {
    // TODO: consider storing the file_name and file_reference and create directory_entry on
    // demand.
    /// Entries.
    entries: Vec<NtfsDirectoryEntry>,

    /// $FILE_NAME with DOS name space.
    pub short_names: HashMap<u64, NtfsFileName>,
}

impl NtfsDirectoryEntries {
    /// Creates a new path.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            short_names: HashMap::new(),
        }
    }

    /// Adds an entry.
    pub fn add(&mut self, file_reference: u64, file_name: NtfsFileName) -> Result<(), ErrorTrace> {
        // TODO: preserve self $FILE_NAME entry.
        if file_name.name_size == 1 && file_name.name.elements[0] == 0x002e {
            // Ignore "."
        } else if file_name.name_space == NTFS_NAME_SPACE_DOS {
            if self.short_names.contains_key(&file_reference) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid directory entry: {}-{} - DOS file name already set",
                    file_reference & 0x0000ffffffffffff,
                    file_reference >> 48,
                )));
            }
            self.short_names.insert(file_reference, file_name);
        } else {
            let directory_entry: NtfsDirectoryEntry =
                NtfsDirectoryEntry::new(file_reference, file_name);
            self.entries.push(directory_entry);
        }
        Ok(())
    }

    /// Determines the number of entries.
    pub fn get_number_of_entries(&self) -> usize {
        self.entries.len()
    }

    /// Retrieves a specific entry.
    pub fn get_entry_by_index(&mut self, index: usize) -> Result<&NtfsDirectoryEntry, ErrorTrace> {
        match self.entries.get_mut(index) {
            Some(directory_entry) => {
                if directory_entry.file_name.name_space == NTFS_NAME_SPACE_WINDOWS
                    && directory_entry.short_file_name.is_some()
                {
                    if let Some(short_name) = self.short_names.get(&directory_entry.file_reference)
                    {
                        // TODO: consider removing the short name.
                        directory_entry.short_file_name = Some(short_name.clone());
                    }
                }
                Ok(directory_entry)
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing directory entry: {}",
                index
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_file_name(name: &str, name_space: u8) -> Result<NtfsFileName, ErrorTrace> {
        let name_data: Vec<u8> = name
            .encode_utf16()
            .flat_map(|value| value.to_le_bytes())
            .collect();

        let mut test_data: Vec<u8> = vec![0; 66];
        test_data[64] = (name_data.len() / 2) as u8;
        test_data[65] = name_space;
        test_data.extend_from_slice(&name_data);

        let mut file_name: NtfsFileName = NtfsFileName::new();
        file_name.read_data(&test_data)?;

        Ok(file_name)
    }

    #[test]
    fn test_add() -> Result<(), ErrorTrace> {
        let mut directory_entries: NtfsDirectoryEntries = NtfsDirectoryEntries::new();

        let file_name: NtfsFileName = get_file_name("TestFile1", NTFS_NAME_SPACE_WINDOWS)?;
        directory_entries.add(0x0001000000000001, file_name)?;

        let file_name: NtfsFileName = get_file_name("TESTF1~1", NTFS_NAME_SPACE_DOS)?;
        directory_entries.add(0x0001000000000002, file_name)?;

        let file_name: NtfsFileName = get_file_name(".", NTFS_NAME_SPACE_WINDOWS)?;
        directory_entries.add(0x0001000000000003, file_name)?;

        assert_eq!(directory_entries.get_number_of_entries(), 1);
        assert_eq!(directory_entries.short_names.len(), 1);

        Ok(())
    }

    #[test]
    fn test_add_with_duplicate_short_name() -> Result<(), ErrorTrace> {
        let mut directory_entries: NtfsDirectoryEntries = NtfsDirectoryEntries::new();

        let file_name: NtfsFileName = get_file_name("TESTF1~1", NTFS_NAME_SPACE_DOS)?;
        let result: Result<(), ErrorTrace> = directory_entries.add(0x0001000000000001, file_name);
        assert!(result.is_ok());

        let file_name: NtfsFileName = get_file_name("TESTF1~2", NTFS_NAME_SPACE_DOS)?;
        let result: Result<(), ErrorTrace> = directory_entries.add(0x0001000000000001, file_name);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_entry_by_index() -> Result<(), ErrorTrace> {
        let mut directory_entries: NtfsDirectoryEntries = NtfsDirectoryEntries::new();

        let file_name: NtfsFileName = get_file_name("TestFile1", NTFS_NAME_SPACE_WINDOWS)?;
        directory_entries.add(0x0001000000000001, file_name)?;

        let file_name: NtfsFileName = get_file_name("TESTF1~1", NTFS_NAME_SPACE_DOS)?;
        directory_entries.add(0x0001000000000001, file_name)?;

        let directory_entry: &NtfsDirectoryEntry = directory_entries.get_entry_by_index(0)?;
        assert_eq!(directory_entry.file_name.name.to_string(), "TestFile1");

        let result: Result<&NtfsDirectoryEntry, ErrorTrace> =
            directory_entries.get_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }
}
