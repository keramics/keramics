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
use keramics_encryption::{AesContext, AesXtsContext, CryptCbc, CryptContext, CryptEcb};

/// Linux Unified Key Setup (LUKS) Disk Encryption cipher context.
#[derive(Clone)]
pub enum LuksCipherContext {
    AesCbc(AesContext),
    AesEcb(AesContext),
    AesXts(AesXtsContext),
}

impl LuksCipherContext {
    /// Decrypts data.
    pub fn decrypt(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        match self {
            LuksCipherContext::AesCbc(context) => {
                context.decrypt_cbc(initialization_vector, encrypted_data, data)
            }
            LuksCipherContext::AesEcb(context) => context.decrypt_ecb(encrypted_data, data),
            LuksCipherContext::AesXts(context) => {
                context.decrypt_xts(initialization_vector, encrypted_data, data)
            }
        }
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        match self {
            LuksCipherContext::AesCbc(context) => context.set_key(key),
            LuksCipherContext::AesEcb(context) => context.set_key(key),
            LuksCipherContext::AesXts(context) => {
                let key_size: usize = key.len() / 2;

                context.set_keys(&key[0..key_size], &key[key_size..])
            }
        }
    }
}
