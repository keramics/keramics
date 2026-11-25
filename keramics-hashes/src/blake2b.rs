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

//! BLAKE2b cryptographic hash function.
//!
//! Provides support for calculating a BLAKE2b hash (RFC 7693).

use std::cmp::min;

use keramics_core::ErrorTrace;
use keramics_types::bytes_to_u64_le;

use super::traits::DigestHashContext;

/// Initialization vector (first 64 bits of the fractional parts of the square roots of primes).
#[rustfmt::skip]
const BLAKE2B_INITIAL_VECTOR: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

/// Message schedule permutation table.
#[rustfmt::skip]
const BLAKE2B_SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// Context for calculating a BLAKE2b hash.
#[derive(Clone)]
pub struct Blake2bContext {
    /// Block buffer.
    block: [u8; 128],

    /// Current offset within the block buffer.
    block_offset: usize,

    /// Digest output size in bytes (1..=64).
    digest_size: usize,

    /// Low 64 bits of the compressed byte counter.
    hash_count_low: u64,

    /// High 64 bits of the compressed byte counter.
    hash_count_high: u64,

    /// Current hash state values.
    hash_values: [u64; 8],

    /// Optional secret key (0..=64 bytes).
    key: Vec<u8>,
}

impl Blake2bContext {
    /// Creates a new context with a specific digest output size.
    pub fn new_with_digest_size(digest_size: usize) -> Result<Self, ErrorTrace> {
        Self::new_with_key(digest_size, &[])
    }

    /// Creates a new context with a specific digest output size and key.
    pub fn new_with_key(digest_size: usize, key: &[u8]) -> Result<Self, ErrorTrace> {
        if digest_size == 0 || digest_size > 64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported digest size: {} value out of bounds",
                digest_size
            )));
        }
        let key_size: usize = key.len();
        if key_size > 64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported key size: {} value out of bounds",
                key_size
            )));
        }
        let mut block: [u8; 128] = [0; 128];
        let block_offset: usize = if key_size > 0 {
            block[0..key_size].copy_from_slice(key);
            128
        } else {
            0
        };
        let mut hash_values: [u64; 8] = BLAKE2B_INITIAL_VECTOR;
        hash_values[0] ^= 0x01010000 ^ ((key_size as u64) << 8) ^ (digest_size as u64);

        Ok(Self {
            block,
            block_offset,
            digest_size,
            hash_count_low: 0,
            hash_count_high: 0,
            hash_values,
            key: key.to_vec(),
        })
    }

    /// Compresses a 128-byte block into the hash state.
    fn compress_block(&mut self, block: &[u8; 128], is_last_block: bool) {
        let mut message: [u64; 16] = [0; 16];
        for (word_index, word) in message.iter_mut().enumerate() {
            let byte_offset: usize = word_index * 8;
            *word = bytes_to_u64_le!(block, byte_offset);
        }
        let mut work_vector: [u64; 16] = [0; 16];
        work_vector[0..8].copy_from_slice(&self.hash_values);
        work_vector[8..16].copy_from_slice(&BLAKE2B_INITIAL_VECTOR);
        work_vector[12] ^= self.hash_count_low;
        work_vector[13] ^= self.hash_count_high;

        if is_last_block {
            work_vector[14] = !work_vector[14];
        }
        for round in 0..12 {
            let sigma_index: usize = round % 10;
            let sigma: &[usize; 16] = &BLAKE2B_SIGMA[sigma_index];

            Self::mix(
                &mut work_vector,
                0,
                4,
                8,
                12,
                message[sigma[0]],
                message[sigma[1]],
            );
            Self::mix(
                &mut work_vector,
                1,
                5,
                9,
                13,
                message[sigma[2]],
                message[sigma[3]],
            );
            Self::mix(
                &mut work_vector,
                2,
                6,
                10,
                14,
                message[sigma[4]],
                message[sigma[5]],
            );
            Self::mix(
                &mut work_vector,
                3,
                7,
                11,
                15,
                message[sigma[6]],
                message[sigma[7]],
            );
            Self::mix(
                &mut work_vector,
                0,
                5,
                10,
                15,
                message[sigma[8]],
                message[sigma[9]],
            );
            Self::mix(
                &mut work_vector,
                1,
                6,
                11,
                12,
                message[sigma[10]],
                message[sigma[11]],
            );
            Self::mix(
                &mut work_vector,
                2,
                7,
                8,
                13,
                message[sigma[12]],
                message[sigma[13]],
            );
            Self::mix(
                &mut work_vector,
                3,
                4,
                9,
                14,
                message[sigma[14]],
                message[sigma[15]],
            );
        }
        for index in 0..8 {
            let upper_index: usize = index + 8;
            self.hash_values[index] ^= work_vector[index] ^ work_vector[upper_index];
        }
    }

    /// Increments the 128-bit compressed byte counter.
    fn increment_counter(&mut self, byte_count: u64) {
        let (new_low, overflowed): (u64, bool) = self.hash_count_low.overflowing_add(byte_count);
        self.hash_count_low = new_low;
        if overflowed {
            self.hash_count_high = self.hash_count_high.wrapping_add(1);
        }
    }

    /// Applies the BLAKE2b G mixing function.
    #[inline(always)]
    fn mix(
        work_vector: &mut [u64; 16],
        index_a: usize,
        index_b: usize,
        index_c: usize,
        index_d: usize,
        value_x: u64,
        value_y: u64,
    ) {
        let mut value_a: u64 = work_vector[index_a];
        let mut value_b: u64 = work_vector[index_b];
        let mut value_c: u64 = work_vector[index_c];
        let mut value_d: u64 = work_vector[index_d];

        value_a = value_a.wrapping_add(value_b).wrapping_add(value_x);
        value_d = (value_d ^ value_a).rotate_right(32);
        value_c = value_c.wrapping_add(value_d);
        value_b = (value_b ^ value_c).rotate_right(24);
        value_a = value_a.wrapping_add(value_b).wrapping_add(value_y);
        value_d = (value_d ^ value_a).rotate_right(16);
        value_c = value_c.wrapping_add(value_d);
        value_b = (value_b ^ value_c).rotate_right(63);

        work_vector[index_a] = value_a;
        work_vector[index_b] = value_b;
        work_vector[index_c] = value_c;
        work_vector[index_d] = value_d;
    }

    /// Resets the context state for reuse.
    fn reset(&mut self) {
        self.block = [0; 128];
        let key_size: usize = self.key.len();
        self.block_offset = if key_size > 0 {
            self.block[0..key_size].copy_from_slice(&self.key);
            128
        } else {
            0
        };
        self.hash_count_low = 0;
        self.hash_count_high = 0;
        self.hash_values = BLAKE2B_INITIAL_VECTOR;
        self.hash_values[0] ^= 0x01010000 ^ ((key_size as u64) << 8) ^ (self.digest_size as u64);
    }
}

impl DigestHashContext for Blake2bContext {
    /// Creates a new 512-bit BLAKE2b context.
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut hash_values: [u64; 8] = BLAKE2B_INITIAL_VECTOR;
        hash_values[0] ^= 0x01010000 ^ 64;

        Self {
            block: [0; 128],
            block_offset: 0,
            digest_size: 64,
            hash_count_low: 0,
            hash_count_high: 0,
            hash_values,
            key: Vec::new(),
        }
    }

    /// Finalizes the digest hash calculation.
    fn finalize(&mut self) -> Vec<u8> {
        let mut hash: Vec<u8> = vec![0; self.digest_size];
        self.finalize_into(&mut hash);
        hash
    }

    /// Finalizes the digest hash calculation into a fixed-size output buffer.
    fn finalize_into(&mut self, output: &mut [u8]) {
        self.increment_counter(self.block_offset as u64);
        self.block[self.block_offset..128].fill(0);

        let final_block: [u8; 128] = self.block;
        self.compress_block(&final_block, true);

        let mut full_hash: [u8; 64] = [0; 64];
        for (word_index, word) in self.hash_values.iter().enumerate() {
            let byte_offset: usize = word_index * 8;
            let byte_end_offset: usize = byte_offset + 8;
            full_hash[byte_offset..byte_end_offset].copy_from_slice(&word.to_le_bytes());
        }
        output[0..self.digest_size].copy_from_slice(&full_hash[0..self.digest_size]);

        self.reset();
    }

    /// Calculates the digest hash of the data.
    fn update(&mut self, data: &[u8]) {
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();

        while data_offset < data_size {
            if self.block_offset == 128 {
                self.increment_counter(128);
                let block_copy: [u8; 128] = self.block;
                self.compress_block(&block_copy, false);
                self.block_offset = 0;
            }
            let available_size: usize = 128 - self.block_offset;
            let remaining_size: usize = data_size - data_offset;
            let copy_size: usize = min(available_size, remaining_size);
            let block_end_offset: usize = self.block_offset + copy_size;
            let data_end_offset: usize = data_offset + copy_size;

            self.block[self.block_offset..block_end_offset]
                .copy_from_slice(&data[data_offset..data_end_offset]);

            self.block_offset = block_end_offset;
            data_offset = data_end_offset;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::formatters::format_as_string;

    #[test]
    fn test_increment_counter_with_overflow() {
        let mut test_context: Blake2bContext = Blake2bContext::new();
        test_context.hash_count_low = u64::MAX - 10;
        test_context.hash_count_high = 0;

        test_context.increment_counter(20);

        assert_eq!(test_context.hash_count_low, 9);
        assert_eq!(test_context.hash_count_high, 1);
    }

    #[test]
    fn test_new_with_unsupported_digest_size() {
        let result: Result<Blake2bContext, ErrorTrace> = Blake2bContext::new_with_digest_size(0);
        assert!(result.is_err());

        let result: Result<Blake2bContext, ErrorTrace> = Blake2bContext::new_with_digest_size(65);
        assert!(result.is_err());

        let oversized_key: [u8; 65] = [0; 65];
        let result: Result<Blake2bContext, ErrorTrace> =
            Blake2bContext::new_with_key(64, &oversized_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_with_key_and_reset() -> Result<(), ErrorTrace> {
        let key: [u8; 64] = core::array::from_fn(|index| index as u8);
        let mut test_context: Blake2bContext = Blake2bContext::new_with_key(64, &key)?;
        test_context.update(&[]);
        let first_hash: Vec<u8> = test_context.finalize();

        let first_hash_string: String = format_as_string(&first_hash);
        assert_eq!(
            first_hash_string,
            "10ebb67700b1868efb4417987acf4690ae9d972fb7a590c2f02871799aaa4786b5e996e8f0f4eb981fc214b005f42d2ff4233499391653df7aefcbc13fc51568"
        );
        // Verify context resets with the key intact for subsequent hashing.
        test_context.update(&[0x00]);
        let second_hash: Vec<u8> = test_context.finalize();

        let second_hash_string: String = format_as_string(&second_hash);
        assert_eq!(
            second_hash_string,
            "961f6dd1e4dd30f63901690c512e78e4b45e4742ed197c3c5e45c549fd25f2e4187b0bc9fe30492b16b0d0bc4ef9b0f34c7003fac09a5ef1532e69430234cebd"
        );
        Ok(())
    }

    #[test]
    fn test_update_and_finalize_with_empty_block() {
        let test_data: [u8; 0] = [];

        let mut test_context: Blake2bContext = Blake2bContext::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"
        );
    }

    #[test]
    fn test_update_and_finalize_with_abc() {
        let mut test_context: Blake2bContext = Blake2bContext::new();
        test_context.update(b"abc");
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923"
        );
    }

    #[test]
    fn test_update_and_finalize_with_variable_digest_size() -> Result<(), ErrorTrace> {
        let mut test_context: Blake2bContext = Blake2bContext::new_with_digest_size(32)?;
        test_context.update(b"abc");
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319"
        );
        Ok(())
    }

    #[test]
    fn test_update_and_finalize_multiple_blocks() {
        let test_data: [u8; 256] = [0x42; 256];

        let mut test_context: Blake2bContext = Blake2bContext::new();
        test_context.update(&test_data[0..128]);
        test_context.update(&test_data[128..256]);
        let split_hash: Vec<u8> = test_context.finalize();

        test_context.update(&test_data);
        let combined_hash: Vec<u8> = test_context.finalize();

        assert_eq!(split_hash, combined_hash);
    }
}
