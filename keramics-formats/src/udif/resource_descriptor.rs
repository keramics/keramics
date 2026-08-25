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
use keramics_types::{bytes_to_u16_be, bytes_to_u32_be};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "resource_identifier", data_type = "u16", format = "hex"),
        field(name = "name_offset", data_type = "u16", format = "hex"),
        field(name = "data_offset", data_type = "BitField32<24>"),
        field(name = "resource_flags", data_type = "BitField32<8>", format = "hex"),
        field(name = "unknown1", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// Universal Disk Image Format (UDIF) resource descriptor.
pub struct UdifResourceDescriptor {
    /// Name offset.
    pub name_offset: u16,

    /// Data offset.
    pub data_offset: u32,
}

impl UdifResourceDescriptor {
    /// Creates a new resource descriptor.
    pub fn new() -> Self {
        Self {
            name_offset: 0,
            data_offset: 0,
        }
    }

    /// Reads the resource descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 12 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let value_32bit: u32 = bytes_to_u32_be!(data, 4);

        self.name_offset = bytes_to_u16_be!(data, 2);
        self.data_offset = value_32bit & 0x00ffffff;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x50, 0x90, 0xff, 0xff, 0x00, 0x00, 0x50, 0x00, 0x04, 0x0c, 0x50, 0x90,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name_offset, 0xffff);
        assert_eq!(test_struct.data_offset, 20480);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceDescriptor::new();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }
}
