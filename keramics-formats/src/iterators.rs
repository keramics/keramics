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

use keramics_core::ErrorTrace;

use super::traits::{FileEntryIterator, PartitionIterator};

/// Generic file entries iterator.
pub struct FileEntriesIterator<'a, T: FileEntryIterator> {
    /// File entry.
    file_entry: &'a mut T,

    /// Number of sub file entries.
    number_of_sub_file_entries: usize,

    /// Sub file entry index.
    sub_file_entry_index: usize,

    /// Value to indicate whether the iterator is initialized.
    is_initialized: bool,
}

impl<'a, T: FileEntryIterator> FileEntriesIterator<'a, T> {
    /// Creates a new iterator.
    pub fn new(file_entry: &'a mut T) -> Self {
        Self {
            file_entry,
            number_of_sub_file_entries: 0,
            sub_file_entry_index: 0,
            is_initialized: false,
        }
    }
}

impl<'a, T: FileEntryIterator> Iterator for FileEntriesIterator<'a, T> {
    type Item = Result<T, ErrorTrace>;

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

/// Generic partitions iterator.
pub struct PartitionsIterator<'a, T: PartitionIterator> {
    /// Volume system.
    volume_system: &'a T,

    /// Number of partitions.
    number_of_partitions: usize,

    /// Partititon index.
    partition_index: usize,
}

impl<'a, T: PartitionIterator> PartitionsIterator<'a, T> {
    /// Creates a new iterator.
    pub fn new(volume_system: &'a T, number_of_partitions: usize) -> Self {
        Self {
            volume_system,
            number_of_partitions,
            partition_index: 0,
        }
    }
}

impl<'a, T: PartitionIterator> Iterator for PartitionsIterator<'a, T> {
    type Item = Result<T::PartitionItem, ErrorTrace>;

    /// Retrieves the next partition.
    fn next(&mut self) -> Option<Self::Item> {
        if self.partition_index >= self.number_of_partitions {
            return None;
        }
        let item: Self::Item = self
            .volume_system
            .get_partition_by_index(self.partition_index);

        self.partition_index += 1;

        Some(item)
    }
}
