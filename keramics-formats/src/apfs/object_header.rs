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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

#[derive(Clone, LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "object_checksum", data_type = "u64", format = "hex"),
        field(name = "object_identifier", data_type = "u64"),
        field(name = "object_transaction_identifier", data_type = "u64"),
        field(name = "object_type", data_type = "u32", format = "hex"),
        field(name = "object_subtype", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Apple File System (APFS) object header.
pub struct ApfsObjectHeader {
    /// Checksum.
    pub checksum: u64,

    /// Identifier.
    pub identifier: u64,

    /// Transaction identifier.
    pub transaction_identifier: u64,

    /// Object type.
    pub object_type: u32,

    /// Object subtype.
    pub object_subtype: u32,
}

impl ApfsObjectHeader {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            checksum: 0,
            identifier: 0,
            transaction_identifier: 0,
            object_type: 0,
            object_subtype: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.checksum = bytes_to_u64_le!(data, 0);
        self.identifier = bytes_to_u64_le!(data, 8);
        self.transaction_identifier = bytes_to_u64_le!(data, 16);
        self.object_type = bytes_to_u32_le!(data, 24);
        self.object_subtype = bytes_to_u32_le!(data, 28);

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
            0xea, 0xd3, 0x7e, 0x6e, 0x43, 0xdb, 0x37, 0x98, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x4e, 0x58, 0x53, 0x42,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsObjectHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.checksum, 0x9837db436e7ed3ea);
        assert_eq!(test_struct.identifier, 1);
        assert_eq!(test_struct.transaction_identifier, 6);
        assert_eq!(test_struct.object_type, 0x80000001);
        assert_eq!(test_struct.object_subtype, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsObjectHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = ApfsObjectHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.checksum, 0x9837db436e7ed3ea);
        assert_eq!(test_struct.identifier, 1);
        assert_eq!(test_struct.transaction_identifier, 6);
        assert_eq!(test_struct.object_type, 0x80000001);
        assert_eq!(test_struct.object_subtype, 0x00000000);

        Ok(())
    }
}
