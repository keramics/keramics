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

use layout_map::LayoutMap;
use types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "signature", data_type = "ByteString<4>")),
        member(field(name = "fixup_values_offset", data_type = "u16")),
        member(field(name = "number_of_fixup_values", data_type = "u16")),
        member(field(name = "journal_sequence_number", data_type = "u64")),
        member(field(name = "sequence_number", data_type = "u16")),
        member(field(name = "reference_count", data_type = "u16")),
        member(field(name = "attributes_offset", data_type = "u16")),
        member(field(name = "flags", data_type = "u16")),
        member(field(name = "used_size", data_type = "u32")),
        member(field(name = "mft_entry_size", data_type = "u32")),
        member(field(name = "base_record_file_reference", data_type = "u64")),
        member(field(name = "first_available_attribute_identifier", data_type = "u16")),
        member(field(name = "unknown1", data_type = "[u8; 2]")),
        member(field(name = "mft_entry_number", data_type = "u32")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) Master File Table (MFT) entry header.
pub struct NtfsMftEntryHeader {
    /// Fix-up values offset.
    pub fixup_values_offset: u16,

    /// Number of fix-up values.
    pub number_of_fixup_values: u16,

    /// Journal sequence number.
    pub journal_sequence_number: u64,

    /// Sequnce number.
    pub sequence_number: u16,

    /// Attributes offset.
    pub attributes_offset: u16,

    /// MFT entry size.
    pub mft_entry_size: u32,

    /// Base record file reference.
    pub base_record_file_reference: u64,
}

impl NtfsMftEntryHeader {
    /// Creates a new MFT entry header.
    pub fn new() -> Self {
        Self {
            fixup_values_offset: 0,
            number_of_fixup_values: 0,
            journal_sequence_number: 0,
            sequence_number: 0,
            attributes_offset: 0,
            mft_entry_size: 0,
            base_record_file_reference: 0,
        }
    }

    /// Reads the MFT entry header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 42 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != NTFS_MFT_ENTRY_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        self.fixup_values_offset = bytes_to_u16_le!(data, 4);
        self.number_of_fixup_values = bytes_to_u16_le!(data, 6);
        self.journal_sequence_number = bytes_to_u64_le!(data, 8);
        self.sequence_number = bytes_to_u16_le!(data, 16);
        self.attributes_offset = bytes_to_u16_le!(data, 20);
        self.mft_entry_size = bytes_to_u32_le!(data, 28);
        self.base_record_file_reference = bytes_to_u64_le!(data, 32);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x46, 0x49, 0x4c, 0x45, 0x30, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x38, 0x00, 0x01, 0x00, 0x98, 0x01, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsMftEntryHeader::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.fixup_values_offset, 48);
        assert_eq!(test_struct.number_of_fixup_values, 3);
        assert_eq!(test_struct.journal_sequence_number, 0);
        assert_eq!(test_struct.sequence_number, 1);
        assert_eq!(test_struct.attributes_offset, 56);
        assert_eq!(test_struct.mft_entry_size, 1024);
        assert_eq!(test_struct.base_record_file_reference, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftEntryHeader::new();
        let result = test_struct.read_data(&test_data[0..41]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = NtfsMftEntryHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
