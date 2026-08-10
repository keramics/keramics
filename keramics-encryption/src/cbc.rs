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

use super::traits::CryptCbc;

/// Context for CBC (Cipher Block Chaining) encryption and decryption.
pub struct CbcContext<T: CryptCbc, const BLOCK_SIZE: usize> {
    /// CBC encryption and decryption context.
    context: T,

    /// Initialization vector.
    initialization_vector: Vec<u8>,
}

impl<T: CryptCbc, const BLOCK_SIZE: usize> CbcContext<T, BLOCK_SIZE> {
    /// Creates a new context.
    pub fn new(initialization_vector: &[u8]) -> Self {
        Self {
            context: T::new(),
            initialization_vector: initialization_vector.to_vec(),
        }
    }

    /// Decrypts data using CBC (Cipher Block Chaining) mode.
    pub fn decrypt(&self, encrypted_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        self.context
            .decrypt_cbc(&self.initialization_vector, encrypted_data, data)
    }

    /// Encrypts data using CBC (Cipher Block Chaining) mode.
    pub fn encrypt(&self, data: &[u8], encrypted_data: &mut [u8]) -> Result<(), ErrorTrace> {
        self.context
            .encrypt_cbc(&self.initialization_vector, data, encrypted_data)
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        self.context.set_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::aes::AesCbcContext;

    #[test]
    fn test_decrypt() -> Result<(), ErrorTrace> {
        let initialization_vector: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let mut cbc_context: AesCbcContext = AesCbcContext::new(&initialization_vector);

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        cbc_context.set_key(&key)?;

        let encrypted_data: [u8; 16] = [
            0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9,
            0x19, 0x7d,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        cbc_context.decrypt(&encrypted_data, &mut data)?;

        let expected_data: [u8; 16] = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
            0x17, 0x2a,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_encrypt() -> Result<(), ErrorTrace> {
        let initialization_vector: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let mut cbc_context: AesCbcContext = AesCbcContext::new(&initialization_vector);

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        cbc_context.set_key(&key)?;

        let data: [u8; 16] = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
            0x17, 0x2a,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 16];
        cbc_context.encrypt(&data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 16] = [
            0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9,
            0x19, 0x7d,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }
}
