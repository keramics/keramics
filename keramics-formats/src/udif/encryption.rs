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
use keramics_encryption::{
    AesContext, CryptCbc, CryptContext, Des3Context, HmacSha1Context, Pbkdf2HmacSha1Context,
    Pkcs7Context,
};

/// Universal Disk Image Format (UDIF) encryption context.
pub enum UdifEncryptionContext {
    Aes(AesContext),
    Des3(Des3Context),
    None,
}

impl UdifEncryptionContext {
    /// Decrypts data.
    pub fn decrypt_cbc(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        match self {
            UdifEncryptionContext::Aes(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            UdifEncryptionContext::Des3(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            UdifEncryptionContext::None => {
                return Err(keramics_core::error_trace_new!("Unable to decrypt data"));
            }
        }
    }
}

/// Universal Disk Image Format (UDIF) HMAC context.
pub enum UdifHmacContext {
    HmacSha1 {
        key: Vec<u8>,
        context: HmacSha1Context,
    },
    None,
}

impl UdifHmacContext {
    /// Calculates a HMAC.
    pub fn calculate_hmac(&mut self, data: &[u8]) -> Result<Vec<u8>, ErrorTrace> {
        match self {
            UdifHmacContext::HmacSha1 { key, context } => {
                let mut hmac: Vec<u8> = vec![0; 20];
                context.calculate_hmac(key, data, &mut hmac)?;
                Ok(hmac)
            }
            UdifHmacContext::None => {
                return Err(keramics_core::error_trace_new!("Unable to calculate HMAC"));
            }
        }
    }
}

/// Universal Disk Image Format (UDIF) key derivation context.
pub enum UdifKeyDerivationContext {
    Pbkdf2HmacSha1(Pbkdf2HmacSha1Context),
}

impl UdifKeyDerivationContext {
    /// Derives a key from the password.
    pub fn derive_key(&mut self, password: &[u8], key: &mut [u8]) -> Result<(), ErrorTrace> {
        match self {
            UdifKeyDerivationContext::Pbkdf2HmacSha1(context) => context.derive_key(password, key),
        }
    }
}

/// Universal Disk Image Format (UDIF) encryption.
pub struct UdifEncryption {}

impl UdifEncryption {
    /// Adds padding.
    pub fn add_padding(
        padding_type: u32,
        block_size: usize,
        data: &mut Vec<u8>,
    ) -> Result<(), ErrorTrace> {
        match padding_type {
            0 => Ok(()),
            2 | 3 => {
                if block_size == 0 || block_size > 255 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid block size value out of bounds"
                    ));
                }
                let data_size: usize = data.len();
                let padding_size: usize = block_size - (data_size % block_size);
                let padding_value: u8 = (padding_type - 2) as u8;

                data.resize(data_size + padding_size, padding_value);

                Ok(())
            }
            7 => {
                let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

                match pkcs7_context.add_padding(block_size, data) {
                    Ok(_) => Ok(()),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to add PKCS7 padding");
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported padding type: {}",
                    padding_type
                )));
            }
        }
    }

    /// Retrieve an encryption context.
    pub fn get_encryption_context(
        encryption_method: u32,
        encryption_mode: u32,
        key: &[u8],
    ) -> Result<Option<UdifEncryptionContext>, ErrorTrace> {
        match encryption_method {
            0x00000011 => match encryption_mode {
                5 | 6 => {
                    let mut context: Des3Context = Des3Context::new();

                    match context.set_key(key) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to set key in DES3-CBC context"
                            );
                            return Err(error);
                        }
                    }
                    Ok(Some(UdifEncryptionContext::Des3(context)))
                }
                _ => Ok(None),
            },
            0x80000001 => match encryption_mode {
                5 | 6 => {
                    let mut context: AesContext = AesContext::new();

                    match context.set_key(key) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to set key in AES-CBC context"
                            );
                            return Err(error);
                        }
                    }
                    Ok(Some(UdifEncryptionContext::Aes(context)))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    /// Retrieve a HMAC context.
    pub fn get_hmac_context(
        hmac_method: u32,
        key: &[u8],
    ) -> Result<Option<UdifHmacContext>, ErrorTrace> {
        match hmac_method {
            91 => Ok(Some(UdifHmacContext::HmacSha1 {
                key: key.to_vec(),
                context: HmacSha1Context::new(),
            })),
            _ => Ok(None),
        }
    }

    /// Retrieve a key derivation context.
    pub fn get_key_derivation_context(
        key_derivation_method: u32,
        salt: &[u8],
        number_of_iterations: usize,
    ) -> Result<Option<UdifKeyDerivationContext>, ErrorTrace> {
        match key_derivation_method {
            103 => Ok(Some(UdifKeyDerivationContext::Pbkdf2HmacSha1(
                Pbkdf2HmacSha1Context::new(salt, number_of_iterations),
            ))),
            _ => Ok(None),
        }
    }

    /// Removes padding.
    pub fn remove_padding<'a>(
        padding_type: u32,
        block_size: usize,
        padded_data: &'a [u8],
    ) -> Result<&'a [u8], ErrorTrace> {
        match padding_type {
            0 => Ok(padded_data),
            2 | 3 => {
                if block_size == 0 || block_size > 255 {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid block size value out of bounds"
                    ));
                }
                let padded_data_size: usize = padded_data.len();

                if padded_data_size < block_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid padded data size value too small"
                    ));
                }
                if padded_data_size % block_size != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid padded data size value not a multitude of block size: {}",
                        block_size
                    )));
                }
                let padding_size: usize = block_size - (padded_data_size % block_size);

                if padding_size == 0 || padding_size > block_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid padding size value out of bounds"
                    ));
                }
                let padding_offset: usize = padded_data_size - padding_size;
                let padding_value: u8 = (padding_type - 2) as u8;

                for byte_value in &padded_data[padding_offset..] {
                    if *byte_value != padding_value {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid padding value at offset: {}",
                            padding_offset
                        )));
                    }
                }
                Ok(&padded_data[0..padding_offset])
            }
            7 => {
                let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

                match pkcs7_context.remove_padding(block_size, padded_data) {
                    Ok(data) => Ok(data),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to remove PKCS7 padding"
                        );
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported padding type: {}",
                    padding_type
                )));
            }
        }
    }
}
