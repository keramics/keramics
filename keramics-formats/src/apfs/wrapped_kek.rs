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
use keramics_encryption::{AesKeyWrapContext, Pbkdf2HmacSha256Context};
use keramics_types::bytes_to_u32_le;

use super::credential::ApfsCredential;
use super::key_bag_packed_value::ApfsKeyBagPackedValue;

/// Apple File System (APFS) wrapped key encryption key (KEK).
pub struct ApfsWrappedKek {
    /// Flags
    pub flags: u32,

    /// Salt.
    pub salt: Vec<u8>,

    /// Number of iterations
    pub number_of_iterations: u64,

    /// Wrapped key data.
    pub wrapped_key_data: Vec<u8>,

    /// Key data.
    pub key_data: Vec<u8>,
}

impl ApfsWrappedKek {
    /// Creates a new wrapped key encryption key (KEK).
    pub fn new() -> Self {
        Self {
            flags: 0,
            salt: Vec::new(),
            number_of_iterations: 0,
            wrapped_key_data: Vec::new(),
            key_data: Vec::new(),
        }
    }

    /// Reads the wrapped key encryption key (KEK) from a buffer.
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
        if packed_value.tag != 0xa3 {
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
                    if packed_value.data_size != 16 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid sub packed value: {} with tag: 0x{:02x} - unsupported volume identifier size",
                            value_index, packed_value.tag
                        )));
                    }
                }
                0x82 => {
                    if packed_value.data_size < 6 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid sub packed value: {} with tag: 0x{:02x} - unsupported metadata size",
                            value_index, packed_value.tag
                        )));
                    }
                    data_offset += 2 + (packed_value.extended_size as usize);
                    self.flags = bytes_to_u32_le!(data, data_offset);
                }
                0x83 => {
                    data_offset += 2 + (packed_value.extended_size as usize);
                    self.wrapped_key_data = data[data_offset..data_end_offset].to_vec();
                }
                0x84 => {
                    if packed_value.data_size == 0 || packed_value.data_size > 8 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid sub packed value: {} with tag: 0x{:02x} - unsupported number of iterations size",
                            value_index, packed_value.tag
                        )));
                    }
                    data_offset += 2 + (packed_value.extended_size as usize);
                    for data_offset in data_offset..data_end_offset {
                        self.number_of_iterations =
                            (self.number_of_iterations << 8) | (data[data_offset] as u64);
                    }
                }
                0x85 => {
                    data_offset += 2 + (packed_value.extended_size as usize);
                    self.salt = data[data_offset..data_end_offset].to_vec();
                }
                _ => {}
            }
            data_offset = data_end_offset;
            value_index += 1;
        }
        Ok(())
    }

    /// Unlocks the key data using a credential.
    pub fn unlock_with_credential(
        &mut self,
        credential: &ApfsCredential,
    ) -> Result<bool, ErrorTrace> {
        match credential {
            ApfsCredential::Passphrase(passphrase) => {
                let key_size: usize = if self.flags & 0x02 != 0 { 16 } else { 32 };

                let mut derived_key: Vec<u8> = vec![0; key_size];

                let mut key_derivation_context: Pbkdf2HmacSha256Context =
                    Pbkdf2HmacSha256Context::new(&self.salt, self.number_of_iterations as usize);

                match key_derivation_context.derive_key(passphrase, &mut derived_key) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to derive key from passphrase"
                        );
                        return Err(error);
                    }
                }
                match self.unlock_with_kek(&derived_key) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to unlock key data using KEK"
                        );
                        Err(error)
                    }
                }
            }
            _ => Ok(false),
        }
    }

    /// Unlocks the key data using a key encrypting key (KEK).
    pub fn unlock_with_kek(&mut self, kek: &[u8]) -> Result<bool, ErrorTrace> {
        let wrapped_key_size: usize = self.wrapped_key_data.len();

        let mut key_wrap_context: AesKeyWrapContext = AesKeyWrapContext::new();

        match key_wrap_context.set_key(kek) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to set key in AES key wrap context"
                );
                return Err(error);
            }
        }
        let mut key_data: Vec<u8> = vec![0; wrapped_key_size];

        match key_wrap_context.unwrap(&self.wrapped_key_data, &mut key_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to unwrap key");
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!("ApfsUnwrappedKeyData", 0, &key_data, wrapped_key_size);

        if &key_data[0..8] == [0xa6; 8] {
            self.key_data = key_data[8..].to_vec();

            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xa3, 0x60, 0x80, 0x01, 0x00, 0x81, 0x10, 0x89, 0x30, 0x13, 0xc3, 0x08, 0x75, 0x47,
            0x3e, 0x91, 0xfe, 0xdb, 0xd8, 0xb9, 0x43, 0x7b, 0x29, 0x82, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x71, 0xd5, 0x83, 0x28, 0x87, 0xc4, 0x71, 0xf4, 0x64, 0x1c, 0x43,
            0x79, 0x71, 0x35, 0xd3, 0x2c, 0xd6, 0x0b, 0x06, 0xf3, 0xf6, 0x3c, 0x09, 0xb4, 0x76,
            0x8e, 0xf0, 0x39, 0x1f, 0x37, 0x3e, 0x5f, 0x59, 0x45, 0x2c, 0x8d, 0x32, 0x8f, 0x6c,
            0x5a, 0xe9, 0x06, 0x0e, 0xde, 0x84, 0x03, 0x01, 0x86, 0xa0, 0x85, 0x10, 0x5b, 0xa1,
            0xee, 0x87, 0x00, 0xb7, 0x77, 0xb0, 0xea, 0xcc, 0xaa, 0xc4, 0xf4, 0xdf, 0xb9, 0x4f,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsWrappedKek::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x00000000);
        assert_eq!(&test_struct.salt, &test_data[82..98]);
        assert_eq!(test_struct.number_of_iterations, 100000);
        assert_eq!(&test_struct.wrapped_key_data, &test_data[35..75]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsWrappedKek::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..1]);
        assert!(result.is_err());
    }
}
