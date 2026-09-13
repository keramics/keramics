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

//! Advanced Encryption Standard (AES) key wrap algorithm.
//!
//! Provides AES key wrap and unwrap support (RFC 3394).

use keramics_core::ErrorTrace;

use crate::aes::AesContext;
use crate::traits::{CryptContext, CryptEcb};

/// Context for AES key wrap and unwrap.
pub struct AesKeyWrapContext {
    /// AES context.
    aes_context: AesContext,
}

impl AesKeyWrapContext {
    /// Creates a new AES key wrap context.
    pub fn new() -> Self {
        Self {
            aes_context: AesContext::new(),
        }
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        self.aes_context.set_key(key)
    }

    /// Unwraps a key.
    pub fn unwrap(&mut self, encrypted_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size < 24 || !encrypted_data_size.is_multiple_of(8) {
            return Err(keramics_core::error_trace_new!(
                "Unsupported encrypted data size"
            ));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let number_of_blocks: usize = (encrypted_data_size / 8) - 1;

        data[0..encrypted_data_size].copy_from_slice(encrypted_data);

        let mut integrity_data: [u8; 16] = [0; 16];

        for round in (0..6).rev() {
            for block_number in (1..=number_of_blocks).rev() {
                let mut block_data: [u8; 16] = [0; 16];
                block_data[0..8].copy_from_slice(&data[0..8]);

                let step_counter: usize = (round * number_of_blocks) + block_number;
                let step_counter_data: &[u8] = &(step_counter as u64).to_be_bytes();
                for byte_index in 0..8 {
                    block_data[byte_index] ^= step_counter_data[byte_index];
                }
                let data_offset: usize = block_number * 8;
                let data_end_offset: usize = data_offset + 8;
                block_data[8..16].copy_from_slice(&data[data_offset..data_end_offset]);

                match self
                    .aes_context
                    .decrypt_ecb(&block_data, &mut integrity_data)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to AES-ECB decrypt block: {} in round: {}",
                                block_number, round
                            )
                        );
                        return Err(error);
                    }
                }
                data[0..8].copy_from_slice(&integrity_data[0..8]);
                data[data_offset..data_end_offset].copy_from_slice(&integrity_data[8..16]);
            }
        }
        Ok(())
    }

    /// Wraps a key.
    pub fn wrap(
        &mut self,
        initialization_vector: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if initialization_vector.len() != 8 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported initialization vector size"
            ));
        }
        if data_size < 16 || !data_size.is_multiple_of(8) {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if data_size + 8 > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let number_of_blocks: usize = data_size / 8;

        encrypted_data[0..8].copy_from_slice(initialization_vector);
        encrypted_data[8..8 + data_size].copy_from_slice(data);

        let mut integrity_data: [u8; 16] = [0; 16];

        for round in 0..6 {
            for block_number in 1..=number_of_blocks {
                let mut block_data: [u8; 16] = [0; 16];

                block_data[0..8].copy_from_slice(&encrypted_data[0..8]);

                let data_offset: usize = block_number * 8;
                let data_end_offset: usize = data_offset + 8;
                block_data[8..16].copy_from_slice(&encrypted_data[data_offset..data_end_offset]);

                match self
                    .aes_context
                    .encrypt_ecb(&block_data, &mut integrity_data)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to AES-ECB encrypt block: {} in round: {}",
                                block_number, round
                            )
                        );
                        return Err(error);
                    }
                }
                let step_counter: usize = (round * number_of_blocks) + block_number;
                let step_counter_data: &[u8] = &(step_counter as u64).to_be_bytes();
                for byte_index in 0..8 {
                    integrity_data[byte_index] ^= step_counter_data[byte_index];
                }
                encrypted_data[0..8].copy_from_slice(&integrity_data[0..8]);
                encrypted_data[data_offset..data_end_offset]
                    .copy_from_slice(&integrity_data[8..16]);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap() -> Result<(), ErrorTrace> {
        let mut aes_key_wrap_context: AesKeyWrapContext = AesKeyWrapContext::new();

        let key: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        aes_key_wrap_context.set_key(&key)?;

        let encrypted_data: [u8; 24] = [
            0x1f, 0xa6, 0x8b, 0x0a, 0x81, 0x12, 0xb4, 0x47, 0xae, 0xf3, 0x4b, 0xd8, 0xfb, 0x5a,
            0x7b, 0x82, 0x9d, 0x3e, 0x86, 0x23, 0x71, 0xd2, 0xcf, 0xe5,
        ];
        let mut data: Vec<u8> = vec![0; 24];
        aes_key_wrap_context.unwrap(&encrypted_data, &mut data)?;

        let expected_data: [u8; 24] = [
            0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_wrap() -> Result<(), ErrorTrace> {
        let mut aes_key_wrap_context: AesKeyWrapContext = AesKeyWrapContext::new();

        let key: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        aes_key_wrap_context.set_key(&key)?;

        let initialization_vector: [u8; 8] = [0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6, 0xa6];
        let data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 24];
        aes_key_wrap_context.wrap(&initialization_vector, &data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 24] = [
            0x1f, 0xa6, 0x8b, 0x0a, 0x81, 0x12, 0xb4, 0x47, 0xae, 0xf3, 0x4b, 0xd8, 0xfb, 0x5a,
            0x7b, 0x82, 0x9d, 0x3e, 0x86, 0x23, 0x71, 0xd2, 0xcf, 0xe5,
        ];
        assert_eq!(encrypted_data, expected_encrypted_data);

        Ok(())
    }
}
