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

use std::sync::Arc;

use keramics_types::{Ucs2CharacterMappings, Ucs2String};

use super::long_name_directory_entry::FatLongNameDirectoryEntry;
use super::short_name_directory_entry::FatShortNameDirectoryEntry;
use super::string::FatString;

/// File Allocation Table (FAT) directory entry.
#[derive(Clone)]
pub struct FatDirectoryEntry {
    /// Identifier.
    pub identifier: u32,

    /// Short name.
    pub short_name: FatShortNameDirectoryEntry,

    /// Long name
    pub long_name: Option<Ucs2String>,
}

impl FatDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(identifier: u32, short_name: FatShortNameDirectoryEntry) -> Self {
        Self {
            identifier,
            short_name,
            long_name: None,
        }
    }

    /// Retrieves the lookup name.
    pub fn get_lookup_name(
        &self,
        case_folding_mappings: &Arc<Ucs2CharacterMappings>,
    ) -> Ucs2String {
        match &self.long_name {
            Some(long_name) => long_name.new_with_case_folding(case_folding_mappings),
            None => self.short_name.get_lookup_name(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<FatString> {
        match self.long_name.as_ref() {
            Some(long_name) => Some(FatString::Ucs2String(long_name.clone())),
            None => Some(FatString::ByteString(self.short_name.name.clone())),
        }
    }

    /// Fills the directory entry based on a long name directory entries.
    pub fn set_long_name(&mut self, long_name_entries: &mut Vec<FatLongNameDirectoryEntry>) {
        if !long_name_entries.is_empty() {
            let mut long_name: Ucs2String = Ucs2String::new();

            for long_name_entry in long_name_entries.iter_mut().rev() {
                long_name.append(&mut long_name_entry.name);
            }
            long_name_entries.clear();

            self.long_name = Some(long_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::ErrorTrace;
    use keramics_encodings::CharacterEncoding;
    use keramics_types::ByteString;

    use crate::fat::short_name_directory_entry_fat12::Fat12ShortNameDirectoryEntry;

    fn get_test_data_fat12() -> Vec<u8> {
        vec![
            0x54, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10, 0x00, 0x7d,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    // TODO: add tests for get_lookup_name

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_fat12();

        let mut short_name: FatShortNameDirectoryEntry = FatShortNameDirectoryEntry::new();
        Fat12ShortNameDirectoryEntry::read_data(&mut short_name, &test_data)?;

        let test_struct: FatDirectoryEntry = FatDirectoryEntry::new(0x00001a80, short_name);

        let name: Option<FatString> = test_struct.get_name();
        assert_eq!(
            name,
            Some(FatString::ByteString(ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: b"TESTDIR1".to_vec(),
            }))
        );

        Ok(())
    }

    // TODO: add tests for set_long_name
}
