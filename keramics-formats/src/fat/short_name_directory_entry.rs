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
use keramics_encodings::CharacterEncoding;
use keramics_layout_map::LayoutMap;
use keramics_types::{ByteString, bytes_to_u16_le, bytes_to_u32_le};

use super::constants::*;

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "name", data_type = "ByteString<8>"),
        field(name = "extension", data_type = "ByteString<3>"),
        field(name = "file_attribute_flags", data_type = "u8", format = "hex"),
        field(name = "flags", data_type = "u8", format = "hex"),
        field(name = "creation_time", data_type = "FatTimeDate10Ms"),
        field(name = "access_date", data_type = "FatDate"),
        field(name = "unknown2", data_type = "[u8; 2]", format = "hex"),
        field(name = "modification_time", data_type = "FatTimeDate"),
        field(name = "data_start_cluster", data_type = "u16"),
        field(name = "data_size", data_type = "u32"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// File Allocation Table (FAT) short name directory entry.
pub struct FatShortNameDirectoryEntry {
    /// Name
    pub name: ByteString,

    /// File attribute flags.
    pub file_attribute_flags: u8,

    /// Flags.
    pub flags: u8,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Access date and time.
    pub access_time: DateTime,

    /// Modifiation date and time.
    pub modification_time: DateTime,

    /// Data start cluster.
    pub data_start_cluster: u16,

    /// Data size.
    pub data_size: u32,
}

impl FatShortNameDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new() -> Self {
        Self {
            name: ByteString::new_with_encoding(&CharacterEncoding::Ascii),
            file_attribute_flags: 0,
            flags: 0,
            creation_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            data_start_cluster: 0,
            data_size: 0,
        }
    }

    /// Reads the directory entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported directory entry data size"
            ));
        }
        self.file_attribute_flags = data[11];
        self.flags = data[12];

        let slice: &[u8] = match data[0..8].iter().rev().position(|value| *value != b' ') {
            Some(data_index) => &data[0..8 - data_index],
            None => &data[0..8],
        };
        for byte_value in slice.iter() {
            if self.flags & 0x08 != 0 && *byte_value >= b'A' && *byte_value <= b'Z' {
                self.name.elements.push(*byte_value + 32);
            } else {
                self.name.elements.push(*byte_value);
            }
        }
        if data[8] != 0 && data[8] != b' ' {
            // Do not add an extension separator for a volume label.
            if self.file_attribute_flags & 0x58 != FAT_FILE_ATTRIBUTE_FLAG_VOLUME_LABEL {
                self.name.elements.push(b'.');
            }
            let slice: &[u8] = match data[8..11].iter().rev().position(|value| *value != b' ') {
                Some(data_index) => &data[8..11 - data_index],
                None => &data[8..11],
            };
            for byte_value in slice.iter() {
                if self.flags & 0x10 != 0 && *byte_value >= b'A' && *byte_value <= b'Z' {
                    self.name.elements.push(*byte_value + 32);
                } else {
                    self.name.elements.push(*byte_value);
                }
            }
        }
        self.creation_time = DateTime::FatTimeDate10Ms(FatTimeDate10Ms::from_bytes(&data[13..18]));
        self.access_time = DateTime::FatDate(FatDate::from_bytes(&data[18..20]));
        self.modification_time = DateTime::FatTimeDate(FatTimeDate::from_bytes(&data[22..26]));

        self.data_start_cluster = bytes_to_u16_le!(data, 26);
        self.data_size = bytes_to_u32_le!(data, 28);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

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

        let mut test_struct = FatShortNameDirectoryEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.name,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'T', b'E', b'S', b'T', b'D', b'I', b'R', b'1'],
            }
        );
        assert_eq!(test_struct.file_attribute_flags, 0x10);
        assert_eq!(test_struct.flags, 0x00);
        assert_eq!(test_struct.data_start_cluster, 3);
        assert_eq!(test_struct.data_size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = FatShortNameDirectoryEntry::new();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = FatShortNameDirectoryEntry::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(
            test_struct.name,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'T', b'E', b'S', b'T', b'D', b'I', b'R', b'1'],
            }
        );
        assert_eq!(test_struct.file_attribute_flags, 0x10);
        assert_eq!(test_struct.data_start_cluster, 3);
        assert_eq!(test_struct.data_size, 0);

        Ok(())
    }
}
