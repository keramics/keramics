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

use keramics_core::ErrorTrace;
use keramics_datetime::{DateTime, PosixTime32};
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u16_le;

use super::file_record::CpioFileRecord;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "u16"),
        field(name = "file_system_identifier", data_type = "u16"),
        field(name = "inode_number", data_type = "u16"),
        field(name = "file_mode", data_type = "u16"),
        field(name = "owner_identifier", data_type = "u16"),
        field(name = "group_identifier", data_type = "u16"),
        field(name = "number_of_links", data_type = "u16"),
        field(name = "device_identifier", data_type = "u16"),
        field(name = "modification_time_upper", data_type = "u16"),
        field(name = "modification_time_lower", data_type = "u16"),
        field(name = "path_size", data_type = "u16"),
        field(name = "data_size_upper", data_type = "u16"),
        field(name = "data_size_lower", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Copy in and out (CPIO) little-endian binary file record
pub struct CpioBinaryFileRecordLittleEndian {}

impl CpioBinaryFileRecordLittleEndian {
    /// Reads the record from a buffer.
    pub fn read_data(file_record: &mut CpioFileRecord, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 26 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..2] != &[0xc7, 0x71] {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        file_record.inode_number = bytes_to_u16_le!(data, 4) as u32;
        file_record.file_mode = bytes_to_u16_le!(data, 6) as u32;
        file_record.owner_identifier = bytes_to_u16_le!(data, 8) as u32;
        file_record.group_identifier = bytes_to_u16_le!(data, 10) as u32;
        file_record.number_of_links = bytes_to_u16_le!(data, 12) as u32;
        file_record.device_identifier = bytes_to_u16_le!(data, 14) as u32;

        let value_16bit_upper: u32 = bytes_to_u16_le!(data, 16) as u32;
        let value_16bit_lower: u32 = bytes_to_u16_le!(data, 18) as u32;
        let timestamp: u32 = (value_16bit_upper << 16) | value_16bit_lower;

        file_record.modification_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::PosixTime32(PosixTime32::new(timestamp as i32))
        };
        file_record.path_size = bytes_to_u16_le!(data, 20) as u32;

        let value_16bit_upper: u32 = bytes_to_u16_le!(data, 22) as u32;
        let value_16bit_lower: u32 = bytes_to_u16_le!(data, 24) as u32;
        file_record.data_size = (value_16bit_upper << 16) | value_16bit_lower;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xc7, 0x71, 0x00, 0x07, 0x12, 0x00, 0xa4, 0x81, 0xe8, 0x03, 0xe8, 0x03, 0x01, 0x00,
            0x00, 0x00, 0x78, 0x67, 0x09, 0xea, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x6d,
            0x6e, 0x74, 0x2f, 0x6b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73, 0x2f, 0x6e, 0x66,
            0x63, 0x5f, 0x74, 0xc3, 0xa9, 0x73, 0x74, 0x66, 0x69, 0x6c, 0xc3, 0xa8, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        CpioBinaryFileRecordLittleEndian::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.inode_number, 18);
        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.device_identifier, 0);
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            })
        );
        assert_eq!(test_struct.path_size, 29);
        assert_eq!(test_struct.data_offset, 0);
        assert_eq!(test_struct.data_size, 0);
        assert_eq!(test_struct.checksum, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        let result: Result<(), ErrorTrace> =
            CpioBinaryFileRecordLittleEndian::read_data(&mut test_struct, &test_data[0..25]);
        assert!(result.is_err());
    }
}
