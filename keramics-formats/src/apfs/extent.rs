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

/// Apple File System (APFS) extent.
#[derive(Clone, Debug)]
pub struct ApfsExtent {
    /// Logical offset.
    pub logical_offset: u64,

    /// Size.
    pub size: u64,

    /// Physical block number.
    pub physical_block_number: u64,

    /// Encryption identifier.
    pub encryption_identifier: u64,
}

impl ApfsExtent {
    /// Creates a new extent.
    pub fn new(
        logical_offset: u64,
        size: u64,
        physical_block_number: u64,
        encryption_identifier: u64,
    ) -> Self {
        Self {
            logical_offset,
            size,
            physical_block_number,
            encryption_identifier,
        }
    }
}
