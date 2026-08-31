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
use keramics_types::ByteString;

use crate::indexed_hash_map::IndexedHashMap;

use super::attribute::XfsAttribute;
use super::attributes_table_entry::XfsAttributesTableEntry;
use super::attributes_table_header::XfsAttributesTableHeader;

/// X File System (XFS) attributes table.
pub struct XfsAttributesTable {
    /// Character encoding.
    character_encoding: CharacterEncoding,
}

impl XfsAttributesTable {
    /// Creates a new attributes table.
    pub fn new(character_encoding: &CharacterEncoding) -> Self {
        Self {
            character_encoding: character_encoding.clone(),
        }
    }

    /// Reads the attributes table from a buffer.
    pub fn read_data(
        &mut self,
        data: &[u8],
        attributes: &mut IndexedHashMap<ByteString, XfsAttribute>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        keramics_core::debug_trace_data!("XfsAttributesTable", 0, data, data_size);

        keramics_core::debug_trace_structure!(XfsAttributesTableHeader::debug_read_data(&data));

        let mut header: XfsAttributesTableHeader = XfsAttributesTableHeader::new();

        match header.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read header");
                return Err(error);
            }
        }
        let mut data_offset: usize = 4;

        for entry_index in 0..header.number_of_entries {
            keramics_core::debug_trace_structure!(XfsAttributesTableEntry::debug_read_data(
                &data[data_offset..]
            ));
            let mut entry: XfsAttributesTableEntry = XfsAttributesTableEntry::new();

            match entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read entry: {}", entry_index)
                    );
                    return Err(error);
                }
            }
            data_offset += 3;

            let name_end_offset: usize = data_offset + (entry.name_size as usize);

            if entry.name_size == 0 || name_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - name size value out of bounds",
                    entry_index
                )));
            }
            let value_data_end_offset: usize = name_end_offset + (entry.value_data_size as usize);

            if value_data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - value data size value out of bounds",
                    entry_index
                )));
            }
            // Ignore the parent attribute (XFS_ATTR_PARENT).
            if entry.attribute_flags & 0x08 == 0 {
                let name: ByteString = XfsAttribute::read_name(
                    &self.character_encoding,
                    entry.attribute_flags,
                    &data[data_offset..name_end_offset],
                );
                let value_data: Vec<u8> = data[name_end_offset..value_data_end_offset].to_vec();

                attributes.insert(name, XfsAttribute::InlineData(value_data));
            }
            data_offset = value_data_end_offset;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x6c, 0x03, 0x00, 0x06, 0x0c, 0x08, 0x78, 0x61, 0x74, 0x74, 0x72, 0x31, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x3f, 0x04, 0x20, 0x65, 0x41, 0xe9, 0x07, 0x25, 0x04,
            0x73, 0x65, 0x6c, 0x69, 0x6e, 0x75, 0x78, 0x75, 0x6e, 0x63, 0x6f, 0x6e, 0x66, 0x69,
            0x6e, 0x65, 0x64, 0x5f, 0x75, 0x3a, 0x6f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x5f, 0x72,
            0x3a, 0x75, 0x6e, 0x6c, 0x61, 0x62, 0x65, 0x6c, 0x65, 0x64, 0x5f, 0x74, 0x3a, 0x73,
            0x30, 0x00, 0x08, 0x19, 0x00, 0x6d, 0x79, 0x78, 0x61, 0x74, 0x74, 0x72, 0x31, 0x4d,
            0x79, 0x20, 0x31, 0x73, 0x74, 0x20, 0x65, 0x78, 0x74, 0x65, 0x6e, 0x64, 0x65, 0x64,
            0x20, 0x61, 0x74, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = XfsAttributesTable::new(&CharacterEncoding::Utf8);
        let mut attributes: IndexedHashMap<ByteString, XfsAttribute> = IndexedHashMap::new();
        test_struct.read_data(&test_data, &mut attributes)?;

        assert_eq!(attributes.len(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = XfsAttributesTable::new(&CharacterEncoding::Utf8);

        let test_data: Vec<u8> = get_test_data();
        let mut attributes: IndexedHashMap<ByteString, XfsAttribute> = IndexedHashMap::new();
        let result = test_struct.read_data(&test_data[0..3], &mut attributes);
        assert!(result.is_err());
    }
}
