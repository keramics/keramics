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
use keramics_datetime::{DateTime, FatDate, FatTimeDate, FatTimeDate10Ms};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

use super::constants::*;
use super::short_name_directory_entry::FatShortNameDirectoryEntry;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "name", data_type = "ByteString<8>"),
        field(name = "extension", data_type = "ByteString<3>"),
        field(name = "file_attribute_flags", data_type = "u8", format = "hex"),
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "creation_time", data_type = "FatTimeDate10Ms"),
        field(name = "access_date", data_type = "FatDate"),
        field(name = "data_start_cluster_upport", data_type = "u16"),
        field(name = "modification_time", data_type = "FatTimeDate"),
        field(name = "data_start_cluster_lower", data_type = "u16"),
        field(name = "data_size", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// File Allocation Table (FAT-32) short name directory entry.
pub struct Fat32ShortNameDirectoryEntry {}

impl Fat32ShortNameDirectoryEntry {
    /// Reads the directory entry from a buffer.
    pub fn read_data(
        directory_entry: &mut FatShortNameDirectoryEntry,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported directory entry data size"
            ));
        }
        directory_entry.file_attribute_flags = data[11];
        directory_entry.flags = data[12];

        let slice: &[u8] = match data[0..8].iter().rev().position(|value| *value != b' ') {
            Some(data_index) => &data[0..8 - data_index],
            None => &data[0..8],
        };
        for byte_value in slice.iter() {
            if directory_entry.flags & 0x08 != 0 && *byte_value >= b'A' && *byte_value <= b'Z' {
                directory_entry.name.elements.push(*byte_value + 32);
            } else {
                directory_entry.name.elements.push(*byte_value);
            }
        }
        if data[8] != 0 && data[8] != b' ' {
            // Do not add an extension separator for a volume label.
            if directory_entry.file_attribute_flags & 0x58 != FAT_FILE_ATTRIBUTE_FLAG_VOLUME_LABEL {
                directory_entry.name.elements.push(b'.');
            }
            let slice: &[u8] = match data[8..11].iter().rev().position(|value| *value != b' ') {
                Some(data_index) => &data[8..11 - data_index],
                None => &data[8..11],
            };
            for byte_value in slice.iter() {
                if directory_entry.flags & 0x10 != 0 && *byte_value >= b'A' && *byte_value <= b'Z' {
                    directory_entry.name.elements.push(*byte_value + 32);
                } else {
                    directory_entry.name.elements.push(*byte_value);
                }
            }
        }
        directory_entry.creation_time =
            DateTime::FatTimeDate10Ms(FatTimeDate10Ms::from_bytes(&data[13..18]));
        directory_entry.access_time = DateTime::FatDate(FatDate::from_bytes(&data[18..20]));
        directory_entry.modification_time =
            DateTime::FatTimeDate(FatTimeDate::from_bytes(&data[22..26]));

        let lower_16bit: u16 = bytes_to_u16_le!(data, 26);
        let upper_16bit: u16 = bytes_to_u16_le!(data, 20);
        directory_entry.data_start_cluster = ((upper_16bit as u32) << 16) | (lower_16bit as u32);

        directory_entry.data_size = bytes_to_u32_le!(data, 28);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_encodings::CharacterEncoding;
    use keramics_types::ByteString;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x54, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10, 0x00, 0x7d,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct: FatShortNameDirectoryEntry = FatShortNameDirectoryEntry::new();
        Fat32ShortNameDirectoryEntry::read_data(&mut test_struct, &test_data)?;

        assert_eq!(
            test_struct.name,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'T', b'E', b'S', b'T', b'D', b'I', b'R', b'1'],
            }
        );
        assert_eq!(test_struct.file_attribute_flags, 0x10);
        assert_eq!(test_struct.flags, 0x00);
        assert_eq!(
            test_struct.creation_time,
            DateTime::FatTimeDate10Ms(FatTimeDate10Ms::new(0x5b53, 0x958f, 0x7d))
        );
        assert_eq!(
            test_struct.access_time,
            DateTime::FatDate(FatDate::new(0x5b53))
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::FatTimeDate(FatTimeDate::new(0x5b53, 0x958f))
        );
        assert_eq!(test_struct.data_start_cluster, 3);
        assert_eq!(test_struct.data_size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct: FatShortNameDirectoryEntry = FatShortNameDirectoryEntry::new();
        let result = Fat32ShortNameDirectoryEntry::read_data(&mut test_struct, &test_data[0..31]);
        assert!(result.is_err());
    }
}
