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

use keramics_core::ErrorTrace;

use super::file_entry::ExtFileEntry;

/// Extended File System (ext) file entries iterator.
pub struct ExtFileEntriesIterator<'a> {
    /// File entry.
    file_entry: &'a mut ExtFileEntry,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,

    /// Value to indicate whether the iterator is initialized.
    is_initialized: bool,
}

impl<'a> ExtFileEntriesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut ExtFileEntry) -> Self {
        Self {
            file_entry,
            number_of_sub_file_entries: 0,
            sub_file_entry_index: 0,
            is_initialized: false,
        }
    }
}

impl<'a> Iterator for ExtFileEntriesIterator<'a> {
    type Item = Result<ExtFileEntry, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_initialized {
            match self.file_entry.get_number_of_sub_file_entries() {
                Ok(number_of_sub_file_entries) => {
                    self.number_of_sub_file_entries = number_of_sub_file_entries;
                }
                Err(error) => return Some(Err(error)),
            }
            self.is_initialized = true;
        }
        if self.sub_file_entry_index >= self.number_of_sub_file_entries {
            return None;
        }
        let item: Self::Item = self
            .file_entry
            .get_sub_file_entry_by_index(self.sub_file_entry_index);

        self.sub_file_entry_index += 1;

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
