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
use std::sync::Arc;

use crate::types::SharedValue;
use crate::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

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
    pub const PATH_PREFIX: &'static str = "/mbr";

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
                    partition_entry.partition_type,
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
    pub(crate) fn get_partition_index_by_path(&self, location: &str) -> io::Result<usize> {
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
    pub(crate) fn get_partition_by_path(&self, location: &str) -> io::Result<Option<MbrPartition>> {
        if location == "/" {
            return Ok(None);
        }
        let partition_index: usize = self.get_partition_index_by_path(location)?;

        let partition: MbrPartition = self.get_partition_by_index(partition_index)?;

        Ok(Some(partition))
    }

    /// Opens a volume system.
    pub fn open(&mut self, file_system: &Arc<VfsFileSystem>, path: &VfsPath) -> io::Result<()> {
        self.data_stream = match file_system.get_data_stream_by_path_and_name(path, None)? {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        };
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

    /// Sets the number of bytes per sector.
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsPath};

    fn get_volume_system() -> io::Result<MbrVolumeSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        volume_system.open(&vfs_file_system, &vfs_path)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_partition_by_index() -> io::Result<()> {
        let volume_system: MbrVolumeSystem = get_volume_system()?;

        let partition: MbrPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 512);
        assert_eq!(partition.size, 1049088);

        Ok(())
    }

    #[test]
    fn get_partition_index_by_path() -> io::Result<()> {
        let volume_system: MbrVolumeSystem = get_volume_system()?;

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
        let volume_system: MbrVolumeSystem = get_volume_system()?;

        let result: Option<MbrPartition> = volume_system.get_partition_by_path("/")?;
        assert!(result.is_none());

        let result: Option<MbrPartition> = volume_system.get_partition_by_path("/mbr2")?;
        assert!(result.is_some());

        let partition: MbrPartition = result.unwrap();
        assert_eq!(partition.offset, 1050112);
        assert_eq!(partition.size, 1573376);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        volume_system.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }
}
