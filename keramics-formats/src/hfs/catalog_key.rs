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
use keramics_encodings::CharacterEncoding;
use keramics_types::{ByteString, Utf16String, bytes_to_u16_be};

use super::catalog_key_extended::HfsExtendedCatalogKey;
use super::catalog_key_standard::HfsStandardCatalogKey;
use super::enums::HfsFormat;
use super::string::HfsString;

/// Hierarchical File System (HFS) catalog key.
pub struct HfsCatalogKey {
    /// Size.
    pub size: usize,

    /// Parent identifier (CNID).
    pub parent_identifier: u32,

    /// Name size.
    pub name_size: u16,
}

impl HfsCatalogKey {
    /// Creates a new catalog key.
    pub fn new() -> Self {
        Self {
            size: 0,
            parent_identifier: 0,
            name_size: 0,
        }
    }

    /// Reads the catalog key for debugging.
    pub fn debug_read_data(format: &HfsFormat, data: &[u8]) -> String {
        match format {
            HfsFormat::Hfs => HfsStandardCatalogKey::debug_read_data(data),
            HfsFormat::HfsPlus | HfsFormat::HfsX => HfsExtendedCatalogKey::debug_read_data(data),
        }
    }

    /// Reads the catalog key from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardCatalogKey::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogKey::read_data(self, data)?;
            }
        }
        Ok(())
    }

    /// Reads the name from a buffer.
    pub fn read_name(
        &mut self,
        format: &HfsFormat,
        encoding: &CharacterEncoding,
        data: &[u8],
    ) -> Result<HfsString, ErrorTrace> {
        let (name_offset, name_size): (usize, usize) = match format {
            HfsFormat::Hfs => (7, self.name_size as usize),
            HfsFormat::HfsPlus | HfsFormat::HfsX => (8, (self.name_size as usize) * 2),
        };
        let name_end_offset: usize = name_offset + name_size;

        if name_end_offset > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid name size: {} value out of bounds",
                name_size
            )));
        }
        keramics_core::debug_trace_data!(
            "HfsCatalogKey name",
            name_offset,
            &data[name_offset..name_end_offset],
            name_size
        );
        let name: HfsString = match format {
            HfsFormat::Hfs => {
                // TODO: handle special characters.
                let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);
                byte_string.read_data(&data[name_offset..name_end_offset]);
                HfsString::ByteString(byte_string)
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                let mut elements: Vec<u16> = Vec::new();

                for data_offset in (name_offset..name_end_offset).step_by(2) {
                    let mut value_16bit: u16 = bytes_to_u16_be!(data, data_offset);

                    value_16bit = match value_16bit {
                        // U+2400 is stored as U+0000
                        0x0000 => 0x2400,
                        // ':' is stored as '/'
                        0x002f => 0x003a,
                        _ => value_16bit,
                    };
                    elements.push(value_16bit);
                }
                HfsString::Utf16String(Utf16String { elements })
            }
        };
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![0x09, 0x00, 0x00, 0x00, 0x00, 0x01, 0x03, 0x6f, 0x73, 0x78];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![
            0x00, 0x0c, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03, 0x00, 0x6f, 0x00, 0x73, 0x00, 0x78,
        ];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogKey::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.size, 10);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 3);

        Ok(())
    }

    #[test]
    fn test_read_name_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogKey::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        let name: HfsString =
            test_struct.read_name(&HfsFormat::Hfs, &CharacterEncoding::MacRoman, &test_data)?;
        assert_eq!(
            name,
            HfsString::ByteString(ByteString {
                encoding: CharacterEncoding::MacRoman,
                elements: vec![0x6f, 0x73, 0x78],
            })
        );
        Ok(())
    }

    #[test]
    fn test_read_data_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsCatalogKey::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.size, 14);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 3);

        Ok(())
    }

    #[test]
    fn test_read_name_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsCatalogKey::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        let name: HfsString = test_struct.read_name(
            &HfsFormat::HfsPlus,
            &CharacterEncoding::MacRoman,
            &test_data,
        )?;
        assert_eq!(
            name,
            HfsString::Utf16String(Utf16String {
                elements: vec![0x006f, 0x0073, 0x0078]
            })
        );
        Ok(())
    }
}
