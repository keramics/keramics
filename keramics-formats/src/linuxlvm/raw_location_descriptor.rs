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

/// Linux Logical Volume Manager (LVM) raw location descriptor.
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "offset", data_type = "u64", format = "hex"),
        field(name = "size", data_type = "u64"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "flags", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data")
)]
pub struct LinuxLvmRawLocationDescriptor {
    /// Offset.
    pub offset: u64,

    /// Size.
    pub size: u64,

    /// Checksum.
    pub checksum: u32,

    /// Flags.
    pub flags: u32,
}

impl LinuxLvmRawLocationDescriptor {
    /// Creates a new raw location descriptor.
    pub fn new() -> Self {
        Self {
            offset: 0,
            size: 0,
            checksum: 0,
            flags: 0,
        }
    }

    /// Reads the raw location descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 24 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.offset = bytes_to_u64_le!(data, 0);
        self.size = bytes_to_u64_le!(data, 8);
        self.checksum = bytes_to_u32_le!(data, 16);
        self.flags = bytes_to_u32_le!(data, 20);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd2, 0x05, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x68, 0x3f, 0x3d, 0x9b, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LinuxLvmRawLocationDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.offset, 0x00000c00);
        assert_eq!(test_struct.size, 1490);
        assert_eq!(test_struct.checksum, 0x9b3d3f68);
        assert_eq!(test_struct.flags, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = LinuxLvmRawLocationDescriptor::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
