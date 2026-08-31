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

use super::file_entry_record::ExFatFileEntryRecord;

/// Extensible File Allocation Table (exFAT) directory entry.
#[derive(Clone)]
pub struct ExFatDirectoryEntry {
    /// Identifier.
    pub identifier: u64,

    /// File entry record.
    pub file_entry_record: ExFatFileEntryRecord,

    /// Name.
    pub name: Ucs2String,

    /// Valid data size.
    pub valid_data_size: u64,

    /// Data start cluster.
    pub data_start_cluster: u32,

    /// Data size.
    pub data_size: u64,

    /// Value to indicate the data stream no-FAT-chain flag was set.
    pub data_stream_no_fat_chain: bool,
}

impl ExFatDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(identifier: u64) -> Self {
        Self {
            identifier,
            file_entry_record: ExFatFileEntryRecord::new(),
            name: Ucs2String::new(),
            valid_data_size: 0,
            data_start_cluster: 0,
            data_size: 0,
            data_stream_no_fat_chain: false,
        }
    }

    /// Retrieves the lookup name.
    pub fn get_lookup_name(
        &self,
        case_folding_mappings: &Arc<Ucs2CharacterMappings>,
    ) -> Ucs2String {
        self.name.new_with_case_folding(case_folding_mappings)
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &Ucs2String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    #[test]
    fn test_get_lookup_name() {
        let mut test_struct = ExFatDirectoryEntry::new(7);
        test_struct.name = Ucs2String::from("TeSt");

        let mappings: Arc<Ucs2CharacterMappings> =
            Arc::new(Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice()));

        let lookup_name: Ucs2String = test_struct.get_lookup_name(&mappings);
        assert_eq!(lookup_name, Ucs2String::from("TEST"));
    }

    #[test]
    fn test_get_name() {
        let mut test_struct = ExFatDirectoryEntry::new(7);
        test_struct.name = Ucs2String::from("TeSt");

        let name: &Ucs2String = test_struct.get_name();
        assert_eq!(name, &Ucs2String::from("TeSt"));
    }
}
