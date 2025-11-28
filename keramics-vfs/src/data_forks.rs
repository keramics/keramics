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

use super::data_fork::VfsDataFork;
use super::file_entry::VfsFileEntry;

/// Virtual File System (VFS) data fork iterator.
pub struct VfsDataForksIterator<'a> {
    /// File entry.
    file_entry: &'a mut VfsFileEntry,

    /// Number of data fork.
    number_of_data_forks: usize,

    /// Extended attribute index.
    data_fork_index: usize,

    /// Value to indicate whether the iterator is initialized.
    is_initialized: bool,
}

impl<'a> VfsDataForksIterator<'a> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut VfsFileEntry) -> Self {
        Self {
            file_entry,
            number_of_data_forks: 0,
            data_fork_index: 0,
            is_initialized: false,
        }
    }
}

impl<'a> Iterator for VfsDataForksIterator<'a> {
    type Item = Result<VfsDataFork, ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_initialized {
            match self.file_entry.get_number_of_data_forks() {
                Ok(number_of_data_forks) => {
                    self.number_of_data_forks = number_of_data_forks;
                }
                Err(error) => return Some(Err(error)),
            }
            self.is_initialized = true;
        }
        if self.data_fork_index >= self.number_of_data_forks {
            return None;
        }
        let item: Self::Item = self.file_entry.get_data_fork_by_index(self.data_fork_index);

        self.data_fork_index += 1;

        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
