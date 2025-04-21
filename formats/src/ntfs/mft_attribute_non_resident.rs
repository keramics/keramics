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
use types::{bytes_to_u16_le, bytes_to_u64_le};

// TODO: add value_condition for total_data_size
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "data_first_vcn", data_type = "u64")),
        member(field(name = "data_last_vcn", data_type = "u64")),
        member(field(name = "data_runs_offset", data_type = "u16")),
        member(field(name = "compression_unit_size", data_type = "u16")),
        member(field(name = "unknown1", data_type = "[u8; 4]")),
        member(field(name = "allocated_data_size", data_type = "u64")),
        member(field(name = "data_size", data_type = "u64")),
        member(field(name = "valid_data_size", data_type = "u64")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) Master File Table (MFT) non-resident attribute.
pub struct NtfsMftAttributeNonResident {
    /// Data first virtual cluster number (VCN).
    pub data_first_vcn: u64,

    /// Data last virtual cluster number (VCN).
    pub data_last_vcn: u64,

    /// Data runs offset.
    pub data_runs_offset: u16,

    /// Compression unit size.
    pub compression_unit_size: u32,

    /// Allocated data size.
    pub allocated_data_size: u64,

    /// Data size.
    pub data_size: u64,

    /// Valid data size.
    pub valid_data_size: u64,
}

impl NtfsMftAttributeNonResident {
    /// Creates a new MFT attribute header.
    pub fn new() -> Self {
        Self {
            data_first_vcn: 0,
            data_last_vcn: 0,
            data_runs_offset: 0,
            compression_unit_size: 0,
            allocated_data_size: 0,
            data_size: 0,
            valid_data_size: 0,
        }
    }

    /// Reads the MFT attribute header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size: usize = data.len();
        if data_size < 48 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        let compression_unit_size: u16 = bytes_to_u16_le!(data, 18);

        if compression_unit_size > 0 {
            if data_size < 56 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported data size"),
                ));
            }
            // The size is calculated as: 2 ^ value
            if compression_unit_size > 31 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Unsupported compression unit size: {} value out of bounds",
                        compression_unit_size
                    ),
                ));
            }
            self.compression_unit_size = 1 << (compression_unit_size as u32);
        }
        self.data_first_vcn = bytes_to_u64_le!(data, 0);
        self.data_last_vcn = bytes_to_u64_le!(data, 8);
        self.data_runs_offset = bytes_to_u16_le!(data, 16);
        self.allocated_data_size = bytes_to_u64_le!(data, 24);
        self.data_size = bytes_to_u64_le!(data, 32);
        self.valid_data_size = bytes_to_u64_le!(data, 40);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsMftAttributeNonResident::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.data_first_vcn, 0);
        assert_eq!(test_struct.data_last_vcn, 63);
        assert_eq!(test_struct.data_runs_offset, 64);
        assert_eq!(test_struct.compression_unit_size, 0);
        assert_eq!(test_struct.allocated_data_size, 262144);
        assert_eq!(test_struct.data_size, 262144);
        assert_eq!(test_struct.valid_data_size, 262144);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftAttributeNonResident::new();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_compression_unit_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[18] = 0xff;

        let mut test_struct = NtfsMftAttributeNonResident::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        // TODO: add test for non-resident attribute with data_size == 56 but with
        // compression_unit_size >= 32
    }
}
