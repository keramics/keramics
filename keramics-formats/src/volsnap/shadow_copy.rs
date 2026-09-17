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

use keramics_datetime::DateTime;
use keramics_types::Uuid;

/// Volume Shadow Snapshot (volsnap) shadow copy.
pub struct VolsnapShadowCopy {
    /// Size.
    pub size: u64,

    /// Creation time.
    pub creation_time: DateTime,

    /// Value to indicate type 2 catalog entry was read.
    pub type2_entry_read: bool,

    /// Store metadata offset.
    pub store_metadata_offset: u64,

    /// Store block list offset.
    pub store_block_list_offset: u64,

    /// Store block range list offset.
    pub store_block_range_list_offset: u64,

    /// Store bitmap offset.
    pub store_bitmap_offset: u64,

    /// Store previous bitmap offset.
    pub store_previous_bitmap_offset: u64,

    /// Value to indicate type 3 catalog entry was read.
    pub type3_entry_read: bool,

    /// Copy identifier.
    pub copy_identifier: Uuid,

    /// Copy set identifier.
    pub copy_set_identifier: Uuid,

    /// Attribute flags.
    pub attribute_flags: u32,

    /// Value to indicate store metadata was read.
    pub store_metadata_read: bool,
}

impl VolsnapShadowCopy {
    /// Creates a new shadow copy.
    pub fn new() -> Self {
        Self {
            size: 0,
            creation_time: DateTime::NotSet,
            type2_entry_read: false,
            store_metadata_offset: 0,
            store_block_list_offset: 0,
            store_block_range_list_offset: 0,
            store_bitmap_offset: 0,
            store_previous_bitmap_offset: 0,
            type3_entry_read: false,
            copy_identifier: Uuid::new(),
            copy_set_identifier: Uuid::new(),
            attribute_flags: 0,
            store_metadata_read: false,
        }
    }
}
