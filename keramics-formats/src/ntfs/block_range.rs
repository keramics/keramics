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

/// New Technologies File System (NTFS) block range type.
#[derive(Clone, Debug, PartialEq)]
pub enum NtfsBlockRangeType {
    InFile,
    Sparse,
}

/// New Technologies File System (NTFS) block range.
#[derive(Debug)]
pub struct NtfsBlockRange {
    /// Virtual (or logical) cluster offset.
    pub virtual_cluster_offset: u64,

    /// (Physical) cluster block number.
    pub cluster_block_number: u64,

    /// Size.
    pub size: u64,

    /// Range type.
    pub range_type: NtfsBlockRangeType,
}

impl NtfsBlockRange {
    /// Creates a new block range.
    pub fn new(
        virtual_cluster_offset: u64,
        cluster_block_number: u64,
        size: u64,
        range_type: NtfsBlockRangeType,
    ) -> Self {
        Self {
            virtual_cluster_offset: virtual_cluster_offset,
            cluster_block_number: cluster_block_number,
            size: size,
            range_type: range_type,
        }
    }
}
