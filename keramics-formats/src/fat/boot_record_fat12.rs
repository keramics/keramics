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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

use super::boot_record::FatBootRecord;
use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "boot_entry_point", data_type = "[u8; 3]", format = "hex")),
        member(field(name = "file_system_signature", data_type = "ByteString<8>")),
        member(field(name = "bytes_per_sector", data_type = "u16")),
        member(field(name = "sectors_per_cluster_block", data_type = "u8")),
        member(field(name = "number_of_reserved_sectors", data_type = "u16")),
        member(field(name = "number_of_allocation_tables", data_type = "u8")),
        member(field(name = "number_of_root_directory_entries", data_type = "u16")),
        member(field(name = "number_of_sectors_16bit", data_type = "u16")),
        member(field(name = "media_descriptor", data_type = "u8")),
        member(field(name = "allocation_table_size_16bit", data_type = "u16")),
        member(field(name = "sectors_per_track", data_type = "u16")),
        member(field(name = "number_of_heads", data_type = "u16")),
        member(field(name = "number_of_hidden_sectors", data_type = "u32")),
        member(field(name = "number_of_sectors_32bit", data_type = "u32")),
        member(field(name = "drive_number", data_type = "u8")),
        member(field(name = "unknown1", data_type = "u8")),
        member(field(name = "extended_boot_signature", data_type = "u8")),
        member(field(name = "volume_serial_number", data_type = "u32")),
        member(field(name = "volume_label", data_type = "ByteString<11>")),
        member(field(name = "file_system_hint", data_type = "ByteString<8>")),
        member(field(name = "bootcode", data_type = "[u8; 448]", format = "hex")),
        member(field(name = "boot_signature", data_type = "[u8; 2]", format = "hex")),
    ),
    method(name = "debug_read_data")
)]
/// File Allocation Table (FAT-12 or FAT-16) boot record.
pub struct Fat12BootRecord {}

impl Fat12BootRecord {
    /// Reads the boot record from a buffer.
    pub fn read_data(boot_record: &mut FatBootRecord, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 512 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported boot record data size"
            ));
        }
        if data[510..512] != FAT_BOOT_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported boot record signature"
            ));
        }
        boot_record.bytes_per_sector = bytes_to_u16_le!(data, 11);

        if !FAT_SUPPORTED_BYTES_PER_SECTOR.contains(&boot_record.bytes_per_sector) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported bytes per sector: {}",
                boot_record.bytes_per_sector
            )));
        }
        boot_record.sectors_per_cluster_block = data[13];

        if !FAT_SUPPORTED_SECTORS_PER_CLUSTER_BLOCK.contains(&boot_record.sectors_per_cluster_block)
        {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported sectors per cluster block: {}",
                boot_record.sectors_per_cluster_block
            )));
        }
        boot_record.number_of_reserved_sectors = bytes_to_u16_le!(data, 14);
        boot_record.number_of_allocation_tables = data[16];
        boot_record.number_of_root_directory_entries = bytes_to_u16_le!(data, 17);

        let number_of_sectors_16bit: u16 = bytes_to_u16_le!(data, 19);

        boot_record.allocation_table_size = bytes_to_u16_le!(data, 22) as u32;

        let number_of_sectors_32bit: u32 = bytes_to_u32_le!(data, 32);

        boot_record.number_of_sectors = if number_of_sectors_32bit != 0 {
            number_of_sectors_32bit
        } else {
            number_of_sectors_16bit as u32
        };
        let extended_boot_signature: u8 = data[38];

        if extended_boot_signature == 0x29 {
            boot_record.volume_serial_number = bytes_to_u32_le!(data, 39);

            let slice: &[u8] = match data[43..54].iter().rev().position(|value| *value != b' ') {
                Some(data_index) => &data[43..54 - data_index],
                None => &data[43..54],
            };
            boot_record.volume_label.elements.extend_from_slice(&slice);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::ByteString;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xeb, 0x3c, 0x90, 0x6d, 0x6b, 0x66, 0x73, 0x2e, 0x66, 0x61, 0x74, 0x00, 0x02, 0x04,
            0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x20, 0xf8, 0x06, 0x00, 0x20, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x5b, 0x0d, 0xf3,
            0x56, 0x46, 0x41, 0x54, 0x31, 0x32, 0x5f, 0x54, 0x45, 0x53, 0x54, 0x20, 0x46, 0x41,
            0x54, 0x31, 0x32, 0x20, 0x20, 0x20, 0x0e, 0x1f, 0xbe, 0x5b, 0x7c, 0xac, 0x22, 0xc0,
            0x74, 0x0b, 0x56, 0xb4, 0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10, 0x5e, 0xeb, 0xf0, 0x32,
            0xe4, 0xcd, 0x16, 0xcd, 0x19, 0xeb, 0xfe, 0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73,
            0x20, 0x6e, 0x6f, 0x74, 0x20, 0x61, 0x20, 0x62, 0x6f, 0x6f, 0x74, 0x61, 0x62, 0x6c,
            0x65, 0x20, 0x64, 0x69, 0x73, 0x6b, 0x2e, 0x20, 0x20, 0x50, 0x6c, 0x65, 0x61, 0x73,
            0x65, 0x20, 0x69, 0x6e, 0x73, 0x65, 0x72, 0x74, 0x20, 0x61, 0x20, 0x62, 0x6f, 0x6f,
            0x74, 0x61, 0x62, 0x6c, 0x65, 0x20, 0x66, 0x6c, 0x6f, 0x70, 0x70, 0x79, 0x20, 0x61,
            0x6e, 0x64, 0x0d, 0x0a, 0x70, 0x72, 0x65, 0x73, 0x73, 0x20, 0x61, 0x6e, 0x79, 0x20,
            0x6b, 0x65, 0x79, 0x20, 0x74, 0x6f, 0x20, 0x74, 0x72, 0x79, 0x20, 0x61, 0x67, 0x61,
            0x69, 0x6e, 0x20, 0x2e, 0x2e, 0x2e, 0x20, 0x0d, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xaa,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct = FatBootRecord::new();

        let test_data: Vec<u8> = get_test_data();
        Fat12BootRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.sectors_per_cluster_block, 4);
        assert_eq!(test_struct.number_of_reserved_sectors, 1);
        assert_eq!(test_struct.number_of_allocation_tables, 2);
        assert_eq!(test_struct.number_of_root_directory_entries, 512);
        assert_eq!(test_struct.allocation_table_size, 6);
        assert_eq!(test_struct.number_of_sectors, 8192);
        assert_eq!(test_struct.volume_serial_number, 0x56f30d5b);
        assert_eq!(test_struct.volume_label, ByteString::from("FAT12_TEST"));

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = FatBootRecord::new();
        let result = Fat12BootRecord::read_data(&mut test_struct, &test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[510] = 0xff;

        let mut test_struct = FatBootRecord::new();
        let result = Fat12BootRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_bytes_per_sector() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[11] = 0xff;

        let mut test_struct = FatBootRecord::new();
        let result = Fat12BootRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_sectors_per_cluster_block() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x7f;

        let mut test_struct = FatBootRecord::new();
        let result = Fat12BootRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());

        let mut test_data: Vec<u8> = get_test_data();
        test_data[13] = 0x81;

        let mut test_struct = FatBootRecord::new();
        let result = Fat12BootRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
