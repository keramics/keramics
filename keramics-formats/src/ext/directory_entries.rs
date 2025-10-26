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

use std::collections::BTreeMap;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use super::block_range::ExtBlockRange;
use super::directory_entry::ExtDirectoryEntry;
use super::directory_tree::ExtDirectoryTree;

/// Extended File System (ext) directory entries.
pub struct ExtDirectoryEntries {
    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Entries.
    pub entries: BTreeMap<ByteString, ExtDirectoryEntry>,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl ExtDirectoryEntries {
    /// Creates new directory entries.
    pub fn new(encoding: &CharacterEncoding) -> Self {
        Self {
            encoding: encoding.clone(),
            entries: BTreeMap::new(),
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(
        &self,
        entry_index: usize,
    ) -> Option<(&ByteString, &ExtDirectoryEntry)> {
        self.entries.iter().nth(entry_index)
    }

    /// Retrieves a specific directory entry by name.
    pub fn get_entry_by_name(
        &self,
        name: &ByteString,
    ) -> Option<(&ByteString, &ExtDirectoryEntry)> {
        self.entries.get_key_value(name)
    }

    /// Retrieves the number of entries.
    pub fn get_number_of_entries(&self) -> usize {
        self.entries.len()
    }

    /// Determines if the directory entries were read.
    pub fn is_read(&self) -> bool {
        return self.is_read;
    }

    /// Reads the directory entries from block data.
    pub fn read_block_data(
        &mut self,
        data_stream: &DataStreamReference,
        block_size: u32,
        block_ranges: &Vec<ExtBlockRange>,
    ) -> Result<(), ErrorTrace> {
        let mut directory_tree: ExtDirectoryTree =
            ExtDirectoryTree::new(&self.encoding, block_size);

        match directory_tree.read_block_data(data_stream, block_ranges, &mut self.entries) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read directory tree from block data"
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads the directory entries from inline data.
    pub fn read_inline_data(&mut self, data: &[u8], block_size: u32) -> Result<(), ErrorTrace> {
        let mut directory_tree: ExtDirectoryTree =
            ExtDirectoryTree::new(&self.encoding, block_size);

        match directory_tree.read_inline_data(data, &mut self.entries) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read directory tree from inline data"
                );
                return Err(error);
            }
        }
        self.is_read = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests
}
