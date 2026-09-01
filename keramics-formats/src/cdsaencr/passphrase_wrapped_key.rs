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
use keramics_types::{bytes_to_u32_be, bytes_to_u64_be};

use super::constants::*;
use super::credential::CdsaEncrCredential;
use super::encryption::{CdsaEncrCipherContext, CdsaEncrEncryption, CdsaEncrKeyDerivationContext};
use super::encryption_type::CdsaEncrEncryptionType;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "key_derivation_method", data_type = "u32"),
        field(name = "number_of_iterations", data_type = "u64"),
        field(name = "salt_size", data_type = "u32"),
        field(name = "salt", data_type = "[u8; 32]", format = "hex"),
        field(name = "initialization_vector_size", data_type = "u32"),
        field(name = "initialization_vector", data_type = "[u8; 32]", format = "hex"),
        field(name = "encryption_key_size", data_type = "u32"),
        field(name = "encryption_method", data_type = "u32", format = "hex"),
        field(name = "padding_type", data_type = "u32"),
        field(name = "encryption_mode", data_type = "u32"),
        field(name = "wrapped_key_data_size", data_type = "u32"),
        field(name = "wrapped_key_data", data_type = "[u8; 64]", format = "hex"),
        field(name = "uknown1", data_type = "[u8; 448]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// Mac OS Encrypted Encoding (cdsaencr) passphrase wrapped key.
pub struct CdsaEncrPassphraseWrappedKey {
    /// Key derivation method.
    pub key_derivation_method: u32,

    /// Number of iterations.
    pub number_of_iterations: u64,

    /// Salt.
    pub salt: Vec<u8>,

    /// Initialization vector size.
    pub initialization_vector_size: usize,

    /// Initialization vector.
    pub initialization_vector: Vec<u8>,

    /// Encryption type.
    pub encryption_type: CdsaEncrEncryptionType,

    /// Padding type.
    pub padding_type: u32,

    /// Wrapped key data.
    pub wrapped_key_data: Vec<u8>,

    /// Key data.
    pub key_data: Vec<u8>,
}

impl CdsaEncrPassphraseWrappedKey {
    /// Creates a new passphrase wrapped key.
    pub fn new() -> Self {
        Self {
            key_derivation_method: 0,
            number_of_iterations: 0,
            salt: Vec::new(),
            initialization_vector_size: 0,
            initialization_vector: Vec::new(),
            encryption_type: CdsaEncrEncryptionType::new(),
            padding_type: 0,
            wrapped_key_data: Vec::new(),
            key_data: Vec::new(),
        }
    }

    /// Reads the passphrase wrapped key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 104 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.key_derivation_method = bytes_to_u32_be!(data, 0);
        self.number_of_iterations = bytes_to_u64_be!(data, 4);

        let salt_size: usize = bytes_to_u32_be!(data, 12) as usize;

        if salt_size > 32 {
            return Err(keramics_core::error_trace_new!(
                "Invalid salt size value out of bounds"
            ));
        }
        let data_end_offset: usize = 16 + salt_size;

        self.salt = data[16..data_end_offset].to_vec();

        self.initialization_vector_size = bytes_to_u32_be!(data, 48) as usize;

        if self.initialization_vector_size > 32 {
            return Err(keramics_core::error_trace_new!(
                "Invalid initialization vector size value out of bounds"
            ));
        }
        let data_end_offset: usize = 52 + self.initialization_vector_size;

        self.initialization_vector = data[52..data_end_offset].to_vec();
        self.encryption_type.key_size = (bytes_to_u32_be!(data, 84) / 8) as usize;
        self.encryption_type.method = bytes_to_u32_be!(data, 88);
        self.padding_type = bytes_to_u32_be!(data, 92);
        self.encryption_type.mode = bytes_to_u32_be!(data, 96);

        let wrapped_key_data_size: usize = bytes_to_u32_be!(data, 100) as usize;

        if wrapped_key_data_size > 64 {
            return Err(keramics_core::error_trace_new!(
                "Invalid wrapped key data size value out of bounds"
            ));
        }
        let data_end_offset: usize = 104 + wrapped_key_data_size;
        self.wrapped_key_data = data[104..data_end_offset].to_vec();

        Ok(())
    }

    /// Unlocks the key.
    pub fn unlock(&mut self, credential: &CdsaEncrCredential) -> Result<bool, ErrorTrace> {
        match credential {
            CdsaEncrCredential::Passphrase(passphrase) => {
                let mut key_derivation_context: CdsaEncrKeyDerivationContext =
                    match CdsaEncrEncryption::get_key_derivation_context(
                        self.key_derivation_method,
                        &self.salt,
                        self.number_of_iterations as usize,
                    ) {
                        Ok(Some(context)) => context,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported key derivation method: {}",
                                self.key_derivation_method
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve key derivation context for method: {}",
                                    self.key_derivation_method
                                )
                            );
                            return Err(error);
                        }
                    };
                let mut key: Vec<u8> = vec![0; self.encryption_type.key_size];

                match key_derivation_context.derive_key(passphrase, &mut key) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to derive key from passphrase"
                        );
                        return Err(error);
                    }
                }
                let mut initialization_vector: Vec<u8> = self.initialization_vector.to_vec();

                match CdsaEncrEncryption::add_padding(2, 16, &mut initialization_vector) {
                    Ok(data) => data,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to add padding to initialization vector"
                        );
                        return Err(error);
                    }
                };
                let cipher_context: CdsaEncrCipherContext =
                    match CdsaEncrEncryption::get_cipher_context(&self.encryption_type, &key) {
                        Ok(Some(context)) => context,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported encryption type: {}",
                                self.encryption_type
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve encryption context for type: {}",
                                    self.encryption_type
                                )
                            );
                            return Err(error);
                        }
                    };
                let mut padded_key_data: Vec<u8> = vec![0; self.wrapped_key_data.len()];

                match cipher_context.decrypt(
                    &mut initialization_vector,
                    &self.wrapped_key_data,
                    &mut padded_key_data,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decrypt passphrase encrypted key data"
                        );
                        return Err(error);
                    }
                }
                keramics_core::debug_trace_data!(
                    "CdsaEncrPaddedKeyData",
                    0,
                    &padded_key_data,
                    padded_key_data.len(),
                );
                let key_data: &[u8] = match CdsaEncrEncryption::remove_padding(
                    self.padding_type,
                    self.initialization_vector_size,
                    &padded_key_data,
                ) {
                    Ok(data) => data,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to remove padding from key data"
                        );
                        return Err(error);
                    }
                };
                let key_data_size: usize = key_data.len();

                if key_data_size < 5 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid key data size value out of bounds"
                    ));
                }
                let signature_offset: usize = key_data_size - 5;

                if &key_data[signature_offset..key_data_size] == CDSAENCR_WRAPPED_KEY_SIGNATURE {
                    self.key_data = key_data[0..signature_offset].to_vec();

                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x00, 0x00, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xc4, 0xb4, 0x00, 0x00,
            0x00, 0x14, 0x61, 0xed, 0x1b, 0xf6, 0xd4, 0x79, 0x40, 0x06, 0x65, 0x08, 0xea, 0x72,
            0x4a, 0x22, 0x91, 0x32, 0x88, 0x4d, 0x9b, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0xcb, 0x45, 0x99, 0x05,
            0x4f, 0xcc, 0xf5, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0xc0, 0x80, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00,
            0x00, 0x06, 0x00, 0x00, 0x00, 0x40, 0xf4, 0x04, 0x5c, 0x99, 0x54, 0x52, 0xd9, 0x2a,
            0xdf, 0xa4, 0x3a, 0x77, 0xbf, 0x22, 0x8e, 0xcf, 0xee, 0x2a, 0x0d, 0x95, 0x61, 0x13,
            0x6f, 0x85, 0xa7, 0x98, 0x4f, 0x13, 0x67, 0xc1, 0xa5, 0x7f, 0xed, 0xac, 0x77, 0x99,
            0xb7, 0x4b, 0x3b, 0xd4, 0x09, 0xf1, 0x68, 0xdc, 0x0b, 0x65, 0x2f, 0xf8, 0x2e, 0xc1,
            0x36, 0x14, 0x0c, 0x3e, 0x27, 0xc3, 0xa9, 0x99, 0x71, 0x41, 0x8c, 0x43, 0x45, 0xbf,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CdsaEncrPassphraseWrappedKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.key_derivation_method, 103);
        assert_eq!(test_struct.number_of_iterations, 312500);
        assert_eq!(test_struct.salt, &test_data[16..36]);
        assert_eq!(test_struct.initialization_vector_size, 8);
        assert_eq!(test_struct.initialization_vector, &test_data[52..60]);
        assert_eq!(test_struct.encryption_type.key_size, 24);
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.padding_type, 7);
        assert_eq!(test_struct.encryption_type.mode, 6);
        assert_eq!(test_struct.wrapped_key_data, &test_data[104..168]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = CdsaEncrPassphraseWrappedKey::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..103]);
        assert!(result.is_err());
    }
}
