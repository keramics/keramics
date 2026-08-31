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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature1", data_type = "[u8; 4]"),
        field(name = "drive_type", data_type = "u16"),
        field(name = "drive_sub_type", data_type = "u16"),
        field(name = "drive_type_name", data_type = "ByteString<16>"),
        field(name = "pack_identifier", data_type = "ByteString<16>"),
        field(name = "bytes_per_sector", data_type = "u32"),
        field(name = "sectors_per_track", data_type = "u32"),
        field(name = "tracks_per_cylinder", data_type = "u32"),
        field(name = "cylinders_per_unit", data_type = "u32"),
        field(name = "sectors_per_cylinder", data_type = "u32"),
        field(name = "sectors_per_unit", data_type = "u32"),
        field(name = "spare_sectors_per_track", data_type = "u16"),
        field(name = "spare_sectors_per_cylinder", data_type = "u16"),
        field(name = "alternate_sectors_per_unit", data_type = "u32"),
        field(name = "rotational_speed", data_type = "u16"),
        field(name = "hardware_sector_interleave", data_type = "u16"),
        field(name = "sector0_skew_per_track", data_type = "u16"),
        field(name = "sector0_skew_per_cylinder", data_type = "u16"),
        field(name = "head_switch_time", data_type = "u32"),
        field(name = "track_to_track_seek_time", data_type = "u32"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "unknown1", data_type = "[u32; 5]"),
        field(name = "unknown2", data_type = "[u32; 5]"),
        field(name = "signature2", data_type = "[u8; 4]"),
        field(name = "checksum", data_type = "u16"),
        field(name = "number_of_entries", data_type = "u16"),
        field(name = "boot_area_size", data_type = "u32"),
        field(name = "maximum_superblock_size", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// BSD disklabel (bsdlabel) header.
pub struct BsdDiskLabelHeader {
    /// The bytes per sector.
    pub bytes_per_sector: u32,

    /// The checksum.
    pub checksum: u16,

    /// The number of entries.
    pub number_of_entries: u16,
}

impl BsdDiskLabelHeader {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            bytes_per_sector: 0,
            checksum: 0,
            number_of_entries: 0,
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 148 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..4] != BSD_DISKLABEL_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported first signature"
            ));
        }
        if &data[132..136] != BSD_DISKLABEL_SIGNATURE {
            return Err(keramics_core::error_trace_new!(
                "Unsupported second signature"
            ));
        }
        self.bytes_per_sector = bytes_to_u32_le!(data, 40);
        self.checksum = bytes_to_u16_le!(data, 136);
        self.number_of_entries = bytes_to_u16_le!(data, 138);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x57, 0x45, 0x56, 0x82, 0x00, 0x00, 0x00, 0x00, 0x61, 0x6d, 0x6e, 0x65, 0x73, 0x69,
            0x61, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x82, 0x00, 0x00, 0x00,
            0x3f, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x10, 0x0e, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x56, 0x82, 0x67, 0x31, 0x08, 0x00,
            0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BsdDiskLabelHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.checksum, 0x3167);
        assert_eq!(test_struct.number_of_entries, 8);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = BsdDiskLabelHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..147]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature1() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = BsdDiskLabelHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature2() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[132] = 0xff;

        let mut test_struct = BsdDiskLabelHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
