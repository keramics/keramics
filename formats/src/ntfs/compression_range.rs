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

/// New Technologies File System (NTFS) compression range type.
#[derive(Clone, Debug, PartialEq)]
pub enum NtfsCompressionRangeType {
    Compressed,
    Uncompressed,
}

/// New Technologies File System (NTFS) compression range.
#[derive(Debug)]
pub struct NtfsCompressionRange {
    /// Offset.
    pub offset: u64,

    /// Size.
    pub size: u64,

    /// Range type.
    pub range_type: NtfsCompressionRangeType,
}

impl NtfsCompressionRange {
    /// Creates a new compression range.
    pub fn new(offset: u64, size: u64, range_type: NtfsCompressionRangeType) -> Self {
        Self {
            offset: offset,
            size: size,
            range_type: range_type,
        }
    }
}
