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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::constants::*;
use super::extended_boot_record::MbrExtendedBootRecord;
use super::master_boot_record::MbrMasterBootRecord;
use super::partition::MbrPartition;
use super::partition_entry::MbrPartitionEntry;

const SUPPORTED_BYTES_PER_SECTOR: [u16; 4] = [512, 1024, 2048, 4096];

/// Master Boot Record (MBR) volume system.
pub struct MbrVolumeSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// First extended boot record offset.
    first_extended_boot_record_offset: u64,

    /// Disk identity.
    pub disk_identity: u32,

    /// Partition entries.
    partition_entries: Vec<MbrPartitionEntry>,
}

impl MbrVolumeSystem {
    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            first_extended_boot_record_offset: 0,
            disk_identity: 0,
            partition_entries: Vec::new(),
        }
    }

    /// Retrieves the number of partitions.
    pub fn get_number_of_partitions(&self) -> usize {
        self.partition_entries.len()
    }

    /// Retrieves a partition by index.
    pub fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<MbrPartition, ErrorTrace> {
        match self.partition_entries.get(partition_index) {
            Some(partition_entry) => {
                let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing data stream"));
                    }
                };
                if self.bytes_per_sector == 0 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported bytes per sector: 0"
                    ));
                }
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
                partition.open(data_stream)?;

                Ok(partition)
            }
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No partition with index: {}",
                    partition_index
                )));
            }
        }
    }

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        self.read_master_boot_record(data_stream)?;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the master and extended boot records.
    fn read_master_boot_record(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut master_boot_record = MbrMasterBootRecord::new();

        master_boot_record.read_at_position(data_stream, SeekFrom::Start(0))?;
        if self.bytes_per_sector == 0 {
            for partition_entry in master_boot_record.partition_entries.iter() {
                if partition_entry.partition_type == 5 || partition_entry.partition_type == 15 {
                    let mut boot_signature: [u8; 2] = [0; 2];

                    for bytes_per_sector in SUPPORTED_BYTES_PER_SECTOR.iter() {
                        let offset: u64 =
                            partition_entry.start_address_lba as u64 * *bytes_per_sector as u64;

                        keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut boot_signature,
                            SeekFrom::Start(offset + 510)
                        );
                        if boot_signature == MBR_BOOT_SIGNATURE {
                            self.bytes_per_sector = *bytes_per_sector;
                            break;
                        }
                    }
                    break;
                }
            }
        }
        let mut entry_index: usize = 0;
        let mut extended_boot_record_offset: u64 = 0;

        while let Some(mut partition_entry) = master_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 5 || partition_entry.partition_type == 15 {
                if self.bytes_per_sector == 0 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported bytes per sector: 0"
                    ));
                }
                if extended_boot_record_offset != 0 {
                    return Err(keramics_core::error_trace_new!(
                        "More than 1 extended partition entry per boot record is not supported"
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

            self.read_extended_boot_record(data_stream, extended_boot_record_offset, 4)?;
        }
        if self.bytes_per_sector == 0 {
            match self
                .partition_entries
                .iter()
                .max_by_key(|element| element.start_address_lba)
            {
                Some(last_partition_entry) => {
                    let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

                    let end_address_lba: u64 = (last_partition_entry.start_address_lba as u64)
                        + (last_partition_entry.number_of_sectors as u64);

                    for bytes_per_sector in SUPPORTED_BYTES_PER_SECTOR.iter() {
                        if end_address_lba > data_stream_size / (*bytes_per_sector as u64) {
                            break;
                        }
                        self.bytes_per_sector = *bytes_per_sector;
                    }
                }
                None => {}
            }
        }
        self.disk_identity = master_boot_record.disk_identity;

        Ok(())
    }

    /// Reads an extended boot record.
    fn read_extended_boot_record(
        &mut self,
        data_stream: &DataStreamReference,
        offset: u64,
        first_entry_index: usize,
    ) -> Result<(), ErrorTrace> {
        if first_entry_index >= 1024 {
            return Err(keramics_core::error_trace_new!(
                "More than 1024 partition entries not supported"
            ));
        }
        let mut extended_boot_record = MbrExtendedBootRecord::new();
        extended_boot_record.read_at_position(data_stream, SeekFrom::Start(offset))?;

        let mut entry_index: usize = 0;
        let mut extended_boot_record_offset: u64 = 0;

        while let Some(mut partition_entry) = extended_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 0 {
                continue;
            }
            if partition_entry.partition_type == 5 {
                if extended_boot_record_offset != 0 {
                    return Err(keramics_core::error_trace_new!(
                        "More than 1 extended partition entry per boot record is not supported"
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
            self.read_extended_boot_record(
                data_stream,
                extended_boot_record_offset,
                first_entry_index + 4,
            )?;
        }
        Ok(())
    }

    /// Sets the number of bytes per sector.
    pub fn set_bytes_per_sector(&mut self, bytes_per_sector: u16) -> Result<(), ErrorTrace> {
        if !SUPPORTED_BYTES_PER_SECTOR.contains(&bytes_per_sector) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported bytes per sector: {}",
                bytes_per_sector
            )));
        }
        self.bytes_per_sector = bytes_per_sector;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_volume_system() -> Result<MbrVolumeSystem, ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/mbr/mbr.raw")?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    #[test]
    fn test_number_of_partitions() -> Result<(), ErrorTrace> {
        let volume_system: MbrVolumeSystem = get_volume_system()?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> Result<(), ErrorTrace> {
        let volume_system: MbrVolumeSystem = get_volume_system()?;

        let partition: MbrPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 512);
        assert_eq!(partition.size, 1049088);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/mbr/mbr.raw")?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_master_boot_record() -> Result<(), ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/mbr/mbr.raw")?;
        volume_system.read_master_boot_record(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    // TODO: add tests for read_extended_boot_record
    // TODO: add tests for set_bytes_per_sector
}
