/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

//! Secure Hash Algorithm 1 (SHA1).
//!
//! Provides support for calculating a SHA1 hash (RFC 1321, FIPS 180-1).

use crate::bytes_to_u32_be;

use super::traits::DigestHashContext;

/// SHA1 block size.
const SHA1_BLOCK_SIZE: usize = 64;

/// SHA1 initial hash values.
const SHA1_HASH_VALUES: [u32; 5] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

/// Context for calculating a SHA1 hash.
pub struct Sha1Context {
    hash_values: [u32; 5],
    number_of_bytes_hashed: u64,
    block_offset: usize,
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
    fn transform_block(
        &self,
        hash_values: &[u32],
        data: &[u8],
        mut data_offset: usize,
    ) -> [u32; 5] {
        let mut values_32bit: [u32; 80] = [0; 80];

        // Break the block of data into 16 x 32-bit big-endian values
        for value_index in 0..16 {
            values_32bit[value_index] = bytes_to_u32_be!(data, data_offset);

            data_offset += 4;
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
        let mut block_hashes: [u32; 5] = [0; 5];
        block_hashes.copy_from_slice(hash_values);

        for value_index in 0..80 {
            let block_hash: u32 = if value_index < 20 {
                0x5a827999_u32.wrapping_add(
                    (block_hashes[1] & block_hashes[2]) | (!(block_hashes[1]) & block_hashes[3]),
                )
            } else if value_index < 40 {
                0x6ed9eba1_u32.wrapping_add(block_hashes[1] ^ block_hashes[2] ^ block_hashes[3])
            } else if value_index < 60 {
                0x8f1bbcdc_u32.wrapping_add(
                    (block_hashes[1] & block_hashes[2])
                        | (block_hashes[1] & block_hashes[3])
                        | (block_hashes[2] & block_hashes[3]),
                )
            } else {
                0xca62c1d6_u32.wrapping_add(block_hashes[1] ^ block_hashes[2] ^ block_hashes[3])
            }
            .wrapping_add(block_hashes[4])
            .wrapping_add(block_hashes[0].rotate_left(5))
            .wrapping_add(values_32bit[value_index]);

            block_hashes[4] = block_hashes[3];
            block_hashes[3] = block_hashes[2];
            block_hashes[2] = block_hashes[1].rotate_left(30);
            block_hashes[1] = block_hashes[0];
            block_hashes[0] = block_hash;
        }
        // Update the hash values
        for value_index in 0..5 {
            block_hashes[value_index] =
                block_hashes[value_index].wrapping_add(hash_values[value_index]);
        }
        block_hashes
    }
}

impl DigestHashContext for Sha1Context {
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

        for block_offset in (0..padding_size).step_by(SHA1_BLOCK_SIZE) {
            let hash_values: [u32; 5] =
                self.transform_block(&self.hash_values, &self.block, block_offset);

            self.hash_values.copy_from_slice(&hash_values);
        }
        let hash: Vec<u8> = self
            .hash_values
            .iter()
            .map(|hash_value| hash_value.to_be_bytes())
            .flatten()
            .collect::<Vec<u8>>();

        self.hash_values = SHA1_HASH_VALUES;
        self.number_of_bytes_hashed = 0;
        self.block_offset = 0;
        self.block.fill(0);

        hash
    }

    /// Calculates the digest hash of the data.
    fn update(&mut self, data: &[u8]) {
        let data_size: usize = data.len();
        let mut data_offset: usize = 0;

        if self.block_offset > 0 {
            while self.block_offset < SHA1_BLOCK_SIZE {
                if data_offset >= data_size {
                    break;
                }
                self.block[self.block_offset] = data[data_offset];
                self.block_offset += 1;

                data_offset += 1;
            }
            if self.block_offset == SHA1_BLOCK_SIZE {
                let hash_values: [u32; 5] = self.transform_block(&self.hash_values, &self.block, 0);

                self.hash_values.copy_from_slice(&hash_values);
                self.number_of_bytes_hashed += SHA1_BLOCK_SIZE as u64;

                self.block_offset = 0;
            }
        }
        while data_offset + SHA1_BLOCK_SIZE < data_size {
            let hash_values: [u32; 5] = self.transform_block(&self.hash_values, &data, data_offset);

            self.hash_values.copy_from_slice(&hash_values);
            self.number_of_bytes_hashed += SHA1_BLOCK_SIZE as u64;

            data_offset += SHA1_BLOCK_SIZE;
        }
        while data_offset < data_size {
            self.block[self.block_offset] = data[data_offset];
            self.block_offset += 1;

            data_offset += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::formatters::format_as_string;

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
