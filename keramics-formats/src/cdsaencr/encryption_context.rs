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

use super::encryption::{CdsaEncrCipherContext, CdsaEncrHmacContext};

/// Mac OS Encrypted Encoding (cdsaencr) encryption context.
#[derive(Clone)]
pub struct CdsaEncrEncryptionContext {
    /// Cipher context.
    pub cipher_context: CdsaEncrCipherContext,

    /// HMAC context.
    pub hmac_context: CdsaEncrHmacContext,
}

impl CdsaEncrEncryptionContext {
    /// Decrypts a block.
    pub(crate) fn decrypt_block(
        &mut self,
        block_number: u32,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let block_number_data: [u8; 4] = block_number.to_be_bytes();

        let initialization_vector: Vec<u8> =
            match self.hmac_context.calculate_hmac(&block_number_data) {
                Ok(data) => data,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to HMAC initialization vector for decrypting block: {}",
                            block_number
                        )
                    );
                    return Err(error);
                }
            };
        match self
            .cipher_context
            .decrypt(&initialization_vector, encrypted_data, data)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt block: {}", block_number)
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_encryption::{AesContext, CryptContext, HmacSha1Context};

    #[test]
    fn test_decrypt_block() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();
        aes_context.set_key(&[0x11; 16])?;
        let mut hmac_context: HmacSha1Context = HmacSha1Context::new();
        hmac_context.set_key(&[0x22; 20]);

        let mut encryption_context: CdsaEncrEncryptionContext = CdsaEncrEncryptionContext {
            cipher_context: CdsaEncrCipherContext::Aes(aes_context.clone()),
            hmac_context: CdsaEncrHmacContext::HmacSha1 {
                context: hmac_context,
            },
        };
        let encrypted_data: [u8; 32] = [0xaa; 32];
        let mut decrypted_data: [u8; 32] = [0; 32];
        encryption_context.decrypt_block(0, &encrypted_data, &mut decrypted_data)?;
        assert_ne!(decrypted_data, [0; 32]);

        // Test invalid non-block-aligned buffer size error.
        let invalid_encrypted_data: [u8; 7] = [0xaa; 7];
        let mut invalid_decrypted_data: [u8; 7] = [0; 7];
        let result: Result<(), ErrorTrace> = encryption_context.decrypt_block(
            1,
            &invalid_encrypted_data,
            &mut invalid_decrypted_data,
        );
        assert!(result.is_err());

        // Test unimplemented HMAC context error.
        let mut invalid_hmac_context: CdsaEncrEncryptionContext = CdsaEncrEncryptionContext {
            cipher_context: CdsaEncrCipherContext::Aes(aes_context),
            hmac_context: CdsaEncrHmacContext::None,
        };
        let result: Result<(), ErrorTrace> =
            invalid_hmac_context.decrypt_block(0, &encrypted_data, &mut decrypted_data);
        assert!(result.is_err());

        Ok(())
    }
}
