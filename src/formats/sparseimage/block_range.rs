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

/// Mac OS sparse image (.sparseimage) block range.
#[derive(Debug)]
pub struct SparseImageBlockRange {
    /// Data offset.
    pub data_offset: u64,
}

impl SparseImageBlockRange {
    /// Creates a new block range.
    pub fn new(data_offset: u64) -> Self {
        Self {
            data_offset: data_offset,
        }
    }
}
