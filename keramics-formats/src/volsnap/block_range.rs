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

/// Volume Shadow Snapshot (volsnap) block range.
#[derive(Debug)]
pub struct VolsnapBlockRange {
    /// Offset.
    pub offset: u64,

    /// Size.
    pub size: u32,

    /// Value to indicate the block range is in the block list.
    pub in_block_list: bool,

    /// Value to indicate the block range is a forwarder.
    pub is_forwarder: bool,

    /// Value to indicate the block range is sparse.
    pub is_sparse: bool,
}

impl VolsnapBlockRange {
    /// Creates a new block range.
    pub fn new(offset: u64, size: u32, in_block_list: bool, is_forwarder: bool) -> Self {
        Self {
            offset,
            size,
            in_block_list,
            is_forwarder,
            is_sparse: false,
        }
    }
}
