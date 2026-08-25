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
use keramics_types::bytes_to_u32_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "resource_data_offset", data_type = "u32", format = "hex"),
        field(name = "resource_map_offset", data_type = "u32", format = "hex"),
        field(name = "resource_data_size", data_type = "u32"),
        field(name = "resource_map_size", data_type = "u32"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Universal Disk Image Format (UDIF) resource fork header.
pub struct UdifResourceForkHeader {
    /// Resource data offset.
    pub resource_data_offset: u32,

    /// Resource map offset.
    pub resource_map_offset: u32,

    /// Resource data size.
    pub resource_data_size: u32,

    /// Resource map size.
    pub resource_map_size: u32,
}

impl UdifResourceForkHeader {
    /// Creates a new resource fork header.
    pub fn new() -> Self {
        Self {
            resource_data_offset: 0,
            resource_map_offset: 0,
            resource_data_size: 0,
            resource_map_size: 0,
        }
    }

    /// Reads the resource fork header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.resource_data_offset = bytes_to_u32_be!(data, 0);
        self.resource_map_offset = bytes_to_u32_be!(data, 4);
        self.resource_data_size = bytes_to_u32_be!(data, 8);
        self.resource_map_size = bytes_to_u32_be!(data, 12);

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
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x2c, 0x00, 0x00, 0x09, 0x2c, 0x00, 0x00,
            0x00, 0xd7,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceForkHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.resource_data_offset, 256);
        assert_eq!(test_struct.resource_map_offset, 2604);
        assert_eq!(test_struct.resource_data_size, 2348);
        assert_eq!(test_struct.resource_map_size, 215);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceForkHeader::new();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = UdifResourceForkHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.resource_data_offset, 256);
        assert_eq!(test_struct.resource_map_offset, 2604);
        assert_eq!(test_struct.resource_data_size, 2348);
        assert_eq!(test_struct.resource_map_size, 215);

        Ok(())
    }
}
