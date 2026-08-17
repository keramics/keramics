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

use keramics_types::Uuid;

use super::enums::PdiSegmentFileType;

/// Parallels Disk Image (PDI) segment file descriptor.
#[derive(Debug)]
pub struct PdiSegmentFileDescriptor {
    /// Path.
    pub(super) path: String,

    /// Image type.
    pub(super) file_type: PdiSegmentFileType,

    /// Snapshot identifier.
    pub(super) snapshot_identifier: Uuid,
}

impl PdiSegmentFileDescriptor {
    /// Creates a new segment file descriptor.
    pub fn new(path: String, file_type: PdiSegmentFileType, snapshot_identifier: Uuid) -> Self {
        Self {
            path,
            file_type,
            snapshot_identifier,
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> &PdiSegmentFileType {
        &self.file_type
    }

    /// Retrieves the path.
    pub fn get_path(&self) -> &str {
        self.path.as_str()
    }

    /// Retrieves the snapshot identifier.
    pub fn get_snapshot_identifier(&self) -> &Uuid {
        &self.snapshot_identifier
    }
}
