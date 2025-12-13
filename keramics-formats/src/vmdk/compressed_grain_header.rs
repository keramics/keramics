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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "logical_sector_number", data_type = "u64"),
        field(name = "compressed_data_size", data_type = "u32"),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// VMware Virtual Disk (VMDK) compressed grain header.
pub struct VmdkCompressedGrainHeader {
    /// Logical sector number.
    pub logical_sector_number: u64,

    /// Compressed data size.
    pub compressed_data_size: u32,
}

impl VmdkCompressedGrainHeader {
    /// Creates a new compressed grain header.
    pub fn new() -> Self {
        Self {
            logical_sector_number: 0,
            compressed_data_size: 0,
        }
    }

    /// Reads the compressed grain header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.logical_sector_number = bytes_to_u64_le!(data, 0);
        self.compressed_data_size = bytes_to_u32_le!(data, 8);

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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VmdkCompressedGrainHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.logical_sector_number, 0);
        assert_eq!(test_struct.compressed_data_size, 512);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VmdkCompressedGrainHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = VmdkCompressedGrainHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.logical_sector_number, 0);
        assert_eq!(test_struct.compressed_data_size, 512);

        Ok(())
    }
}
