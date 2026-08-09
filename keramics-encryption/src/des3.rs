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

//! Tripple Data Encryption Standard (DES3) encryption.
//!
//! Provides DES3 encryption and decryption support.

use keramics_core::ErrorTrace;
use keramics_types::bytes_to_u64_be;

use super::cbc::CbcContext;
use super::traits::{CryptCbc, CryptContext};

/// DES3 block size.
const DES3_BLOCK_SIZE: usize = 8;

/// DES3 supported key sizes.
const DES3_SUPPORTED_KEY_SIZES: [usize; 6] = [7, 8, 14, 16, 21, 24];

/// DES3 permutation values.
const DES3_PERMUTATION_VALUES: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2, 60, 52, 44, 36, 28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6,
    64, 56, 48, 40, 32, 24, 16, 8, 57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3, 61,
    53, 45, 37, 29, 21, 13, 5, 63, 55, 47, 39, 31, 23, 15, 7,
];

/// DES3 inverse permutation values.
const DES3_INVERSE_PERMUTATION_VALUES: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32, 39, 7, 47, 15, 55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29, 36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26, 33, 1, 41, 9, 49, 17, 57, 25,
];

/// DES3 expansion values.
const DES3_EXPANSION_VALUES: [u8; 48] = [
    32, 1, 2, 3, 4, 5, 4, 5, 6, 7, 8, 9, 8, 9, 10, 11, 12, 13, 12, 13, 14, 15, 16, 17, 16, 17, 18,
    19, 20, 21, 20, 21, 22, 23, 24, 25, 24, 25, 26, 27, 28, 29, 28, 29, 30, 31, 32, 1,
];

/// DES3 post substitution permutation values.
const DES3_POST_SUBSTITUTION_PERMUTATION_VALUES: [u8; 32] = [
    16, 7, 20, 21, 29, 12, 28, 17, 1, 15, 23, 26, 5, 18, 31, 10, 2, 8, 24, 14, 32, 27, 3, 9, 19,
    13, 30, 6, 22, 11, 4, 25,
];

/// DES3 substitution-boxes (S-boxes).
const DES3_SBOXES: [[u8; 64]; 8] = [
    [
        14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7, 0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12,
        11, 9, 5, 3, 8, 4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0, 15, 12, 8, 2, 4, 9,
        1, 7, 5, 11, 3, 14, 10, 0, 6, 13,
    ],
    [
        15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10, 3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1,
        10, 6, 9, 11, 5, 0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15, 13, 8, 10, 1, 3, 15,
        4, 2, 11, 6, 7, 12, 0, 5, 14, 9,
    ],
    [
        10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8, 13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5,
        14, 12, 11, 15, 1, 13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7, 1, 10, 13, 0, 6,
        9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12,
    ],
    [
        7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15, 13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2,
        12, 1, 10, 14, 9, 10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4, 3, 15, 0, 6, 10, 1,
        13, 8, 9, 4, 5, 11, 12, 7, 2, 14,
    ],
    [
        2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9, 14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15,
        10, 3, 9, 8, 6, 4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14, 11, 8, 12, 7, 1, 14,
        2, 13, 6, 15, 0, 9, 10, 4, 5, 3,
    ],
    [
        12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11, 10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13,
        14, 0, 11, 3, 8, 9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6, 4, 3, 2, 12, 9, 5,
        15, 10, 11, 14, 1, 7, 6, 0, 8, 13,
    ],
    [
        4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1, 13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5,
        12, 2, 15, 8, 6, 1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2, 6, 11, 13, 8, 1, 4,
        10, 7, 9, 5, 0, 15, 14, 2, 3, 12,
    ],
    [
        13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7, 1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6,
        11, 0, 14, 9, 2, 7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8, 2, 1, 14, 7, 4, 10,
        8, 13, 15, 12, 9, 0, 3, 5, 6, 11,
    ],
];

/// DES3 permuted choice values.
const DES3_PERMUTED_CHOICE_VALUES1: [u8; 56] = [
    57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18, 10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60,
    52, 44, 36, 63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22, 14, 6, 61, 53, 45, 37, 29,
    21, 13, 5, 28, 20, 12, 4,
];

const DES3_PERMUTED_CHOICE_VALUES2: [u8; 48] = [
    14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10, 23, 19, 12, 4, 26, 8, 16, 7, 27, 20, 13, 2, 41, 52,
    31, 37, 47, 55, 30, 40, 51, 45, 33, 48, 44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
];

/// DES3 shift iterations.
const DES3_SHIFT_ITERATIONS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

/// Context for DES3 encryption.
pub struct Des3Context {
    /// Key values.
    key_values: Vec<u64>,
}

impl Des3Context {
    /// Calculates the initial permutation value.
    #[inline(always)]
    fn calculate_initial_permutation_value(&self, input_value: u64) -> (u64, u64) {
        let mut value_64bit: u64 = 0;

        for number_of_bits in DES3_PERMUTATION_VALUES.iter() {
            value_64bit = (value_64bit << 1) | ((input_value >> (64 - *number_of_bits)) & 0x01);
        }
        (value_64bit & 0xffffffff, value_64bit >> 32)
    }

    /// Calculates the inverse permutation.
    #[inline(always)]
    fn calculate_inverse_permutation(
        &self,
        permutation_lower_32bit: u64,
        permutation_upper_32bit: u64,
    ) -> u64 {
        let mut value_64bit: u64 = 0;

        let permulation_value: u64 = (permutation_upper_32bit << 32) | permutation_lower_32bit;

        // Calculate the inverse permutation.
        for number_of_bits in DES3_INVERSE_PERMUTATION_VALUES.iter() {
            value_64bit =
                (value_64bit << 1) | ((permulation_value >> (64 - *number_of_bits)) & 0x01);
        }
        value_64bit
    }

    /// Calculates a permutation with a specific sub key.
    #[inline(always)]
    fn calculate_permutation_with_sub_key(
        &self,
        sub_key: u64,
        permutation_lower_32bit: u64,
        permutation_upper_32bit: u64,
    ) -> (u64, u64) {
        let mut function_result: u64 = 0;
        let mut sbox_value: u64 = 0;
        let mut value_64bit: u64 = 0;

        for number_of_bits in DES3_EXPANSION_VALUES.iter() {
            value_64bit =
                (value_64bit << 1) | ((permutation_lower_32bit >> (32 - *number_of_bits)) & 0x01);
        }
        value_64bit ^= sub_key;

        for index in 0..8 {
            let mut sbox_index: u64 = index * 6;

            let mut row_bit_mask: u64 =
                ((value_64bit & (0x0000840000000000 >> sbox_index)) >> (42 - sbox_index)) & 0xff;
            row_bit_mask = (row_bit_mask >> 4) | (row_bit_mask & 0x01);

            let column_bit_mask: u64 =
                ((value_64bit & (0x0000780000000000 >> sbox_index)) >> (43 - sbox_index)) & 0xff;

            sbox_index = (row_bit_mask << 4) | column_bit_mask;

            sbox_value =
                (sbox_value << 4) | (DES3_SBOXES[index as usize][sbox_index as usize] as u64);
        }
        for number_of_bits in DES3_POST_SUBSTITUTION_PERMUTATION_VALUES.iter() {
            function_result =
                (function_result << 1) | ((sbox_value >> (32 - *number_of_bits)) & 0x01);
        }
        // Note that the lower and upper 32-bit values are deliberately swapped.
        (
            permutation_upper_32bit ^ function_result,
            permutation_lower_32bit,
        )
    }

    /// Calculate the sub keys.
    #[inline(always)]
    fn calculate_sub_keys(&self, key_value: u64) -> [u64; 16] {
        // Calculate the key schedule.
        let mut value_64bit: u64 = 0;

        for number_of_bits in DES3_PERMUTED_CHOICE_VALUES1.iter() {
            value_64bit = (value_64bit << 1) | ((key_value >> (64 - *number_of_bits)) & 0x01);
        }
        let mut choice_lower_28bit: u64 = value_64bit & 0x0fffffff;
        let mut choice_upper_28bit: u64 = value_64bit >> 28;

        // Calculate the sub keys.
        let mut sub_keys: [u64; 16] = [0; 16];

        for sub_key_index in 0..16 {
            let iterations: u8 = DES3_SHIFT_ITERATIONS[sub_key_index];

            for _ in 0..iterations {
                choice_lower_28bit = ((choice_lower_28bit << 1) & 0x0fffffff)
                    | ((choice_lower_28bit >> 27) & 0x00000001);
                choice_upper_28bit = ((choice_upper_28bit << 1) & 0x0fffffff)
                    | ((choice_upper_28bit >> 27) & 0x00000001);
            }
            value_64bit = (choice_upper_28bit << 28) | choice_lower_28bit;

            let mut sub_key_value: u64 = 0;

            for number_of_bits in DES3_PERMUTED_CHOICE_VALUES2.iter() {
                sub_key_value =
                    (sub_key_value << 1) | ((value_64bit >> (56 - *number_of_bits)) & 0x01);
            }
            sub_keys[sub_key_index] = sub_key_value;
        }
        sub_keys
    }

    /// Decrypts a 64-bit value (or 8 byte block).
    #[inline(always)]
    fn decrypt_block(&self, key_value: u64, input_value: u64) -> u64 {
        let sub_keys: [u64; 16] = self.calculate_sub_keys(key_value);

        let (mut permutation_lower_32bit, mut permutation_upper_32bit): (u64, u64) =
            self.calculate_initial_permutation_value(input_value);

        for sub_key in sub_keys.iter().rev() {
            (permutation_lower_32bit, permutation_upper_32bit) = self
                .calculate_permutation_with_sub_key(
                    *sub_key,
                    permutation_lower_32bit,
                    permutation_upper_32bit,
                );
        }
        // Note that calculate_permutation_with_sub_key swapped the lower and upper 32-bit parts.
        self.calculate_inverse_permutation(permutation_upper_32bit, permutation_lower_32bit)
    }

    /// Decrypts data using ECB (Electronic CodeBook) mode.
    pub fn decrypt_ecb(&self, encrypted_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.key_values.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size % DES3_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid encrypted data size value not a multitude of block size: {}",
                DES3_BLOCK_SIZE
            ),));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let mut data_offset: usize = 0;

        for block_data in encrypted_data.chunks_exact(DES3_BLOCK_SIZE) {
            let mut block_value: u64 = bytes_to_u64_be!(block_data, 0);

            block_value = self.decrypt_block(self.key_values[2], block_value);
            block_value = self.encrypt_block(self.key_values[1], block_value);
            block_value = self.decrypt_block(self.key_values[0], block_value);

            let data_end_offset: usize = data_offset + DES3_BLOCK_SIZE;
            data[data_offset..data_end_offset].copy_from_slice(&block_value.to_be_bytes());

            data_offset = data_end_offset;
        }
        Ok(())
    }

    /// Encrypts a 64-bit value (or 8 byte block).
    #[inline(always)]
    fn encrypt_block(&self, key_value: u64, input_value: u64) -> u64 {
        let sub_keys: [u64; 16] = self.calculate_sub_keys(key_value);

        let (mut permutation_lower_32bit, mut permutation_upper_32bit): (u64, u64) =
            self.calculate_initial_permutation_value(input_value);

        for sub_key in sub_keys.iter() {
            (permutation_lower_32bit, permutation_upper_32bit) = self
                .calculate_permutation_with_sub_key(
                    *sub_key,
                    permutation_lower_32bit,
                    permutation_upper_32bit,
                );
        }
        // Note that calculate_permutation_with_sub_key swapped the lower and upper 32-bit parts.
        self.calculate_inverse_permutation(permutation_upper_32bit, permutation_lower_32bit)
    }

    /// Encrypts data using ECB (Electronic CodeBook) mode.
    pub fn encrypt_ecb(&self, data: &[u8], encrypted_data: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.key_values.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let data_size: usize = data.len();

        if data_size % DES3_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid data size value not a multitude of block size: {}",
                DES3_BLOCK_SIZE
            )));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let mut data_offset: usize = 0;

        for block_data in data.chunks_exact(DES3_BLOCK_SIZE) {
            let mut block_value: u64 = bytes_to_u64_be!(block_data, 0);

            block_value = self.encrypt_block(self.key_values[0], block_value);
            block_value = self.decrypt_block(self.key_values[1], block_value);
            block_value = self.encrypt_block(self.key_values[2], block_value);

            let data_end_offset: usize = data_offset + DES3_BLOCK_SIZE;
            encrypted_data[data_offset..data_end_offset]
                .copy_from_slice(&block_value.to_be_bytes());

            data_offset = data_end_offset;
        }
        Ok(())
    }
}

impl CryptContext for Des3Context {
    /// Creates a new context.
    fn new() -> Self {
        Self {
            key_values: Vec::new(),
        }
    }

    /// Sets the key.
    fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        let key_size: usize = key.len();

        if !DES3_SUPPORTED_KEY_SIZES.contains(&key_size) {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        self.key_values = vec![0; 3];

        let value_64bit: u64 = match key_size {
            7 | 14 | 21 => bytes_to_u64_be!(key, 0) >> 8,
            _ => bytes_to_u64_be!(key, 0),
        };
        self.key_values[0] = value_64bit;

        self.key_values[1] = match key_size {
            14 | 21 => bytes_to_u64_be!(key, 7) >> 8,
            16 | 24 => {
                bytes_to_u64_be!(key, 8)
            }
            _ => value_64bit,
        };
        self.key_values[2] = match key_size {
            21 => {
                let value_64bit: u64 = bytes_to_u64_be!(key, 14);
                (value_64bit << 8) | (key[20] as u64)
            }
            24 => {
                bytes_to_u64_be!(key, 16)
            }
            _ => value_64bit,
        };
        Ok(())
    }
}

impl CryptCbc for Des3Context {
    /// Decrypts data using CBC (Cipher Block Chaining) mode.
    fn decrypt_cbc(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.key_values.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        if initialization_vector.len() < 8 {
            return Err(keramics_core::error_trace_new!(
                "Invalid initialization vector value too small"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size % 8 != 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data size value not a multitude of block size: 8"
            ));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let mut initialization_vector_64bit: u64 = bytes_to_u64_be!(initialization_vector, 0);
        let mut data_offset: usize = 0;

        for block_data in encrypted_data.chunks_exact(DES3_BLOCK_SIZE) {
            let input_value: u64 = bytes_to_u64_be!(block_data, 0);

            let mut block_value: u64 = self.decrypt_block(self.key_values[2], input_value);
            block_value = self.encrypt_block(self.key_values[1], block_value);
            block_value = self.decrypt_block(self.key_values[0], block_value);

            block_value ^= initialization_vector_64bit;

            let data_end_offset: usize = data_offset + 8;
            data[data_offset..data_end_offset].copy_from_slice(&block_value.to_be_bytes());

            initialization_vector_64bit = input_value;
            data_offset = data_end_offset;
        }
        Ok(())
    }

    /// Encrypts data using CBC (Cipher Block Chaining) mode.
    fn encrypt_cbc(
        &self,
        initialization_vector: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.key_values.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        if initialization_vector.len() < 8 {
            return Err(keramics_core::error_trace_new!(
                "Invalid initialization vector value too small"
            ));
        }
        let data_size: usize = data.len();

        if data_size % 8 != 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid data size value not a multitude of block size: 8"
            ));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let mut initialization_vector_64bit: u64 = bytes_to_u64_be!(initialization_vector, 0);
        let mut data_offset: usize = 0;

        for block_data in data.chunks_exact(DES3_BLOCK_SIZE) {
            let mut block_value: u64 = bytes_to_u64_be!(block_data, 0);
            block_value ^= initialization_vector_64bit;

            block_value = self.encrypt_block(self.key_values[0], block_value);
            block_value = self.decrypt_block(self.key_values[1], block_value);
            block_value = self.encrypt_block(self.key_values[2], block_value);

            let data_end_offset: usize = data_offset + 8;
            encrypted_data[data_offset..data_end_offset]
                .copy_from_slice(&block_value.to_be_bytes());

            initialization_vector_64bit = block_value;
            data_offset = data_end_offset;
        }
        Ok(())
    }
}

/// Context for DES3-CBC
pub type Des3CbcContext = CbcContext<Des3Context, 8>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_block() -> Result<(), ErrorTrace> {
        let des3_context: Des3Context = Des3Context::new();

        let output_value: u64 = des3_context.encrypt_block(0x9837239487, 0x2983123819080ac1);
        assert_eq!(output_value, 0xa9494d9bbdc2873f);

        Ok(())
    }

    #[test]
    fn test_decrypt_cbc() -> Result<(), ErrorTrace> {
        let mut des3_context: Des3Context = Des3Context::new();

        des3_context.set_key(b"This is a key123")?;

        let encrypted_data: [u8; 32] = [
            0x65, 0x86, 0x6b, 0x09, 0x01, 0x57, 0xd7, 0x64, 0xe4, 0xa4, 0xb3, 0x7e, 0x80, 0xd3,
            0xc3, 0x7f, 0x71, 0x7b, 0x45, 0x7d, 0x3a, 0x4c, 0x0a, 0x20, 0x2e, 0x32, 0xd1, 0xcf,
            0x8a, 0xf1, 0xa0, 0x21,
        ];
        let mut data: Vec<u8> = vec![0; 32];
        des3_context.decrypt_cbc(b"This IV!", &encrypted_data, &mut data)?;

        assert_eq!(&data, b"This is secret encrypted text!!!");

        Ok(())
    }

    #[test]
    fn test_decrypt_ecb() -> Result<(), ErrorTrace> {
        let mut des3_context: Des3Context = Des3Context::new();

        let key: [u8; 8] = [0x01, 0xea, 0x97, 0xbf, 0x45, 0x1c, 0xa8, 0x15];
        des3_context.set_key(&key)?;

        let encrypted_data: [u8; 8] = [0xc2, 0x0d, 0x08, 0x10, 0x9a, 0x04, 0x04, 0xbf];
        let mut data: Vec<u8> = vec![0; 8];
        des3_context.decrypt_ecb(&encrypted_data, &mut data)?;

        let expected_data: [u8; 8] = [0x40, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decrypt_block() -> Result<(), ErrorTrace> {
        let des3_context: Des3Context = Des3Context::new();

        let output_value: u64 = des3_context.decrypt_block(0x3719827398, 0x344720e90cdc908f);
        assert_eq!(output_value, 0x6d0ee7e5792e2a93);

        Ok(())
    }

    #[test]
    fn test_encrypt_cbc() -> Result<(), ErrorTrace> {
        let mut des3_context: Des3Context = Des3Context::new();

        des3_context.set_key(b"This is a key123")?;

        let mut encrypted_data: Vec<u8> = vec![0; 32];
        des3_context.encrypt_cbc(
            b"This IV!",
            b"This is secret encrypted text!!!",
            &mut encrypted_data,
        )?;

        let expected_encrypted_data: [u8; 32] = [
            0x65, 0x86, 0x6b, 0x09, 0x01, 0x57, 0xd7, 0x64, 0xe4, 0xa4, 0xb3, 0x7e, 0x80, 0xd3,
            0xc3, 0x7f, 0x71, 0x7b, 0x45, 0x7d, 0x3a, 0x4c, 0x0a, 0x20, 0x2e, 0x32, 0xd1, 0xcf,
            0x8a, 0xf1, 0xa0, 0x21,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_ecb() -> Result<(), ErrorTrace> {
        let mut des3_context: Des3Context = Des3Context::new();

        let key: [u8; 8] = [0x01, 0xea, 0x97, 0xbf, 0x45, 0x1c, 0xa8, 0x15];
        des3_context.set_key(&key)?;

        let data: [u8; 8] = [0x40, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        let mut encrypted_data: Vec<u8> = vec![0; 8];
        des3_context.encrypt_ecb(&data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 8] = [0xc2, 0x0d, 0x08, 0x10, 0x9a, 0x04, 0x04, 0xbf];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }
}
