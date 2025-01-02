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

use std::io;

use crate::vfs::file_entry::VfsFileEntry;

/// Virtual File System (VFS) file entries iterator.
pub struct VfsFileEntriesIterator<'a> {
    /// File entry.
    file_entry: &'a mut VfsFileEntry,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,
}

impl<'a> VfsFileEntriesIterator<'a> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut VfsFileEntry, number_of_sub_file_entries: usize) -> Self {
        Self {
            file_entry: file_entry,
            number_of_sub_file_entries: number_of_sub_file_entries,
            sub_file_entry_index: 0,
        }
    }
}

impl<'a> Iterator for VfsFileEntriesIterator<'a> {
    type Item = io::Result<VfsFileEntry>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
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

// TODO: add tests
