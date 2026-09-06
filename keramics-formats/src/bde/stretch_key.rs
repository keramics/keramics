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
use keramics_types::bytes_to_u16_le;

use super::metadata_entry_header::BdeMetadataEntryHeader;
use super::metadata_property::BdeMetadataProperty;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "encryption_method", data_type = "u16", format = "hex"),
        field(name = "unknown1", data_type = "u16"),
        field(name = "salt", data_type = "[u8; 16]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) stretch key.
pub struct BdeStretchKey {
    /// Encryption method.
    pub encryption_method: u16,

    /// Salt.
    pub salt: Vec<u8>,

    /// Properties.
    pub properties: Vec<BdeMetadataProperty>,
}

impl BdeStretchKey {
    /// Creates a new stretch key.
    pub fn new() -> Self {
        Self {
            encryption_method: 0,
            salt: Vec::new(),
            properties: Vec::new(),
        }
    }

    /// Reads the stretch key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 20 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.encryption_method = bytes_to_u16_le!(data, 0);
        self.salt = data[4..20].to_vec();

        let mut data_offset: usize = 20;
        let mut entry_index: usize = 0;

        while data_offset < data_size - 8 {
            let data_end_offset: usize = data_offset + 8;

            if &data[data_offset..data_end_offset] == &[0; 8] {
                break;
            }
            keramics_core::debug_trace_structure!(BdeMetadataEntryHeader::debug_read_data(
                &data[data_offset..]
            ));
            let mut entry_header: BdeMetadataEntryHeader = BdeMetadataEntryHeader::new();

            match entry_header.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read metadata entry: {} header", entry_index),
                    );
                    return Err(error);
                }
            }
            if entry_header.entry_type != 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid metadata entry: {} unsupported entry type: 0x{:04x}",
                    entry_index, entry_header.entry_type
                )));
            }
            if entry_header.entry_size < 8
                || (entry_header.entry_size as usize) > data_size - data_offset
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid metadata entry: {} size value out of bounds",
                    entry_index
                )));
            }
            let entry_data_size: usize = (entry_header.entry_size as usize) - 8;

            data_offset += 8;

            let data_end_offset: usize = data_offset + entry_data_size;

            keramics_core::debug_trace_data!(
                "BdeMetadataPropertyData",
                data_offset,
                &data[data_offset..data_end_offset],
                entry_data_size
            );
            self.properties.push(BdeMetadataProperty::new(
                entry_header.value_type,
                data_offset,
                entry_data_size,
            ));
            data_offset = data_end_offset;
            entry_index += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x01, 0x10, 0x00, 0x00, 0xfe, 0xdc, 0xfa, 0xa5, 0xe2, 0x6e, 0xe3, 0x88, 0x0d, 0x2b,
            0xdb, 0x2e, 0xe4, 0xe4, 0x42, 0x8f, 0x50, 0x00, 0x00, 0x00, 0x05, 0x00, 0x01, 0x00,
            0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x02, 0x00, 0x00, 0x00, 0x1f, 0x36,
            0xdb, 0xbe, 0x1c, 0x3e, 0x0a, 0xd2, 0x95, 0x02, 0x90, 0x07, 0xf8, 0xd8, 0x1e, 0xa0,
            0xb4, 0x83, 0x96, 0xae, 0x1a, 0xa4, 0xb1, 0xa1, 0xab, 0x59, 0x0f, 0xda, 0xeb, 0xc7,
            0x4c, 0x70, 0x23, 0x49, 0xb8, 0x86, 0xbd, 0x9c, 0x79, 0x53, 0xc7, 0x51, 0x48, 0x09,
            0x9b, 0x52, 0xeb, 0xe7, 0xc3, 0xd9, 0x06, 0xf2, 0x47, 0x08, 0x0f, 0xeb, 0x46, 0x6a,
            0x94, 0x19,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeStretchKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.encryption_method, 0x1001);
        assert_eq!(&test_struct.salt, &test_data[4..20]);
        assert_eq!(test_struct.properties.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeStretchKey::new();
        let result = test_struct.read_data(&test_data[0..19]);
        assert!(result.is_err());
    }
}
