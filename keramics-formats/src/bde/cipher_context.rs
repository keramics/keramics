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
