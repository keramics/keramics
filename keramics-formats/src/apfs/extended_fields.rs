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

use std::collections::HashMap;

use keramics_core::ErrorTrace;

use crate::util::calculate_alignment_padding;

use super::extended_fields_entry::ApfsExtendedFieldsEntry;
use super::extended_fields_header::ApfsExtendedFieldsHeader;

/// Apple File System (APFS) extended fields.
pub struct ApfsExtendedFields {
    /// Fields.
    pub fields: HashMap<u8, Vec<u8>>,
}

impl ApfsExtendedFields {
    /// Creates new extended fields.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Retrieves a specific extended field.
    pub fn get(&self, field_type: &u8) -> Option<&[u8]> {
        self.fields.get(field_type).map(|value| value.as_slice())
    }

    /// Reads the extended fields from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        keramics_core::debug_trace_structure!(ApfsExtendedFieldsHeader::debug_read_data(data));
        let mut extended_fields_header: ApfsExtendedFieldsHeader = ApfsExtendedFieldsHeader::new();

        match extended_fields_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read extended fields header"
                );
                return Err(error);
            }
        }
        let mut data_offset: usize = 4;

        if (extended_fields_header.number_of_fields as usize) > ((data_size - data_offset) / 4) {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of extended fields value out of bounds"
            ));
        }
        if (extended_fields_header.data_size as usize) > data_size - data_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid extended fields data size value out of bounds"
            ));
        }
        let mut value_data_offset: usize =
            data_offset + ((extended_fields_header.number_of_fields as usize) * 4);

        for field_index in 0..extended_fields_header.number_of_fields {
            keramics_core::debug_trace_structure!(ApfsExtendedFieldsEntry::debug_read_data(
                &data[data_offset..]
            ));
            let mut extended_fields_entry: ApfsExtendedFieldsEntry = ApfsExtendedFieldsEntry::new();

            match extended_fields_entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read extended fields entry: {}", field_index)
                    );
                    return Err(error);
                }
            }
            data_offset += 4;

            let value_data_end_offset: usize =
                value_data_offset + (extended_fields_entry.data_size as usize);

            if value_data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid extended field entry: {} value data size value out of bounds",
                    field_index
                )));
            }
            keramics_core::debug_trace_data!(
                format!("ApfsExtendedFieldData: {}", field_index),
                data_offset,
                &data[value_data_offset..value_data_end_offset],
                extended_fields_entry.data_size,
            );
            if self.fields.contains_key(&extended_fields_entry.field_type) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid extended field entry: {} - field of type: {} already set",
                    field_index, extended_fields_entry.field_type
                )));
            }
            self.fields.insert(
                extended_fields_entry.field_type,
                data[value_data_offset..value_data_end_offset].to_vec(),
            );
            value_data_offset = value_data_end_offset;

            let alignment_padding: usize =
                calculate_alignment_padding(extended_fields_entry.data_size as usize, 8);

            if alignment_padding > 0 {
                // TODO: debug print alignment padding.
                value_data_offset += alignment_padding;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x01, 0x00, 0x08, 0x00, 0x04, 0x02, 0x05, 0x00, 0x72, 0x6f, 0x6f, 0x74, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsExtendedFields::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.fields.len(), 1);
        assert_eq!(
            test_struct.fields.get(&4),
            Some(test_data[8..13].to_vec()).as_ref()
        );

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsExtendedFields::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
