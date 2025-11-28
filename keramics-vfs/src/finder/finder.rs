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
use keramics_formats::Path;

use crate::file_entry::VfsFileEntry;
use crate::file_system::VfsFileSystem;

/// Virtual File System (VFS) finder state.
struct VfsFinderState {
    /// File entry.
    file_entry: VfsFileEntry,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,

    /// Value to indicate the state has been initialized.
    is_initialized: bool,
}

impl VfsFinderState {
    /// Creates a new finder state.
    fn new(file_entry: VfsFileEntry) -> Self {
        Self {
            file_entry,
            number_of_sub_file_entries: 0,
            sub_file_entry_index: 0,
            is_initialized: false,
        }
    }
}

/// Virtual File System (VFS) finder.
pub struct VfsFinder<'a> {
    /// File system.
    file_system: &'a VfsFileSystem,

    /// Path.
    path: Path,

    /// Finder states.
    states: Vec<VfsFinderState>,

    /// Value to indicate the finder has started searching.
    search_started: bool,

    /// Value to indicate the finder encountered an error.
    error_encountered: bool,
}

// TODO: add support for filters (FindSpecs).

impl<'a> VfsFinder<'a> {
    /// Creates a new finder.
    pub fn new(file_system: &'a VfsFileSystem) -> Self {
        Self {
            file_system,
            path: Path::from("/"),
            states: Vec::new(),
            search_started: false,
            error_encountered: false,
        }
    }

    /// Retrieves the current path.
    pub fn get_path(&self) -> &Path {
        &self.path
    }
}

impl<'a> Iterator for VfsFinder<'a> {
    type Item = Result<(VfsFileEntry, Path), ErrorTrace>;

    /// Retrieves the next file entry.
    fn next(&mut self) -> Option<Self::Item> {
        if !self.search_started {
            self.search_started = true;
            match self.file_system.get_root_file_entry() {
                Ok(result) => match result {
                    Some(file_entry) => {
                        self.states.push(VfsFinderState::new(file_entry));
                    }
                    None => return None,
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve root file entry"
                    );
                    return Some(Err(error));
                }
            };
        }
        if self.error_encountered {
            self.path.components.pop();
            self.error_encountered = false;
        }
        while let Some(mut state) = self.states.pop() {
            if !state.is_initialized {
                match state.file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => {
                        state.number_of_sub_file_entries = number_of_sub_file_entries
                    }
                    Err(mut error) => {
                        self.error_encountered = true;

                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of sub file entries"
                        );
                        return Some(Err(error));
                    }
                }
                state.is_initialized = true;
            }
            if state.sub_file_entry_index >= state.number_of_sub_file_entries {
                let path: Path = self.path.clone();

                self.path.components.pop();

                return Some(Ok((state.file_entry, path)));
            }
            let result: Result<VfsFileEntry, ErrorTrace> = state
                .file_entry
                .get_sub_file_entry_by_index(state.sub_file_entry_index);
            let sub_file_entry_index: usize = state.sub_file_entry_index;

            state.sub_file_entry_index += 1;
            self.states.push(state);

            match result {
                Ok(file_entry) => {
                    match file_entry.get_name() {
                        Some(name) => self.path.push(name),
                        None => {
                            return Some(Err(keramics_core::error_trace_new!(
                                "Missing name for file entry"
                            )));
                        }
                    }
                    self.states.push(VfsFinderState::new(file_entry));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve sub file entry: {}",
                            sub_file_entry_index
                        )
                    );
                    return Some(Err(error));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
