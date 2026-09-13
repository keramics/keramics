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

use keramics_encryption::AesXtsContext;

/// Apple File System (APFS) encryption context.
pub struct ApfsEncryptionContext {
    /// Cipher context.
    cipher_context: AesXtsContext,

    /// Bytes per sector.
    bytes_per_sector: u16,
}

impl ApfsEncryptionContext {
    /// Creates a new encryption context.
    pub fn new(bytes_per_sector: u16) -> Self {
        Self {
            cipher_context: AesXtsContext::new(),
            bytes_per_sector,
        }
    }

    /// Decrypts a block.
    pub fn decrypt_block(
        &self,
        block_offset: u64,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();
        let mut tweak_value: [u8; 16] = [0; 16];
        let mut sector_number: u64 = block_offset / (self.bytes_per_sector as u64);

        while data_offset < data_size {
            tweak_value[0..8].copy_from_slice(&sector_number.to_le_bytes());

            let data_end_offset: usize = data_offset + (self.bytes_per_sector as usize);

            match self.cipher_context.decrypt_xts(
                &tweak_value,
                &encrypted_data[data_offset..data_end_offset],
                &mut data[data_offset..data_end_offset],
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to decrypt sector: {}", sector_number)
                    );
                    return Err(error);
                }
            }
            data_offset = data_end_offset;
            sector_number += 1;
        }
        Ok(())
    }

    /// Sets the keys.
    pub fn set_keys(&mut self, key: &[u8], tweak_key: &[u8]) -> Result<(), ErrorTrace> {
        self.cipher_context.set_keys(key, tweak_key)
    }
}
