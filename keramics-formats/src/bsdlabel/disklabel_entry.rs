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
use keramics_types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "number_of_sectors", data_type = "u32"),
        field(name = "start_sector", data_type = "u32"),
        field(name = "fragment_size", data_type = "u32"),
        field(name = "file_system_type", data_type = "u8"),
        field(name = "fragments_per_block", data_type = "u8"),
        field(name = "unknown1", data_type = "[u8; 2]"),
    ),
    methods("debug_read_data")
)]
/// BSD disklabel (bsdlabel) entry.
pub struct BsdDiskLabelEntry {
    /// The entry index.
    pub entry_index: u16,

    /// The number of sectors.
    pub number_of_sectors: u32,

    /// The start sector.
    pub start_sector: u32,
}

impl BsdDiskLabelEntry {
    /// Creates a new disklabel entry.
    pub fn new() -> Self {
        Self {
            entry_index: 0,
            number_of_sectors: 0,
            start_sector: 0,
        }
    }

    /// Reads the disklabel entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.number_of_sectors = bytes_to_u32_le!(data, 0);
        self.start_sector = bytes_to_u32_le!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xf0, 0x1f, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BsdDiskLabelEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.number_of_sectors, 8176);
        assert_eq!(test_struct.start_sector, 16);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = BsdDiskLabelEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
