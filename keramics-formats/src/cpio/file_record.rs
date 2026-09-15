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
use keramics_datetime::DateTime;

/// Copy in and out (CPIO) file record
pub struct CpioFileRecord {
    /// Inode number.
    pub inode_number: u32,

    /// File mode.
    pub file_mode: u32,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// Number of links.
    pub number_of_links: u32,

    /// Device identifier.
    pub device_identifier: u32,

    /// Modification date and time.
    pub modification_time: DateTime,

    /// Path size.
    pub path_size: u32,

    /// Data size.
    pub data_size: u32,

    /// Checksum
    pub checksum: u32,
}

impl CpioFileRecord {
    /// Creates a new record.
    pub fn new() -> Self {
        Self {
            inode_number: 0,
            file_mode: 0,
            owner_identifier: 0,
            group_identifier: 0,
            number_of_links: 0,
            device_identifier: 0,
            modification_time: DateTime::NotSet,
            path_size: 0,
            data_size: 0,
            checksum: 0,
        }
    }
}
