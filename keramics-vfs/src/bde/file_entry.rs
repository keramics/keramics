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
use keramics_formats::bde::BdeEncryptedVolume;

use crate::enums::VfsFileType;

/// BitLocker Drive Encryption (BDE) encrypted volume file entry.
pub enum BdeFileEntry {
    /// Root file entry.
    Root {
        /// File.
        encrypted_volume: Arc<BdeEncryptedVolume>,
    },

    /// Unlocked volume file entry.
    UnlockedVolume {
        /// File.
        encrypted_volume: Arc<BdeEncryptedVolume>,
    },
}

impl BdeFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            BdeFileEntry::Root { .. } => Ok(None),
            BdeFileEntry::UnlockedVolume {
                encrypted_volume, ..
            } => Ok(encrypted_volume.get_data_stream()),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            BdeFileEntry::Root { .. } => VfsFileType::Directory,
            BdeFileEntry::UnlockedVolume { .. } => VfsFileType::File,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            BdeFileEntry::Root { .. } => PathComponent::Root,
            BdeFileEntry::UnlockedVolume { .. } => PathComponent::from("bde1"),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            BdeFileEntry::Root { .. } => 0,
            BdeFileEntry::UnlockedVolume {
                encrypted_volume, ..
            } => encrypted_volume.get_volume_size(),
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            BdeFileEntry::Root { .. } => 1,
            BdeFileEntry::UnlockedVolume { .. } => 0,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<BdeFileEntry, ErrorTrace> {
        match self {
            BdeFileEntry::Root { encrypted_volume } => {
                if sub_file_entry_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "No sub file entry with index: {}",
                        sub_file_entry_index
                    )));
                }
                Ok(BdeFileEntry::UnlockedVolume {
                    encrypted_volume: encrypted_volume.clone(),
                })
            }
            BdeFileEntry::UnlockedVolume { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            BdeFileEntry::Root { .. } => true,
            BdeFileEntry::UnlockedVolume { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::RwLock;

    use keramics_core::open_os_data_stream;
    use keramics_formats::RangeStream;
    use keramics_formats::vhd::VhdFile;

    use crate::tests::get_test_data_path;

    fn get_encrypted_volume() -> Result<Arc<BdeEncryptedVolume>, ErrorTrace> {
        let mut encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

        let path_string: String = get_test_data_path("bde/bde_aes128.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        encrypted_volume.read_data_stream(&data_stream)?;

        Ok(Arc::new(encrypted_volume))
    }

    fn get_root_file_entry(bde_encrypted_volume: &Arc<BdeEncryptedVolume>) -> BdeFileEntry {
        BdeFileEntry::Root {
            encrypted_volume: bde_encrypted_volume.clone(),
        }
    }

    fn get_unlocked_volume_file_entry(
        bde_encrypted_volume: &Arc<BdeEncryptedVolume>,
    ) -> BdeFileEntry {
        BdeFileEntry::UnlockedVolume {
            encrypted_volume: bde_encrypted_volume.clone(),
        }
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: BdeFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: BdeFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("bde1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: BdeFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 65994752);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry: BdeFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);

        let sub_file_entry: BdeFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("bde1"));

        let result: Result<BdeFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let test_encrypted_volume: Arc<BdeEncryptedVolume> = get_encrypted_volume()?;

        let file_entry: BdeFileEntry = get_root_file_entry(&test_encrypted_volume);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: BdeFileEntry = get_unlocked_volume_file_entry(&test_encrypted_volume);
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
