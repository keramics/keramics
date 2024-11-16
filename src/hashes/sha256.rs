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

//! 256-bit Secure Hash Algorithm 2 (SHA-256).
//!
//! Provides support for calculating a SHA-256 hash (RFC 6234, FIPS 180-2).

/// SHA-256 block size.
const SHA256_BLOCK_SIZE: usize = 64;

/// The first 32-bits of the fractional parts of the square roots of the primes in [2, 19]
const SHA256_PRIME_SQUARE_ROOTS: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// The first 32-bits of the fractional parts of the cube roots of the primes in [2, 311]
const SHA256_PRIME_CUBE_ROOTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Context for calculating a SHA-256 hash.
pub struct Sha256Context {
    hash_values: [u32; 8],
    number_of_bytes_hashed: u64,
    block_offset: usize,
    block: [u8; SHA256_BLOCK_SIZE * 2],
}

impl Sha256Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            hash_values: SHA256_PRIME_SQUARE_ROOTS,
            number_of_bytes_hashed: 0,
            block_offset: 0,
            block: [0; SHA256_BLOCK_SIZE * 2],
        }
    }

    /// Finalizes the hash calculation.
    pub fn finalize(&mut self) -> Vec<u8> {
        let bit_size: u64 = (self.number_of_bytes_hashed + self.block_offset as u64) * 8;

        // Add padding with a size of 56 mod 64
        let padding_size: usize = SHA256_BLOCK_SIZE * if self.block_offset >= 56 { 2 } else { 1 };

        let bit_size_block_offset: usize = padding_size - 8;

        // The first byte of the padding contains 0x80
        self.block[self.block_offset] = 0x80;
        self.block[self.block_offset + 1..bit_size_block_offset].fill(0);
        self.block[bit_size_block_offset..padding_size].copy_from_slice(&bit_size.to_be_bytes());

        for block_offset in (0..padding_size).step_by(SHA256_BLOCK_SIZE) {
            let hash_values: [u32; 8] =
                self.transform_block(&self.hash_values, &self.block, block_offset);

            self.hash_values.copy_from_slice(&hash_values);
        }
        let hash: Vec<u8> = self
            .hash_values
            .iter()
            .map(|hash_value| hash_value.to_be_bytes())
            .flatten()
            .collect::<Vec<u8>>();

        self.hash_values = SHA256_PRIME_SQUARE_ROOTS;
        self.number_of_bytes_hashed = 0;
        self.block_offset = 0;
        self.block.fill(0);

        hash
    }

    /// Calculates the hash of a block of data.
    #[inline(always)]
    fn transform_block(
        &self,
        hash_values: &[u32],
        data: &[u8],
        mut data_offset: usize,
    ) -> [u32; 8] {
        let mut values_32bit: [u32; 64] = [0; 64];

        // Break the block of data into 16 x 32-bit big-endian values
        for value_index in 0..16 {
            values_32bit[value_index] = crate::bytes_to_u32_be!(data, data_offset);

            data_offset += 4;
        }
        // Extend to 64 x 32-bit values
        for value_index in 16..64 {
            let s0: u32 = values_32bit[value_index - 15].rotate_right(7)
                ^ values_32bit[value_index - 15].rotate_right(18)
                ^ (values_32bit[value_index - 15] >> 3);

            let s1: u32 = values_32bit[value_index - 2].rotate_right(17)
                ^ values_32bit[value_index - 2].rotate_right(19)
                ^ (values_32bit[value_index - 2] >> 10);

            values_32bit[value_index] = values_32bit[value_index - 16]
                .wrapping_add(s0)
                .wrapping_add(values_32bit[value_index - 7])
                .wrapping_add(s1);
        }
        // Calculate the hash values
        let mut block_hashes: [u32; 8] = [0; 8];
        block_hashes.copy_from_slice(hash_values);

        for value_index in 0..64 {
            let s0: u32 = block_hashes[0].rotate_right(2)
                ^ block_hashes[0].rotate_right(13)
                ^ block_hashes[0].rotate_right(22);

            let s1: u32 = block_hashes[4].rotate_right(6)
                ^ block_hashes[4].rotate_right(11)
                ^ block_hashes[4].rotate_right(25);

            let t1: u32 = block_hashes[7]
                .wrapping_add(s1)
                .wrapping_add(
                    (block_hashes[4] & block_hashes[5]) ^ (!(block_hashes[4]) & block_hashes[6]),
                )
                .wrapping_add(SHA256_PRIME_CUBE_ROOTS[value_index])
                .wrapping_add(values_32bit[value_index]);

            let t2: u32 = s0.wrapping_add(
                (block_hashes[0] & block_hashes[1])
                    ^ (block_hashes[0] & block_hashes[2])
                    ^ (block_hashes[1] & block_hashes[2]),
            );

            block_hashes[7] = block_hashes[6];
            block_hashes[6] = block_hashes[5];
            block_hashes[5] = block_hashes[4];
            block_hashes[4] = block_hashes[3].wrapping_add(t1);
            block_hashes[3] = block_hashes[2];
            block_hashes[2] = block_hashes[1];
            block_hashes[1] = block_hashes[0];
            block_hashes[0] = t1.wrapping_add(t2);
        }
        // Update the hash values
        for value_index in 0..8 {
            block_hashes[value_index] =
                block_hashes[value_index].wrapping_add(hash_values[value_index]);
        }
        block_hashes
    }

    /// Calculates the hash of the data.
    pub fn update(&mut self, data: &[u8]) {
        let data_size: usize = data.len();
        let mut data_offset: usize = 0;

        if self.block_offset > 0 {
            while self.block_offset < SHA256_BLOCK_SIZE {
                if data_offset >= data_size {
                    break;
                }
                self.block[self.block_offset] = data[data_offset];
                self.block_offset += 1;

                data_offset += 1;
            }
            if self.block_offset == SHA256_BLOCK_SIZE {
                let hash_values: [u32; 8] = self.transform_block(&self.hash_values, &self.block, 0);

                self.hash_values.copy_from_slice(&hash_values);
                self.number_of_bytes_hashed += SHA256_BLOCK_SIZE as u64;

                self.block_offset = 0;
            }
        }
        while data_offset + SHA256_BLOCK_SIZE < data_size {
            let hash_values: [u32; 8] = self.transform_block(&self.hash_values, &data, data_offset);

            self.hash_values.copy_from_slice(&hash_values);
            self.number_of_bytes_hashed += SHA256_BLOCK_SIZE as u64;

            data_offset += SHA256_BLOCK_SIZE;
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

        let mut test_context: Sha256Context = Sha256Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
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

        let mut test_context: Sha256Context = Sha256Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "a644092a1de8e05e17908ce565d55fcf39e30585565d96bf1c13eeb9f3401803"
        );
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

        let mut test_context: Sha256Context = Sha256Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(
            test_hash_string,
            "d19ddbd98476519a07cd8917b95eb609e5b50e8088ad68cd7426e8139d5bffc2"
        );
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

        let mut test_context: Sha256Context = Sha256Context::new();

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
        assert_eq!(
            test_hash_string,
            "d19ddbd98476519a07cd8917b95eb609e5b50e8088ad68cd7426e8139d5bffc2"
        );
    }
}
