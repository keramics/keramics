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

use std::cmp::max;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::constants::*;
use super::extended_boot_record::MbrExtendedBootRecord;
use super::master_boot_record::MbrMasterBootRecord;
use super::partition::MbrPartition;
use super::partition_entry::MbrPartitionEntry;
use super::partitions::MbrPartitionsIterator;

/// Master Boot Record (MBR) volume system.
pub struct MbrVolumeSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    pub bytes_per_sector: u32,

    /// Disk identity.
    pub disk_identity: u32,

    /// Partition entries.
    partition_entries: Vec<MbrPartitionEntry>,
}

impl MbrVolumeSystem {
    const SUPPORTED_BYTES_PER_SECTOR: [u32; 4] = [512, 1024, 2048, 4096];

    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
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
                let partition_offset: u64 =
                    partition_entry.start_address_lba * (self.bytes_per_sector as u64);
                let partition_size: u64 =
                    (partition_entry.number_of_sectors as u64) * (self.bytes_per_sector as u64);

                let mut partition: MbrPartition = MbrPartition::new(
                    partition_entry.index,
                    partition_offset,
                    partition_size,
                    partition_entry.partition_type,
                    partition_entry.flags,
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

    /// Retrieves a partitions iterator.
    pub fn partitions(&self) -> MbrPartitionsIterator<'_> {
        MbrPartitionsIterator::new(self, self.partition_entries.len())
    }

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_boot_records(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read boot records");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the master and extended boot records.
    fn read_boot_records(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut boot_signature: [u8; 2] = [0; 2];
        let mut master_boot_record = MbrMasterBootRecord::new();

        match master_boot_record.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read master boot record at offset: 0 (0x00000000)"
                );
                return Err(error);
            }
        }
        if self.bytes_per_sector == 0 {
            for partition_entry in master_boot_record.partition_entries.iter() {
                if partition_entry.partition_type == 5 || partition_entry.partition_type == 15 {
                    for bytes_per_sector in Self::SUPPORTED_BYTES_PER_SECTOR.iter() {
                        let offset: u64 =
                            partition_entry.start_address_lba * (*bytes_per_sector as u64);

                        keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut boot_signature,
                            SeekFrom::Start(offset + 510)
                        );
                        if &boot_signature == MBR_BOOT_SIGNATURE {
                            self.bytes_per_sector = *bytes_per_sector;
                            break;
                        }
                    }
                    break;
                }
            }
        }
        let mut entry_index: usize = 0;
        let mut extended_boot_record_lba: u64 = 0;

        while let Some(mut partition_entry) = master_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 0x05 || partition_entry.partition_type == 0x0f {
                if self.bytes_per_sector == 0 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported bytes per sector: 0"
                    ));
                }
                if extended_boot_record_lba != 0 {
                    return Err(keramics_core::error_trace_new!(
                        "More than 1 extended partition entry per boot record is not supported"
                    ));
                }
                extended_boot_record_lba = partition_entry.start_address_lba;
            } else if partition_entry.partition_type != 0 {
                partition_entry.index = entry_index;
                self.partition_entries.push(partition_entry);
            }
            entry_index += 1;
        }
        if extended_boot_record_lba != 0 {
            let extended_boot_record_offset: u64 =
                extended_boot_record_lba * (self.bytes_per_sector as u64);

            match self.read_extended_boot_record(
                data_stream,
                extended_boot_record_offset,
                4,
                extended_boot_record_lba,
                extended_boot_record_lba,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read first extended boot record"
                    );
                    return Err(error);
                }
            }
        }
        let mut partition_entries: Vec<MbrPartitionEntry> = self.partition_entries.clone();
        partition_entries.sort_by_key(|partition_entry| partition_entry.start_address_lba);

        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);
        let mut last_end_address_lba: u64 = 0;

        for partition_entry in partition_entries.iter() {
            if partition_entry.start_address_lba < last_end_address_lba {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported overlapping partition entries"
                ));
            }
            last_end_address_lba =
                partition_entry.start_address_lba + (partition_entry.number_of_sectors as u64);
            let end_offset: u64 = (last_end_address_lba as u64) * (self.bytes_per_sector as u64);

            // TODO: mark the partition as corrupt
            if end_offset > data_stream_size {
                return Err(keramics_core::error_trace_new!(
                    "Invalid partition entry size value out of bounds"
                ));
            }
        }
        if self.bytes_per_sector == 0 {
            let mut largest_bytes_per_sector: u32 = 0;

            if let Some(last_partition_entry) = partition_entries.last() {
                let start_address_lba: u64 = last_partition_entry.start_address_lba;
                let end_address_lba: u64 =
                    start_address_lba + (last_partition_entry.number_of_sectors as u64);

                for bytes_per_sector in Self::SUPPORTED_BYTES_PER_SECTOR.iter().rev() {
                    // The partition last LBA should not exceed the data stream size.
                    if end_address_lba > data_stream_size / (*bytes_per_sector as u64) {
                        continue;
                    }
                    let offset: u64 = start_address_lba * (*bytes_per_sector as u64);

                    keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut boot_signature,
                        SeekFrom::Start(offset + 510)
                    );
                    // Some file systems like FAT and NTFS use the MBR boot signature in their boot sectors.
                    if &boot_signature == MBR_BOOT_SIGNATURE {
                        self.bytes_per_sector = *bytes_per_sector;

                        break;
                    }
                    largest_bytes_per_sector = max(largest_bytes_per_sector, *bytes_per_sector);
                }
                if self.bytes_per_sector == 0 && largest_bytes_per_sector == 512 {
                    self.bytes_per_sector = 512;
                }
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
        first_extended_boot_record_lba: u64,
        current_extended_boot_record_lba: u64,
    ) -> Result<(), ErrorTrace> {
        if first_entry_index >= 1024 {
            return Err(keramics_core::error_trace_new!(
                "More than 1024 partition entries not supported"
            ));
        }
        let mut extended_boot_record = MbrExtendedBootRecord::new();

        match extended_boot_record.read_at_position(data_stream, SeekFrom::Start(offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read extended boot record at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        let mut next_extended_boot_record_offset: u64 = 0;
        let mut next_extended_boot_record_lba: u64 = 0;

        if let Some(mut partition_entry) = extended_boot_record.partition_entries.get(1) {
            if partition_entry.partition_type != 0 {
                if partition_entry.partition_type != 0x05 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported partition entry: 1 - unusuppored type: 0x{:02x}",
                        partition_entry.partition_type
                    )));
                }
                next_extended_boot_record_lba =
                    first_extended_boot_record_lba + partition_entry.start_address_lba;
                next_extended_boot_record_offset =
                    next_extended_boot_record_lba * (self.bytes_per_sector as u64);

                // TODO check bounds
            }
        }
        for index in 2..4 {
            if let Some(mut partition_entry) = extended_boot_record.partition_entries.get(index) {
                if partition_entry.partition_type != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported partition entry: {} - unsupported type: 0x{:02x}",
                        index, partition_entry.partition_type
                    )));
                }
            }
        }
        if let Some(mut partition_entry) = extended_boot_record.partition_entries.pop_front() {
            if partition_entry.partition_type == 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported partition entry: 0 - unsupported type: 0x{:02x}",
                    partition_entry.partition_type
                )));
            }
            partition_entry.index = first_entry_index;
            partition_entry.start_address_lba += current_extended_boot_record_lba;

            self.partition_entries.push(partition_entry);
        }
        if next_extended_boot_record_offset != 0 {
            match self.read_extended_boot_record(
                data_stream,
                next_extended_boot_record_offset,
                first_entry_index + 4,
                first_extended_boot_record_lba,
                next_extended_boot_record_lba,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read next extended boot record"
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Sets the number of bytes per sector.
    pub fn set_bytes_per_sector(&mut self, bytes_per_sector: u32) -> Result<(), ErrorTrace> {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<MbrVolumeSystem, ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
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
    fn test_partitions() -> Result<(), ErrorTrace> {
        let volume_system: MbrVolumeSystem = get_volume_system()?;

        let mut partitions_iterator: MbrPartitionsIterator = volume_system.partitions();

        let result: Option<Result<MbrPartition, ErrorTrace>> = partitions_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<MbrPartition, ErrorTrace>> = partitions_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    #[test]
    fn test_read_boot_records() -> Result<(), ErrorTrace> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_boot_records(&data_stream)?;

        assert_eq!(volume_system.get_number_of_partitions(), 2);

        Ok(())
    }

    // TODO: add tests for read_extended_boot_record
    // TODO: add tests for set_bytes_per_sector
}
