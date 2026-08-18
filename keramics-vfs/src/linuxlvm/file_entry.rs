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
use keramics_formats::linuxlvm::{LinuxLvmVolume, LinuxLvmVolumeSystem};

use crate::enums::VfsFileType;

/// Linux Logical Volume Manager (LVM) file entry.
pub enum LinuxLvmFileEntry {
    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<LinuxLvmVolumeSystem>,
    },

    /// Volume file entry.
    Volume {
        /// File name index.
        name_index: usize,

        /// Volume.
        volume: LinuxLvmVolume,
    },
}

impl LinuxLvmFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            LinuxLvmFileEntry::Root { .. } => Ok(None),
            LinuxLvmFileEntry::Volume { volume, .. } => Ok(Some(volume.get_data_stream())),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            LinuxLvmFileEntry::Root { .. } => VfsFileType::Directory,
            LinuxLvmFileEntry::Volume { .. } => VfsFileType::File,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<&str> {
        match self {
            LinuxLvmFileEntry::Root { .. } => None,
            LinuxLvmFileEntry::Volume { volume, .. } => Some(volume.get_identifier()),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            LinuxLvmFileEntry::Root { .. } => PathComponent::Root,
            LinuxLvmFileEntry::Volume { name_index, .. } => {
                PathComponent::from(format!("lvm{}", name_index + 1))
            }
        }
    }

    /// Retrieves the volume number.
    pub fn get_volume_number(&self) -> Option<usize> {
        match self {
            LinuxLvmFileEntry::Root { .. } => None,
            LinuxLvmFileEntry::Volume { volume, .. } => Some(volume.get_volume_index() + 1),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            LinuxLvmFileEntry::Root { .. } => 0,
            LinuxLvmFileEntry::Volume { volume, .. } => volume.get_volume_size(),
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            LinuxLvmFileEntry::Root { volume_system } => volume_system.get_number_of_volumes(),
            LinuxLvmFileEntry::Volume { .. } => 0,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<LinuxLvmFileEntry, ErrorTrace> {
        match self {
            LinuxLvmFileEntry::Root { volume_system } => {
                match volume_system.get_volume_by_index(sub_file_entry_index) {
                    Ok(lvm_volume) => Ok(LinuxLvmFileEntry::Volume {
                        name_index: sub_file_entry_index,
                        volume: lvm_volume,
                    }),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve volume: {}", sub_file_entry_index)
                        );
                        return Err(error);
                    }
                }
            }
            LinuxLvmFileEntry::Volume { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            LinuxLvmFileEntry::Root { .. } => true,
            LinuxLvmFileEntry::Volume { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_formats::linuxlvm::LinuxLvmDataFileDescriptor;
    use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<LinuxLvmVolumeSystem, ErrorTrace> {
        let path_string: String = get_test_data_path("linuxlvm");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(
                PathComponent::from("lvm2.raw"),
                0,
            )];

        let mut volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

        volume_system.open(&file_resolver, &data_file_descriptors)?;

        Ok(volume_system)
    }

    fn get_root_file_entry(lvm_volume_system: &Arc<LinuxLvmVolumeSystem>) -> LinuxLvmFileEntry {
        LinuxLvmFileEntry::Root {
            volume_system: lvm_volume_system.clone(),
        }
    }

    fn get_volume_file_entry(
        lvm_volume_system: &Arc<LinuxLvmVolumeSystem>,
    ) -> Result<LinuxLvmFileEntry, ErrorTrace> {
        let lvm_volume: LinuxLvmVolume = lvm_volume_system.get_volume_by_index(0)?;

        Ok(LinuxLvmFileEntry::Volume {
            name_index: 0,
            volume: lvm_volume,
        })
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let result: Option<&str> = file_entry.get_identifier();
        assert!(result.is_none());

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let identifier: &str = file_entry.get_identifier().unwrap();
        assert_eq!(identifier, "TDUzsI-6K36-Qipq-T835-ynjD-dCFB-Z36vG4");

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("lvm1"));

        Ok(())
    }

    #[test]
    fn test_get_volume_number() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, None);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, Some(1));
        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 2);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);

        let sub_file_entry: LinuxLvmFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("lvm1"));

        let result: Result<LinuxLvmFileEntry, ErrorTrace> =
            file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let lvm_volume_system: Arc<LinuxLvmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: LinuxLvmFileEntry = get_root_file_entry(&lvm_volume_system);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: LinuxLvmFileEntry = get_volume_file_entry(&lvm_volume_system)?;
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
