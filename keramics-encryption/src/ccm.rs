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

use super::traits::CryptCcm;

/// CCM (Counter with Cipher Block Chaining message authentication code (CBC-MAC)) encryption and
/// decryption (RFC 3610).
pub struct CcmContext<T: CryptCcm> {
    /// CBC encryption and decryption context.
    context: T,

    /// Nonce.
    nonce: Vec<u8>,

    /// Associated data (AAD).
    associated_data: Vec<u8>,
}

impl<T: CryptCcm> CcmContext<T> {
    /// Creates a new context.
    pub fn new(nonce: &[u8], associated_data: &[u8]) -> Self {
        Self {
            context: T::new(),
            nonce: nonce.to_vec(),
            associated_data: associated_data.to_vec(),
        }
    }

    /// Decrypts data using CCM (Counter with CBC-MAC) mode.
    pub fn decrypt(
        &self,
        encrypted_data: &[u8],
        data: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        self.context.decrypt_ccm(
            &self.nonce,
            &self.associated_data,
            encrypted_data,
            data,
            tag,
        )
    }

    /// Encrypts data using CCM (Counter with CBC-MAC) mode.
    pub fn encrypt(
        &self,
        data: &[u8],
        encrypted_data: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        self.context.encrypt_ccm(
            &self.nonce,
            &self.associated_data,
            data,
            encrypted_data,
            tag,
        )
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        self.context.set_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::aes::AesCcmContext;

    #[test]
    fn test_decrypt() -> Result<(), ErrorTrace> {
        let nonce: [u8; 13] = [
            0x00, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5,
        ];
        let associated_data: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let mut ccm_context: AesCcmContext = AesCcmContext::new(&nonce, &associated_data);

        let key: [u8; 16] = [
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd,
            0xce, 0xcf,
        ];
        ccm_context.set_key(&key)?;

        let encrypted_data: [u8; 23] = [
            0x58, 0x8c, 0x97, 0x9a, 0x61, 0xc6, 0x63, 0xd2, 0xf0, 0x66, 0xd0, 0xc2, 0xc0, 0xf9,
            0x89, 0x80, 0x6d, 0x5f, 0x6b, 0x61, 0xda, 0xc3, 0x84,
        ];
        let mut data: Vec<u8> = vec![0; 23];
        let mut tag: Vec<u8> = vec![0; 8];
        ccm_context.decrypt(&encrypted_data, &mut data, &mut tag)?;

        let expected_data: [u8; 23] = [
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        ];
        assert_eq!(&data, &expected_data);

        let expected_tag: [u8; 8] = [0x17, 0xe8, 0xd1, 0x2c, 0xfd, 0xf9, 0x26, 0xe0];
        assert_eq!(&tag, &expected_tag);

        Ok(())
    }

    #[test]
    fn test_encrypt() -> Result<(), ErrorTrace> {
        let nonce: [u8; 13] = [
            0x00, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5,
        ];
        let associated_data: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let mut ccm_context: AesCcmContext = AesCcmContext::new(&nonce, &associated_data);

        let key: [u8; 16] = [
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd,
            0xce, 0xcf,
        ];
        ccm_context.set_key(&key)?;

        let data: [u8; 23] = [
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 23];
        let mut tag: Vec<u8> = vec![0; 8];
        ccm_context.encrypt(&data, &mut encrypted_data, &mut tag)?;

        let expected_encrypted_data: [u8; 23] = [
            0x58, 0x8c, 0x97, 0x9a, 0x61, 0xc6, 0x63, 0xd2, 0xf0, 0x66, 0xd0, 0xc2, 0xc0, 0xf9,
            0x89, 0x80, 0x6d, 0x5f, 0x6b, 0x61, 0xda, 0xc3, 0x84,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        let expected_tag: [u8; 8] = [0x17, 0xe8, 0xd1, 0x2c, 0xfd, 0xf9, 0x26, 0xe0];
        assert_eq!(&tag, &expected_tag);

        Ok(())
    }
}
