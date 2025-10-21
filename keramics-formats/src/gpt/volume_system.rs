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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use super::partition::GptPartition;
use super::partition_entry::GptPartitionEntry;
use super::partition_table_header::GptPartitionTableHeader;

const SUPPORTED_BYTES_PER_SECTOR: [u16; 2] = [512, 4096];

/// GUID Partition Table (GPT) volume system.
pub struct GptVolumeSystem {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Disk identifier.
    pub disk_identifier: Uuid,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Partition entries.
    // TODO: use a HashMap for lookup by identifier.
    partition_entries: Vec<GptPartitionEntry>,
}

impl GptVolumeSystem {
    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
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
    pub fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<GptPartition, ErrorTrace> {
        match self.partition_entries.get(partition_index) {
            Some(partition_entry) => {
                let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing data stream"));
                    }
                };
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
                match partition.open(data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open partition: {}", partition_index)
                        );
                        return Err(error);
                    }
                }
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

    // TODO: add get_partition_index_by_identifier

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_partition_table(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read partition table");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the partition table.
    fn read_partition_table(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut partition_table_header = GptPartitionTableHeader::new();

        if self.bytes_per_sector != 0 {
            match partition_table_header
                .read_at_position(data_stream, SeekFrom::Start(self.bytes_per_sector as u64))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read partition table header"
                    );
                    return Err(error);
                }
            }
        } else {
            for bytes_per_sector in SUPPORTED_BYTES_PER_SECTOR.iter() {
                match partition_table_header
                    .read_at_position(data_stream, SeekFrom::Start(*bytes_per_sector as u64))
                {
                    Ok(_) => self.bytes_per_sector = *bytes_per_sector,
                    Err(_) => {}
                };
                if self.bytes_per_sector != 0 {
                    break;
                }
            }
            if self.bytes_per_sector == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported bytes per sector: 0"
                ));
            }
        }
        let backup_partition_table_offset: u64 =
            partition_table_header.backup_header_block_number * self.bytes_per_sector as u64;

        let mut backup_partition_table_header = GptPartitionTableHeader::new();

        if backup_partition_table_offset > 0 {
            match backup_partition_table_header
                .read_at_position(data_stream, SeekFrom::Start(backup_partition_table_offset))
            {
                Ok(read_count) => read_count,
                Err(_) => {
                    if self.mediator.debug_output {
                        self.mediator.debug_print(format!(
                            "Invalid backup partition table block number falling back to last block"
                        ));
                    }
                    match backup_partition_table_header.read_at_position(
                        data_stream,
                        SeekFrom::End(-(self.bytes_per_sector as i64)),
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read backup partition table"
                            );
                            return Err(error);
                        }
                    }
                }
            };
        }
        // TODO: compare primary with backup partition table header.

        if !partition_table_header.disk_identifier.is_nil() {
            self.disk_identifier = partition_table_header.disk_identifier;
        }
        if partition_table_header.entry_data_size != 128 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported partition table entry data size: {}",
                partition_table_header.entry_data_size
            )));
        }
        let maximum_number_of_entries: u32 =
            (32 * self.bytes_per_sector as u32) / partition_table_header.entry_data_size;

        if partition_table_header.number_of_entries > maximum_number_of_entries {
            return Err(keramics_core::error_trace_new!(format!(
                "Number of partition entries: {} value out of bounds: {}",
                partition_table_header.number_of_entries, maximum_number_of_entries
            )));
        }
        let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

        let mut entry_data_offset: u64 =
            partition_table_header.entries_start_block_number * self.bytes_per_sector as u64;
        let mut entry_data: Vec<u8> = vec![0; partition_table_header.entry_data_size as usize];

        for entry_index in 0..partition_table_header.number_of_entries {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut entry_data,
                SeekFrom::Start(entry_data_offset)
            );
            entry_data_offset += partition_table_header.entry_data_size as u64;

            crc32_context.update(&entry_data);

            let mut partition_entry = GptPartitionEntry::new(entry_index as usize);

            match partition_entry.read_data(&entry_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read partition table entry"
                    );
                    return Err(error);
                }
            }
            if !partition_entry.type_identifier.is_nil() {
                // TODO: check upper bound with size or area_end_block_number
                if partition_entry.start_block_number
                    < partition_table_header.area_start_block_number
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Partition entry: {} start block number: {} value out of bounds: {} - {}",
                        entry_index,
                        partition_entry.start_block_number,
                        partition_table_header.area_start_block_number,
                        partition_table_header.area_end_block_number,
                    )));
                }
                if partition_entry.end_block_number < partition_entry.start_block_number {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Partition entry: {} end block number: {} value out of bounds: {} - {}",
                        entry_index,
                        partition_entry.end_block_number,
                        partition_entry.start_block_number,
                        partition_table_header.area_end_block_number,
                    )));
                }
                self.partition_entries.push(partition_entry);
            }
        }
        let calculated_checksum: u32 = crc32_context.finalize();

        if partition_table_header.entries_data_checksum != 0
            && partition_table_header.entries_data_checksum != calculated_checksum
        {
            return Err(keramics_core::error_trace_new!(format!(
                "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                partition_table_header.entries_data_checksum, calculated_checksum
            )));
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

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<GptVolumeSystem, ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("gpt/gpt.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_number_of_partitions() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        let partition: GptPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 1048576);
        assert_eq!(partition.size, 1048576);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("gpt/gpt.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_partition_table() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("gpt/gpt.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_partition_table(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    // TODO: add tests for set_bytes_per_sector
}
