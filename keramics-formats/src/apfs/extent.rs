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
pub struct ApfsExtent {
    /// Extent (logical) offset.
    pub extent_offset: u64,

    /// Extent size.
    pub extent_size: u64,

    /// (Physical) block number.
    pub block_number: u64,

    /// Encryption identifier.
    pub encryption_identifier: u64,
}

impl ApfsExtent {
    /// Creates a new extent.
    pub fn new(
        extent_offset: u64,
        extent_size: u64,
        encryption_identifier: u64,
        block_number: u64,
    ) -> Self {
        Self {
            extent_offset,
            extent_size,
            block_number,
            encryption_identifier,
        }
    }
}
