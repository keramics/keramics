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

use crate::checksums::ReversedCrc32Context;
use crate::mediator::{Mediator, MediatorReference};
use crate::types::{SharedValue, Uuid};
use crate::vfs::{
    VfsDataStreamReference, VfsFileEntry, VfsFileEntryReference, VfsFileSystem,
    VfsFileSystemReference, VfsPath, VfsPathType, WrapperVfsFileEntry,
};

use super::partition::GptPartition;
use super::partition_entry::GptPartitionEntry;
use super::partition_table_header::GptPartitionTableHeader;

const SUPPORTED_BYTES_PER_SECTOR: [u16; 2] = [512, 4096];

/// GUID Partition Table (GPT) volume system.
pub struct GptVolumeSystem {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Disk identifier.
    pub disk_identifier: Uuid,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Partition entries.
    partition_entries: Vec<GptPartitionEntry>,
}

impl GptVolumeSystem {
    const PATH_PREFIX: &'static str = "/gpt";

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: SharedValue::none(),
            disk_identifier: Uuid::new(),
            bytes_per_sector: 0,
            partition_entries: Vec::new(),
        }
    }

    /// Retrieves the number of partitions.
    pub fn get_number_of_partitions(&self) -> usize {
        self.partition_entries.len()
    }

    /// Retrieves a partition by index.
    pub fn get_partition_by_index(&self, partition_index: usize) -> io::Result<GptPartition> {
        match self.partition_entries.get(partition_index) {
            Some(partition_entry) => {
                let partition_offset: u64 =
                    partition_entry.start_block_number as u64 * self.bytes_per_sector as u64;
                let partition_size: u64 = (partition_entry.end_block_number
                    - partition_entry.start_block_number
                    + 1) as u64
                    * self.bytes_per_sector as u64;

                let mut partition: GptPartition = GptPartition::new(
                    partition_entry.index,
                    partition_offset,
                    partition_size,
                    &partition_entry.type_identifier,
                    &partition_entry.identifier,
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
        if !location.starts_with(GptVolumeSystem::PATH_PREFIX) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported path: {}", location),
            ));
        }
        // TODO: add support for identifier comparison /gpt{UUID}

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
    fn get_partition_by_path(&self, location: &str) -> io::Result<Option<GptPartition>> {
        if location == "/" {
            return Ok(None);
        }
        let partition_index: usize = self.get_partition_index_by_path(location)?;

        let partition: GptPartition = self.get_partition_by_index(partition_index)?;

        Ok(Some(partition))
    }

    // TODO: add get_partition_index_by_identifier

    /// Reads the partition table.
    fn read_partition_table(&mut self) -> io::Result<()> {
        let mut partition_table_header = GptPartitionTableHeader::new();

        if self.bytes_per_sector != 0 {
            partition_table_header.read_at_position(
                &self.data_stream,
                io::SeekFrom::Start(self.bytes_per_sector as u64),
            )?;
        } else {
            for bytes_per_sector in SUPPORTED_BYTES_PER_SECTOR.iter() {
                match partition_table_header.read_at_position(
                    &self.data_stream,
                    io::SeekFrom::Start(*bytes_per_sector as u64),
                ) {
                    Ok(_) => self.bytes_per_sector = *bytes_per_sector,
                    Err(_) => {}
                };
                if self.bytes_per_sector != 0 {
                    break;
                }
            }
            if self.bytes_per_sector == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported bytes per sector: 0"),
                ));
            }
        }
        let backup_partition_table_offset: u64 =
            partition_table_header.backup_header_block_number * self.bytes_per_sector as u64;

        let mut backup_partition_table_header = GptPartitionTableHeader::new();

        if backup_partition_table_offset > 0 {
            match backup_partition_table_header.read_at_position(
                &self.data_stream,
                io::SeekFrom::Start(backup_partition_table_offset),
            ) {
                Ok(_) => {}
                Err(_) => {
                    if self.mediator.debug_output {
                        self.mediator.debug_print(format!(
                            "Invalid backup partition table block number falling back to last block"
                        ));
                    }
                    backup_partition_table_header.read_at_position(
                        &self.data_stream,
                        io::SeekFrom::End(-(self.bytes_per_sector as i64)),
                    )?;
                }
            };
        }
        // TODO: compare primary with backup partition table header.

        if !partition_table_header.disk_identifier.is_nil() {
            self.disk_identifier = partition_table_header.disk_identifier;
        }
        if partition_table_header.entry_data_size < 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported partition table entry data size: {}",
                    partition_table_header.entry_data_size
                ),
            ));
        }
        let maximum_number_of_entries: u32 =
            (32 * self.bytes_per_sector as u32) / partition_table_header.entry_data_size;

        if partition_table_header.number_of_entries > maximum_number_of_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Number of partition entries: {} value out of bounds: {}",
                    partition_table_header.number_of_entries, maximum_number_of_entries
                ),
            ));
        }
        let entries_start_offset: u64 =
            partition_table_header.entries_start_block_number * self.bytes_per_sector as u64;

        match self.data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.seek(io::SeekFrom::Start(entries_start_offset))?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

        let mut entry_data: Vec<u8> = vec![0; partition_table_header.entry_data_size as usize];

        for entry_index in 0..partition_table_header.number_of_entries {
            match self.data_stream.with_write_lock() {
                Ok(mut data_stream) => data_stream.read_exact(&mut entry_data)?,
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            crc32_context.update(&entry_data);

            let mut partition_entry = GptPartitionEntry::new(entry_index as usize);
            partition_entry.read_data(&entry_data)?;

            if !partition_entry.type_identifier.is_nil() {
                // TODO: check upper bound with size or area_end_block_number
                if partition_entry.start_block_number
                    < partition_table_header.area_start_block_number
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Partition entry: {} start block number: {} value out of bounds: {} - {}",
                            entry_index, partition_entry.start_block_number,
                            partition_table_header.area_start_block_number,
                            partition_table_header.area_end_block_number,
                        ),
                    ));
                }
                if partition_entry.end_block_number < partition_entry.start_block_number {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Partition entry: {} end block number: {} value out of bounds: {} - {}",
                            entry_index,
                            partition_entry.end_block_number,
                            partition_entry.start_block_number,
                            partition_table_header.area_end_block_number,
                        ),
                    ));
                }
                self.partition_entries.push(partition_entry);
            }
        }
        let calculated_checksum: u32 = crc32_context.finalize();

        if partition_table_header.entries_data_checksum != 0
            && partition_table_header.entries_data_checksum != calculated_checksum
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    partition_table_header.entries_data_checksum, calculated_checksum
                ),
            ));
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

impl VfsFileSystem for GptVolumeSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        if path.path_type != VfsPathType::Gpt {
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

    /// Opens a file system.
    fn open(
        &mut self,
        parent_file_system: &VfsFileSystemReference,
        path: &VfsPath,
    ) -> io::Result<()> {
        self.data_stream = match parent_file_system.with_write_lock() {
            Ok(file_system) => file_system.open_data_stream(path, None)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.read_partition_table()
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<VfsFileEntryReference> {
        if path.path_type != VfsPathType::Gpt {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        let partition: Option<GptPartition> = self.get_partition_by_path(&path.location)?;

        let mut file_entry: WrapperVfsFileEntry =
            WrapperVfsFileEntry::new::<GptPartition>(partition);
        file_entry.open(path)?;

        Ok(Box::new(file_entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileType};

    fn get_volume_system() -> io::Result<GptVolumeSystem> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut volume_system = GptVolumeSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        volume_system.open(&parent_file_system, &vfs_path)?;

        Ok(volume_system)
    }

    #[test]
    fn test_file_entry_exists() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "./bogus2", None);
        assert_eq!(volume_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name() -> io::Result<()> {
        let volume_system = GptVolumeSystem::new();

        let directory_name: &str = volume_system.get_directory_name("/gpt1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let partition: GptPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 1048576);
        assert_eq!(partition.size, 65536);

        Ok(())
    }

    #[test]
    fn get_partition_index_by_path() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let partition_index: usize = volume_system.get_partition_index_by_path("/gpt1")?;
        assert_eq!(partition_index, 0);

        let result = volume_system.get_partition_index_by_path("/bogus1");
        assert!(result.is_err());

        let result = volume_system.get_partition_index_by_path("/gpt99");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_partition_by_path() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let result: Option<GptPartition> = volume_system.get_partition_by_path("/")?;
        assert!(result.is_none());

        let result: Option<GptPartition> = volume_system.get_partition_by_path("/gpt2")?;
        assert!(result.is_some());

        let partition: GptPartition = result.unwrap();
        assert_eq!(partition.offset, 2097152);
        assert_eq!(partition.size, 65536);

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut volume_system = GptVolumeSystem::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        volume_system.open(&parent_file_system, &vfs_path)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_root() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/", Some(os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            volume_system.open_file_entry(&test_vfs_path)?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_of_partition() -> io::Result<()> {
        let volume_system = get_volume_system()?;

        let os_vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/gpt2", Some(os_vfs_path));
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
}
