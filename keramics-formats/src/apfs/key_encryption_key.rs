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

use super::credential::ApfsCredential;
use super::key_bag_packed_value::ApfsKeyBagPackedValue;
use super::wrapped_kek::ApfsWrappedKek;

/// Apple File System (APFS) key encryption key (KEK).
pub struct ApfsKeyEncryptionKey {
    /// HMAC
    pub hmac: Vec<u8>,

    /// Wrapped KEK.
    pub wrapped_kek: ApfsWrappedKek,
}

impl ApfsKeyEncryptionKey {
    /// Creates a new key encryption key (KEK).
    pub fn new() -> Self {
        Self {
            hmac: Vec::new(),
            wrapped_kek: ApfsWrappedKek::new(),
        }
    }

    /// Reads the key encryption key (KEK) from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        keramics_core::debug_trace_structure!(ApfsKeyBagPackedValue::debug_read_data(data));

        let mut packed_value: ApfsKeyBagPackedValue = ApfsKeyBagPackedValue::new();

        match packed_value.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read main packed value: 0");
                return Err(error);
            }
        }
        if packed_value.tag != 0x30 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported packed main packed value tag: 0x{:02x}",
                packed_value.tag
            )));
        }
        let mut data_offset: usize = 2 + (packed_value.extended_size as usize);
        let mut value_index: usize = 0;

        if (packed_value.data_size as usize) > data_size - data_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid main packed value data size value out of bounds"
            ));
        }
        while data_offset < data_size {
            let data_end_offset: usize = data_offset + 2;

            if data_size - data_offset < 2 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid sub packed value: {} header size value out of bounds",
                    value_index
                )));
            }
            if data[data_offset..data_end_offset] == [0; 2] {
                break;
            }
            keramics_core::debug_trace_structure!(ApfsKeyBagPackedValue::debug_read_data(
                &data[data_offset..]
            ));
            let mut packed_value: ApfsKeyBagPackedValue = ApfsKeyBagPackedValue::new();

            match packed_value.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read sub packed value: {}", value_index)
                    );
                    return Err(error);
                }
            }
            if (packed_value.data_size as usize) > data_size - data_end_offset {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid sub packed value: {} with tag: 0x{:02x} - data size value out of bounds",
                    value_index, packed_value.tag
                )));
            }
            let data_end_offset: usize = data_offset
                + 2
                + (packed_value.extended_size as usize)
                + (packed_value.data_size as usize);

            match packed_value.tag {
                0x81 => {
                    data_offset += 2 + (packed_value.extended_size as usize);

                    keramics_core::debug_trace_data!(
                        format!("ApfsKeyBagPackedValueData: {}", value_index),
                        data_offset,
                        &data[data_offset..data_end_offset],
                        packed_value.data_size,
                    );
                    if packed_value.data_size != 32 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid sub packed value: {} with tag: 0x{:02x} - unsupported HMAC size",
                            value_index, packed_value.tag
                        )));
                    }
                    self.hmac = data[data_offset..data_end_offset].to_vec();
                }
                0x82 => {
                    data_offset += 2 + (packed_value.extended_size as usize);

                    keramics_core::debug_trace_data!(
                        format!("ApfsKeyBagPackedValueData: {}", value_index),
                        data_offset,
                        &data[data_offset..data_end_offset],
                        packed_value.data_size,
                    );
                    if packed_value.data_size != 8 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid sub packed value: {} with tag: 0x{:02x} - unsupported salt? size",
                            value_index, packed_value.tag
                        )));
                    }
                }
                0xa3 => {
                    keramics_core::debug_trace_data!(
                        format!("ApfsWrappedKek: {}", value_index),
                        data_offset,
                        &data[data_offset..data_end_offset],
                        packed_value.data_size,
                    );
                    match self
                        .wrapped_kek
                        .read_data(&data[data_offset..data_end_offset])
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read sub packed value: {} with tag: 0x{:02x}",
                                    value_index, packed_value.tag
                                )
                            );
                            return Err(error);
                        }
                    }
                }
                _ => {}
            }
            data_offset = data_end_offset;
            value_index += 1;
        }
        Ok(())
    }

    /// Unlocks the key using a credential.
    pub fn unlock_with_credential(
        &mut self,
        credential: &ApfsCredential,
    ) -> Result<bool, ErrorTrace> {
        self.wrapped_kek.unlock_with_credential(credential)
    }

    /// Unlocks the key using a key encryption key (KEK).
    pub fn unlock_with_kek(&mut self, kek: &[u8]) -> Result<bool, ErrorTrace> {
        self.wrapped_kek.unlock_with_kek(kek)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x30, 0x81, 0x91, 0x80, 0x01, 0x00, 0x81, 0x20, 0xe7, 0xfd, 0xdd, 0x8d, 0x2d, 0x96,
            0xf9, 0xa4, 0x4a, 0x02, 0xa4, 0xa4, 0x73, 0x23, 0xcb, 0x6c, 0xf0, 0xe3, 0xd2, 0xf5,
            0xbb, 0x1d, 0x77, 0xc9, 0x23, 0x75, 0x1a, 0xa1, 0x40, 0x55, 0x49, 0x99, 0x82, 0x08,
            0x71, 0xeb, 0x66, 0x6d, 0x46, 0x1d, 0x84, 0xeb, 0xa3, 0x60, 0x80, 0x01, 0x00, 0x81,
            0x10, 0x89, 0x30, 0x13, 0xc3, 0x08, 0x75, 0x47, 0x3e, 0x91, 0xfe, 0xdb, 0xd8, 0xb9,
            0x43, 0x7b, 0x29, 0x82, 0x08, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x71, 0xd5, 0x83,
            0x28, 0x87, 0xc4, 0x71, 0xf4, 0x64, 0x1c, 0x43, 0x79, 0x71, 0x35, 0xd3, 0x2c, 0xd6,
            0x0b, 0x06, 0xf3, 0xf6, 0x3c, 0x09, 0xb4, 0x76, 0x8e, 0xf0, 0x39, 0x1f, 0x37, 0x3e,
            0x5f, 0x59, 0x45, 0x2c, 0x8d, 0x32, 0x8f, 0x6c, 0x5a, 0xe9, 0x06, 0x0e, 0xde, 0x84,
            0x03, 0x01, 0x86, 0xa0, 0x85, 0x10, 0x5b, 0xa1, 0xee, 0x87, 0x00, 0xb7, 0x77, 0xb0,
            0xea, 0xcc, 0xaa, 0xc4, 0xf4, 0xdf, 0xb9, 0x4f,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsKeyEncryptionKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(&test_struct.hmac, &test_data[8..40]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsKeyEncryptionKey::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..1]);
        assert!(result.is_err());
    }
}
