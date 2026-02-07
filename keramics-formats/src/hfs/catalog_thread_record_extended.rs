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
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "parent_identifier", data_type = "u32"),
        field(name = "name_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) catalog thread record.
pub struct HfsExtendedCatalogThreadRecord {}

impl HfsExtendedCatalogThreadRecord {
    /// Reads the catalog thread record from a buffer.
    pub fn read_data(
        thread_record: &mut HfsCatalogThreadRecord,
        data: &[u8],
    ) -> Result<(), ErrorTrace> {
        if data.len() < 10 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        thread_record.record_type = bytes_to_u16_be!(data, 0);

        if thread_record.record_type != 0x0003 && thread_record.record_type != 0x0004 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported record type: 0x{:04x}",
                thread_record.record_type
            )));
        }
        thread_record.parent_identifier = bytes_to_u32_be!(data, 4);
        thread_record.name_size = bytes_to_u16_be!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0c, 0x00, 0x68, 0x00, 0x66,
            0x00, 0x73, 0x00, 0x70, 0x00, 0x6c, 0x00, 0x75, 0x00, 0x73, 0x00, 0x5f, 0x00, 0x74,
            0x00, 0x65, 0x00, 0x73, 0x00, 0x74,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogThreadRecord::new();
        HfsExtendedCatalogThreadRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0003);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 12);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogThreadRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsExtendedCatalogThreadRecord::read_data(&mut test_struct, &test_data[0..9]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_record_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = HfsCatalogThreadRecord::new();
        let result = HfsExtendedCatalogThreadRecord::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
