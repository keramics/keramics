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

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::ByteString;

use super::partition_map_entry::ApmPartitionMapEntry;

/// Apple Partition Map (APM) partition.
pub struct ApmPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// The current offset.
    current_offset: u64,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The size of the partition.
    pub(super) size: u64,

    /// The partition type identifier.
    type_identifier: ByteString,

    /// The name.
    name: ByteString,

    /// The status flags.
    status_flags: u32,
}

impl ApmPartition {
    /// Creates a new partition.
    pub(super) fn new(data_stream: &DataStreamReference, bytes_per_sector: u16) -> Self {
        Self {
            data_stream: data_stream.clone(),
            bytes_per_sector,
            current_offset: 0,
            offset: 0,
            size: 0,
            type_identifier: ByteString::new(),
            name: ByteString::new(),
            status_flags: 0,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &ByteString {
        &self.name
    }

    /// Retrieves the partition offset.
    pub fn get_partition_offset(&self) -> u64 {
        self.offset
    }

    /// Retrieves the partition size.
    pub fn get_partition_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the status flags.
    pub fn get_status_flags(&self) -> u32 {
        self.status_flags
    }

    /// Retrieves the type identifier.
    pub fn get_type_identifier(&self) -> &ByteString {
        &self.type_identifier
    }

    /// Opens a partition.
    pub(super) fn open(
        &mut self,
        partition_entry: &ApmPartitionMapEntry,
    ) -> Result<(), ErrorTrace> {
        self.offset = (partition_entry.start_sector as u64) * (self.bytes_per_sector as u64);
        self.size = (partition_entry.number_of_sectors as u64) * (self.bytes_per_sector as u64);
        self.type_identifier = partition_entry.type_identifier.clone();
        self.name = partition_entry.name.clone();
        self.status_flags = partition_entry.status_flags;

        Ok(())
    }
}

impl DataStream for ApmPartition {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = keramics_core::data_stream_read_at_position!(
            &self.data_stream,
            &mut buf[0..read_size],
            SeekFrom::Start(self.offset + self.current_offset)
        );
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset value out of bounds"
                    ));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<ApmPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();
        partition_entry.start_sector = 64;
        partition_entry.number_of_sectors = 8112;
        partition_entry.name = ByteString::from("identifier");
        partition_entry.type_identifier = ByteString::from("type_identifier");
        partition_entry.status_flags = 0x40000033;

        let mut partition = ApmPartition::new(&data_stream, 512);
        partition.open(&partition_entry)?;

        Ok(partition)
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();
        partition_entry.start_sector = 64;
        partition_entry.number_of_sectors = 8112;
        partition_entry.name = ByteString::from("identifier");
        partition_entry.type_identifier = ByteString::from("type_identifier");
        partition_entry.status_flags = 0x40000033;

        let mut partition = ApmPartition::new(&data_stream, 512);
        partition.open(&partition_entry)?;

        Ok(())
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        partition.seek(SeekFrom::Start(1024))?;

        let offset: u64 = partition.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let size: u64 = partition.get_size()?;
        assert_eq!(size, 4153344);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let offset: u64 = partition.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let offset: u64 = partition.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, partition.size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let offset = partition.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = partition.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let result: Result<u64, ErrorTrace> = partition.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;

        let offset: u64 = partition.seek(SeekFrom::End(512))?;
        assert_eq!(offset, partition.size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;
        partition.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = partition.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x48, 0x2b, 0x00, 0x04, 0x00, 0x00, 0x01, 0x00, 0x31, 0x30, 0x2e, 0x30, 0x00, 0x00,
            0x00, 0x00, 0xdd, 0x46, 0x8d, 0xdf, 0xdd, 0x46, 0x71, 0xc2, 0x00, 0x00, 0x00, 0x00,
            0xdd, 0x46, 0x71, 0xbf, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x03, 0xf6, 0x00, 0x00, 0x02, 0xdf, 0x00, 0x00, 0x01, 0x65,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xbb, 0x3e, 0x73, 0x0f, 0x40, 0xa9, 0x79, 0xed,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00,
            0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00,
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x63, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut partition: ApmPartition = get_partition()?;
        partition.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = partition.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
