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
use keramics_encryption::{AesContext, CryptCbc, CryptContext};

use super::encryption_type::QcowEncryptionType;

/// QEMU Copy-On-Write (QCOW) encryption context.
#[derive(Clone)]
pub struct QcowEncryptionContext {
    /// Cipher context.
    cipher_context: AesContext,
}

impl QcowEncryptionContext {
    /// Decrypts a sector.
    pub fn decrypt_sector(
        &self,
        sector_number: u64,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut initialization_vector: [u8; 16] = [0; 16];

        initialization_vector[0..8].copy_from_slice(&sector_number.to_le_bytes());

        match self
            .cipher_context
            .decrypt_cbc(&initialization_vector, encrypted_data, data)
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

/// QEMU Copy-On-Write (QCOW) encryption context.
pub struct QcowEncryption {}

impl QcowEncryption {
    /// Retrieves an encryption context.
    pub fn get_encryption_context(
        encryption_type: &QcowEncryptionType,
        key: &[u8],
    ) -> Result<Option<QcowEncryptionContext>, ErrorTrace> {
        let mut cipher_context: AesContext = match encryption_type.method {
            1 => AesContext::new(),
            _ => return Ok(None),
        };
        match cipher_context.set_key(key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        Ok(Some(QcowEncryptionContext { cipher_context }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_encryption_context_and_decrypt_sector() -> Result<(), ErrorTrace> {
        // Test unsupported encryption method returns None.
        let unsupported_type: QcowEncryptionType = QcowEncryptionType::new(0);
        let context: Option<QcowEncryptionContext> =
            QcowEncryption::get_encryption_context(&unsupported_type, &[0; 16])?;
        assert!(context.is_none());

        // Test invalid key length returns error.
        let aes_type: QcowEncryptionType = QcowEncryptionType::new(1);
        let result: Result<Option<QcowEncryptionContext>, ErrorTrace> =
            QcowEncryption::get_encryption_context(&aes_type, &[0; 5]);
        assert!(result.is_err());

        // Test valid AES-128-CBC sector decryption.
        let context: QcowEncryptionContext =
            QcowEncryption::get_encryption_context(&aes_type, &[0x11; 16])?.unwrap();
        let encrypted_data: [u8; 32] = [0x22; 32];
        let mut decrypted_data: [u8; 32] = [0; 32];
        context.decrypt_sector(1, &encrypted_data, &mut decrypted_data)?;
        assert_ne!(decrypted_data, encrypted_data);

        // Test decrypt_sector with invalid (non-block-aligned) buffer size.
        let mut short_output: [u8; 10] = [0; 10];
        let result: Result<(), ErrorTrace> =
            context.decrypt_sector(1, &encrypted_data[0..10], &mut short_output);
        assert!(result.is_err());

        Ok(())
    }
}
