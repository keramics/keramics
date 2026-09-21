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

/// Volume Shadow Snapshot (volsnap) block overlay range.
#[derive(Debug)]
pub struct VolsnapBlockOverlayRange {
    /// End offset.
    pub end_offset: u32,

    /// Value to indicate the bit in the allocation bitmap is set.
    pub is_set: bool,
}

impl VolsnapBlockOverlayRange {
    /// Creates a new block overlay range.
    pub fn new(end_offset: u32, is_set: bool) -> Self {
        Self { end_offset, is_set }
    }
}
