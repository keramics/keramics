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

#[derive(Clone, Debug, LayoutMap, PartialEq)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "logical_data_offset", data_type = "u64", format = "hex"),
        field(name = "relative_block_offset", data_type = "u64", format = "hex"),
        field(name = "physical_data_offset", data_type = "u64", format = "hex"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "bitmap", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) block descriptor.
pub struct VolsnapBlockDescriptor {
    /// Logical data offset.
    pub logical_data_offset: u64,

    /// Relative block offset.
    pub relative_block_offset: u64,

    /// Physical data offset.
    pub physical_data_offset: u64,

    /// Flags.
    pub flags: u32,

    /// Bitmap.
    pub bitmap: u32,

    /// Overlay.
    pub overlay: Option<Box<VolsnapBlockDescriptor>>,
}

impl VolsnapBlockDescriptor {
    /// Creates a new block descriptor.
    pub fn new() -> Self {
        Self {
            logical_data_offset: 0,
            relative_block_offset: 0,
            physical_data_offset: 0,
            flags: 0,
            bitmap: 0,
            overlay: None,
        }
    }

    /// Determines if the block descriptor is a forwarder.
    pub fn is_forwarder(&self) -> bool {
        self.flags & 0x00000001 != 0
    }

    /// Determines if the block descriptor is an overlay.
    pub fn is_overlay(&self) -> bool {
        self.flags & 0x00000002 != 0
    }

    /// Determines if the block descriptor is unused.
    pub fn is_unused(&self) -> bool {
        self.flags & 0x00000004 != 0
    }

    /// Reads the block descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.logical_data_offset = bytes_to_u64_le!(data, 0);
        self.relative_block_offset = bytes_to_u64_le!(data, 8);
        self.physical_data_offset = bytes_to_u64_le!(data, 16);
        self.flags = bytes_to_u32_le!(data, 24);
        self.bitmap = bytes_to_u32_le!(data, 28);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0xc0, 0x84, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xc0, 0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapBlockDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.logical_data_offset, 0x0284c000);
        assert_eq!(test_struct.relative_block_offset, 0x0000c000);
        assert_eq!(test_struct.physical_data_offset, 0x02a9c000);
        assert_eq!(test_struct.flags, 0x00000000);
        assert_eq!(test_struct.bitmap, 0x00000000);
        assert_eq!(test_struct.overlay, None);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapBlockDescriptor::new();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }
}
