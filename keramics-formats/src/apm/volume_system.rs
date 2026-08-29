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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::PartitionIterator;

use super::constants::*;
use super::partition::ApmPartition;
use super::partition_map_entry::ApmPartitionMapEntry;
use super::partitions::ApmPartitionsIterator;

/// Apple Partition Map (APM) volume system.
pub struct ApmVolumeSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Partition map entries.
    partition_map_entries: Vec<ApmPartitionMapEntry>,
}

impl ApmVolumeSystem {
    const SUPPORTED_BYTES_PER_SECTOR: [u16; 2] = [512, 2048];

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            partition_map_entries: Vec::new(),
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the number of partitions.
    pub fn get_number_of_partitions(&self) -> usize {
        self.partition_map_entries.len()
    }

    /// Retrieves a partitions iterator.
    pub fn partitions(&self) -> ApmPartitionsIterator<'_> {
        ApmPartitionsIterator::new(self, self.partition_map_entries.len())
    }

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_partition_map(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read partition map");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the partition map.
    fn read_partition_map(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut partition_map_signature: [u8; 80] = [0; 80];
        let mut number_of_entries: u32 = 0;
        let mut partition_map_entry_index: u32 = 0;

        for bytes_per_sector in Self::SUPPORTED_BYTES_PER_SECTOR.iter() {
            let offset: u64 = *bytes_per_sector as u64;

            keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut partition_map_signature,
                SeekFrom::Start(offset)
            );
            if &partition_map_signature[0..2] == APM_PARTITION_MAP_SIGNATURE
                && &partition_map_signature[48..67] == APM_PARTITION_MAP_TYPE
            {
                self.bytes_per_sector = *bytes_per_sector;
                break;
            }
        }
        if self.bytes_per_sector == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported bytes per sector: 0"
            ));
        }
        let mut partition_map_entry_offset: u64 = self.bytes_per_sector as u64;

        loop {
            let mut partition_map_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();

            match partition_map_entry
                .read_at_position(data_stream, SeekFrom::Start(partition_map_entry_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read partition map entry at offset: {} (0x{:08x})",
                            partition_map_entry_offset, partition_map_entry_offset
                        )
                    );
                    return Err(error);
                }
            }
            if partition_map_entry_index == 0 {
                if partition_map_entry.type_identifier != APM_PARTITION_MAP_TYPE.as_slice() {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported partition map entry: {} unsupported partition type",
                        partition_map_entry_index,
                    )));
                }
                number_of_entries = partition_map_entry.number_of_entries;
            } else if partition_map_entry.number_of_entries != number_of_entries {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported partition map entry: {} number of entries: {} value out of bounds",
                    partition_map_entry_index, partition_map_entry.number_of_entries,
                )));
            } else {
                self.partition_map_entries.push(partition_map_entry);
            }
            partition_map_entry_index += 1;
            partition_map_entry_offset += self.bytes_per_sector as u64;

            if partition_map_entry_index >= number_of_entries {
                break;
            }
        }
        Ok(())
    }
}

impl PartitionIterator for ApmVolumeSystem {
    type PartitionItem = ApmPartition;

    /// Retrieves a partition by index.
    fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<Self::PartitionItem, ErrorTrace> {
        match self.partition_map_entries.get(partition_index) {
            Some(partition_entry) => match self.data_stream.as_ref() {
                Some(data_stream) => Ok(ApmPartition::new(
                    &data_stream,
                    self.bytes_per_sector,
                    partition_entry,
                )),
                None => Err(keramics_core::error_trace_new!("Missing data stream")),
            },
            None => Err(keramics_core::error_trace_new!(format!(
                "No partition with index: {}",
                partition_index
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{open_fake_data_stream, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<ApmVolumeSystem, ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    fn get_test_partition_map_data(number_of_entries: u32, partition_type: &[u8]) -> Vec<u8> {
        let mut test_data: Vec<u8> = vec![0; 2048];

        let entry: Vec<u8> = get_test_partition_map_entry_data(number_of_entries, partition_type);
        test_data[512..1024].copy_from_slice(&entry);

        test_data
    }

    fn get_test_partition_map_entry_data(number_of_entries: u32, partition_type: &[u8]) -> Vec<u8> {
        let mut test_data: Vec<u8> = vec![0; 512];

        test_data[0] = 0x50;
        test_data[1] = 0x4d;
        test_data[4..8].copy_from_slice(&number_of_entries.to_be_bytes());
        test_data[48..48 + partition_type.len()].copy_from_slice(partition_type);

        test_data
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let bytes_per_sector: u16 = volume_system.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_number_of_partitions() -> Result<(), ErrorTrace> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let number_of_partitions: usize = volume_system.get_number_of_partitions();
        assert_eq!(number_of_partitions, 2);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> Result<(), ErrorTrace> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let partition: ApmPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 32768);
        assert_eq!(partition.size, 4153344);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index_without_data_stream() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();
        volume_system
            .partition_map_entries
            .push(ApmPartitionMapEntry::new());

        let result = volume_system.get_partition_by_index(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_partition_by_index_with_unsupported_partition_index() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();
        volume_system
            .partition_map_entries
            .push(ApmPartitionMapEntry::new());

        let result = volume_system.get_partition_by_index(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_partitions() -> Result<(), ErrorTrace> {
        let volume_system: ApmVolumeSystem = get_volume_system()?;

        let mut partitions_iterator: ApmPartitionsIterator = volume_system.partitions();

        let result: Option<Result<ApmPartition, ErrorTrace>> = partitions_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<ApmPartition, ErrorTrace>> = partitions_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_unsupported_bytes_per_sector() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let test_data: Vec<u8> = vec![0; 2048];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let result = volume_system.read_data_stream(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_map() -> Result<(), ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_partition_map(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_partition_map_with_2048_bytes_per_sector() -> Result<(), ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let mut test_data: Vec<u8> = vec![0; 16384];
        let entry: Vec<u8> =
            get_test_partition_map_entry_data(1, APM_PARTITION_MAP_TYPE.as_slice());
        test_data[2048..2560].copy_from_slice(&entry);

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);
        volume_system.read_partition_map(&data_stream)?;

        assert_eq!(volume_system.get_bytes_per_sector(), 2048);
        assert_eq!(volume_system.get_number_of_partitions(), 0);

        Ok(())
    }

    #[test]
    fn test_read_partition_map_with_unsupported_bytes_per_sector() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let test_data: Vec<u8> = vec![0; 2048];
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let result = volume_system.read_partition_map(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_map_with_unsupported_partition_type() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let test_data: Vec<u8> = get_test_partition_map_data(2, b"Apple_HFS");
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let result = volume_system.read_partition_map(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_map_with_inconsistent_number_of_entries() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let mut test_data: Vec<u8> =
            get_test_partition_map_data(2, APM_PARTITION_MAP_TYPE.as_slice());
        let entry: Vec<u8> = get_test_partition_map_entry_data(3, b"Apple_HFS");
        test_data[1024..1536].copy_from_slice(&entry);

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let result = volume_system.read_partition_map(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_map_with_truncated_partition_map_entry() {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let mut test_data: Vec<u8> =
            get_test_partition_map_data(2, APM_PARTITION_MAP_TYPE.as_slice());
        test_data.truncate(1024);

        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let result = volume_system.read_partition_map(&data_stream);
        assert!(result.is_err());
    }
}
