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

use super::extended_attribute::ExtExtendedAttribute;
use super::file_entry::ExtFileEntry;

/// Extended File System (ext) extended attributes iterator.
pub struct ExtExtendedAttributesIterator<'a> {
    /// File entry.
    file_entry: &'a mut ExtFileEntry,

    /// Number of extended attributes.
    number_of_extended_attributes: usize,

    /// Partititon index.
    extended_attribute_index: usize,

    /// Value to indicate whether the iterator is initialized.
    is_initialized: bool,
}

impl<'a> ExtExtendedAttributesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut ExtFileEntry) -> Self {
        Self {
            file_entry,
            number_of_extended_attributes: 0,
            extended_attribute_index: 0,
            is_initialized: false,
        }
    }
}

impl<'a> Iterator for ExtExtendedAttributesIterator<'a> {
    type Item = Result<ExtExtendedAttribute, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_initialized {
            match self.file_entry.get_number_of_extended_attributes() {
                Ok(number_of_extended_attributes) => {
                    self.number_of_extended_attributes = number_of_extended_attributes;
                }
                Err(error) => return Some(Err(error)),
            }
            self.is_initialized = true;
        }
        if self.extended_attribute_index >= self.number_of_extended_attributes {
            return None;
        }
        let item: Self::Item = self
            .file_entry
            .get_extended_attribute_by_index(self.extended_attribute_index);
        self.extended_attribute_index += 1;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
