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
use keramics_encryption::{AesContext, AesXtsContext, CryptContext};

use super::encryption_context::BdeEncryptionContext;
use super::encryption_type::BdeEncryptionType;

/// BitLocker Drive Encryption (BDE) cipher context.
#[derive(Clone)]
pub enum BdeCipherContext {
    AesCbc(AesContext),
    AesXts(AesXtsContext),
}

impl BdeCipherContext {
    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        match self {
            BdeCipherContext::AesCbc(context) => context.set_key(key),
            BdeCipherContext::AesXts(context) => {
                let key_size: usize = key.len() / 2;

                context.set_keys(&key[0..key_size], &key[key_size..])
            }
        }
    }
}

/// BitLocker Drive Encryption (BDE) encryption.
pub struct BdeEncryption {}

impl BdeEncryption {
    /// Retrieves a encryption context.
    pub fn get_encryption_context(
        bytes_per_sector: u16,
        encryption_type: &BdeEncryptionType,
        key_data: &[u8],
    ) -> Result<Option<BdeEncryptionContext>, ErrorTrace> {
        let mut cipher_context: BdeCipherContext = match encryption_type.method {
            0x8000..=0x8003 => BdeCipherContext::AesCbc(AesContext::new()),
            0x8004..=0x8005 => BdeCipherContext::AesXts(AesXtsContext::new()),
            _ => return Ok(None),
        };
        let key_size: usize = encryption_type.get_key_size();
        let key_data_size: usize = key_data.len();

        if key_data_size < key_size {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        match cipher_context.set_key(&key_data[0..key_size]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        let diffuser_context: Option<AesContext> = match encryption_type.method {
            0x8000 | 0x8001 => {
                if key_data_size < 32 + key_size {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported diffuser key size"
                    ));
                }
                let mut aes_context: AesContext = AesContext::new();

                match aes_context.set_key(&key_data[32..32 + key_size]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to set key in diffuser context"
                        );
                        return Err(error);
                    }
                }
                Some(aes_context)
            }
            _ => None,
        };
        Ok(Some(BdeEncryptionContext::new(
            bytes_per_sector,
            cipher_context,
            diffuser_context,
        )))
    }
}
