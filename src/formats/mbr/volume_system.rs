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
use std::rc::Rc;

use crate::types::SharedValue;
use crate::vfs::{
    VfsDataStreamReference, VfsFileEntry, VfsFileEntryReference, VfsFileSystem,
    VfsFileSystemReference, VfsPath, VfsPathType, VfsResolver, VfsResolverReference,
    WrapperVfsFileEntry,
};

use super::constants::*;
use super::extended_boot_record::MbrExtendedBootRecord;
use super::master_boot_record::MbrMasterBootRecord;
use super::partition::MbrPartition;
use super::partition_entry::MbrPartitionEntry;

const SUPPORTED_BYTES_PER_SECTOR: [u16; 4] = [512, 1024, 2048, 4096];

/// Master Boot Record (MBR) volume system.
pub struct MbrVolumeSystem {
    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// First extended boot record offset.
    first_extended_boot_record_offset: u64,

    /// Partition entries.
    partition_entries: Vec<MbrPartitionEntry>,
}

impl MbrVolumeSystem {
    const PATH_PREFIX: &'static str = "/mbr";

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: SharedValue::none(),
            bytes_per_sector: 0,
            first_extended_boot_record_offset: 0,
            partition_entries: Vec::new(),
        }
    }

    /// Retrieves the number of partitions.
    pub fn get_number_of_partitions(&self) -> usize {
        self.partition_entries.len()
    }

    /// Retrieves a partition by index.
    pub fn get_partition_by_index(&self, partition_index: usize) -> io::Result<MbrPartition> {
        match self.partition_entries.get(partition_index) {
            Some(partition_entry) => {
                let mut partition_offset: u64 =
                    partition_entry.start_address_lba as u64 * self.bytes_per_sector as u64;
                let partition_size: u64 =
                    partition_entry.number_of_sectors as u64 * self.bytes_per_sector as u64;

                if partition_entry.index >= 4 {
                    partition_offset += self.first_extended_boot_record_offset;
                }
                let mut partition: MbrPartition = MbrPartition::new(
                    partition_entry.index,
                    partition_offset,
                    partition_size,
                    partition_entry.flags,
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
        if !location.starts_with(MbrVolumeSystem::PATH_PREFIX) {
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
        if partition_index == 0 || partition_index > self.partition_entries.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        Ok(partition_index - 1)
    }

    /// Retrieves the partition with the specific location.
    fn get_partition_by_path(&self, location: &str) -> io::Result<Option<MbrPartition>> {
        if location == "/" {
            return Ok(None);
        }
        let partition_index: usize = self.get_partition_index_by_path(location)?;

        let partition: MbrPartition = self.get_partition_by_index(partition_index)?;

        Ok(Some(partition))
    }

    /// Opens a volume system.
    pub fn open(&mut self, file_system: &dyn VfsFileSystem, path: &VfsPath) -> io::Result<()> {
        self.data_stream = file_system.open_data_stream(path, None)?;

        self.read_master_boot_record()
    }

    /// Reads the master and extended boot records.
    fn read_master_boot_record(&mut self) -> io::Result<()> {
        let mut master_boot_record = MbrMasterBootRecord::new();

        master_boot_record.read_at_position(&self.data_stream, io::SeekFrom::Start(0))?;
        if self.bytes_per_sector == 0 {
            for partition_entry in master_boot_record.partition_entries.iter() {
                if partition_entry.partition_type == 5 || partition_entry.partition_type == 15 {
                    let mut boot_signature: [u8; 2] = [0; 2];

                    for bytes_per_sector in SUPPORTED_BYTES_PER_SECTOR.iter() {
                        let offset: u64 =
                            partition_entry.start_address_lba as u64 * *bytes_per_sector as u64;

                        match self.data_stream.with_write_lock() {
                            Ok(mut data_stream) => data_stream.read_at_position(
                                &mut boot_signature,
                                io::SeekFrom::Start(offset + 510),
                            )?,
                            Err(error) => return Err(crate::error_to_io_error!(error)),
                        };
                        if boot_signature == MBR_BOOT_SIGNATURE {
                            self.bytes_per_sector = *bytes_per_sector;
                            break;
                        }
                    }
                    break;
                }
            }
        }
        if self.bytes_per_sector == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported bytes per sector: 0"),
            ));
        }
        let mut entry_index: usize = 0;
        let mut extended_boot_record_offset: u64 = 0;

        while let Some(mut partition_entry) = master_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 5 || partition_entry.partition_type == 15 {
                if extended_boot_record_offset != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("More than 1 extended partition entry per boot record is not supported."),
                    ));
                }
                extended_boot_record_offset =
                    partition_entry.start_address_lba as u64 * self.bytes_per_sector as u64;
            } else if partition_entry.partition_type != 0 {
                partition_entry.index = entry_index;
                self.partition_entries.push(partition_entry);
            }
            entry_index += 1;
        }
        if extended_boot_record_offset != 0 {
            self.first_extended_boot_record_offset = extended_boot_record_offset;

            self.read_extended_boot_record(extended_boot_record_offset, 4)?;
        }
        Ok(())
    }

    /// Reads an extended boot record.
    fn read_extended_boot_record(
        &mut self,
        offset: u64,
        first_entry_index: usize,
    ) -> io::Result<()> {
        if first_entry_index >= 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("More than 1024 partition entries not supported."),
            ));
        }
        let mut extended_boot_record = MbrExtendedBootRecord::new();
        extended_boot_record.read_at_position(&self.data_stream, io::SeekFrom::Start(offset))?;

        let mut entry_index: usize = 0;
        let mut extended_boot_record_offset: u64 = 0;

        while let Some(mut partition_entry) = extended_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 0 {
                continue;
            }
            if partition_entry.partition_type == 5 {
                if extended_boot_record_offset != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("More than 1 extended partition entry per boot record is not supported."),
                    ));
                }
                extended_boot_record_offset = self.first_extended_boot_record_offset
                    + (partition_entry.start_address_lba as u64 * self.bytes_per_sector as u64);
            } else if partition_entry.partition_type != 0 {
                partition_entry.index = first_entry_index + entry_index;
                self.partition_entries.push(partition_entry);
            }
            entry_index += 1;
        }
        if extended_boot_record_offset != 0 {
            self.read_extended_boot_record(extended_boot_record_offset, first_entry_index + 4)?;
        }
        Ok(())
    }

    pub fn set_bytes_per_sector(&mut self, bytes_per_sector: u16) -> io::Result<()> {
        if !SUPPORTED_BYTES_PER_SECTOR.contains(&bytes_per_sector) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported bytes per sector: {}", bytes_per_sector),
            ));
        }
        self.bytes_per_sector = bytes_per_sector;

        Ok(())
    }
}

impl VfsFileSystem for MbrVolumeSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        if path.path_type != VfsPathType::Mbr {
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

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<VfsFileEntryReference> {
        if path.path_type != VfsPathType::Mbr {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let partition: Option<MbrPartition> = self.get_partition_by_path(&path.location)?;

        let mut file_entry: WrapperVfsFileEntry =
            WrapperVfsFileEntry::new::<MbrPartition>(partition);
        file_entry.open(path)?;

        Ok(Box::new(file_entry))
    }

    /// Opens a file system.
    fn open_with_resolver(&mut self, path: &VfsPath) -> io::Result<()> {
        if path.path_type != VfsPathType::Mbr {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        if path.location != "/" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Location in path is not /",
            ));
        }
        let parent_path: Rc<VfsPath> = match path.get_parent() {
            Some(value) => value,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent path",
                ));
            }
        };
        let parent_file_system_path: VfsPath = VfsPath::new_from_path(&parent_path, "/");

        let vfs_resolver: VfsResolverReference = VfsResolver::current();
        let parent_file_system: VfsFileSystemReference =
            vfs_resolver.open_file_system(&parent_file_system_path)?;

        match parent_file_system.with_write_lock() {
            Ok(file_system) => self.open(file_system.as_ref(), parent_path.as_ref())?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType};

    fn get_volume_system() -> io::Result<MbrVolumeSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

        let mut volume_system = MbrVolumeSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        match vfs_file_system.with_write_lock() {
            Ok(file_system) => volume_system.open(file_system.as_ref(), &vfs_path)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        Ok(volume_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "./bogus2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let volume_system = MbrVolumeSystem::new();

        let directory_name: &str = volume_system.get_directory_name("/mbr1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let partition: MbrPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 512);
        assert_eq!(partition.size, 66048);

        Ok(())
    }

    #[test]
    fn get_partition_index_by_path() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let partition_index: usize = volume_system.get_partition_index_by_path("/mbr1")?;
        assert_eq!(partition_index, 0);

        let result = volume_system.get_partition_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = volume_system.get_partition_index_by_path("/mbr99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_partition_by_path() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let result: Option<MbrPartition> = volume_system.get_partition_by_path("/")?;
        assert!(result.is_none());

        let result: Option<MbrPartition> = volume_system.get_partition_by_path("/mbr2")?;
        assert!(result.is_some());

        let partition: MbrPartition = result.unwrap();
        assert_eq!(partition.offset, 67072);
        assert_eq!(partition.size, 66048);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference = vfs_context.open_file_system(&vfs_path)?;

        let mut volume_system = MbrVolumeSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        match vfs_file_system.with_write_lock() {
            Ok(file_system) => volume_system.open(file_system.as_ref(), &vfs_path)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            volume_system.open_file_entry(&test_vfs_path)?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_partition() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/mbr2", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            volume_system.open_file_entry(&test_vfs_path)?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::NotSet, "/", None);

        let result = volume_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_open_with_resolver() -> io::Result<()> {
        let mut volume_system = MbrVolumeSystem::new();

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/", Some(os_vfs_path));
        volume_system.open_with_resolver(&vfs_path)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }
}
