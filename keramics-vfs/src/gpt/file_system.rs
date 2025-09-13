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

use keramics_formats::gpt::{GptPartition, GptVolumeSystem};

use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::GptFileEntry;

/// GUID Partition Table (GPT) file system.
pub struct GptFileSystem {
    /// Volume system.
    volume_system: Arc<GptVolumeSystem>,

    /// Number of partitions.
    number_of_partitions: usize,
}

impl GptFileSystem {
    pub const PATH_PREFIX: &'static str = "/gpt";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            volume_system: Arc::new(GptVolumeSystem::new()),
            number_of_partitions: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        let location: &String = match path {
            VfsPath::Gpt { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            return Ok(true);
        }
        match self.get_partition_index_by_path(&location) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(&self, path: &VfsPath) -> io::Result<Option<GptFileEntry>> {
        let location: &String = match path {
            VfsPath::Gpt { location, .. } => location,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        if location == "/" {
            let gpt_file_entry: GptFileEntry = self.get_root_file_entry()?;

            return Ok(Some(gpt_file_entry));
        }
        match self.get_partition_index_by_path(location) {
            Ok(partition_index) => {
                let gpt_partition: GptPartition =
                    self.volume_system.get_partition_by_index(partition_index)?;

                Ok(Some(GptFileEntry::Partition {
                    index: partition_index,
                    partition: Arc::new(RwLock::new(gpt_partition)),
                }))
            }
            Err(_) => Ok(None),
        }
    }

    /// Retrieves the partition index with the specific location.
    // TODO: return None instead of Err
    fn get_partition_index_by_path(&self, location: &String) -> io::Result<usize> {
        if !location.starts_with(GptFileSystem::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        };
        let partition_index: usize = match location[4..].parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported path: {}", location),
                ))
            }
        };
        if partition_index == 0 || partition_index > self.number_of_partitions {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        Ok(partition_index - 1)
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<GptFileEntry> {
        Ok(GptFileEntry::Root {
            volume_system: self.volume_system.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        path: &VfsPath,
    ) -> io::Result<()> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                ))
            }
        };
        match file_system.get_data_stream_by_path_and_name(path, None)? {
            Some(data_stream) => match Arc::get_mut(&mut self.volume_system) {
                Some(volume_system) => {
                    volume_system.read_data_stream(&data_stream)?;

                    self.number_of_partitions = volume_system.get_number_of_partitions();
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Missing volume system",
                    ))
                }
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::{VfsFileType, VfsPathType};
    use crate::file_system::VfsFileSystem;

    fn get_file_system() -> io::Result<(GptFileSystem, VfsPath)> {
        let mut gpt_file_system: GptFileSystem = GptFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        gpt_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        Ok((gpt_file_system, parent_vfs_path))
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let (gpt_file_system, parent_vfs_path): (GptFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/");
        let result: bool = gpt_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/gpt1");
        let result: bool = gpt_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/bogus1");
        let result: bool = gpt_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let (gpt_file_system, parent_vfs_path): (GptFileSystem, VfsPath) = get_file_system()?;

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/");
        let result: Option<GptFileEntry> = gpt_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let gpt_file_entry: GptFileEntry = result.unwrap();

        let name: Option<String> = gpt_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = gpt_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/gpt1");
        let result: Option<GptFileEntry> = gpt_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let gpt_file_entry: GptFileEntry = result.unwrap();

        let name: Option<String> = gpt_file_entry.get_name();
        assert_eq!(name, Some("gpt1".to_string()));

        let file_type: VfsFileType = gpt_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = parent_vfs_path.new_child(VfsPathType::Gpt, "/bogus1");
        let result: Option<GptFileEntry> = gpt_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn get_partition_index_by_path() -> io::Result<()> {
        let (gpt_file_system, _): (GptFileSystem, VfsPath) = get_file_system()?;

        let path: String = "/".to_string();
        let result = gpt_file_system.get_partition_index_by_path(&path);
        assert!(result.is_err());

        let path: String = "/gpt1".to_string();
        let partition_index: usize = gpt_file_system.get_partition_index_by_path(&path)?;
        assert_eq!(partition_index, 0);

        let path: String = "/gpt99".to_string();
        let result = gpt_file_system.get_partition_index_by_path(&path);
        assert!(result.is_err());

        let path: String = "/bogus1".to_string();
        let result = gpt_file_system.get_partition_index_by_path(&path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> io::Result<()> {
        let (gpt_file_system, _): (GptFileSystem, VfsPath) = get_file_system()?;

        let gpt_file_entry: GptFileEntry = gpt_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = gpt_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut gpt_file_system: GptFileSystem = GptFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let parent_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        gpt_file_system.open(Some(&parent_file_system), &parent_vfs_path)?;

        assert_eq!(gpt_file_system.number_of_partitions, 2);

        Ok(())
    }
}
