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

use keramics_types::bytes_to_u16_le;
use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "signature", data_type = "ByteString<4>")),
        member(field(name = "fixup_values_offset", data_type = "u16")),
        member(field(name = "number_of_fixup_values", data_type = "u16")),
        member(field(name = "journal_sequence_number", data_type = "u64")),
        member(field(name = "vcn", data_type = "u64")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) index entry header.
pub struct NtfsIndexEntryHeader {
    /// Fix-up values offset.
    pub fixup_values_offset: u16,

    /// Number of fix-up values.
    pub number_of_fixup_values: u16,
}

impl NtfsIndexEntryHeader {
    /// Creates a new index entry header.
    pub fn new() -> Self {
        Self {
            fixup_values_offset: 0,
            number_of_fixup_values: 0,
        }
    }

    /// Reads the index entry header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 24 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != NTFS_INDEX_ENTRY_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported NTFS index entry header signature"),
            ));
        }
        self.fixup_values_offset = bytes_to_u16_le!(data, 4);
        self.number_of_fixup_values = bytes_to_u16_le!(data, 6);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x49, 0x4e, 0x44, 0x58, 0x28, 0x00, 0x09, 0x00, 0xc1, 0xa9, 0x1b, 0x19, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsIndexEntryHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.fixup_values_offset, 40);
        assert_eq!(test_struct.number_of_fixup_values, 9);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsIndexEntryHeader::new();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = NtfsIndexEntryHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
