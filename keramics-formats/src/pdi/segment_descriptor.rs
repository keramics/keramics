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

use super::segment_file_descriptor::PdiSegmentFileDescriptor;

/// Parallels Disk Image (PDI) segment descriptor.
pub struct PdiSegmentDescriptor {
    /// Start sector.
    pub(super) start_sector: u64,

    /// End sector.
    pub(super) end_sector: u64,

    /// Size.
    pub(super) size: u64,

    /// Files.
    pub(super) files: Vec<PdiSegmentFileDescriptor>,
}

impl PdiSegmentDescriptor {
    /// Creates a new segment descriptor.
    pub fn new(
        start_sector: u64,
        end_sector: u64,
        size: u64,
        files: Vec<PdiSegmentFileDescriptor>,
    ) -> Self {
        Self {
            start_sector,
            end_sector,
            size,
            files,
        }
    }

    /// Retrieves the number of files.
    pub fn get_number_of_files(&self) -> usize {
        self.files.len()
    }

    /// Retrieves a file by index.
    pub fn get_file_by_index(&self, file_index: usize) -> Option<&PdiSegmentFileDescriptor> {
        self.files.get(file_index)
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        self.size
    }
}
