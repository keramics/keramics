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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_encodings::CharacterEncoding;
use keramics_types::{ByteString, Ucs2String};

use super::enums::FatFormat;
use super::short_name_directory_entry_fat12::Fat12ShortNameDirectoryEntry;
use super::short_name_directory_entry_fat32::Fat32ShortNameDirectoryEntry;

#[derive(Clone)]
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
    pub data_start_cluster: u32,

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

    /// Retrieves the lookup name.
    pub fn get_lookup_name(&self) -> Ucs2String {
        let elements: Vec<u16> = self
            .name
            .elements
            .iter()
            .map(|element| {
                if *element >= b'a' && *element <= b'z' {
                    (*element - 32) as u16
                } else {
                    *element as u16
                }
            })
            .collect();

        Ucs2String { elements }
    }

    /// Reads the directory entry from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
        format: &FatFormat,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; 32];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("FatShortNameDirectoryEntry", offset, &data, 32);

        if format == &FatFormat::Fat32 {
            keramics_core::debug_trace_structure!(Fat32ShortNameDirectoryEntry::debug_read_data(
                &data
            ));
            match Fat32ShortNameDirectoryEntry::read_data(self, &data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read FAT-32 short name directory entry"
                    );
                    return Err(error);
                }
            }
        } else {
            keramics_core::debug_trace_structure!(Fat12ShortNameDirectoryEntry::debug_read_data(
                &data
            ));
            match Fat12ShortNameDirectoryEntry::read_data(self, &data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read FAT-12 or FAT-16 short name directory entry"
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data_fat12() -> Vec<u8> {
        return vec![
            0x54, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10, 0x00, 0x7d,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    // TODO: add tests for get_lookup_name

    #[test]
    fn test_read_at_position_fat12() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_fat12();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = FatShortNameDirectoryEntry::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0), &FatFormat::Fat12)?;

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
