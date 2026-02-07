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
use keramics_types::{ByteString, Utf16String};

use super::catalog_thread_record_extended::HfsExtendedCatalogThreadRecord;
use super::catalog_thread_record_standard::HfsStandardCatalogThreadRecord;
use super::enums::HfsFormat;
use super::string::HfsString;

/// Hierarchical File System (HFS) catalog thread record.
pub struct HfsCatalogThreadRecord {
    /// Record type.
    pub record_type: u16,

    /// Parent identifier (CNID).
    pub parent_identifier: u32,

    /// Name size.
    pub name_size: u16,

    /// Name.
    pub name: HfsString,
}

impl HfsCatalogThreadRecord {
    /// Creates a new catalog thread record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            parent_identifier: 0,
            name_size: 0,
            name: HfsString::new(),
        }
    }

    /// Reads the catalog thread record for debugging.
    pub fn debug_read_data(format: &HfsFormat, data: &[u8]) -> String {
        match format {
            HfsFormat::Hfs => HfsStandardCatalogThreadRecord::debug_read_data(data),
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogThreadRecord::debug_read_data(data)
            }
        }
    }

    /// Reads the catalog thread record from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardCatalogThreadRecord::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogThreadRecord::read_data(self, data)?;
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
            HfsFormat::Hfs => (15, self.name_size as usize),
            HfsFormat::HfsPlus | HfsFormat::HfsX => (10, (self.name_size as usize) * 2),
        };
        let name_end_offset: usize = name_offset + name_size;

        if name_end_offset > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid name size: {} value out of bounds",
                name_size
            )));
        }
        keramics_core::debug_trace_data!(
            "HfsCatalogThreadRecord name",
            name_offset,
            &data[name_offset..name_end_offset],
            name_size
        );
        let name: HfsString = match format {
            HfsFormat::Hfs => {
                let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);
                byte_string.read_data(&data[name_offset..name_end_offset]);
                HfsString::ByteString(byte_string)
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                let utf16_string: Utf16String =
                    Utf16String::from_be_bytes(&data[name_offset..name_end_offset]);
                HfsString::Utf16String(utf16_string)
            }
        };
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x68, 0x66, 0x73, 0x5f, 0x74, 0x65, 0x73, 0x74,
        ];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![
            0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0c, 0x00, 0x68, 0x00, 0x66,
            0x00, 0x73, 0x00, 0x70, 0x00, 0x6c, 0x00, 0x75, 0x00, 0x73, 0x00, 0x5f, 0x00, 0x74,
            0x00, 0x65, 0x00, 0x73, 0x00, 0x74,
        ];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogThreadRecord::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0300);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 8);

        Ok(())
    }

    #[test]
    fn test_read_name_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogThreadRecord::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        let name: HfsString =
            test_struct.read_name(&HfsFormat::Hfs, &CharacterEncoding::MacRoman, &test_data)?;
        assert_eq!(
            name,
            HfsString::ByteString(ByteString {
                encoding: CharacterEncoding::MacRoman,
                elements: vec![0x68, 0x66, 0x73, 0x5f, 0x74, 0x65, 0x73, 0x74],
            })
        );
        Ok(())
    }

    #[test]
    fn test_read_data_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsCatalogThreadRecord::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0003);
        assert_eq!(test_struct.parent_identifier, 1);
        assert_eq!(test_struct.name_size, 12);

        Ok(())
    }

    #[test]
    fn test_read_name_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsCatalogThreadRecord::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        let name: HfsString = test_struct.read_name(
            &HfsFormat::HfsPlus,
            &CharacterEncoding::MacRoman,
            &test_data,
        )?;
        assert_eq!(
            name,
            HfsString::Utf16String(Utf16String {
                elements: vec![
                    0x0068, 0x0066, 0x0073, 0x0070, 0x006c, 0x0075, 0x0073, 0x005f, 0x0074, 0x0065,
                    0x0073, 0x0074,
                ]
            })
        );
        Ok(())
    }
}
