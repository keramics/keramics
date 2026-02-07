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
use keramics_types::{bytes_to_u16_be, bytes_to_u32_be};

use super::catalog_thread_record::HfsCatalogThreadRecord;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "record_type", data_type = "u16", format = "hex"),
        field(name = "unknown1", data_type = "[u8; 8]"),
        field(name = "parent_identifier", data_type = "u32"),
        field(name = "name_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS standard) catalog thread record.
pub struct HfsStandardCatalogThreadRecord {}

impl HfsStandardCatalogThreadRecord {
    /// Reads the catalog thread record from a buffer.
    pub fn read_data(
        thread_record: &mut HfsCatalogThreadRecord,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 15 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        thread_record.record_type = bytes_to_u16_be!(data, 0);

        if thread_record.record_type != 0x0300 && thread_record.record_type != 0x0400 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                thread_record.record_type
            )));
        }
        thread_record.parent_identifier = bytes_to_u32_be!(data, 10);
        thread_record.name_size = data[14] as u16;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x68, 0x66, 0x73, 0x5f, 0x74, 0x65, 0x73, 0x74,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogThreadRecord::new();
        HfsStandardCatalogThreadRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0300);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 8);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogThreadRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsStandardCatalogThreadRecord::read_data(&mut test_struct, &test_data[0..14]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_record_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsCatalogThreadRecord::new();
        let result = HfsStandardCatalogThreadRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
