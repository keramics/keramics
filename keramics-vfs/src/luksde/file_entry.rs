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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::PathComponent;
use keramics_formats::luksde::LuksEncryptedVolume;

use crate::enums::VfsFileType;

/// Linux Unified Key Setup (LUKS) Disk Encryption file entry.
pub enum LuksFileEntry {
    /// Root file entry.
    Root {
        /// Encrypted volume.
        encrypted_volume: Arc<LuksEncryptedVolume>,
    },

    /// Unlocked volume file entry.
    UnlockedVolume {
        /// Encrypted volume.
        encrypted_volume: Arc<LuksEncryptedVolume>,
    },
}

impl LuksFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            LuksFileEntry::Root { .. } => Ok(None),
            LuksFileEntry::UnlockedVolume {
                encrypted_volume, ..
            } => Ok(encrypted_volume.get_data_stream()),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            LuksFileEntry::Root { .. } => VfsFileType::Directory,
            LuksFileEntry::UnlockedVolume { .. } => VfsFileType::File,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            LuksFileEntry::Root { .. } => PathComponent::Root,
            LuksFileEntry::UnlockedVolume { .. } => PathComponent::from("luks1"),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            LuksFileEntry::Root { .. } => 0,
            LuksFileEntry::UnlockedVolume {
                encrypted_volume, ..
            } => encrypted_volume.get_volume_size(),
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            LuksFileEntry::Root { .. } => 1,
            LuksFileEntry::UnlockedVolume { .. } => 0,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<LuksFileEntry, ErrorTrace> {
        match self {
            LuksFileEntry::Root { encrypted_volume } => {
                if sub_file_entry_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No sub file entry with index: {}",
                        sub_file_entry_index
                    )));
                }
                Ok(LuksFileEntry::UnlockedVolume {
                    encrypted_volume: encrypted_volume.clone(),
                })
            }
            LuksFileEntry::UnlockedVolume { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            LuksFileEntry::Root { .. } => true,
            LuksFileEntry::UnlockedVolume { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_encrypted_volume() -> Result<Arc<LuksEncryptedVolume>, ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        Ok(Arc::new(encrypted_volume))
    }

    fn get_root_file_entry(luks_encrypted_volume: &Arc<LuksEncryptedVolume>) -> LuksFileEntry {
        LuksFileEntry::Root {
            encrypted_volume: luks_encrypted_volume.clone(),
        }
    }

    fn get_unlocked_volume_file_entry(
        luks_encrypted_volume: &Arc<LuksEncryptedVolume>,
    ) -> LuksFileEntry {
        LuksFileEntry::UnlockedVolume {
            encrypted_volume: luks_encrypted_volume.clone(),
        }
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: LuksFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: LuksFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("luks1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: LuksFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 2097152);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry: LuksFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);

        let sub_file_entry: LuksFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("luks1"));

        let result: Result<LuksFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<LuksEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: LuksFileEntry = get_root_file_entry(&test_encrypted_volume);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: LuksFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
