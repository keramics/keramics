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

/// Virtual Hard Disk (VHD) block range type.
#[derive(Clone, Debug, PartialEq)]
pub enum VhdBlockRangeType {
    InFile,
    InParent,
    Sparse,
}

/// Virtual Hard Disk (VHD) block range.
#[derive(Debug)]
pub struct VhdBlockRange {
    /// Media offset.
    pub media_offset: u64,

    /// Data offset.
    pub data_offset: u64,

    /// Size.
    pub size: u64,

    /// Range type.
    pub range_type: VhdBlockRangeType,
}

impl VhdBlockRange {
    /// Creates a new block range.
    pub fn new(
        media_offset: u64,
        data_offset: u64,
        size: u64,
        range_type: VhdBlockRangeType,
    ) -> Self {
        Self {
            media_offset: media_offset,
            data_offset: data_offset,
            size: size,
            range_type: range_type,
        }
    }
}
