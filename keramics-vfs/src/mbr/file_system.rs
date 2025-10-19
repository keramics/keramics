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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::mbr::MbrVolumeSystem;

use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

use super::file_entry::MbrFileEntry;

/// Master Boot Record (MBR) file system.
pub struct MbrFileSystem {
    /// Volume system.
    volume_system: Arc<MbrVolumeSystem>,

    /// Number of partitions.
    number_of_partitions: usize,
}

impl MbrFileSystem {
    pub const PATH_PREFIX: &'static str = "/mbr";

    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            volume_system: Arc::new(MbrVolumeSystem::new()),
            number_of_partitions: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path) => {
                let number_of_components: usize = string_path.components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(false);
                }
                if string_path.components[0] != "" {
                    return Ok(false);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    return Ok(true);
                }
                match self.get_partition_index(&string_path.components[1]) {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<MbrFileEntry>, ErrorTrace> {
        match vfs_path {
            VfsPath::String(string_path) => {
                let number_of_components: usize = string_path.components.len();
                if number_of_components == 0 || number_of_components > 2 {
                    return Ok(None);
                }
                if string_path.components[0] != "" {
                    return Ok(None);
                }
                // A single empty component represents "/".
                if number_of_components == 1 {
                    let mbr_file_entry: MbrFileEntry = self.get_root_file_entry()?;

                    return Ok(Some(mbr_file_entry));
                }
                let partition_index: usize =
                    match self.get_partition_index(&string_path.components[1]) {
                        Some(partition_index) => partition_index,
                        None => return Ok(None),
                    };
                match self.volume_system.get_partition_by_index(partition_index) {
                    Ok(mbr_partition) => Ok(Some(MbrFileEntry::Partition {
                        index: partition_index,
                        partition: Arc::new(RwLock::new(mbr_partition)),
                    })),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve MBR partition: {}", partition_index)
                        );
                        return Err(error);
                    }
                }
            }
            _ => Err(keramics_core::error_trace_new!("Unsupported VFS path type")),
        }
    }

    /// Retrieves the partition index.
    fn get_partition_index(&self, file_name: &String) -> Option<usize> {
        if !file_name.starts_with("mbr") {
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
    pub fn get_root_file_entry(&self) -> Result<MbrFileEntry, ErrorTrace> {
        Ok(MbrFileEntry::Root {
            volume_system: self.volume_system.clone(),
        })
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let vfs_path: &VfsPath = vfs_location.get_path();

        match Arc::get_mut(&mut self.volume_system) {
            Some(volume_system) => {
                match Self::open_volume_system(volume_system, file_system, vfs_path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open MBR volume system"
                        );
                        return Err(error);
                    }
                }
                self.number_of_partitions = volume_system.get_number_of_partitions();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to MBR volume system"
                ));
            }
        }
        Ok(())
    }

    /// Opens a MBR volume system.
    pub(crate) fn open_volume_system(
        volume_system: &mut MbrVolumeSystem,
        file_system: &VfsFileSystemReference,
        vfs_path: &VfsPath,
    ) -> Result<(), ErrorTrace> {
        let result: Option<DataStreamReference> =
            match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        match volume_system.read_data_stream(&data_stream) {
            Ok(()) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read MBR volume system from data stream"
                );
                return Err(error);
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

    fn get_file_system() -> Result<MbrFileSystem, ErrorTrace> {
        let mut mbr_file_system: MbrFileSystem = MbrFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/mbr/mbr.raw");
        mbr_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        Ok(mbr_file_system)
    }

    #[test]
    fn test_file_entry_exists() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/");
        let result: bool = mbr_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr1");
        let result: bool = mbr_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/bogus1");
        let result: bool = mbr_file_system.file_entry_exists(&vfs_path)?;
        assert_eq!(result, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let mbr_file_entry: MbrFileEntry = result.unwrap();

        let name: Option<String> = mbr_file_entry.get_name();
        assert!(name.is_none());

        let file_type: VfsFileType = mbr_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr1");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let mbr_file_entry: MbrFileEntry = result.unwrap();

        let name: Option<String> = mbr_file_entry.get_name();
        assert_eq!(name, Some(String::from("mbr1")));

        let file_type: VfsFileType = mbr_file_entry.get_file_type();
        assert!(file_type == VfsFileType::File);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/bogus1");
        let result: Option<MbrFileEntry> = mbr_file_system.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_partition_index() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let file_name: String = String::from("mbr1");
        let partition_index: Option<usize> = mbr_file_system.get_partition_index(&file_name);
        assert_eq!(partition_index, Some(0));

        let file_name: String = String::from("mbr99");
        let partition_index: Option<usize> = mbr_file_system.get_partition_index(&file_name);
        assert!(partition_index.is_none());

        let file_name: String = String::from("bogus1");
        let partition_index: Option<usize> = mbr_file_system.get_partition_index(&file_name);
        assert!(partition_index.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_file_entry() -> Result<(), ErrorTrace> {
        let mbr_file_system: MbrFileSystem = get_file_system()?;

        let mbr_file_entry: MbrFileEntry = mbr_file_system.get_root_file_entry()?;

        let file_type: VfsFileType = mbr_file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut mbr_file_system: MbrFileSystem = MbrFileSystem::new();

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let parent_vfs_location: VfsLocation = new_os_vfs_location("../test_data/mbr/mbr.raw");
        mbr_file_system.open(Some(&parent_file_system), &parent_vfs_location)?;

        assert_eq!(mbr_file_system.number_of_partitions, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
}
