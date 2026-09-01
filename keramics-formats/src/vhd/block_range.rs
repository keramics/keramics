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

/// Virtual Hard Disk (VHD) block range type.
#[derive(Debug, PartialEq)]
pub enum VhdBlockRangeType {
    InFile,
    InParent,
    Sparse,
}

/// Virtual Hard Disk (VHD) block range.
#[derive(Debug, PartialEq)]
pub struct VhdBlockRange {
    /// Logical offset.
    pub logical_offset: u64,

    /// Physical offset.
    pub physical_offset: u64,

    /// Size.
    pub size: u64,

    /// Range type.
    pub range_type: VhdBlockRangeType,
}

impl VhdBlockRange {
    /// Creates a new block range.
    pub fn new(
        logical_offset: u64,
        physical_offset: u64,
        size: u64,
        range_type: VhdBlockRangeType,
    ) -> Self {
        Self {
            logical_offset,
            physical_offset,
            size,
            range_type,
        }
    }
}
