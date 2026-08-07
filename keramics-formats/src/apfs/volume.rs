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
use keramics_types::{ByteString, Uuid};

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

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&ByteString> {
        if self.superblock.volume_label.is_empty() {
            None
        } else {
            Some(&self.superblock.volume_label)
        }
    }

    /// Opens a volume.
    pub(super) fn open(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::apfs::ApfsContainer;
    use crate::tests::get_test_data_path;

    fn get_volume() -> Result<ApfsVolume, ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        container.get_volume_by_index(0)
    }

    #[test]
    fn test_get_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_feature_flags();
        assert_eq!(feature_flags, 0x00000002);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let identifier: &Uuid = volume.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "33d13da9-f1c8-4d2a-b9c7-71ab9dbe5fe2"
        );
        Ok(())
    }

    #[test]
    fn test_get_incompatible_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_incompatible_feature_flags();
        assert_eq!(feature_flags, 0x00000001);

        Ok(())
    }

    #[test]
    fn test_get_read_only_compatible_feature_flags() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let feature_flags: u64 = volume.get_read_only_compatible_feature_flags();
        assert_eq!(feature_flags, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let volume: ApfsVolume = get_volume()?;

        let volume_label: Option<&ByteString> = volume.get_volume_label();
        assert_eq!(volume_label, Some(ByteString::from("apfs_test")).as_ref());

        Ok(())
    }

    // TODO: add tests for open
}
