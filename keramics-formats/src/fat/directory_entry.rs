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

use std::collections::HashMap;
use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_types::Ucs2String;

use super::long_name_directory_entry::FatLongNameDirectoryEntry;
use super::short_name_directory_entry::FatShortNameDirectoryEntry;

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
            identifier: identifier,
            short_name: short_name,
            long_name: None,
        }
    }

    /// Retrieves the lookup name.
    pub fn get_lookup_name(&self, case_folding_mappings: &Arc<HashMap<u16, u16>>) -> Ucs2String {
        match &self.long_name {
            Some(long_name) => Ucs2String::new_with_case_folding(long_name, case_folding_mappings),
            None => Ucs2String {
                elements: self
                    .short_name
                    .name
                    .elements
                    .iter()
                    .map(|element| {
                        if *element >= b'a' && *element <= b'z' {
                            (*element - 32) as u16
                        } else {
                            *element as u16
                        }
                    })
                    .collect::<Vec<u16>>(),
            },
        }
    }

    /// Fills the directory entry based on a long name directory entries.
    pub fn set_long_name(&mut self, long_name_entries: &mut Vec<FatLongNameDirectoryEntry>) {
        if !long_name_entries.is_empty() {
            let mut long_name: Ucs2String = Ucs2String::new();

            for long_name_entry in long_name_entries.iter_mut().rev() {
                long_name
                    .elements
                    .append(&mut long_name_entry.name.elements);
            }
            long_name_entries.clear();

            self.long_name = Some(long_name);
        }
    }
}
