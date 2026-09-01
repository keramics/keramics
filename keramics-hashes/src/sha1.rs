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

//! Secure Hash Algorithm 1 (SHA-1).
//!
//! Provides support for calculating a SHA-1 hash (RFC 1321, FIPS 180-1).

use std::cmp::min;
use std::slice::ChunksExact;

use super::traits::DigestHashContext;

/// SHA-1 block size.
const SHA1_BLOCK_SIZE: usize = 64;

/// SHA-1 initial hash values.
const SHA1_HASH_VALUES: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

/// SHA-1 transform step for rounds 0 to 19
macro_rules! sha1_transform_step1 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $value_32bit:expr) => {
        let initial_hash: u32 = ($block_hash1 & $block_hash2) | (!$block_hash1 & $block_hash3);

        $block_hash4 = $block_hash4
            .wrapping_add($block_hash0.rotate_left(5))
            .wrapping_add(initial_hash)
            .wrapping_add(0x5a827999)
            .wrapping_add($value_32bit);

        $block_hash1 = $block_hash1.rotate_left(30);
    };
}

/// SHA-1 transform step for rounds 20 to 39
macro_rules! sha1_transform_step2 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $value_32bit:expr) => {
        let initial_hash: u32 = $block_hash1 ^ $block_hash2 ^ $block_hash3;

        $block_hash4 = $block_hash4
            .wrapping_add($block_hash0.rotate_left(5))
            .wrapping_add(initial_hash)
            .wrapping_add(0x6ed9eba1)
            .wrapping_add($value_32bit);

        $block_hash1 = $block_hash1.rotate_left(30);
    };
}

/// SHA-1 transform step for rounds 40 to 59
macro_rules! sha1_transform_step3 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $value_32bit:expr) => {
        let initial_hash: u32 = ($block_hash1 & $block_hash2)
            | ($block_hash1 & $block_hash3)
            | ($block_hash2 & $block_hash3);

        $block_hash4 = $block_hash4
            .wrapping_add($block_hash0.rotate_left(5))
            .wrapping_add(initial_hash)
            .wrapping_add(0x8f1bbcdc)
            .wrapping_add($value_32bit);

        $block_hash1 = $block_hash1.rotate_left(30);
    };
}

/// SHA-1 transform step for rounds 60 to 79
macro_rules! sha1_transform_step4 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $value_32bit:expr) => {
        let initial_hash: u32 = $block_hash1 ^ $block_hash2 ^ $block_hash3;

        $block_hash4 = $block_hash4
            .wrapping_add($block_hash0.rotate_left(5))
            .wrapping_add(initial_hash)
            .wrapping_add(0xca62c1d6)
            .wrapping_add($value_32bit);

        $block_hash1 = $block_hash1.rotate_left(30);
    };
}

/// SHA-1 transform step for a group of 5 32-bit values for rounds 0 to 19
macro_rules! sha1_transform_group_step1 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $values_32bit:expr, $index:expr) => {
        sha1_transform_step1!(
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $values_32bit[$index]
        );
        sha1_transform_step1!(
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $values_32bit[$index + 1]
        );
        sha1_transform_step1!(
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $values_32bit[$index + 2]
        );
        sha1_transform_step1!(
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $values_32bit[$index + 3]
        );
        sha1_transform_step1!(
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $values_32bit[$index + 4]
        );
    };
}

/// SHA-1 transform step for a group of 5 32-bit values for rounds 20 to 39
macro_rules! sha1_transform_group_step2 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $values_32bit:expr, $index:expr) => {
        sha1_transform_step2!(
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $values_32bit[$index]
        );
        sha1_transform_step2!(
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $values_32bit[$index + 1]
        );
        sha1_transform_step2!(
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $values_32bit[$index + 2]
        );
        sha1_transform_step2!(
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $values_32bit[$index + 3]
        );
        sha1_transform_step2!(
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $values_32bit[$index + 4]
        );
    };
}

/// SHA-1 transform step for a group of 5 32-bit values for rounds 40 to 59
macro_rules! sha1_transform_group_step3 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $values_32bit:expr, $index:expr) => {
        sha1_transform_step3!(
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $values_32bit[$index]
        );
        sha1_transform_step3!(
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $values_32bit[$index + 1]
        );
        sha1_transform_step3!(
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $values_32bit[$index + 2]
        );
        sha1_transform_step3!(
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $values_32bit[$index + 3]
        );
        sha1_transform_step3!(
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $values_32bit[$index + 4]
        );
    };
}

/// SHA-1 transform step for a group of 5 32-bit values for rounds 60 to 79
macro_rules! sha1_transform_group_step4 {
    ($block_hash0:expr, $block_hash1:expr, $block_hash2:expr, $block_hash3:expr, $block_hash4:expr, $values_32bit:expr, $index:expr) => {
        sha1_transform_step4!(
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $values_32bit[$index]
        );
        sha1_transform_step4!(
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $values_32bit[$index + 1]
        );
        sha1_transform_step4!(
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $block_hash2,
            $values_32bit[$index + 2]
        );
        sha1_transform_step4!(
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $block_hash1,
            $values_32bit[$index + 3]
        );
        sha1_transform_step4!(
            $block_hash1,
            $block_hash2,
            $block_hash3,
            $block_hash4,
            $block_hash0,
            $values_32bit[$index + 4]
        );
    };
}

/// Context for calculating a SHA-1 hash.
#[derive(Clone)]
pub struct Sha1Context {
    /// Hash values.
    hash_values: [u32; 5],

    /// Number of bytes hashed.
    number_of_bytes_hashed: u64,

    /// Block offset.
    block_offset: usize,

    /// Block.
    block: [u8; SHA1_BLOCK_SIZE * 2],
}

impl Sha1Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            hash_values: SHA1_HASH_VALUES,
            number_of_bytes_hashed: 0,
            block_offset: 0,
            block: [0; SHA1_BLOCK_SIZE * 2],
        }
    }

    /// Calculates the hash of a block of data.
    #[inline(always)]
    fn transform_block(&self, hash_values: &mut [u32], data: &[u8]) {
        let mut values_32bit: [u32; 80] = [0; 80];

        // Break the block of data into 16 x 32-bit big-endian values
        for (index, chunk) in data[0..SHA1_BLOCK_SIZE].chunks_exact(4).enumerate() {
            values_32bit[index] = u32::from_be_bytes(chunk.try_into().unwrap());
        }
        // Extend to 80 x 32-bit values
        for value_index in 16..80 {
            values_32bit[value_index] = values_32bit[value_index - 3]
                ^ values_32bit[value_index - 8]
                ^ values_32bit[value_index - 14]
                ^ values_32bit[value_index - 16];
            values_32bit[value_index] = values_32bit[value_index].rotate_left(1);
        }
        // Calculate the hash values
        let mut block_hash0: u32 = hash_values[0];
        let mut block_hash1: u32 = hash_values[1];
        let mut block_hash2: u32 = hash_values[2];
        let mut block_hash3: u32 = hash_values[3];
        let mut block_hash4: u32 = hash_values[4];

        for index in (0..20).step_by(5) {
            sha1_transform_group_step1!(
                block_hash0,
                block_hash1,
                block_hash2,
                block_hash3,
                block_hash4,
                values_32bit,
                index
            );
        }
        for index in (20..40).step_by(5) {
            sha1_transform_group_step2!(
                block_hash0,
                block_hash1,
                block_hash2,
                block_hash3,
                block_hash4,
                values_32bit,
                index
            );
        }
        for index in (40..60).step_by(5) {
            sha1_transform_group_step3!(
                block_hash0,
                block_hash1,
                block_hash2,
                block_hash3,
                block_hash4,
                values_32bit,
                index
            );
        }
        for index in (60..80).step_by(5) {
            sha1_transform_group_step4!(
                block_hash0,
                block_hash1,
                block_hash2,
                block_hash3,
                block_hash4,
                values_32bit,
                index
            );
        }
        // Update the hash values
        hash_values[0] = block_hash0.wrapping_add(hash_values[0]);
        hash_values[1] = block_hash1.wrapping_add(hash_values[1]);
        hash_values[2] = block_hash2.wrapping_add(hash_values[2]);
        hash_values[3] = block_hash3.wrapping_add(hash_values[3]);
        hash_values[4] = block_hash4.wrapping_add(hash_values[4]);
    }
}

impl DigestHashContext for Sha1Context {
    /// Creates a new context.
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    /// Finalizes the hash calculation.
    fn finalize(&mut self) -> Vec<u8> {
        let bit_size: u64 = (self.number_of_bytes_hashed + self.block_offset as u64) * 8;

        // Add padding with a size of 56 mod 64
        let padding_size: usize = SHA1_BLOCK_SIZE * if self.block_offset >= 56 { 2 } else { 1 };

        let bit_size_block_offset: usize = padding_size - 8;

        // The first byte of the padding contains 0x80
        self.block[self.block_offset] = 0x80;
        self.block[self.block_offset + 1..bit_size_block_offset].fill(0);
        self.block[bit_size_block_offset..padding_size].copy_from_slice(&bit_size.to_be_bytes());

        let mut hash_values: [u32; 5] = [0; 5];
        hash_values.copy_from_slice(&self.hash_values);

        for chunk in self.block[0..padding_size].chunks_exact(SHA1_BLOCK_SIZE) {
            self.transform_block(&mut hash_values, chunk);
        }
        let hash: Vec<u8> = hash_values
            .iter()
            .flat_map(|hash_value| hash_value.to_be_bytes())
            .collect::<Vec<u8>>();

        self.hash_values = SHA1_HASH_VALUES;
        self.number_of_bytes_hashed = 0;
        self.block_offset = 0;
        self.block.fill(0);

        hash
    }

    /// Calculates the digest hash of the data.
    fn update(&mut self, data: &[u8]) {
        let mut hash_values: [u32; 5] = [0; 5];
        hash_values.copy_from_slice(&self.hash_values);

        let data_offset: usize = if self.block_offset == 0 {
            0
        } else {
            let remaining_block_size: usize = SHA1_BLOCK_SIZE - self.block_offset;
            let data_end_offset: usize = min(remaining_block_size, data.len());

            for byte_value in data[0..data_end_offset].iter() {
                self.block[self.block_offset] = *byte_value;
                self.block_offset += 1;
            }
            if self.block_offset == SHA1_BLOCK_SIZE {
                self.transform_block(&mut hash_values, &self.block);

                self.number_of_bytes_hashed += SHA1_BLOCK_SIZE as u64;

                self.block_offset = 0;
            }
            data_end_offset
        };
        let mut chunks: ChunksExact<'_, u8> = data[data_offset..].chunks_exact(SHA1_BLOCK_SIZE);

        for chunk in &mut chunks {
            self.transform_block(&mut hash_values, chunk);

            self.number_of_bytes_hashed += SHA1_BLOCK_SIZE as u64;
        }
        self.hash_values.copy_from_slice(&hash_values);

        let remainder: &[u8] = chunks.remainder();

        for byte_value in remainder.iter() {
            self.block[self.block_offset] = *byte_value;
            self.block_offset += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::formatters::format_as_string;

    #[test]
    fn test_update_and_finalize_with_empty_block() {
        let test_data: [u8; 0] = [];

        let mut test_context: Sha1Context = Sha1Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_update_and_finalize_with_single_block() {
        let test_data: [u8; 63] = [
            0xff, 0xf0, 0x0f, 0xff, 0xff, 0x00, 0x06, 0x00, 0xff, 0xff, 0xf0, 0x07, 0xff, 0xe0,
            0x04, 0x00, 0x03, 0x00, 0x00, 0x03, 0xf0, 0xff, 0xff, 0x00, 0x03, 0xff, 0xfb, 0xff,
            0xc3, 0xff, 0xf0, 0x07, 0xff, 0xff, 0xc7, 0x00, 0x7f, 0x80, 0x00, 0x03, 0xff, 0xf8,
            0x00, 0x1f, 0xe1, 0xff, 0xf8, 0x63, 0xfc, 0x00, 0x3f, 0xc0, 0x9f, 0xff, 0xf8, 0x00,
            0x00, 0x7f, 0xff, 0x1f, 0xff, 0xfc, 0x00,
        ];

        let mut test_context: Sha1Context = Sha1Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "3acbf874199763eba20f3789dfc59572aca4cf33");
    }

    #[test]
    fn test_update_and_finalize_multiple_blocks() {
        let test_data: [u8; 128] = [
            0xff, 0xff, 0xff, 0x00, 0x00, 0x03, 0xc0, 0x00, 0x00, 0x01, 0xff, 0xff, 0xf8, 0x00,
            0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0x00, 0x00, 0x0f, 0x00, 0x00, 0xff, 0xff, 0xf8,
            0x80, 0x00, 0xf8, 0x00, 0x0f, 0xc0, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0xff,
            0xff, 0xff, 0xf8, 0x0f, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x18, 0x00, 0x00, 0x7f, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x03, 0xff, 0xff, 0xff, 0x00, 0x7f, 0xff, 0xff, 0xfc, 0x00,
            0x03, 0xc0, 0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xf0, 0x00, 0x07, 0xff, 0xff, 0x80,
            0x01, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x0f, 0xff, 0xfe, 0x07, 0xff, 0xff, 0xf8, 0x00,
            0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x03, 0xe0, 0x00, 0x07, 0xff, 0xf0, 0x0f, 0xff,
            0xf0, 0x00, 0x00, 0xff, 0xff, 0xf8, 0x7f, 0xc0, 0x03, 0xc0, 0x3f, 0xff, 0xe0, 0x00,
            0x00, 0x00,
        ];

        let mut test_context: Sha1Context = Sha1Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "ede4deb4293cfe4138c2c056b7c46ff821cc0acc");
    }

    #[test]
    fn test_incremental_update_and_finalize() {
        let test_data: [u8; 128] = [
            0xff, 0xff, 0xff, 0x00, 0x00, 0x03, 0xc0, 0x00, 0x00, 0x01, 0xff, 0xff, 0xf8, 0x00,
            0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0x00, 0x00, 0x0f, 0x00, 0x00, 0xff, 0xff, 0xf8,
            0x80, 0x00, 0xf8, 0x00, 0x0f, 0xc0, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x00, 0x00, 0xff,
            0xff, 0xff, 0xf8, 0x0f, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x18, 0x00, 0x00, 0x7f, 0xff,
            0xff, 0xff, 0x00, 0x00, 0x03, 0xff, 0xff, 0xff, 0x00, 0x7f, 0xff, 0xff, 0xfc, 0x00,
            0x03, 0xc0, 0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xf0, 0x00, 0x07, 0xff, 0xff, 0x80,
            0x01, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x0f, 0xff, 0xfe, 0x07, 0xff, 0xff, 0xf8, 0x00,
            0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x03, 0xe0, 0x00, 0x07, 0xff, 0xf0, 0x0f, 0xff,
            0xf0, 0x00, 0x00, 0xff, 0xff, 0xf8, 0x7f, 0xc0, 0x03, 0xc0, 0x3f, 0xff, 0xe0, 0x00,
            0x00, 0x00,
        ];

        let mut test_context: Sha1Context = Sha1Context::new();

        let data_size: usize = test_data.len();
        let mut data_offset: usize = 0;
        let mut data_end_offset: usize = 32;
        while data_end_offset < data_size {
            test_context.update(&test_data[data_offset..data_end_offset]);
            data_offset = data_end_offset;
            data_end_offset += 32;
        }
        if data_offset < data_size {
            test_context.update(&test_data[data_offset..]);
        }
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "ede4deb4293cfe4138c2c056b7c46ff821cc0acc");
    }
}
