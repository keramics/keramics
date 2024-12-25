/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

/// Extended File System (ext) block range type.
#[derive(Clone, Debug, PartialEq)]
pub enum ExtBlockRangeType {
    InFile,
    Sparse,
}

/// Extended File System (ext) block range.
#[derive(Clone, Debug)]
pub struct ExtBlockRange {
    /// Logical block number.
    pub logical_block_number: u64,

    /// Physical block number.
    pub physical_block_number: u64,

    /// Number of blocks.
    pub number_of_blocks: u64,

    /// Range type.
    pub range_type: ExtBlockRangeType,
}

impl ExtBlockRange {
    /// Creates a new block range.
    pub fn new(
        logical_block_number: u64,
        physical_block_number: u64,
        number_of_blocks: u64,
        range_type: ExtBlockRangeType,
    ) -> Self {
        Self {
            logical_block_number: logical_block_number,
            physical_block_number: physical_block_number,
            number_of_blocks: number_of_blocks,
            range_type: range_type,
        }
    }
}
