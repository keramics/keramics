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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::Ucs2String;

#[derive(Debug, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "sequence_number", data_type = "u8"),
        field(name = "name_segment1", data_type = "Ucs2String<5>"),
        field(name = "unknown1", data_type = "[u8; 2]", format = "hex"),
        field(name = "name_checksum", data_type = "u8", format = "hex"),
        field(name = "name_segment2", data_type = "Ucs2String<6>"),
        field(name = "unknown2", data_type = "[u8; 2]", format = "hex"),
        field(name = "name_segment3", data_type = "Ucs2String<2>"),
    ),
    method(name = "debug_read_data")
)]
/// File Allocation Table (FAT) long name directory entry.
pub struct FatLongNameDirectoryEntry {
    /// Sequence number
    pub sequence_number: u8,

    /// Name
    pub name: Ucs2String,
}

impl FatLongNameDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new() -> Self {
        Self {
            sequence_number: 0,
            name: Ucs2String::new(),
        }
    }

    /// Reads the directory entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported directory entry data size"
            ));
        }
        self.sequence_number = data[0];

        self.name.read_data_le(&data[1..11]);

        if self.name.len() == 5 || data[14..16] != [0xff, 0xff] {
            self.name.read_data_le(&data[14..26]);
        }
        if self.name.len() == 11 || data[28..30] != [0xff, 0xff] {
            self.name.read_data_le(&data[28..32]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x41, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x0f, 0x00, 0x81,
            0x69, 0x00, 0x72, 0x00, 0x31, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = FatLongNameDirectoryEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.sequence_number, 0x41);
        assert_eq!(test_struct.name, Ucs2String::from("testdir1"));

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = FatLongNameDirectoryEntry::new();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }
}
