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

use super::catalog_key::HfsCatalogKey;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "data_size", data_type = "u16"),
        group(
            size_condition = ">= 6",
            field(name = "parent_identifier", data_type = "u32"),
        ),
        group(size_condition = ">= 8", field(name = "name_size", data_type = "u16"),)
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) catalog key.
pub struct HfsExtendedCatalogKey {}

impl HfsExtendedCatalogKey {
    /// Reads the catalog key from a buffer.
    pub fn read_data(key: &mut HfsCatalogKey, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 2 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let key_data_size: usize = bytes_to_u16_be!(data, 0) as usize;

        if key_data_size > data_size - 2 {
            return Err(keramics_core::error_trace_new!(
                "Invalid key data size value out of bounds"
            ));
        }
        key.size = 2 + key_data_size;

        if key_data_size >= 4 {
            key.parent_identifier = bytes_to_u32_be!(data, 2);
        }
        if key_data_size >= 6 {
            key.name_size = bytes_to_u16_be!(data, 6);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x0c, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x78,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsCatalogKey::new();
        HfsExtendedCatalogKey::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.size, 14);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsCatalogKey::new();

        let test_data: Vec<u8> = get_test_data();
        let result = HfsExtendedCatalogKey::read_data(&mut test_struct, &test_data[0..1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_key_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[1] = 0xff;

        let mut test_struct = HfsCatalogKey::new();
        let result = HfsExtendedCatalogKey::read_data(&mut test_struct, &test_data);
        assert!(result.is_err());
    }
}
