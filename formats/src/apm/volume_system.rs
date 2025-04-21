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

use core::DataStreamReference;

use super::constants::*;
use super::partition::ApmPartition;
use super::partition_map_entry::ApmPartitionMapEntry;

/// Apple Partition Map (APM) volume system.
pub struct ApmVolumeSystem {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Partition map entries.
    partition_map_entries: Vec<ApmPartitionMapEntry>,
}

impl ApmVolumeSystem {
    pub const PATH_PREFIX: &'static str = "/apm";

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: DataStreamReference::none(),
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
    pub fn get_partition_index_by_path(&self, location: &str) -> io::Result<usize> {
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
    pub fn get_partition_by_path(&self, location: &str) -> io::Result<Option<ApmPartition>> {
        if location == "/" {
            return Ok(None);
        }
        let partition_index: usize = self.get_partition_index_by_path(location)?;

        let partition: ApmPartition = self.get_partition_by_index(partition_index)?;

        Ok(Some(partition))
    }

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        self.read_partition_map(data_stream)?;

        self.data_stream = data_stream.clone();

        Ok(())
    }

    /// Reads the partition map.
    fn read_partition_map(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        let mut number_of_entries: u32 = 0;
        let mut partition_map_entry_index: u32 = 0;
        let mut partition_map_entry_offset: u64 = 512;

        self.bytes_per_sector = 512;

        loop {
            let mut partition_map_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();

            partition_map_entry
                .read_at_position(data_stream, io::SeekFrom::Start(partition_map_entry_offset))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    use core::open_os_data_stream;

    fn get_volume_system() -> io::Result<ApmVolumeSystem> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/apm/apm.dmg")?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_number_of_partitions() -> io::Result<()> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

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
    fn test_read_data_stream() -> io::Result<()> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/apm/apm.dmg")?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_partition_map() -> io::Result<()> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/apm/apm.dmg")?;
        volume_system.read_partition_map(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }
}
