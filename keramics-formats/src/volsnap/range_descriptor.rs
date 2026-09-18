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
use keramics_types::bytes_to_u64_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "range_offset", data_type = "u64", format = "hex"),
        field(name = "relative_block_offset", data_type = "u64", format = "hex"),
        field(name = "range_size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) range descriptor.
pub struct VolsnapRangeDescriptor {
    /// Range offset.
    pub range_offset: u64,

    /// Range size.
    pub range_size: u64,
}

impl VolsnapRangeDescriptor {
    /// Creates a new range descriptor.
    pub fn new() -> Self {
        Self {
            range_offset: 0,
            range_size: 0,
        }
    }

    /// Reads the range descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 24 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.range_offset = bytes_to_u64_le!(data, 0);
        self.range_size = bytes_to_u64_le!(data, 16);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0xc0, 0xa9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x40, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapRangeDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.range_offset, 0x04a9c000);
        assert_eq!(test_struct.range_size, 33505280);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapRangeDescriptor::new();
        let result = test_struct.read_data(&test_data[0..23]);
        assert!(result.is_err());
    }
}
