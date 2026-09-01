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

/// Mac OS sparse image (.sparseimage) block range type.
#[derive(Clone, Debug, PartialEq)]
pub enum SparseImageBlockRangeType {
    InFile,
    Sparse,
}

/// Mac OS sparse image (.sparseimage) block range.
#[derive(Clone, Debug)]
pub struct SparseImageBlockRange {
    /// Logical offset.
    pub logical_offset: u64,

    /// Physical band number.
    pub physical_band_number: u32,

    /// Number of bands.
    pub number_of_bands: u32,

    /// Range type.
    pub range_type: SparseImageBlockRangeType,
}

impl SparseImageBlockRange {
    /// Creates a new block range.
    pub fn new(
        logical_offset: u64,
        physical_band_number: u32,
        number_of_bands: u32,
        range_type: SparseImageBlockRangeType,
    ) -> Self {
        Self {
            logical_offset,
            physical_band_number,
            number_of_bands,
            range_type,
        }
    }
}
