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
        field(name = "file_system_signature1", data_type = "ByteString<8>"),
        field(name = "bytes_per_sector", data_type = "u16"),
        field(name = "sectors_per_cluster_block", data_type = "u8"),
        field(name = "number_of_reserved_sectors", data_type = "u16"),
        field(name = "number_of_allocation_tables", data_type = "u8"),
        field(name = "number_of_root_directory_entries", data_type = "u16"),
        field(name = "number_of_sectors_16bit", data_type = "u16"),
        field(name = "media_descriptor", data_type = "u8"),
        field(name = "allocation_table_size_16bit", data_type = "u16"),
        field(name = "sectors_per_track", data_type = "u16"),
        field(name = "number_of_heads", data_type = "u16"),
        field(name = "number_of_hidden_sectors", data_type = "u32"),
        field(name = "number_of_sectors_32bit", data_type = "u32"),
        field(name = "allocation_table_size_32bit", data_type = "u32"),
        field(name = "extended_flags", data_type = "u16"),
        field(name = "revision_minor_number", data_type = "u8"),
        field(name = "revision_major_number", data_type = "u8"),
        field(name = "root_directory_cluster_block_number", data_type = "u32"),
        field(name = "fsinfo_sector_number", data_type = "u16"),
        field(name = "boot_sector_number", data_type = "u16"),
        field(name = "unknown1", data_type = "[u8; 12]", format = "hex"),
        field(name = "drive_number", data_type = "u8"),
        field(name = "unknown2", data_type = "u8"),
        field(name = "extended_boot_signature", data_type = "u8", format = "hex"),
        field(name = "volume_serial_number", data_type = "u32", format = "hex"),
        field(name = "volume_label", data_type = "ByteString<11>"),
        field(name = "file_system_signature2", data_type = "ByteString<8>"),
        field(name = "bootcode", data_type = "[u8; 70]", format = "hex"),
        field(name = "identifier", data_type = "Uuid"),
        field(name = "metadata_block_offset1", data_type = "u64", format = "hex"),
        field(name = "metadata_block_offset2", data_type = "u64", format = "hex"),
        field(name = "metadata_block_offset3", data_type = "u64", format = "hex"),
        field(name = "unknown3", data_type = "[u8; 310]", format = "hex"),
        field(name = "boot_signature", data_type = "[u8; 2]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) boot record.
pub struct BdeBootRecord {
    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Number of sectors.
    pub number_of_sectors: u32,

    /// Metadata block offset 1.
    pub metadata_block_offset1: u64,

    /// Metadata block offset 2.
    pub metadata_block_offset2: u64,

    /// Metadata block offset 3.
    pub metadata_block_offset3: u64,
}

impl BdeBootRecord {
    /// Creates a new boot record.
    pub fn new() -> Self {
        Self {
            bytes_per_sector: 0,
            number_of_sectors: 0,
            metadata_block_offset1: 0,
            metadata_block_offset2: 0,
            metadata_block_offset3: 0,
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
        if &data[160..176] != BDE_IDENTIFIER
            && &data[160..176] != BDE_USED_DISK_SPACE_ONLY_IDENTIFIER
        {
            return Err(keramics_core::error_trace_new!("Unsupported identifier"));
        }
        self.bytes_per_sector = bytes_to_u16_le!(data, 11);

        let number_of_sectors_16bit: u16 = bytes_to_u16_le!(data, 19);

        if number_of_sectors_16bit != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported number of sectors 16-bit: {}",
                number_of_sectors_16bit
            )));
        }
        self.number_of_sectors = bytes_to_u32_le!(data, 32);

        self.metadata_block_offset1 = bytes_to_u64_le!(data, 176);
        self.metadata_block_offset2 = bytes_to_u64_le!(data, 184);
        self.metadata_block_offset3 = bytes_to_u64_le!(data, 192);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xeb, 0x58, 0x90, 0x2d, 0x46, 0x56, 0x45, 0x2d, 0x46, 0x53, 0x2d, 0x00, 0x02, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00, 0x3f, 0x00, 0xff, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x1f, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x00, 0x00, 0x00,
            0x00, 0x4e, 0x4f, 0x20, 0x4e, 0x41, 0x4d, 0x45, 0x20, 0x20, 0x20, 0x20, 0x46, 0x41,
            0x54, 0x33, 0x32, 0x20, 0x20, 0x20, 0x33, 0xc9, 0x8e, 0xd1, 0xbc, 0xf4, 0x7b, 0x8e,
            0xc1, 0x8e, 0xd9, 0xbd, 0x00, 0x7c, 0xa0, 0xfb, 0x7d, 0xb4, 0x7d, 0x8b, 0xf0, 0xac,
            0x98, 0x40, 0x74, 0x0c, 0x48, 0x74, 0x0e, 0xb4, 0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10,
            0xeb, 0xef, 0xa0, 0xfd, 0x7d, 0xeb, 0xe6, 0xcd, 0x16, 0xcd, 0x19, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0xd6, 0x67, 0x49, 0x29, 0x2e, 0xd8, 0x4a,
            0x83, 0x99, 0xf6, 0xa3, 0x39, 0xe3, 0xd0, 0x01, 0x00, 0x00, 0x10, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x50, 0x95, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x1a, 0x0b,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0d, 0x0a, 0x52, 0x65, 0x6d, 0x6f, 0x76, 0x65, 0x20, 0x64,
            0x69, 0x73, 0x6b, 0x73, 0x20, 0x6f, 0x72, 0x20, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x20,
            0x6d, 0x65, 0x64, 0x69, 0x61, 0x2e, 0xff, 0x0d, 0x0a, 0x44, 0x69, 0x73, 0x6b, 0x20,
            0x65, 0x72, 0x72, 0x6f, 0x72, 0xff, 0x0d, 0x0a, 0x50, 0x72, 0x65, 0x73, 0x73, 0x20,
            0x61, 0x6e, 0x79, 0x20, 0x6b, 0x65, 0x79, 0x20, 0x74, 0x6f, 0x20, 0x72, 0x65, 0x73,
            0x74, 0x61, 0x72, 0x74, 0x0d, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78,
            0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78,
            0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78,
            0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78,
            0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78,
            0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x00, 0x1f, 0x2c, 0x55, 0xaa,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.number_of_sectors, 0);
        assert_eq!(test_struct.metadata_block_offset1, 0x02100000);
        assert_eq!(test_struct.metadata_block_offset2, 0x06955000);
        assert_eq!(test_struct.metadata_block_offset3, 0x0b1aa000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeBootRecord::new();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[3] = 0xff;

        let mut test_struct = BdeBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_identifier() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[160] = 0xff;

        let mut test_struct = BdeBootRecord::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
