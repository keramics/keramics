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

/// Parallels Disk Image (PDI) block range type.
#[derive(Debug, PartialEq)]
pub enum PdiBlockRangeType {
    InFile,
    InParentOrSparse,
}

/// Parallels Disk Image (PDI) block range.
#[derive(Debug, PartialEq)]
pub struct PdiBlockRange {
    /// Extent offset.
    pub extent_offset: u64,

    /// Data offset.
    pub data_offset: u64,

    /// Size.
    pub size: u64,

    /// Range type.
    pub range_type: PdiBlockRangeType,
}

impl PdiBlockRange {
    /// Creates a new block range.
    pub fn new(
        extent_offset: u64,
        data_offset: u64,
        size: u64,
        range_type: PdiBlockRangeType,
    ) -> Self {
        Self {
            extent_offset,
            data_offset,
            size,
            range_type,
        }
    }
}
