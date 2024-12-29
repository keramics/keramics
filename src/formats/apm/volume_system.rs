/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::types::SharedValue;
use crate::vfs::{
    VfsDataStreamReference, VfsFileEntryReference, VfsFileSystem, VfsFileSystemReference,
    VfsPathReference, VfsPathType, WrapperVfsFileEntry,
};

use super::constants::*;
use super::partition::ApmPartition;
use super::partition_map_entry::ApmPartitionMapEntry;

/// Apple Partition Map (APM) volume system.
pub struct ApmVolumeSystem {
    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Partition map entries.
    partition_map_entries: Vec<ApmPartitionMapEntry>,
}

impl ApmVolumeSystem {
    const PATH_PREFIX: &'static str = "/apm";

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: SharedValue::none(),
            bytes_per_sector: 0,
            partition_map_entries: Vec::new(),
        }
    }

    /// Retrieves the number of partitions.
    pub fn get_number_of_partitions(&self) -> usize {
        self.partition_map_entries.len()
    }

    /// Retrieves a partition by index.
    pub fn get_partition_by_index(&self, partition_index: usize) -> io::Result<ApmPartition> {
        match self.partition_map_entries.get(partition_index) {
            Some(partition_entry) => {
                let partition_offset: u64 =
                    partition_entry.start_sector as u64 * self.bytes_per_sector as u64;
                let partition_size: u64 =
                    partition_entry.number_of_sectors as u64 * self.bytes_per_sector as u64;

                let mut partition: ApmPartition = ApmPartition::new(
                    partition_offset,
                    partition_size,
                    &partition_entry.type_identifier,
                    &partition_entry.name,
                    partition_entry.status_flags,
                );
                partition.open(&self.data_stream)?;

                Ok(partition)
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("No partition with index: {}", partition_index),
                ))
            }
        }
    }

    /// Retrieves the partition index with the specific location.
    fn get_partition_index_by_path(&self, location: &str) -> io::Result<usize> {
        if !location.starts_with(ApmVolumeSystem::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        let partition_index: usize = match location[4..].parse::<usize>() {
            Ok(value) => value,
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported path: {}", location),
                ))
            }
        };
        if partition_index == 0 || partition_index > self.partition_map_entries.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        Ok(partition_index - 1)
    }

    /// Retrieves the partition with the specific location.
    fn get_partition_by_path(&self, location: &str) -> io::Result<Option<ApmPartition>> {
        if location == "/" {
            return Ok(None);
        }
        let partition_index: usize = self.get_partition_index_by_path(location)?;

        let partition: ApmPartition = self.get_partition_by_index(partition_index)?;

        Ok(Some(partition))
    }

    /// Reads the partition map.
    fn read_partition_map(&mut self) -> io::Result<()> {
        let mut number_of_entries: u32 = 0;
        let mut partition_map_entry_index: u32 = 0;
        let mut partition_map_entry_offset: u64 = 512;

        self.bytes_per_sector = 512;

        loop {
            let mut partition_map_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();

            partition_map_entry.read_at_position(
                &self.data_stream,
                io::SeekFrom::Start(partition_map_entry_offset),
            )?;
            if partition_map_entry_index == 0 {
                if partition_map_entry.type_identifier.elements != APM_PARTITION_MAP_TYPE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported partition map entry: {} unsupported partition type",
                            partition_map_entry_index,
                        ),
                    ));
                }
                number_of_entries = partition_map_entry.number_of_entries;
            } else if partition_map_entry.number_of_entries != number_of_entries {
                return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported partition map entry: {} number of entries: {} value out of bounds",
                            partition_map_entry_index, partition_map_entry.number_of_entries,
                        ),
                    ));
            } else {
                self.partition_map_entries.push(partition_map_entry);
            }
            partition_map_entry_index += 1;
            partition_map_entry_offset += 512;

            if partition_map_entry_index >= number_of_entries {
                break;
            }
        }
        Ok(())
    }
}

impl VfsFileSystem for ApmVolumeSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPathReference) -> io::Result<bool> {
        if path.path_type != VfsPathType::Apm {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        if path.location == "/" {
            return Ok(true);
        }
        match self.get_partition_index_by_path(&path.location) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Retrieves the path type.
    fn get_vfs_path_type(&self) -> VfsPathType {
        VfsPathType::Apm
    }

    /// Opens a volume system.
    fn open(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        let result: Option<VfsDataStreamReference> = match file_system.with_write_lock() {
            Ok(file_system) => file_system.open_data_stream(path, None)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.data_stream = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.location),
                ))
            }
        };
        self.read_partition_map()
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(
        &self,
        path: &VfsPathReference,
    ) -> io::Result<Option<VfsFileEntryReference>> {
        if path.path_type != VfsPathType::Apm {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let partition: Option<ApmPartition> = match self.get_partition_by_path(&path.location) {
            Ok(partition) => partition,
            Err(_) => return Ok(None),
        };
        let mut file_entry: WrapperVfsFileEntry =
            WrapperVfsFileEntry::new::<ApmPartition>(partition);
        file_entry.initialize(path)?;

        Ok(Some(Box::new(file_entry)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType, VfsPath};

    fn get_volume_system() -> io::Result<ApmVolumeSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        volume_system.open(&vfs_file_system, &vfs_path)?;

        Ok(volume_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "./bogus2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let directory_name: &str = volume_system.get_directory_name("/apm1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let partition: ApmPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 32768);
        assert_eq!(partition.size, 4153344);

        Ok(())
    }

    #[test]
    fn get_partition_index_by_path() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let partition_index: usize = volume_system.get_partition_index_by_path("/apm1")?;
        assert_eq!(partition_index, 0);

        let result = volume_system.get_partition_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = volume_system.get_partition_index_by_path("/apm99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_partition_by_path() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let result: Option<ApmPartition> = volume_system.get_partition_by_path("/")?;
        assert!(result.is_none());

        let result: Option<ApmPartition> = volume_system.get_partition_by_path("/apm2")?;
        assert!(result.is_some());

        let partition: ApmPartition = result.unwrap();
        assert_eq!(partition.offset, 4186112);
        assert_eq!(partition.size, 8192);

        Ok(())
    }

    #[test]
    fn test_get_vfs_path_type() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let vfs_path_type: VfsPathType = volume_system.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Apm);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        volume_system.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            volume_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_partition() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/apm2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            volume_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_non_existing() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/bogus2", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            volume_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let test_vfs_path: VfsPathReference = VfsPath::new(VfsPathType::NotSet, "/", None);

        let result = volume_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
