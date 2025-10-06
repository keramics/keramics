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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

const SUPPORTED_BYTES_PER_SECTOR: [u16; 5] = [256, 512, 1024, 2048, 4096];
const SUPPORTED_CLUSTER_BLOCK_SIZE: [u32; 14] = [
    256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152,
];

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "boot_entry_point", data_type = "[u8; 3]", format = "hex")),
        member(field(name = "file_system_signature", data_type = "ByteString<8>")),
        member(field(name = "bytes_per_sector", data_type = "u16")),
        member(field(name = "sectors_per_cluster_block", data_type = "u8")),
        member(field(name = "unknown1", data_type = "[u8; 2]")),
        member(field(name = "number_of_file_allocation_tables", data_type = "u8")),
        member(field(name = "number_of_root_directory_entries", data_type = "u16")),
        member(field(name = "number_of_sectors_16bit", data_type = "u16")),
        member(field(name = "media_descriptor", data_type = "u8")),
        member(field(name = "sectors_pre_file_allocation_table", data_type = "u16")),
        member(field(name = "sectors_per_track", data_type = "u16")),
        member(field(name = "number_of_heads", data_type = "u16")),
        member(field(name = "number_of_hidden_sectors", data_type = "u32")),
        member(field(name = "number_of_sectors_32bit", data_type = "u32")),
        member(field(name = "unknown3", data_type = "[u8; 4]")),
        member(field(name = "number_of_sectors_64bit", data_type = "u64")),
        member(field(name = "mft_cluster_block_number", data_type = "u64")),
        member(field(name = "mirror_mft_cluster_block_number", data_type = "u64")),
        member(field(name = "mft_entry_size", data_type = "u32")),
        member(field(name = "index_entry_size", data_type = "u32")),
        member(field(name = "volume_serial_number", data_type = "u64", format = "hex")),
        member(field(name = "checksum", data_type = "u32", format = "hex")),
        member(field(name = "bootcode", data_type = "[u8; 426]", format = "hex")),
        member(field(name = "boot_signature", data_type = "[u8; 2]", format = "hex")),
    ),
    method(name = "debug_read_data"),
    method(name = "read_at_position")
)]
/// New Technologies File System (NTFS) boot record.
pub struct NtfsBootRecord {
    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Cluster block size.
    pub cluster_block_size: u32,

    /// MFT block number.
    pub mft_block_number: u64,

    /// Mirror MFT block number.
    pub mirror_mft_block_number: u64,

    /// MFT entry size.
    pub mft_entry_size: u32,

    /// Index entry size.
    pub index_entry_size: u32,

    /// Volume serial number.
    pub volume_serial_number: u64,
}

impl NtfsBootRecord {
    /// Creates a new boot record.
    pub fn new() -> Self {
        Self {
            bytes_per_sector: 0,
            cluster_block_size: 0,
            mft_block_number: 0,
            mirror_mft_block_number: 0,
            mft_entry_size: 0,
            index_entry_size: 0,
            volume_serial_number: 0,
        }
    }

    /// Reads the boot record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 512 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported NTFS boot record data size"
            ));
        }
        if data[3..11] != NTFS_FILE_SYSTEM_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported file system signature"
            ));
        }
        self.bytes_per_sector = bytes_to_u16_le!(data, 11);
        self.mft_block_number = bytes_to_u64_le!(data, 48);
        self.mirror_mft_block_number = bytes_to_u64_le!(data, 56);
        self.volume_serial_number = bytes_to_u64_le!(data, 72);

        if !SUPPORTED_BYTES_PER_SECTOR.contains(&self.bytes_per_sector) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported bytes per sector: {}",
                self.bytes_per_sector
            )));
        }
        let sectors_per_cluster_block: u32 = data[13] as u32;

        self.cluster_block_size = if sectors_per_cluster_block <= 128 {
            sectors_per_cluster_block
        } else {
            // The size is calculated as: 2 ^ ( 256 - value ).
            let exponent: u32 = 256 - sectors_per_cluster_block;
            if exponent > 12 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported sectors per cluster block: {} value out of bounds",
                    sectors_per_cluster_block
                )));
            }
            1 << exponent
        };
        self.cluster_block_size *= self.bytes_per_sector as u32;

        if !SUPPORTED_CLUSTER_BLOCK_SIZE.contains(&self.cluster_block_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported sectors per cluster block: {}",
                sectors_per_cluster_block
            )));
        }
        let mft_entry_size: u32 = bytes_to_u32_le!(data, 64);

        if mft_entry_size == 0 || mft_entry_size > 255 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT entry size: {} value out of bounds",
                mft_entry_size
            )));
        }
        self.mft_entry_size = if mft_entry_size < 128 {
            mft_entry_size * self.cluster_block_size
        } else {
            // The size is calculated as: 2 ^ ( 256 - value ).
            let exponent: u32 = 256 - mft_entry_size;
            if exponent > 32 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported MFT entry size: {} value out of bounds",
                    mft_entry_size
                )));
            }
            1 << exponent
        };
        // Note that 42 is the minimum MFT entry size and 65535 is chosen given the fix-up values
        // and attributes offsets of the MFT entry are 16-bit.
        if self.mft_entry_size < 42 || self.mft_entry_size > 65535 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT entry size: {} value out of bounds",
                mft_entry_size
            )));
        }
        let index_entry_size: u32 = bytes_to_u32_le!(data, 68);

        if index_entry_size == 0 || index_entry_size > 255 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported index entry size: {} value out of bounds",
                index_entry_size
            )));
        }
        self.index_entry_size = if index_entry_size < 128 {
            index_entry_size * self.cluster_block_size
        } else {
            // The size is calculated as: 2 ^ ( 256 - value ).
            let exponent: u32 = 256 - index_entry_size;
            if exponent > 32 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported index entry size: {} value out of bounds",
                    index_entry_size
                )));
            }
            1 << exponent
        };
        // Note that 32 is the minimum index entry size and 16777216 is an arbitrary chosen limit.
        if self.index_entry_size < 32 || self.index_entry_size > 16777216 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported index entry size: {} value out of bounds",
                index_entry_size
            )));
        }
        let number_of_sectors: u64 = bytes_to_u64_le!(data, 40);

        if number_of_sectors > u64::MAX / (self.bytes_per_sector as u64) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported number of sectors: {} value out of bounds",
                number_of_sectors
            )));
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
        return vec![
            0xeb, 0x52, 0x90, 0x4e, 0x54, 0x46, 0x53, 0x20, 0x20, 0x20, 0x20, 0x00, 0x02, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00, 0x3f, 0x00, 0x20, 0x00,
            0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x80, 0x00, 0xc0, 0x3e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xeb, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x60, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x23, 0x56, 0xed, 0x50, 0x92, 0xed, 0x50, 0xba, 0x00, 0x00, 0x00, 0x00,
            0xfa, 0x33, 0xc0, 0x8e, 0xd0, 0xbc, 0x00, 0x7c, 0xfb, 0xb8, 0xc0, 0x07, 0x8e, 0xd8,
            0xe8, 0x16, 0x00, 0xb8, 0x00, 0x0d, 0x8e, 0xc0, 0x33, 0xdb, 0xc6, 0x06, 0x0e, 0x00,
            0x10, 0xe8, 0x53, 0x00, 0x68, 0x00, 0x0d, 0x68, 0x6a, 0x02, 0xcb, 0x8a, 0x16, 0x24,
            0x00, 0xb4, 0x08, 0xcd, 0x13, 0x73, 0x05, 0xb9, 0xff, 0xff, 0x8a, 0xf1, 0x66, 0x0f,
            0xb6, 0xc6, 0x40, 0x66, 0x0f, 0xb6, 0xd1, 0x80, 0xe2, 0x3f, 0xf7, 0xe2, 0x86, 0xcd,
            0xc0, 0xed, 0x06, 0x41, 0x66, 0x0f, 0xb7, 0xc9, 0x66, 0xf7, 0xe1, 0x66, 0xa3, 0x20,
            0x00, 0xc3, 0xb4, 0x41, 0xbb, 0xaa, 0x55, 0x8a, 0x16, 0x24, 0x00, 0xcd, 0x13, 0x72,
            0x0f, 0x81, 0xfb, 0x55, 0xaa, 0x75, 0x09, 0xf6, 0xc1, 0x01, 0x74, 0x04, 0xfe, 0x06,
            0x14, 0x00, 0xc3, 0x66, 0x60, 0x1e, 0x06, 0x66, 0xa1, 0x10, 0x00, 0x66, 0x03, 0x06,
            0x1c, 0x00, 0x66, 0x3b, 0x06, 0x20, 0x00, 0x0f, 0x82, 0x3a, 0x00, 0x1e, 0x66, 0x6a,
            0x00, 0x66, 0x50, 0x06, 0x53, 0x66, 0x68, 0x10, 0x00, 0x01, 0x00, 0x80, 0x3e, 0x14,
            0x00, 0x00, 0x0f, 0x85, 0x0c, 0x00, 0xe8, 0xb3, 0xff, 0x80, 0x3e, 0x14, 0x00, 0x00,
            0x0f, 0x84, 0x61, 0x00, 0xb4, 0x42, 0x8a, 0x16, 0x24, 0x00, 0x16, 0x1f, 0x8b, 0xf4,
            0xcd, 0x13, 0x66, 0x58, 0x5b, 0x07, 0x66, 0x58, 0x66, 0x58, 0x1f, 0xeb, 0x2d, 0x66,
            0x33, 0xd2, 0x66, 0x0f, 0xb7, 0x0e, 0x18, 0x00, 0x66, 0xf7, 0xf1, 0xfe, 0xc2, 0x8a,
            0xca, 0x66, 0x8b, 0xd0, 0x66, 0xc1, 0xea, 0x10, 0xf7, 0x36, 0x1a, 0x00, 0x86, 0xd6,
            0x8a, 0x16, 0x24, 0x00, 0x8a, 0xe8, 0xc0, 0xe4, 0x06, 0x0a, 0xcc, 0xb8, 0x01, 0x02,
            0xcd, 0x13, 0x0f, 0x82, 0x19, 0x00, 0x8c, 0xc0, 0x05, 0x20, 0x00, 0x8e, 0xc0, 0x66,
            0xff, 0x06, 0x10, 0x00, 0xff, 0x0e, 0x0e, 0x00, 0x0f, 0x85, 0x6f, 0xff, 0x07, 0x1f,
            0x66, 0x61, 0xc3, 0xa0, 0xf8, 0x01, 0xe8, 0x09, 0x00, 0xa0, 0xfb, 0x01, 0xe8, 0x03,
            0x00, 0xfb, 0xeb, 0xfe, 0xb4, 0x01, 0x8b, 0xf0, 0xac, 0x3c, 0x00, 0x74, 0x09, 0xb4,
            0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10, 0xeb, 0xf2, 0xc3, 0x0d, 0x0a, 0x41, 0x20, 0x64,
            0x69, 0x73, 0x6b, 0x20, 0x72, 0x65, 0x61, 0x64, 0x20, 0x65, 0x72, 0x72, 0x6f, 0x72,
            0x20, 0x6f, 0x63, 0x63, 0x75, 0x72, 0x72, 0x65, 0x64, 0x00, 0x0d, 0x0a, 0x4e, 0x54,
            0x4c, 0x44, 0x52, 0x20, 0x69, 0x73, 0x20, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6e, 0x67,
            0x00, 0x0d, 0x0a, 0x4e, 0x54, 0x4c, 0x44, 0x52, 0x20, 0x69, 0x73, 0x20, 0x63, 0x6f,
            0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x00, 0x0d, 0x0a, 0x50, 0x72, 0x65,
            0x73, 0x73, 0x20, 0x43, 0x74, 0x72, 0x6c, 0x2b, 0x41, 0x6c, 0x74, 0x2b, 0x44, 0x65,
            0x6c, 0x20, 0x74, 0x6f, 0x20, 0x72, 0x65, 0x73, 0x74, 0x61, 0x72, 0x74, 0x0d, 0x0a,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x83, 0xa0, 0xb3, 0xc9, 0x00, 0x00, 0x55, 0xaa,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = NtfsBootRecord::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.mft_block_number, 5355);
        assert_eq!(test_struct.mirror_mft_block_number, 8032);
        assert_eq!(test_struct.mft_entry_size, 1024);
        assert_eq!(test_struct.index_entry_size, 4096);
        assert_eq!(test_struct.volume_serial_number, 0xba50ed9250ed5623);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[3] = 0xff;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_bytes_per_sector() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[11] = 0xff;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_sectors_per_cluster_block() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x7f;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x81;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_mft_entry_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[64] = 0x00;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[65] = 0x01;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[64] = 0x81;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_index_entry_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[68] = 0x00;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[69] = 0x01;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[68] = 0x81;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_number_of_sectors() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[47] = 0xff;

        let mut test_struct = NtfsBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = NtfsBootRecord::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.mft_block_number, 5355);
        assert_eq!(test_struct.mirror_mft_block_number, 8032);
        assert_eq!(test_struct.mft_entry_size, 1024);
        assert_eq!(test_struct.index_entry_size, 4096);
        assert_eq!(test_struct.volume_serial_number, 0xba50ed9250ed5623);

        Ok(())
    }
}
