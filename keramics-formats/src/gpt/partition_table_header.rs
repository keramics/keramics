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

use keramics_checksums::ReversedCrc32Context;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le, Uuid};
use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<8>"),
        field(name = "minor_format_version", data_type = "u16"),
        field(name = "major_format_version", data_type = "u16"),
        field(name = "header_data_size", data_type = "u32"),
        field(name = "header_data_checksum", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "header_block_number", data_type = "u64"),
        field(name = "backup_header_block_number", data_type = "u64"),
        field(name = "area_start_block_number", data_type = "u64"),
        field(name = "area_end_block_number", data_type = "u64"),
        field(name = "disk_identifier", data_type = "uuid"),
        field(name = "entries_start_block_number", data_type = "u64"),
        field(name = "number_of_entries", data_type = "u32"),
        field(name = "entry_data_size", data_type = "u32"),
        field(name = "entries_data_checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// GUID Partition Table (GPT) partition table header.
pub struct GptPartitionTableHeader {
    pub backup_header_block_number: u64,
    pub area_start_block_number: u64,
    pub area_end_block_number: u64,
    pub disk_identifier: Uuid,
    pub entries_start_block_number: u64,
    pub number_of_entries: u32,
    pub entry_data_size: u32,
    pub entries_data_checksum: u32,
}

impl GptPartitionTableHeader {
    /// Creates a new partition table header.
    pub fn new() -> Self {
        Self {
            backup_header_block_number: 0,
            area_start_block_number: 0,
            area_end_block_number: 0,
            disk_identifier: Uuid::new(),
            entries_start_block_number: 0,
            number_of_entries: 0,
            entry_data_size: 0,
            entries_data_checksum: 0,
        }
    }

    /// Reads the partition table header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 92 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..8] != GPT_PARTITION_TABLE_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        let format_version: u32 = bytes_to_u32_le!(data, 8);

        if format_version != 0x00010000 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported format version: 0x{:08x}", format_version),
            ));
        }
        let header_data_size: u32 = bytes_to_u32_le!(data, 12);

        if header_data_size != 92 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported header data size: {}", header_data_size),
            ));
        }
        let stored_checksum: u32 = bytes_to_u32_le!(data, 16);

        let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);
        crc32_context.update(&data[0..16]);
        crc32_context.update(&[0; 4]);
        crc32_context.update(&data[20..header_data_size as usize]);

        let calculated_checksum: u32 = crc32_context.finalize();

        if stored_checksum != 0 && stored_checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    stored_checksum, calculated_checksum
                ),
            ));
        }
        self.backup_header_block_number = bytes_to_u64_le!(data, 32);
        self.area_start_block_number = bytes_to_u64_le!(data, 40);
        self.area_end_block_number = bytes_to_u64_le!(data, 48);
        self.disk_identifier = Uuid::from_le_bytes(&data[56..72]);
        self.entries_start_block_number = bytes_to_u64_le!(data, 72);
        self.number_of_entries = bytes_to_u32_le!(data, 80);
        self.entry_data_size = bytes_to_u32_le!(data, 84);
        self.entries_data_checksum = bytes_to_u32_le!(data, 88);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54, 0x00, 0x00, 0x01, 0x00, 0x5c, 0x00,
            0x00, 0x00, 0x35, 0x50, 0xdc, 0x20, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xde, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x7a, 0x65, 0x6e, 0xe8, 0x40, 0xd8, 0x09, 0x4c, 0xaf, 0xe3, 0xa1, 0xa5, 0xf6, 0x65,
            0xcf, 0x44, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x80, 0x00, 0x00, 0x00, 0x1e, 0xf9, 0xb8, 0xac,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = GptPartitionTableHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.backup_header_block_number, 8191);
        assert_eq!(test_struct.area_start_block_number, 34);
        assert_eq!(test_struct.area_end_block_number, 8158);
        assert_eq!(
            test_struct.disk_identifier.to_string(),
            "e86e657a-d840-4c09-afe3-a1a5f665cf44"
        );
        assert_eq!(test_struct.entries_start_block_number, 2);
        assert_eq!(test_struct.number_of_entries, 128);
        assert_eq!(test_struct.entry_data_size, 128);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = GptPartitionTableHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..91]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[8] = 0xff;

        let mut test_struct = GptPartitionTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_header_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[12] = 0xff;

        let mut test_struct = GptPartitionTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_checksum_mismatch() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[16] = 0xff;

        let mut test_struct = GptPartitionTableHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
