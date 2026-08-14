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
use keramics_formats::apfs::{ApfsContainer, ApfsFileSystem, ApfsVolume};
use keramics_types::Uuid;

use crate::enums::VfsFileType;

/// Apple File System (APFS) container file entry.
pub enum ApfsContainerFileEntry {
    /// Root file entry.
    Root {
        /// Volume system.
        container: Arc<ApfsContainer>,
    },

    /// Volume file entry.
    Volume {
        /// Volume index.
        index: usize,

        /// Volume.
        volume: Arc<ApfsVolume>,

        /// Size.
        size: u64,
    },
}

impl ApfsContainerFileEntry {
    /// Retrieves an APFS file system.
    pub(crate) fn get_apfs_file_system(&self) -> Result<Option<ApfsFileSystem>, ErrorTrace> {
        match self {
            ApfsContainerFileEntry::Root { .. } => Ok(None),
            ApfsContainerFileEntry::Volume { volume, .. } => match volume.get_file_system() {
                Ok(apfs_file_system) => Ok(Some(apfs_file_system)),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve APFS file system",
                    );
                    return Err(error);
                }
            },
        }
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            ApfsContainerFileEntry::Root { .. } => Ok(None),
            ApfsContainerFileEntry::Volume { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            ApfsContainerFileEntry::Root { .. } => VfsFileType::Directory,
            ApfsContainerFileEntry::Volume { .. } => VfsFileType::File,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<Uuid> {
        match self {
            ApfsContainerFileEntry::Root { .. } => None,
            ApfsContainerFileEntry::Volume { volume, .. } => Some(volume.get_identifier().clone()),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            ApfsContainerFileEntry::Root { .. } => PathComponent::Root,
            ApfsContainerFileEntry::Volume { index, .. } => {
                PathComponent::from(format!("apfs{}", index + 1))
            }
        }
    }

    /// Retrieves the volume number.
    pub fn get_volume_number(&self) -> Option<usize> {
        match self {
            ApfsContainerFileEntry::Root { .. } => None,
            ApfsContainerFileEntry::Volume { volume, .. } => Some(volume.get_volume_index() + 1),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            ApfsContainerFileEntry::Root { .. } => 0,
            ApfsContainerFileEntry::Volume { size, .. } => *size,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            ApfsContainerFileEntry::Root { container } => container.get_number_of_volumes(),
            ApfsContainerFileEntry::Volume { .. } => 0,
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<ApfsContainerFileEntry, ErrorTrace> {
        match self {
            ApfsContainerFileEntry::Root { container } => {
                match container.get_volume_by_index(sub_file_entry_index) {
                    Ok(apfs_volume) => {
                        let volume_size: u64 = apfs_volume.get_size();

                        Ok(ApfsContainerFileEntry::Volume {
                            index: sub_file_entry_index,
                            volume: Arc::new(apfs_volume),
                            size: volume_size,
                        })
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve APFS volume: {}", sub_file_entry_index)
                        );
                        return Err(error);
                    }
                }
            }
            ApfsContainerFileEntry::Volume { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            ApfsContainerFileEntry::Root { .. } => true,
            ApfsContainerFileEntry::Volume { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_container() -> Result<ApfsContainer, ErrorTrace> {
        let mut container: ApfsContainer = ApfsContainer::new();

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        Ok(container)
    }

    // TODO: add tests for get_apfs_file_system

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let identifier: Option<Uuid> = file_entry.get_identifier();
        assert_eq!(identifier, None);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let identifier: Option<Uuid> = file_entry.get_identifier();
        assert_eq!(
            identifier,
            Some(Uuid {
                part1: 0x33d13da9,
                part2: 0xf1c8,
                part3: 0x4d2a,
                part4: 0xb9c7,
                part5: 0x71ab9dbe5fe2,
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("apfs1"));

        Ok(())
    }

    #[test]
    fn test_get_volume_number() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, None);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let volume_number: Option<usize> = file_entry.get_volume_number();
        assert_eq!(volume_number, Some(1));
        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let size: u64 = file_entry.get_size();
        assert_eq!(size, 77824);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        let sub_file_entry: ApfsContainerFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("apfs1"));

        let result: Result<ApfsContainerFileEntry, ErrorTrace> =
            file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let apfs_container: Arc<ApfsContainer> = Arc::new(get_container()?);

        let file_entry = ApfsContainerFileEntry::Root {
            container: apfs_container.clone(),
        };
        assert_eq!(file_entry.is_root_file_entry(), true);

        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let volume_size: u64 = apfs_volume.get_size();
        let file_entry = ApfsContainerFileEntry::Volume {
            index: 0,
            volume: Arc::new(apfs_volume),
            size: volume_size,
        };
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
