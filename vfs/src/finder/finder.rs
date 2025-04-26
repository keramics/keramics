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

use crate::file_entry::VfsFileEntry;
use crate::file_system::VfsFileSystem;
use crate::string::VfsString;

/// Virtual File System (VFS) finder state.
struct VfsFinderState {
    /// File entry.
    file_entry: VfsFileEntry,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,
}

impl VfsFinderState {
    /// Creates a new iterator.
    fn new(file_entry: VfsFileEntry, number_of_sub_file_entries: usize) -> Self {
        Self {
            file_entry: file_entry,
            number_of_sub_file_entries: number_of_sub_file_entries,
            sub_file_entry_index: 0,
        }
    }
}

/// Virtual File System (VFS) finder.
pub struct VfsFinder<'a> {
    /// File system.
    file_system: &'a VfsFileSystem,

    /// Path components.
    pub path_components: Vec<VfsString>,

    /// Finder states.
    states: Vec<VfsFinderState>,

    /// Value to indicate the finder has started searching.
    search_started: bool,
}

// TODO: add support for filters (FindSpecs).

impl<'a> VfsFinder<'a> {
    /// Creates a new finder.
    pub fn new(file_system: &'a VfsFileSystem) -> Self {
        Self {
            file_system: file_system,
            path_components: Vec::new(),
            states: Vec::new(),
            search_started: false,
        }
    }
}

impl<'a> Iterator for VfsFinder<'a> {
    type Item = io::Result<(VfsFileEntry, Vec<VfsString>)>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.search_started {
            self.search_started = true;
            match self.file_system.get_root_file_entry() {
                Ok(result) => match result {
                    Some(mut file_entry) => {
                        // TODO: if file_entry.get_number_of_sub_file_entries() fails return file_entry and error.
                        let number_of_sub_file_entries: usize =
                            match file_entry.get_number_of_sub_file_entries() {
                                Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                                Err(error) => return Some(Err(error)),
                            };
                        match file_entry.get_name() {
                            Some(name) => self.path_components.push(name),
                            None => self.path_components.push(VfsString::Empty),
                        };
                        self.states
                            .push(VfsFinderState::new(file_entry, number_of_sub_file_entries));
                    }
                    None => return None,
                },
                Err(error) => return Some(Err(error)),
            };
        }
        while let Some(mut state) = self.states.pop() {
            if state.sub_file_entry_index >= state.number_of_sub_file_entries {
                let path_components: Vec<VfsString> = self.path_components.clone();

                self.path_components.pop();

                return Some(Ok((state.file_entry, path_components)));
            }
            let result: io::Result<VfsFileEntry> = state
                .file_entry
                .get_sub_file_entry_by_index(state.sub_file_entry_index);
            state.sub_file_entry_index += 1;
            self.states.push(state);

            match result {
                Ok(mut file_entry) => {
                    // TODO: if file_entry.get_number_of_sub_file_entries() fails return file_entry and error.
                    let number_of_sub_file_entries: usize =
                        match file_entry.get_number_of_sub_file_entries() {
                            Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                            Err(error) => return Some(Err(error)),
                        };
                    match file_entry.get_name() {
                        Some(name) => self.path_components.push(name),
                        None => self.path_components.push(VfsString::Empty),
                    };
                    self.states
                        .push(VfsFinderState::new(file_entry, number_of_sub_file_entries));
                }
                Err(error) => return Some(Err(error)),
            }
        }
        None
    }
}

// TODO: add tests
