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
use keramics_encryption::{CryptCbc, CryptEcb};

use super::encryption::BdeCipherContext;

/// BitLocker disk encryption (BDE) block stream.
#[derive(Clone)]
pub struct BdeEncryptionContext {
    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Cipher context.
    pub cipher_context: BdeCipherContext,

    /// Diffuser context.
    pub diffuser_context: Option<u32>,
}

impl BdeEncryptionContext {
    /// Creates a new encryption context.
    pub fn new(bytes_per_sector: u16, cipher_context: BdeCipherContext) -> Self {
        Self {
            bytes_per_sector,
            cipher_context,
            diffuser_context: None,
        }
    }

    /// Decrypts a sector.
    pub fn decrypt_sector(
        &self,
        sector_data_offset: u64,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let sector_number: u64 = sector_data_offset / (self.bytes_per_sector as u64);
        let mut initialization_vector: [u8; 16] = [0; 16];

        match &self.cipher_context {
            BdeCipherContext::AesCbc(aes_context) => {
                let mut block_key_data: [u8; 16] = [0; 16];
                block_key_data[0..8].copy_from_slice(&sector_data_offset.to_le_bytes());

                match aes_context.encrypt_ecb(&block_key_data, &mut initialization_vector) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to encrypt initialization vector for sector: {}",
                                sector_number
                            )
                        );
                        return Err(error);
                    }
                }
                if let Some(diffuser_context) = self.diffuser_context {
                    todo!();
                }
                match aes_context.decrypt_cbc(&initialization_vector, encrypted_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to decrypt sector: {}", sector_number)
                        );
                        return Err(error);
                    }
                }
                if let Some(diffuser_context) = self.diffuser_context {
                    todo!();
                }
            }
            BdeCipherContext::AesXts(xts_context) => {
                initialization_vector[0..8].copy_from_slice(&sector_number.to_le_bytes());

                match xts_context.decrypt_xts(&initialization_vector, encrypted_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to decrypt sector: {}", sector_number)
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}
