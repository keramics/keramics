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
use keramics_datetime::{DateTime, FatTimeDate, FatTimeDate10Ms};
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u16_le;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_code", data_type = "u8", format = "hex"),
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "set_checksum", data_type = "u16"),
        field(name = "file_attribute_flags", data_type = "u16", format = "hex"),
        field(name = "unknown2", data_type = "u16"),
        field(name = "creation_time", data_type = "FatTimeDate"),
        field(name = "modification_time", data_type = "FatTimeDate"),
        field(name = "access_time", data_type = "FatTimeDate"),
        field(name = "creation_time_fraction", data_type = "u8"),
        field(name = "modification_time_fraction", data_type = "u8"),
        field(name = "creation_time_utc_offset", data_type = "u8", format = "hex"),
        field(
            name = "modification_time_utc_offset",
            data_type = "u8",
            format = "hex"
        ),
        field(name = "access_time_utc_offset", data_type = "u8", format = "hex"),
        field(name = "unknown4", data_type = "[u8; 7]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Extensible File Allocation Table (exFAT) file entry (directory) record.
pub struct ExFatFileEntryRecord {
    /// File attribute flags.
    pub file_attribute_flags: u16,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Access date and time.
    pub access_time: DateTime,

    /// Modifiation date and time.
    pub modification_time: DateTime,
}

impl ExFatFileEntryRecord {
    /// Creates a new file entry record.
    pub fn new() -> Self {
        Self {
            file_attribute_flags: 0,
            creation_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
        }
    }

    /// Reads the file entry record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let type_code: u8 = data[0];

        if type_code != 0x85 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported type code: 0x{:02x}",
                type_code
            )));
        }
        self.file_attribute_flags = bytes_to_u16_le!(data, 4);

        if &data[8..12] == &[0; 4] && data[20] == 0 {
            self.creation_time = DateTime::NotSet;
        } else {
            let fat_time: u16 = bytes_to_u16_le!(data, 8);
            let fat_date: u16 = bytes_to_u16_le!(data, 10);
            let fraction: u8 = data[20];

            let mut fat_time_date: FatTimeDate10Ms =
                FatTimeDate10Ms::new(fat_date, fat_time, fraction);
            fat_time_date.set_utc_offset(data[22]);

            self.creation_time = DateTime::FatTimeDate10Ms(fat_time_date);
        }
        if &data[12..16] == &[0; 4] && data[21] == 0 {
            self.modification_time = DateTime::NotSet;
        } else {
            let fat_time: u16 = bytes_to_u16_le!(data, 12);
            let fat_date: u16 = bytes_to_u16_le!(data, 14);
            let fraction: u8 = data[21];

            let mut fat_time_date: FatTimeDate10Ms =
                FatTimeDate10Ms::new(fat_date, fat_time, fraction);
            fat_time_date.set_utc_offset(data[23]);

            self.modification_time = DateTime::FatTimeDate10Ms(fat_time_date);
        }
        if &data[16..20] == &[0; 4] {
            self.access_time = DateTime::NotSet;
        } else {
            let mut fat_time_date: FatTimeDate = FatTimeDate::from_bytes(&data[16..20]);
            fat_time_date.set_utc_offset(data[24]);

            self.access_time = DateTime::FatTimeDate(fat_time_date);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        vec![
            0x85, 0x02, 0xeb, 0x8b, 0x20, 0x00, 0x00, 0x00, 0xcd, 0x62, 0x15, 0x5d, 0xcd, 0x62,
            0x15, 0x5d, 0xcd, 0x62, 0x15, 0x5d, 0x25, 0x25, 0x80, 0x80, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatFileEntryRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.file_attribute_flags, 0x0020);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatFileEntryRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_type_code() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = ExFatFileEntryRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = ExFatFileEntryRecord::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.file_attribute_flags, 0x0020);

        Ok(())
    }
}
