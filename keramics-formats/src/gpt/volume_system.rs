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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

#[cfg(feature = "debug-trace")]
use keramics_core::DebugTrace;

use crate::traits::PartitionIterator;

use super::partition::GptPartition;
use super::partition_entry::GptPartitionEntry;
use super::partition_table_header::GptPartitionTableHeader;
use super::partitions::GptPartitionsIterator;

/// GUID Partition Table (GPT) volume system.
pub struct GptVolumeSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Disk identifier.
    disk_identifier: Uuid,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Partition entries.
    // TODO: use a HashMap for lookup by identifier.
    partition_entries: Vec<GptPartitionEntry>,
}

impl GptVolumeSystem {
    const SUPPORTED_BYTES_PER_SECTOR: [u16; 4] = [512, 1024, 2048, 4096];

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            disk_identifier: Uuid::new(),
            bytes_per_sector: 0,
            partition_entries: Vec::new(),
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the disk identifier.
    pub fn get_disk_identifier(&self) -> &Uuid {
        &self.disk_identifier
    }

    // TODO: add get_partition_index_by_identifier

    /// Retrieves a partitions iterator.
    pub fn partitions(&self) -> GptPartitionsIterator<'_> {
        GptPartitionsIterator::new(self, self.partition_entries.len())
    }

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
            for bytes_per_sector in Self::SUPPORTED_BYTES_PER_SECTOR.iter() {
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
                    #[cfg(feature = "debug-trace")]
                    DebugTrace::static_scope(|debug_trace| {
                        debug_trace.print("Invalid backup partition table block number falling back to last block");
                    });
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
        if partition_table_header.number_of_entries > 256 {
            return Err(keramics_core::error_trace_new!(format!(
                "Number of partition entries: {} value out of bounds: 256",
                partition_table_header.number_of_entries
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
        if !Self::SUPPORTED_BYTES_PER_SECTOR.contains(&bytes_per_sector) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported bytes per sector: {}",
                bytes_per_sector
            )));
        }
        self.bytes_per_sector = bytes_per_sector;

        Ok(())
    }
}

impl PartitionIterator for GptVolumeSystem {
    type PartitionItem = GptPartition;

    /// Retrieves the number of partitions.
    fn get_number_of_partitions(&self) -> usize {
        self.partition_entries.len()
    }

    /// Retrieves a partition by index.
    fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<Self::PartitionItem, ErrorTrace> {
        match self.partition_entries.get(partition_index) {
            Some(partition_entry) => match self.data_stream.as_ref() {
                Some(data_stream) => Ok(GptPartition::new(
                    data_stream,
                    self.bytes_per_sector,
                    &partition_entry,
                )),
                None => Err(keramics_core::error_trace_new!("Missing data stream")),
            },
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No partition with index: {}",
                    partition_index
                )));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{open_fake_data_stream, open_os_data_stream};

    use crate::tests::get_test_data_path;

    const TEST_PARTITION_TABLE_HEADER: [u8; 92] = [
        0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54, 0x00, 0x00, 0x01, 0x00, 0x5c, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x66, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11,
        0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x02, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    const TEST_PARTITION_TABLE_ENTRY: [u8; 128] = [
        0xaf, 0x3d, 0xc6, 0x0f, 0x83, 0x84, 0x72, 0x47, 0x8e, 0x79, 0x3d, 0x69, 0xd8, 0x47, 0x7d,
        0xe4, 0x8c, 0x58, 0x25, 0x1e, 0xa9, 0x27, 0x94, 0x40, 0x86, 0x8c, 0x2f, 0x25, 0x70, 0x21,
        0xf8, 0x7b, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x08, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4c, 0x00, 0x69, 0x00,
        0x6e, 0x00, 0x75, 0x00, 0x78, 0x00, 0x20, 0x00, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00, 0x65,
        0x00, 0x73, 0x00, 0x79, 0x00, 0x73, 0x00, 0x74, 0x00, 0x65, 0x00, 0x6d, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    fn get_test_data() -> Vec<u8> {
        let mut test_data: Vec<u8> = vec![0; 4096];

        test_data[512..604].copy_from_slice(&TEST_PARTITION_TABLE_HEADER[..]);
        test_data[544..552].copy_from_slice(&3584_u64.to_le_bytes());
        test_data[592..596].copy_from_slice(&1_u32.to_le_bytes());
        test_data[596..600].copy_from_slice(&128_u32.to_le_bytes());

        test_data[1024..1152].copy_from_slice(&TEST_PARTITION_TABLE_ENTRY[..]);
        test_data[1056..1064].copy_from_slice(&2048_u64.to_le_bytes());
        test_data[1064..1072].copy_from_slice(&4095_u64.to_le_bytes());

        test_data[3584..3676].copy_from_slice(&TEST_PARTITION_TABLE_HEADER[..]);
        test_data[3616..3624].copy_from_slice(&1_u64.to_le_bytes());

        test_data
    }

    fn get_volume_system() -> Result<GptVolumeSystem, ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        let bytes_per_sector: u16 = volume_system.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_disk_identifier() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        let disk_identifier: &Uuid = volume_system.get_disk_identifier();
        assert_eq!(
            disk_identifier.to_string(),
            "b182deb3-9c86-4892-9e88-9297a4909855"
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_partitions() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        let number_of_partitions: usize = volume_system.get_number_of_partitions();
        assert_eq!(number_of_partitions, 2);

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
    fn test_partitions() -> Result<(), ErrorTrace> {
        let volume_system: GptVolumeSystem = get_volume_system()?;

        let mut partitions_iterator: GptPartitionsIterator = volume_system.partitions();

        let result: Option<Result<GptPartition, ErrorTrace>> = partitions_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<GptPartition, ErrorTrace>> = partitions_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_partition_table() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_partition_table(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_partition_table_with_invalid_backup_header() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[544..552].copy_from_slice(&3_u64.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());
        volume_system.read_partition_table(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_invalid_backup_header() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[544..552].copy_from_slice(&3_u64.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 1);
        assert_eq!(volume_system.get_bytes_per_sector(), 512);

        Ok(())
    }

    #[test]
    fn test_read_partition_table_with_invalid_backup_fallback() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[3584..3676].fill(0);

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_partition_by_index_with_invalid_index() {
        let volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let result: Result<GptPartition, ErrorTrace> = volume_system.get_partition_by_index(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_entry_start() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[1056..1064].copy_from_slice(&2047_u64.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_entry_end() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[1064..1072].copy_from_slice(&2047_u64.to_le_bytes());
        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_entry_checksum() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[600..604].copy_from_slice(&0xdeadbeef_u32.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_entry_size() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[596..600].copy_from_slice(&129_u32.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_number_of_entries() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let mut test_data: Vec<u8> = get_test_data();
        test_data[592..596].copy_from_slice(&257_u32.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_partition_table_with_invalid_bytes_per_sector() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let test_data: Vec<u8> = vec![0; 4096];
        let data_stream: DataStreamReference = open_fake_data_stream(test_data.as_slice());

        let result = volume_system.read_partition_table(&data_stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_bytes_per_sector() -> Result<(), ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        volume_system.set_bytes_per_sector(512)?;
        assert_eq!(volume_system.get_bytes_per_sector(), 512);

        volume_system.set_bytes_per_sector(4096)?;
        assert_eq!(volume_system.get_bytes_per_sector(), 4096);

        Ok(())
    }

    #[test]
    fn test_set_bytes_per_sector_with_unsupported_value() {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let result = volume_system.set_bytes_per_sector(256);
        assert!(result.is_err());
    }
}
