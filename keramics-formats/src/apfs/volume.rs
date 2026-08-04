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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use super::volume_superblock::ApfsVolumeSuperblock;

/// Apple File System (APFS) volume.
pub struct ApfsVolume {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Volume superblock.
    superblock: ApfsVolumeSuperblock,
}

impl ApfsVolume {
    /// Creates a volume.
    pub(super) fn new(superblock: ApfsVolumeSuperblock) -> Self {
        Self {
            data_stream: None,
            superblock,
        }
    }

    /// Retrieves the feature flags.
    pub fn get_feature_flags(&self) -> u64 {
        self.superblock.feature_flags
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.superblock.volume_identifier
    }

    /// Retrieves the incompatible feature flags.
    pub fn get_incompatible_feature_flags(&self) -> u64 {
        self.superblock.incompatible_feature_flags
    }

    /// Retrieves the read-only compatible feature flags.
    pub fn get_read_only_compatible_feature_flags(&self) -> u64 {
        self.superblock.read_only_compatible_feature_flags
    }

    /// Opens a volume.
    pub(super) fn open(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}
