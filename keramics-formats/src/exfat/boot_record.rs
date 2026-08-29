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

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "boot_entry_point", data_type = "[u8; 3]", format = "hex"),
        field(name = "file_system_signature", data_type = "ByteString<8>"),
        field(name = "unknown1", data_type = "[u8; 53]"),
        field(name = "partition_offset", data_type = "u64", format = "hex"),
        field(name = "number_of_sectors", data_type = "u64"),
        field(name = "allocation_table_offset", data_type = "u32", format = "hex"),
        field(name = "allocation_table_size", data_type = "u32"),
        field(name = "cluster_heap_start_sector", data_type = "u32"),
        field(name = "number_of_clusters", data_type = "u32"),
        field(name = "root_directory_cluster_block_number", data_type = "u32"),
        field(name = "volume_serial_number", data_type = "u32", format = "hex"),
        field(name = "revision_minor_number", data_type = "u8"),
        field(name = "revision_major_number", data_type = "u8"),
        field(name = "volume_flags", data_type = "u16", format = "hex"),
        field(name = "bytes_per_sector", data_type = "u8"),
        field(name = "sectors_per_cluster_block", data_type = "u8"),
        field(name = "number_of_allocation_tables", data_type = "u8"),
        field(name = "drive_number", data_type = "u8"),
        field(name = "unknown2", data_type = "u8"),
        field(name = "unknown3", data_type = "[u8; 7]"),
        field(name = "bootcode", data_type = "[u8; 390]", format = "hex"),
        field(name = "boot_signature", data_type = "[u8; 2]", format = "hex"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Extensible File Allocation Table (exFAT) boot record.
pub struct ExFatBootRecord {
    /// Number of sectors.
    pub number_of_sectors: u64,

    /// File allocation table offset.
    pub allocation_table_offset: u32,

    /// File allocation table size.
    pub allocation_table_size: u32,

    /// Cluster heap start sector.
    pub cluster_heap_start_sector: u32,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Sectors per cluster block.
    pub sectors_per_cluster_block: u32,

    /// Number of allocation tables.
    pub number_of_allocation_tables: u8,

    /// Root directory cluster block number.
    pub root_directory_cluster_block_number: u32,

    /// Volume serial number.
    pub volume_serial_number: u32,
}

impl ExFatBootRecord {
    /// Creates a new boot record.
    pub fn new() -> Self {
        Self {
            number_of_sectors: 0,
            allocation_table_offset: 0,
            allocation_table_size: 0,
            cluster_heap_start_sector: 0,
            bytes_per_sector: 0,
            sectors_per_cluster_block: 0,
            number_of_allocation_tables: 0,
            root_directory_cluster_block_number: 0,
            volume_serial_number: 0,
        }
    }

    /// Reads the boot record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[510..512] != EXFAT_BOOT_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let bytes_per_sector: u8 = data[108];

        if bytes_per_sector < 9 || bytes_per_sector > 12 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid bytes per sector: {} value out of bounds",
                bytes_per_sector,
            )));
        }
        let sectors_per_cluster_block: u8 = data[109];

        if sectors_per_cluster_block > 25 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid sectors per cluster block: {} value out of bounds",
                sectors_per_cluster_block,
            )));
        }
        self.number_of_sectors = bytes_to_u64_le!(data, 72);
        self.allocation_table_offset = bytes_to_u32_le!(data, 80);
        self.allocation_table_size = bytes_to_u32_le!(data, 84);
        self.cluster_heap_start_sector = bytes_to_u32_le!(data, 88);

        if self.allocation_table_size == 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported allocation table size: {}",
                self.allocation_table_size
            )));
        }
        self.bytes_per_sector = 1 << (bytes_per_sector as u16);
        self.sectors_per_cluster_block = 1 << (sectors_per_cluster_block as u32);

        self.root_directory_cluster_block_number = bytes_to_u32_le!(data, 96);

        if self.root_directory_cluster_block_number < 2 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported root directory cluster block number: {}",
                self.root_directory_cluster_block_number
            )));
        }
        self.volume_serial_number = bytes_to_u32_le!(data, 100);
        self.number_of_allocation_tables = data[110];

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
            0xeb, 0x76, 0x90, 0x45, 0x58, 0x46, 0x41, 0x54, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
            0x30, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x0f, 0x00,
            0x00, 0x00, 0x02, 0x73, 0xef, 0x7a, 0x00, 0x01, 0x00, 0x00, 0x09, 0x00, 0x01, 0x80,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x1f, 0xbe, 0x95, 0x7c, 0xac,
            0x22, 0xc0, 0x74, 0x0b, 0x56, 0xb4, 0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10, 0x5e, 0xeb,
            0xf0, 0x32, 0xe4, 0xcd, 0x16, 0xcd, 0x19, 0xeb, 0xfe, 0x54, 0x68, 0x69, 0x73, 0x20,
            0x65, 0x78, 0x46, 0x41, 0x54, 0x2f, 0x47, 0x50, 0x54, 0x20, 0x76, 0x6f, 0x6c, 0x75,
            0x6d, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6e, 0x6f, 0x74, 0x20, 0x62, 0x6f, 0x6f, 0x74,
            0x61, 0x62, 0x6c, 0x65, 0x2e, 0x0d, 0x0a, 0x50, 0x6c, 0x65, 0x61, 0x73, 0x65, 0x20,
            0x69, 0x6e, 0x73, 0x65, 0x72, 0x74, 0x20, 0x61, 0x20, 0x62, 0x6f, 0x6f, 0x74, 0x61,
            0x62, 0x6c, 0x65, 0x20, 0x64, 0x69, 0x73, 0x6b, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x70,
            0x72, 0x65, 0x73, 0x73, 0x20, 0x61, 0x6e, 0x79, 0x20, 0x6b, 0x65, 0x79, 0x20, 0x74,
            0x6f, 0x20, 0x74, 0x72, 0x79, 0x20, 0x61, 0x67, 0x61, 0x69, 0x6e, 0x2e, 0x0d, 0x0a,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xaa,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExFatBootRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_sectors, 8192);
        assert_eq!(test_struct.allocation_table_offset, 2048);
        assert_eq!(test_struct.allocation_table_size, 48);
        assert_eq!(test_struct.cluster_heap_start_sector, 4096);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.sectors_per_cluster_block, 1);
        assert_eq!(test_struct.number_of_allocation_tables, 1);
        assert_eq!(test_struct.volume_serial_number, 0x7aef7302);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ExFatBootRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[510] = 0xff;

        let mut test_struct = ExFatBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = ExFatBootRecord::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.number_of_sectors, 8192);
        assert_eq!(test_struct.allocation_table_offset, 2048);
        assert_eq!(test_struct.allocation_table_size, 48);
        assert_eq!(test_struct.cluster_heap_start_sector, 4096);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.sectors_per_cluster_block, 1);
        assert_eq!(test_struct.number_of_allocation_tables, 1);
        assert_eq!(test_struct.volume_serial_number, 0x7aef7302);

        Ok(())
    }
}
