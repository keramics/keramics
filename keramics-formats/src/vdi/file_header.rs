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
use keramics_types::{Uuid, bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "pre_header", data_type = "ByteString<64>"),
        field(name = "signature", data_type = "[u8; 4]", format = "hex"),
        field(name = "minor_format_version", data_type = "u16"),
        field(name = "major_format_version", data_type = "u16"),
        field(name = "header_size", data_type = "u32"),
        field(name = "image_type", data_type = "u32"),
        field(name = "image_flags", data_type = "u32", format = "hex"),
        field(name = "description", data_type = "ByteString<256>"),
        field(name = "block_map_offset", data_type = "u32", format = "hex"),
        field(name = "data_offset", data_type = "u32", format = "hex"),
        field(name = "number_of_cylinders", data_type = "u32"),
        field(name = "number_of_heads", data_type = "u32"),
        field(name = "number_of_sectors", data_type = "u32"),
        field(name = "bytes_per_sector", data_type = "u32"),
        field(name = "unknown1", data_type = "[u8; 4]"),
        field(name = "data_size", data_type = "u64"),
        field(name = "block_size", data_type = "u32"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "number_of_blocks", data_type = "u32"),
        field(name = "number_of_allocated_blocks", data_type = "u32"),
        field(name = "identifier", data_type = "Uuid"),
        field(name = "snapshot_identifier", data_type = "Uuid"),
        field(name = "link_identifier", data_type = "Uuid"),
        field(name = "parent_identifier", data_type = "Uuid"),
        field(name = "unknown2", data_type = "[u8; 56]"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Virtual Disk Image (VDI) file header.
pub struct VdiFileHeader {
    /// Minor format version
    pub minor_format_version: u16,

    /// Major format version
    pub major_format_version: u16,

    /// Image type.
    pub image_type: u32,

    /// Block map offset.
    pub block_map_offset: u32,

    /// Data offset.
    pub data_offset: u32,

    /// Data size.
    pub data_size: u64,

    /// Block size.
    pub block_size: u32,

    /// Number of blocks.
    pub number_of_blocks: u32,

    /// Identifier.
    pub identifier: Uuid,

    /// Parent identifier.
    pub parent_identifier: Option<Uuid>,
}

impl VdiFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            minor_format_version: 0,
            major_format_version: 0,
            image_type: 0,
            block_map_offset: 0,
            data_offset: 0,
            data_size: 0,
            block_size: 0,
            number_of_blocks: 0,
            identifier: Uuid::new(),
            parent_identifier: None,
        }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 76 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[64..68] != VDI_FILE_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        // TODO: check format version

        let header_size: usize = bytes_to_u32_le!(data, 72) as usize;

        if header_size < 360 || header_size > data_size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported header size value out of bounds"
            ));
        }
        self.minor_format_version = bytes_to_u16_le!(data, 68);
        self.major_format_version = bytes_to_u16_le!(data, 70);
        self.image_type = bytes_to_u32_le!(data, 76);
        self.block_map_offset = bytes_to_u32_le!(data, 340);
        self.data_offset = bytes_to_u32_le!(data, 344);
        self.data_size = bytes_to_u64_le!(data, 368);
        self.block_size = bytes_to_u32_le!(data, 376);
        self.number_of_blocks = bytes_to_u32_le!(data, 384);
        self.identifier = Uuid::from_le_bytes(&data[392..408]);

        if header_size >= 392 {
            let parent_identifier: Uuid = Uuid::from_le_bytes(&data[440..456]);

            if !parent_identifier.is_nil() {
                self.parent_identifier = Some(parent_identifier);
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

    fn get_test_data() -> Vec<u8> {
        vec![
            0x3c, 0x3c, 0x3c, 0x20, 0x51, 0x45, 0x4d, 0x55, 0x20, 0x56, 0x4d, 0x20, 0x56, 0x69,
            0x72, 0x74, 0x75, 0x61, 0x6c, 0x20, 0x44, 0x69, 0x73, 0x6b, 0x20, 0x49, 0x6d, 0x61,
            0x67, 0x65, 0x20, 0x3e, 0x3e, 0x3e, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x10, 0xda, 0xbe, 0x01, 0x00,
            0x01, 0x00, 0x80, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0xe8, 0x7a, 0x43, 0x88, 0x31, 0x96, 0x48, 0x4c, 0x81, 0xc3, 0xe7, 0x87, 0xcc, 0x58,
            0x65, 0x01, 0x50, 0x36, 0xa0, 0x65, 0x9f, 0x9b, 0x2c, 0x46, 0x93, 0xb1, 0x34, 0xe1,
            0x6e, 0x13, 0x76, 0x76, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VdiFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.minor_format_version, 1);
        assert_eq!(test_struct.major_format_version, 1);
        assert_eq!(test_struct.image_type, 1);
        assert_eq!(test_struct.block_map_offset, 0x00000200);
        assert_eq!(test_struct.data_offset, 0x00000400);
        assert_eq!(test_struct.data_size, 4194304);
        assert_eq!(test_struct.block_size, 1048576);
        assert_eq!(test_struct.number_of_blocks, 4);
        assert_eq!(
            test_struct.identifier.to_string(),
            "88437ae8-9631-4c48-81c3-e787cc586501",
        );
        assert_eq!(test_struct.parent_identifier, None);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VdiFileHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..75]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[64] = 0xff;

        let mut test_struct = VdiFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_header_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[75] = 0xff;

        let mut test_struct = VdiFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = VdiFileHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        Ok(())
    }
}
