/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::sync::{Arc, RwLock};

use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};

use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::ApmFileEntry;

/// Apple Partition Map (APM) file system.
pub struct ApmFileSystem {
    /// Volume system.
    volume_system: Arc<ApmVolumeSystem>,

    /// Number of partitions.
    number_of_partitions: usize,
}

impl ApmFileSystem {
    pub const PATH_PREFIX: &'static str = "/apm";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            volume_system: Arc::new(ApmVolumeSystem::new()),
            number_of_partitions: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> io::Result<bool> {
        match vfs_path {
            VfsPath::String(string_path_components) => {
                let number_of_components: usize = string_path_components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(false);
                }
                if string_path_components[0] != "" {
                    return Ok(false);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    return Ok(true);
                }
                match self.get_partition_index(&string_path_components[1]) {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported VFS path type",
            )),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, vfs_path: &VfsPath) -> io::Result<Option<ApmFileEntry>> {
        match vfs_path {
            VfsPath::String(string_path_components) => {
                let number_of_components: usize = string_path_components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(None);
                }
                if string_path_components[0] != "" {
                    return Ok(None);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    let apm_file_entry: ApmFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(apm_file_entry));
                }
                match self.get_partition_index(&string_path_components[1]) {
                    Some(partition_index) => {
                        let apm_partition: ApmPartition =
                            self.volume_system.get_partition_by_index(partition_index)?;

                        Ok(Some(ApmFileEntry::Partition {
                            index: partition_index,
                            partition: Arc::new(RwLock::new(apm_partition)),
                        }))
                    }
                    None => Ok(None),
                }
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported VFS path type",
            )),
        }
    }

    /// Retrieves the partition index.
    fn get_partition_index(&self, file_name: &String) -> Option<usize> {
        if !file_name.starts_with("apm") {
            return None;
        }
        match file_name[3..].parse::<usize>() {
            Ok(partition_index) => {
                if partition_index > 0 && partition_index <= self.number_of_partitions {
                    Some(partition_index - 1)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<ApmFileEntry> {
        Ok(ApmFileEntry::Root {
            volume_system: self.volume_system.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> io::Result<()> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                ));
            }
        };
        let vfs_path: &VfsPath = vfs_location.get_path();
        match file_system.get_data_stream_by_path_and_name(vfs_path, None)? {
            Some(data_stream) => match Arc::get_mut(&mut self.volume_system) {
                Some(volume_system) => {
                    volume_system.read_data_stream(&data_stream)?;

                    self.number_of_partitions = volume_system.get_number_of_partitions();
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Missing volume system",
                    ));
                }
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", vfs_path.to_string()),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::new_os_vfs_location;

    fn get_file_system() -> io::Result<ApmFileSystem> {
        let mut apm_file_system: ApmFileSystem = ApmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/apm/apm.dmg");
        apm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(apm_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/");
        let result: bool = apm_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/apm1");
        let result: bool = apm_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/bogus1");
        let result: bool = apm_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/");
        let result: Option<ApmFileEntry> = apm_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let apm_file_entry: ApmFileEntry = result.unwrap();

        let name: Option<String> = apm_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = apm_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/apm1");
        let result: Option<ApmFileEntry> = apm_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let apm_file_entry: ApmFileEntry = result.unwrap();

        let name: Option<String> = apm_file_entry.get_name();
        assert_eq!(name, Some("apm1".to_string()));

        let file_type: VfsFileType = apm_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/bogus1");
        let result: Option<ApmFileEntry> = apm_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_partition_index() -> io::Result<()> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let file_name: String = "apm1".to_string();
        let partition_index: Option<usize> = apm_file_system.get_partition_index(&file_name);
        assert_eq!(partition_index, Some(0));

        let file_name: String = "apm99".to_string();
        let partition_index: Option<usize> = apm_file_system.get_partition_index(&file_name);
        assert!(partition_index.is_none());

        let file_name: String = "bogus1".to_string();
        let partition_index: Option<usize> = apm_file_system.get_partition_index(&file_name);
        assert!(partition_index.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> io::Result<()> {
        let apm_file_system: ApmFileSystem = get_file_system()?;

        let apm_file_entry: ApmFileEntry = apm_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = apm_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut apm_file_system: ApmFileSystem = ApmFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/apm/apm.dmg");
        apm_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(apm_file_system.number_of_partitions, 2);

        Ok(())
    }
}
