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

use datetime::DateTime;

#[derive(Clone)]
/// New Technologies File System (NTFS) directory entry.
pub struct NtfsDirectoryEntry {
    /// File reference.
    pub file_reference: u64,

    /// Creation time.
    pub creation_time: DateTime,

    /// Modification time.
    pub modification_time: DateTime,

    /// Entry modification time.
    pub entry_modification_time: DateTime,

    /// Access time.
    pub access_time: DateTime,

    /// Data size.
    pub data_size: u64,

    /// File attribute flags.
    pub file_attribute_flags: u32,
}

impl NtfsDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(
        file_reference: u64,
        creation_time: DateTime,
        modification_time: DateTime,
        entry_modification_time: DateTime,
        access_time: DateTime,
        data_size: u64,
        file_attribute_flags: u32,
    ) -> Self {
        Self {
            file_reference: file_reference,
            creation_time: creation_time,
            modification_time: modification_time,
            entry_modification_time: entry_modification_time,
            access_time: access_time,
            data_size: data_size,
            file_attribute_flags: file_attribute_flags,
        }
    }
}
