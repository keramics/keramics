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

        let mut initialization_vector: Vec<u8> =
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
            .decrypt(&mut initialization_vector, encrypted_data, data)
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
