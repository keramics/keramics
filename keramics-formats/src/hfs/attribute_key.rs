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
use keramics_types::{Utf16String, bytes_to_u16_be, bytes_to_u32_be};

use super::string::HfsString;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "data_size", data_type = "u16"),
        group(
            size_condition = ">= 14",
            field(name = "unknown1", data_type = "[u8; 2]"),
            field(name = "identifier", data_type = "u32"),
            field(name = "unknown2", data_type = "[u8; 4]"),
            field(name = "name_size", data_type = "u16"),
        ),
    ),
    methods("debug_read_data")
)]
/// Hierarchical File System (HFS extended) attribute key.
pub struct HfsAttributeKey {
    /// Size.
    pub size: usize,

    /// Identifier (CNID).
    pub identifier: u32,

    /// Name size.
    pub name_size: u16,
}

impl HfsAttributeKey {
    /// Creates a new attribute key.
    pub fn new() -> Self {
        Self {
            size: 0,
            identifier: 0,
            name_size: 0,
        }
    }

    /// Reads the attribute key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
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
        self.size = 2 + key_data_size;

        if key_data_size >= 6 {
            self.identifier = bytes_to_u32_be!(data, 4);
        }
        if key_data_size >= 10 {
            self.name_size = bytes_to_u16_be!(data, 12);
        }
        Ok(())
    }

    /// Reads the name from a buffer.
    pub fn read_name(&mut self, data: &[u8]) -> Result<HfsString, ErrorTrace> {
        let name_size: usize = (self.name_size as usize) * 2;
        let name_end_offset: usize = 14 + name_size;

        if name_end_offset > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid name size: {} value out of bounds",
                name_size
            )));
        }
        keramics_core::debug_trace_data!(
            "HfsAttributeKey name",
            14,
            &data[14..name_end_offset],
            name_size
        );
        let utf16_string: Utf16String = Utf16String::from_be_bytes(&data[14..name_end_offset]);
        let name: HfsString = HfsString::Utf16String(utf16_string);

        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsAttributeKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.size, 14);
        assert_eq!(test_struct.identifier, 1);
        assert_eq!(test_struct.name_size, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = HfsAttributeKey::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_key_size() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[1] = 0xff;

        let mut test_struct = HfsAttributeKey::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_name() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = HfsAttributeKey::new();
        test_struct.read_data(&test_data)?;

        let name: HfsString = test_struct.read_name(&test_data)?;
        assert_eq!(
            name,
            HfsString::Utf16String(Utf16String { elements: vec![] })
        );
        Ok(())
    }
}
