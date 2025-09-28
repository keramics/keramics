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

use keramics_types::{Utf16String, Uuid, bytes_to_u64_le};
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "type_identifier", data_type = "uuid"),
        field(name = "identifier", data_type = "uuid"),
        field(name = "start_block_number", data_type = "u64"),
        field(name = "end_block_number", data_type = "u64"),
        field(name = "attribute_flags", data_type = "u64", format = "hex"),
        field(name = "name", data_type = "Utf16String<36>"),
    ),
    method(name = "debug_read_data")
)]
/// GUID Partition Table (GPT) partition entry.
pub struct GptPartitionEntry {
    pub index: usize,
    pub type_identifier: Uuid,
    pub identifier: Uuid,
    pub start_block_number: u64,
    pub end_block_number: u64,
    pub attribute_flags: u64,
    pub name: Utf16String,
}

impl GptPartitionEntry {
    /// Creates a new partition entry.
    pub fn new(index: usize) -> Self {
        Self {
            index: index,
            type_identifier: Uuid::new(),
            identifier: Uuid::new(),
            start_block_number: 0,
            end_block_number: 0,
            attribute_flags: 0,
            name: Utf16String::new(),
        }
    }

    /// Reads the partition entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported GPT partition entry data size"),
            ));
        }
        self.type_identifier = Uuid::from_le_bytes(&data[0..16]);
        self.identifier = Uuid::from_le_bytes(&data[16..32]);
        self.start_block_number = bytes_to_u64_le!(data, 32);
        self.end_block_number = bytes_to_u64_le!(data, 40);
        self.attribute_flags = bytes_to_u64_le!(data, 48);
        self.name = Utf16String::from_le_bytes(&data[56..128]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xaf, 0x3d, 0xc6, 0x0f, 0x83, 0x84, 0x72, 0x47, 0x8e, 0x79, 0x3d, 0x69, 0xd8, 0x47,
            0x7d, 0xe4, 0x8c, 0x58, 0x25, 0x1e, 0xa9, 0x27, 0x94, 0x40, 0x86, 0x8c, 0x2f, 0x25,
            0x70, 0x21, 0xf8, 0x7b, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x4c, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x75, 0x00, 0x78, 0x00, 0x20, 0x00, 0x66, 0x00,
            0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x73, 0x00, 0x79, 0x00, 0x73, 0x00, 0x74, 0x00,
            0x65, 0x00, 0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = GptPartitionEntry::new(1);
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.type_identifier.to_string(),
            "0fc63daf-8483-4772-8e79-3d69d8477de4"
        );
        assert_eq!(
            test_struct.identifier.to_string(),
            "1e25588c-27a9-4094-868c-2f257021f87b"
        );
        assert_eq!(test_struct.start_block_number, 2048);
        assert_eq!(test_struct.end_block_number, 2175);
        assert_eq!(test_struct.attribute_flags, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = GptPartitionEntry::new(1);
        let result = test_struct.read_data(&test_data[0..127]);
        assert!(result.is_err());
    }
}
