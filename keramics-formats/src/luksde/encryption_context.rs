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

use super::cipher_context::LuksCipherContext;
use super::encryption::LuksInitializationVectorContext;

/// Linux Unified Key Setup (LUKS) Disk Encryption encryption context.
pub struct LuksEncryptionContext {
    /// Cipher context.
    pub cipher_context: LuksCipherContext,

    /// Initialization vector context.
    pub initialization_vector_context: LuksInitializationVectorContext,
}

impl LuksEncryptionContext {
    /// Decrypts a sector.
    pub fn decrypt_sector(
        &self,
        sector_number: u64,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut initialization_vector: [u8; 16] = [0; 16];

        match self
            .initialization_vector_context
            .derive_initialization_vector(sector_number, &mut initialization_vector)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to derive initialization vector for sector: {}",
                        sector_number
                    )
                );
                return Err(error);
            }
        }
        match self
            .cipher_context
            .decrypt(&initialization_vector, encrypted_data, data)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt sector: {}", sector_number)
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

    use keramics_encryption::{AesContext, CryptContext};

    #[test]
    fn test_decrypt_sector() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();
        aes_context.set_key(&[0x11; 16])?;
        let encryption_context: LuksEncryptionContext = LuksEncryptionContext {
            cipher_context: LuksCipherContext::AesCbc(aes_context),
            initialization_vector_context: LuksInitializationVectorContext::Plain64,
        };
        let encrypted_data: [u8; 32] = [0xaa; 32];
        let mut decrypted_data: [u8; 32] = [0; 32];
        encryption_context.decrypt_sector(0, &encrypted_data, &mut decrypted_data)?;
        assert_ne!(decrypted_data, [0; 32]);

        // Test cipher decryption error with invalid non-block-aligned buffer.
        let invalid_encrypted_data: [u8; 7] = [0xaa; 7];
        let mut invalid_decrypted_data: [u8; 7] = [0; 7];
        let result: Result<(), ErrorTrace> = encryption_context.decrypt_sector(
            0,
            &invalid_encrypted_data,
            &mut invalid_decrypted_data,
        );
        assert!(result.is_err());

        Ok(())
    }
}
