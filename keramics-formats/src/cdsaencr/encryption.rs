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

use super::encryption_type::CdsaEncrEncryptionType;

/// Mac OS Encrypted Encoding (cdsaencr) cipher context.
#[derive(Clone)]
pub enum CdsaEncrCipherContext {
    Aes(AesContext),
    Des3(Des3Context),
    None,
}

impl CdsaEncrCipherContext {
    /// Decrypts data.
    pub fn decrypt(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        match self {
            CdsaEncrCipherContext::Aes(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            CdsaEncrCipherContext::Des3(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            CdsaEncrCipherContext::None => Err(keramics_core::error_trace_new!("Not implemented")),
        }
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        match self {
            CdsaEncrCipherContext::Aes(context) => context.set_key(key),
            CdsaEncrCipherContext::Des3(context) => context.set_key(key),
            CdsaEncrCipherContext::None => Err(keramics_core::error_trace_new!("Not implemented")),
        }
    }
}

/// Mac OS Encrypted Encoding (cdsaencr) HMAC context.
#[derive(Clone)]
pub enum CdsaEncrHmacContext {
    HmacSha1 {
        key: Vec<u8>,
        context: HmacSha1Context,
    },
    None,
}

impl CdsaEncrHmacContext {
    /// Calculates a HMAC.
    pub fn calculate_hmac(&mut self, data: &[u8]) -> Result<Vec<u8>, ErrorTrace> {
        match self {
            CdsaEncrHmacContext::HmacSha1 { key, context } => {
                let mut hmac: Vec<u8> = vec![0; 20];
                context.calculate_hmac(key, data, &mut hmac)?;
                Ok(hmac)
            }
            CdsaEncrHmacContext::None => Err(keramics_core::error_trace_new!("Not implemented")),
        }
    }
}

/// Mac OS Encrypted Encoding (cdsaencr) key derivation context.
pub enum CdsaEncrKeyDerivationContext {
    Pbkdf2HmacSha1(Pbkdf2HmacSha1Context),
}

impl CdsaEncrKeyDerivationContext {
    /// Derives a key from the password.
    pub fn derive_key(&mut self, password: &[u8], key: &mut [u8]) -> Result<(), ErrorTrace> {
        match self {
            CdsaEncrKeyDerivationContext::Pbkdf2HmacSha1(context) => {
                context.derive_key(password, key)
            }
        }
    }
}

/// Mac OS Encrypted Encoding (cdsaencr) encryption.
pub struct CdsaEncrEncryption {}

impl CdsaEncrEncryption {
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

    /// Retrieves a cipher context.
    pub fn get_cipher_context(
        encryption_type: &CdsaEncrEncryptionType,
        key: &[u8],
    ) -> Result<Option<CdsaEncrCipherContext>, ErrorTrace> {
        let mut cipher_context: CdsaEncrCipherContext = match encryption_type.method {
            0x00000011 => match encryption_type.mode {
                5 | 6 => CdsaEncrCipherContext::Des3(Des3Context::new()),
                _ => return Ok(None),
            },
            0x80000001 => match encryption_type.mode {
                5 | 6 => CdsaEncrCipherContext::Aes(AesContext::new()),
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        match cipher_context.set_key(key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        Ok(Some(cipher_context))
    }

    /// Retrieves a HMAC context.
    pub fn get_hmac_context(
        hmac_method: u32,
        key: &[u8],
    ) -> Result<Option<CdsaEncrHmacContext>, ErrorTrace> {
        match hmac_method {
            91 => Ok(Some(CdsaEncrHmacContext::HmacSha1 {
                key: key.to_vec(),
                context: HmacSha1Context::new(),
            })),
            _ => Ok(None),
        }
    }

    /// Retrieves a key derivation context.
    pub fn get_key_derivation_context(
        key_derivation_method: u32,
        salt: &[u8],
        number_of_iterations: usize,
    ) -> Result<Option<CdsaEncrKeyDerivationContext>, ErrorTrace> {
        match key_derivation_method {
            103 => Ok(Some(CdsaEncrKeyDerivationContext::Pbkdf2HmacSha1(
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

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_encryption::{AesContext, Des3Context};

    #[test]
    fn test_add_padding_with_no_padding() -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0x01, 0x02, 0x03];
        CdsaEncrEncryption::add_padding(0, 8, &mut data)?;

        assert_eq!(data, vec![0x01, 0x02, 0x03]);

        Ok(())
    }

    #[test]
    fn test_add_padding_with_zero_padding() -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        CdsaEncrEncryption::add_padding(2, 8, &mut data)?;

        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x00, 0x00, 0x00]);

        Ok(())
    }

    #[test]
    fn test_add_padding_with_one_padding() -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        CdsaEncrEncryption::add_padding(3, 8, &mut data)?;

        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x01, 0x01, 0x01]);

        Ok(())
    }

    #[test]
    fn test_add_padding_with_pkcs7_padding() -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0x74, 0x65, 0x73, 0x74, 0x69];
        CdsaEncrEncryption::add_padding(7, 8, &mut data)?;

        assert_eq!(data, vec![0x74, 0x65, 0x73, 0x74, 0x69, 0x03, 0x03, 0x03]);

        Ok(())
    }

    #[test]
    fn test_add_padding_with_unsupported_block_size() {
        let mut data: Vec<u8> = vec![0x01, 0x02, 0x03];

        let result = CdsaEncrEncryption::add_padding(2, 0, &mut data);
        assert!(result.is_err());

        let result = CdsaEncrEncryption::add_padding(7, 256, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_padding_with_unsupported_padding_type() {
        let mut data: Vec<u8> = vec![0x01, 0x02, 0x03];

        let result = CdsaEncrEncryption::add_padding(1, 8, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cipher_context_with_aes() -> Result<(), ErrorTrace> {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x80000001,
            mode: 5,
            key_size: 16,
        };
        let key: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let cipher_context: CdsaEncrCipherContext =
            CdsaEncrEncryption::get_cipher_context(&encryption_type, &key)?.unwrap();

        assert!(matches!(cipher_context, CdsaEncrCipherContext::Aes(_)));
        Ok(())
    }

    #[test]
    fn test_get_cipher_context_with_des3() -> Result<(), ErrorTrace> {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x00000011,
            mode: 6,
            key_size: 24,
        };
        let key: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        let cipher_context: CdsaEncrCipherContext =
            CdsaEncrEncryption::get_cipher_context(&encryption_type, &key)?.unwrap();

        assert!(matches!(cipher_context, CdsaEncrCipherContext::Des3(_)));
        Ok(())
    }

    #[test]
    fn test_get_cipher_context_with_unsupported_method() -> Result<(), ErrorTrace> {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x0000002a,
            mode: 5,
            key_size: 16,
        };
        let cipher_context: Option<CdsaEncrCipherContext> =
            CdsaEncrEncryption::get_cipher_context(&encryption_type, &[])?;

        assert!(cipher_context.is_none());

        Ok(())
    }

    #[test]
    fn test_get_cipher_context_with_unsupported_mode() -> Result<(), ErrorTrace> {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x80000001,
            mode: 2,
            key_size: 16,
        };
        let key: Vec<u8> = vec![0; 16];

        let cipher_context: Option<CdsaEncrCipherContext> =
            CdsaEncrEncryption::get_cipher_context(&encryption_type, &key)?;

        assert!(cipher_context.is_none());

        Ok(())
    }

    #[test]
    fn test_get_cipher_context_with_unsupported_key() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x80000001,
            mode: 5,
            key_size: 16,
        };
        let key: Vec<u8> = vec![0; 4];

        let result = CdsaEncrEncryption::get_cipher_context(&encryption_type, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_context_decrypt_with_aes() -> Result<(), ErrorTrace> {
        let initialization_vector: Vec<u8> = vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let key: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let encrypted_data: Vec<u8> = vec![
            0x76, 0xd0, 0x62, 0x7d, 0xa1, 0xd2, 0x90, 0x43, 0x6e, 0x21, 0xa4, 0xaf, 0x7f, 0xca,
            0x94, 0xb7,
        ];
        let mut data: Vec<u8> = vec![0; 16];

        let mut context: CdsaEncrCipherContext = CdsaEncrCipherContext::Aes(AesContext::new());
        context.set_key(&key)?;
        context.decrypt(&initialization_vector, &encrypted_data, &mut data)?;

        let expected_data: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_cipher_context_decrypt_with_des3() -> Result<(), ErrorTrace> {
        let initialization_vector: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let key: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        let encrypted_data: Vec<u8> = vec![
            0x61, 0x09, 0xaa, 0xd6, 0xf5, 0xfa, 0xd5, 0xf5, 0x71, 0xf7, 0x7e, 0x2d, 0xcc, 0x05,
            0x4d, 0x55,
        ];
        let mut data: Vec<u8> = vec![0; 16];

        let mut context: CdsaEncrCipherContext = CdsaEncrCipherContext::Des3(Des3Context::new());
        context.set_key(&key)?;
        context.decrypt(&initialization_vector, &encrypted_data, &mut data)?;

        let expected_data: Vec<u8> = vec![
            0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21,
            0x22, 0x23,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_cipher_context_decrypt_with_no_context() {
        let context: CdsaEncrCipherContext = CdsaEncrCipherContext::None;

        let mut data: Vec<u8> = vec![0; 16];
        let result = context.decrypt(&[], &[0u8; 16], &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_context_set_key_with_no_context() {
        let mut context: CdsaEncrCipherContext = CdsaEncrCipherContext::None;

        let result = context.set_key(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_hmac_with_hmac_sha1() -> Result<(), ErrorTrace> {
        let key: Vec<u8> = vec![0x0b; 20];

        let mut hmac_context: CdsaEncrHmacContext =
            CdsaEncrEncryption::get_hmac_context(91, &key)?.unwrap();

        let hmac: Vec<u8> = hmac_context.calculate_hmac(b"Hi There")?;

        let expected_hmac: Vec<u8> = vec![
            0xb6, 0x17, 0x31, 0x86, 0x55, 0x05, 0x72, 0x64, 0xe2, 0x8b, 0xc0, 0xb6, 0xfb, 0x37,
            0x8c, 0x8e, 0xf1, 0x46, 0xbe, 0x00,
        ];
        assert_eq!(hmac, expected_hmac);

        Ok(())
    }

    #[test]
    fn test_get_hmac_context_with_unsupported_method() -> Result<(), ErrorTrace> {
        let hmac_context: Option<CdsaEncrHmacContext> =
            CdsaEncrEncryption::get_hmac_context(92, &[])?;

        assert!(hmac_context.is_none());

        Ok(())
    }

    #[test]
    fn test_calculate_hmac_with_no_context() {
        let mut hmac_context: CdsaEncrHmacContext = CdsaEncrHmacContext::None;

        let result = hmac_context.calculate_hmac(b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_key_with_pbkdf2_hmac_sha1() -> Result<(), ErrorTrace> {
        let mut key_derivation_context: CdsaEncrKeyDerivationContext =
            CdsaEncrEncryption::get_key_derivation_context(103, b"salt", 1)?.unwrap();

        let mut key: Vec<u8> = vec![0; 20];
        match &mut key_derivation_context {
            CdsaEncrKeyDerivationContext::Pbkdf2HmacSha1(context) => {
                context.derive_key(b"password", &mut key)?
            }
        }
        let expected_key: Vec<u8> = vec![
            0x0c, 0x60, 0xc8, 0x0f, 0x96, 0x1f, 0x0e, 0x71, 0xf3, 0xa9, 0xb5, 0x24, 0xaf, 0x60,
            0x12, 0x06, 0x2f, 0xe0, 0x37, 0xa6,
        ];
        assert_eq!(key, expected_key);

        Ok(())
    }

    #[test]
    fn test_get_key_derivation_context_with_unsupported_method() -> Result<(), ErrorTrace> {
        let key_derivation_context: Option<CdsaEncrKeyDerivationContext> =
            CdsaEncrEncryption::get_key_derivation_context(104, b"salt", 1)?;

        assert!(key_derivation_context.is_none());

        Ok(())
    }

    #[test]
    fn test_remove_padding_with_no_padding() -> Result<(), ErrorTrace> {
        let data: &[u8] = CdsaEncrEncryption::remove_padding(0, 8, &[0x01, 0x02, 0x03])?;

        assert_eq!(data, &[0x01, 0x02, 0x03]);

        Ok(())
    }

    #[test]
    fn test_remove_padding_with_zero_padding() -> Result<(), ErrorTrace> {
        let padded_data: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let data: &[u8] = CdsaEncrEncryption::remove_padding(2, 8, padded_data)?;

        assert_eq!(data, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

        Ok(())
    }

    #[test]
    fn test_remove_padding_with_one_padding() -> Result<(), ErrorTrace> {
        let padded_data: &[u8] = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01,
        ];
        let data: &[u8] = CdsaEncrEncryption::remove_padding(3, 8, padded_data)?;

        assert_eq!(data, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

        Ok(())
    }

    #[test]
    fn test_remove_padding_with_pkcs7_padding() -> Result<(), ErrorTrace> {
        let data: &[u8] = CdsaEncrEncryption::remove_padding(
            7,
            8,
            &[0x74, 0x65, 0x73, 0x74, 0x04, 0x04, 0x04, 0x04],
        )?;
        assert_eq!(data, &[0x74, 0x65, 0x73, 0x74]);

        Ok(())
    }

    #[test]
    fn test_remove_padding_with_unsupported_block_size() {
        let padded_data: &[u8] = &[0x01, 0x02, 0x03, 0x04];

        let result = CdsaEncrEncryption::remove_padding(2, 0, padded_data);
        assert!(result.is_err());

        let result = CdsaEncrEncryption::remove_padding(7, 256, padded_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_padding_with_invalid_padded_data_size() {
        let padded_data: &[u8] = &[0x01, 0x02, 0x03];

        let result = CdsaEncrEncryption::remove_padding(2, 8, padded_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_padding_with_invalid_padding() {
        let padded_data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x09];

        let result = CdsaEncrEncryption::remove_padding(2, 8, padded_data);
        assert!(result.is_err());

        let padded_data: &[u8] = &[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x10];

        let result = CdsaEncrEncryption::remove_padding(7, 8, padded_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_padding_with_unsupported_padding_type() {
        let padded_data: &[u8] = &[0x01, 0x02, 0x03];

        let result = CdsaEncrEncryption::remove_padding(1, 8, padded_data);
        assert!(result.is_err());
    }
}
