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

use crate::path_component::PathComponent;

use super::directory_entry::HfsDirectoryEntry;
use super::string::HfsString;

/// Hierarchical File System (HFS) directory entries.
pub struct HfsDirectoryEntries {
    /// Entries.
    entries: BTreeMap<HfsString, HfsDirectoryEntry>,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl HfsDirectoryEntries {
    /// Creates new directory entries.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(
        &self,
        entry_index: usize,
    ) -> Option<(&HfsString, &HfsDirectoryEntry)> {
        self.entries.iter().nth(entry_index)
    }

    /// Retrieves a specific directory entry by name.
    pub fn get_entry_by_name(
        &self,
        name: &PathComponent,
    ) -> Result<Option<(&HfsString, &HfsDirectoryEntry)>, ErrorTrace> {
        let lookup_name: HfsString = match name.to_utf16_string() {
            Ok(utf16_string) => HfsString::Utf16String(utf16_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to convert path component to UTF-16 string"
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
        name: HfsString,
        entry: HfsDirectoryEntry,
    ) -> Option<HfsDirectoryEntry> {
        self.entries.insert(name, entry)
    }

    /// Determines if the directory entries were read.
    pub fn is_read(&self) -> bool {
        return self.is_read;
    }
}
