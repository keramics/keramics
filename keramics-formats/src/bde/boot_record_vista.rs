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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "boot_entry_point", data_type = "[u8; 3]", format = "hex"),
        field(name = "file_system_signature", data_type = "ByteString<8>"),
        field(name = "bytes_per_sector", data_type = "u16"),
        field(name = "sectors_per_cluster_block", data_type = "u8"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "number_of_allocation_tables", data_type = "u8"),
        field(name = "number_of_root_directory_entries", data_type = "u16"),
        field(name = "number_of_sectors_16bit", data_type = "u16"),
        field(name = "media_descriptor", data_type = "u8"),
        field(name = "allocation_table_size_16bit", data_type = "u16"),
        field(name = "sectors_per_track", data_type = "u16"),
        field(name = "number_of_heads", data_type = "u16"),
        field(name = "number_of_hidden_sectors", data_type = "u32"),
        field(name = "number_of_sectors_32bit", data_type = "u32"),
        field(name = "unknown2", data_type = "[u8; 4]"),
        field(name = "number_of_sectors_64bit", data_type = "u64"),
        field(name = "mft_cluster_block_number", data_type = "u64"),
        field(name = "metadata_cluster_block_number", data_type = "u64"),
        field(name = "mft_entry_size", data_type = "u32"),
        field(name = "index_entry_size", data_type = "u32"),
        field(name = "volume_serial_number", data_type = "u64", format = "hex"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "bootcode", data_type = "[u8; 426]", format = "hex"),
        field(name = "boot_signature", data_type = "[u8; 2]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) boot record used by Windows Vista.
pub struct BdeBootRecordVista {
    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Cluster block size.
    pub cluster_block_size: u32,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Metadata cluster block number.
    pub metadata_cluster_block_number: u64,
}

impl BdeBootRecordVista {
    const SUPPORTED_BYTES_PER_SECTOR: [u16; 5] = [256, 512, 1024, 2048, 4096];

    const SUPPORTED_CLUSTER_BLOCK_SIZE: [u32; 14] = [
        256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576,
        2097152,
    ];

    /// Creates a new boot record.
    pub fn new() -> Self {
        Self {
            bytes_per_sector: 0,
            cluster_block_size: 0,
            number_of_sectors: 0,
            metadata_cluster_block_number: 0,
        }
    }

    /// Reads the boot record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[3..11] != BDE_FILE_SYSTEM_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.bytes_per_sector = bytes_to_u16_le!(data, 11);

        if !Self::SUPPORTED_BYTES_PER_SECTOR.contains(&self.bytes_per_sector) {
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

        if !Self::SUPPORTED_CLUSTER_BLOCK_SIZE.contains(&self.cluster_block_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported sectors per cluster block: {}",
                sectors_per_cluster_block
            )));
        }
        let number_of_sectors_16bit: u16 = bytes_to_u16_le!(data, 19);

        if number_of_sectors_16bit != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported number of sectors 16-bit: {}",
                number_of_sectors_16bit
            )));
        }
        let number_of_sectors_32bit: u32 = bytes_to_u32_le!(data, 32);

        if number_of_sectors_32bit != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported number of sectors 32-bit: {}",
                number_of_sectors_32bit
            )));
        }
        self.number_of_sectors = bytes_to_u64_le!(data, 40);

        if self.number_of_sectors > u64::MAX / (self.bytes_per_sector as u64) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported number of sectors: {} value out of bounds",
                self.number_of_sectors
            )));
        }
        self.metadata_cluster_block_number = bytes_to_u64_le!(data, 56);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xeb, 0x52, 0x90, 0x2d, 0x46, 0x56, 0x45, 0x2d, 0x46, 0x53, 0x2d, 0x00, 0x02, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00, 0x3f, 0x00, 0xf0, 0x00,
            0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x80, 0x00, 0xf7, 0x2f,
            0x76, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x04, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0xaa, 0x12, 0x9e, 0xac, 0x56, 0x9e, 0xac, 0x74, 0x00, 0x00, 0x00, 0x00,
            0xfa, 0x33, 0xc0, 0x8e, 0xd0, 0xbc, 0x00, 0x7c, 0xfb, 0x68, 0xc0, 0x07, 0x1f, 0x1e,
            0x68, 0x66, 0x00, 0xcb, 0x88, 0x16, 0x0e, 0x00, 0x66, 0x81, 0x3e, 0x03, 0x00, 0x4e,
            0x54, 0x46, 0x53, 0x75, 0x15, 0xb4, 0x41, 0xbb, 0xaa, 0x55, 0xcd, 0x13, 0x72, 0x0c,
            0x81, 0xfb, 0x55, 0xaa, 0x75, 0x06, 0xf7, 0xc1, 0x01, 0x00, 0x75, 0x03, 0xe9, 0xd2,
            0x00, 0x1e, 0x83, 0xec, 0x18, 0x68, 0x1a, 0x00, 0xb4, 0x48, 0x8a, 0x16, 0x0e, 0x00,
            0x8b, 0xf4, 0x16, 0x1f, 0xcd, 0x13, 0x9f, 0x83, 0xc4, 0x18, 0x9e, 0x58, 0x1f, 0x72,
            0xe1, 0x3b, 0x06, 0x0b, 0x00, 0x75, 0xdb, 0xa3, 0x0f, 0x00, 0xc1, 0x2e, 0x0f, 0x00,
            0x04, 0x1e, 0x5a, 0x33, 0xdb, 0xb9, 0x00, 0x20, 0x2b, 0xc8, 0x66, 0xff, 0x06, 0x11,
            0x00, 0x03, 0x16, 0x0f, 0x00, 0x8e, 0xc2, 0xff, 0x06, 0x16, 0x00, 0xe8, 0x40, 0x00,
            0x2b, 0xc8, 0x77, 0xef, 0xb8, 0x00, 0xbb, 0xcd, 0x1a, 0x66, 0x23, 0xc0, 0x75, 0x2d,
            0x66, 0x81, 0xfb, 0x54, 0x43, 0x50, 0x41, 0x75, 0x24, 0x81, 0xf9, 0x02, 0x01, 0x72,
            0x1e, 0x16, 0x68, 0x07, 0xbb, 0x16, 0x68, 0x70, 0x0e, 0x16, 0x68, 0x09, 0x00, 0x66,
            0x53, 0x66, 0x53, 0x66, 0x55, 0x16, 0x16, 0x16, 0x68, 0xb8, 0x01, 0x66, 0x61, 0x0e,
            0x07, 0xcd, 0x1a, 0xe9, 0x6a, 0x01, 0x90, 0x90, 0x66, 0x60, 0x1e, 0x06, 0x66, 0xa1,
            0x11, 0x00, 0x66, 0x03, 0x06, 0x1c, 0x00, 0x1e, 0x66, 0x68, 0x00, 0x00, 0x00, 0x00,
            0x66, 0x50, 0x06, 0x53, 0x68, 0x01, 0x00, 0x68, 0x10, 0x00, 0xb4, 0x42, 0x8a, 0x16,
            0x0e, 0x00, 0x16, 0x1f, 0x8b, 0xf4, 0xcd, 0x13, 0x66, 0x59, 0x5b, 0x5a, 0x66, 0x59,
            0x66, 0x59, 0x1f, 0x0f, 0x82, 0x16, 0x00, 0x66, 0xff, 0x06, 0x11, 0x00, 0x03, 0x16,
            0x0f, 0x00, 0x8e, 0xc2, 0xff, 0x0e, 0x16, 0x00, 0x75, 0xbc, 0x07, 0x1f, 0x66, 0x61,
            0xc3, 0xa0, 0xf8, 0x01, 0xe8, 0x08, 0x00, 0xa0, 0xfb, 0x01, 0xe8, 0x02, 0x00, 0xeb,
            0xfe, 0xb4, 0x01, 0x8b, 0xf0, 0xac, 0x3c, 0x00, 0x74, 0x09, 0xb4, 0x0e, 0xbb, 0x07,
            0x00, 0xcd, 0x10, 0xeb, 0xf2, 0xc3, 0x0d, 0x0a, 0x41, 0x20, 0x64, 0x69, 0x73, 0x6b,
            0x20, 0x72, 0x65, 0x61, 0x64, 0x20, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x20, 0x6f, 0x63,
            0x63, 0x75, 0x72, 0x72, 0x65, 0x64, 0x00, 0x0d, 0x0a, 0x42, 0x4f, 0x4f, 0x54, 0x4d,
            0x47, 0x52, 0x20, 0x69, 0x73, 0x20, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6e, 0x67, 0x00,
            0x0d, 0x0a, 0x42, 0x4f, 0x4f, 0x54, 0x4d, 0x47, 0x52, 0x20, 0x69, 0x73, 0x20, 0x63,
            0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x00, 0x0d, 0x0a, 0x50, 0x72,
            0x65, 0x73, 0x73, 0x20, 0x43, 0x74, 0x72, 0x6c, 0x2b, 0x41, 0x6c, 0x74, 0x2b, 0x44,
            0x65, 0x6c, 0x20, 0x74, 0x6f, 0x20, 0x72, 0x65, 0x73, 0x74, 0x61, 0x72, 0x74, 0x0d,
            0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x80, 0x9d, 0xb2, 0xca, 0x00, 0x00, 0x55, 0xaa,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecordVista::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.cluster_block_size, 4096);
        assert_eq!(test_struct.number_of_sectors, 192294903);
        assert_eq!(test_struct.metadata_cluster_block_number, 6916);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecordVista::new();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[3] = 0xff;

        let mut test_struct = BdeBootRecordVista::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_bytes_per_sector() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[11] = 0xff;

        let mut test_struct = BdeBootRecordVista::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_sectors_per_cluster_block() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x7f;

        let mut test_struct = BdeBootRecordVista::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x81;

        let mut test_struct = BdeBootRecordVista::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
