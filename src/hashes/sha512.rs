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

//! 512-bit Secure Hash Algorithm 2 (SHA-512).
//!
//! Provides support for calculating a SHA-512 hash (RFC 6234, FIPS 180-2).

/// SHA-512 block size.
const SHA512_BLOCK_SIZE: usize = 128;

/// The first 64-bits of the fractional parts of the square roots of the primes in [2, 19]
#[rustfmt::skip]
const SHA512_PRIME_SQUARE_ROOTS: [u64; 8] = [
    0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
    0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
];

/// The first 64-bits of the fractional parts of the cube roots of the primes in [2, 409]
#[rustfmt::skip]
const SHA512_PRIME_CUBE_ROOTS: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

/// Context for calculating a SHA-512 hash.
pub struct Sha512Context {
    hash_values: [u64; 8],
    number_of_bytes_hashed: u64,
    block_offset: usize,
    block: [u8; SHA512_BLOCK_SIZE * 2],
}

impl Sha512Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            hash_values: SHA512_PRIME_SQUARE_ROOTS,
            number_of_bytes_hashed: 0,
            block_offset: 0,
            block: [0; SHA512_BLOCK_SIZE * 2],
        }
    }

    /// Finalizes the hash calculation.
    pub fn finalize(&mut self) -> Vec<u8> {
        let bit_size: u64 = (self.number_of_bytes_hashed + self.block_offset as u64) * 8;

        // Add padding with a size of 112 mod 128
        let padding_size: usize = SHA512_BLOCK_SIZE * if self.block_offset >= 112 { 2 } else { 1 };

        let bit_size_block_offset: usize = padding_size - 8;

        // The first byte of the padding contains 0x80
        self.block[self.block_offset] = 0x80;
        self.block[self.block_offset + 1..bit_size_block_offset].fill(0);
        self.block[bit_size_block_offset..padding_size].copy_from_slice(&bit_size.to_be_bytes());

        for block_offset in (0..padding_size).step_by(SHA512_BLOCK_SIZE) {
            let hash_values: [u64; 8] =
                self.transform_block(&self.hash_values, &self.block, block_offset);

            self.hash_values.copy_from_slice(&hash_values);
        }
        let hash: Vec<u8> = self
            .hash_values
            .iter()
            .map(|hash_value| hash_value.to_be_bytes())
            .flatten()
            .collect::<Vec<u8>>();

        self.hash_values = SHA512_PRIME_SQUARE_ROOTS;
        self.number_of_bytes_hashed = 0;
        self.block_offset = 0;
        self.block.fill(0);

        hash
    }

    /// Calculates the hash of a block of data.
    #[inline(always)]
    fn transform_block(
        &self,
        hash_values: &[u64],
        data: &[u8],
        mut data_offset: usize,
    ) -> [u64; 8] {
        let mut values_64bit: [u64; 80] = [0; 80];

        // Break the block of data into 16 x 64-bit big-endian values
        for value_index in 0..16 {
            values_64bit[value_index] = crate::bytes_to_u64_be!(data, data_offset);

            data_offset += 8;
        }
        // Extend to 80 x 64-bit values
        for value_index in 16..80 {
            let s0: u64 = values_64bit[value_index - 15].rotate_right(1)
                ^ values_64bit[value_index - 15].rotate_right(8)
                ^ (values_64bit[value_index - 15] >> 7);

            let s1: u64 = values_64bit[value_index - 2].rotate_right(19)
                ^ values_64bit[value_index - 2].rotate_right(61)
                ^ (values_64bit[value_index - 2] >> 6);

            values_64bit[value_index] = values_64bit[value_index - 16]
                .wrapping_add(s0)
                .wrapping_add(values_64bit[value_index - 7])
                .wrapping_add(s1);
        }
        // Calculate the hash values
        let mut block_hashes: [u64; 8] = [0; 8];
        block_hashes.copy_from_slice(hash_values);

        for value_index in 0..80 {
            let s0: u64 = block_hashes[0].rotate_right(28)
                ^ block_hashes[0].rotate_right(34)
                ^ block_hashes[0].rotate_right(39);

            let s1: u64 = block_hashes[4].rotate_right(14)
                ^ block_hashes[4].rotate_right(18)
                ^ block_hashes[4].rotate_right(41);

            let t1: u64 = block_hashes[7]
                .wrapping_add(s1)
                .wrapping_add(
                    (block_hashes[4] & block_hashes[5]) ^ (!(block_hashes[4]) & block_hashes[6]),
                )
                .wrapping_add(SHA512_PRIME_CUBE_ROOTS[value_index])
                .wrapping_add(values_64bit[value_index]);

            let t2: u64 = s0.wrapping_add(
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
            while self.block_offset < SHA512_BLOCK_SIZE {
                if data_offset >= data_size {
                    break;
                }
                self.block[self.block_offset] = data[data_offset];
                self.block_offset += 1;

                data_offset += 1;
            }
            if self.block_offset == SHA512_BLOCK_SIZE {
                let hash_values: [u64; 8] = self.transform_block(&self.hash_values, &self.block, 0);

                self.hash_values.copy_from_slice(&hash_values);
                self.number_of_bytes_hashed += SHA512_BLOCK_SIZE as u64;

                self.block_offset = 0;
            }
        }
        while data_offset + SHA512_BLOCK_SIZE < data_size {
            let hash_values: [u64; 8] = self.transform_block(&self.hash_values, &data, data_offset);

            self.hash_values.copy_from_slice(&hash_values);
            self.number_of_bytes_hashed += SHA512_BLOCK_SIZE as u64;

            data_offset += SHA512_BLOCK_SIZE;
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

        let mut test_context: Sha512Context = Sha512Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
    }

    #[test]
    fn test_update_and_finalize_with_single_block() {
        let test_data: [u8; 127] = [
            0x00, 0x00, 0x00, 0x0f, 0xff, 0xc0, 0x00, 0x0f, 0xf8, 0x00, 0x00, 0x00, 0xff, 0xfc,
            0x00, 0x00, 0x0f, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x00, 0x03, 0xff, 0x00, 0xff, 0xfc,
            0x07, 0xff, 0xf0, 0x1f, 0xff, 0xfe, 0x0f, 0xff, 0xff, 0xfd, 0xff, 0xff, 0xff, 0xf0,
            0x1f, 0xff, 0xf0, 0x00, 0x07, 0xf8, 0xff, 0xf8, 0x00, 0x00, 0x00, 0xff, 0xff, 0xc0,
            0x3f, 0xff, 0xff, 0xff, 0x80, 0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0x00, 0x00, 0x03,
            0xff, 0xff, 0xf0, 0x00, 0x07, 0xff, 0xff, 0x00, 0x00, 0x3f, 0xff, 0xf0, 0x01, 0xff,
            0xff, 0xc0, 0x01, 0xff, 0xff, 0xff, 0x00, 0x3f, 0xff, 0xf8, 0x1f, 0xff, 0xff, 0xfe,
            0x1f, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x7f, 0xe0, 0x00, 0x07, 0xff, 0xff, 0xfe,
            0x00, 0x00, 0x00, 0x03, 0xe0, 0x07, 0xff, 0xc0, 0x03, 0xfc, 0x00, 0x07, 0xff, 0xff,
            0xff,
        ];

        let mut test_context: Sha512Context = Sha512Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "c6a5f4bbdb075c17ebcf4131de0fe33d3e2bb6edb5af7c277b472ea7847b11d2aa2598cb7ca75e4fe94c264bd2942bd82fc60b5045bd7c5cbc31325954713dfc");
    }

    #[test]
    fn test_update_and_finalize_multiple_blocks() {
        let test_data: [u8; 257] = [
            0x00, 0x00, 0x07, 0xff, 0xff, 0x00, 0x00, 0x07, 0xff, 0xff, 0xf0, 0x01, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x80, 0x00, 0x00, 0x00, 0x06, 0x00, 0x0f, 0x7f, 0xff, 0xff, 0xff,
            0xc0, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x1f, 0xf0, 0xff, 0xff, 0xff, 0xff,
            0x80, 0x00, 0x70, 0x00, 0x00, 0x00, 0x3f, 0xff, 0x80, 0x0f, 0xff, 0xff, 0xff, 0xf0,
            0x00, 0x7f, 0xff, 0xc0, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x7f,
            0xff, 0xff, 0xff, 0xfe, 0x1f, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x03, 0xff, 0xff, 0xff,
            0xfc, 0x00, 0x00, 0x00, 0x7f, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x03, 0xff, 0xff,
            0x03, 0xff, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x00, 0x01, 0xff,
            0xe0, 0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xf0, 0x00, 0x1f, 0xff, 0xff, 0xff,
            0x00, 0x00, 0x01, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x00, 0x00, 0x78, 0x07, 0xff, 0x00,
            0x00, 0x00, 0xff, 0x80, 0x1f, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x03, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x80, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x07, 0xff, 0xff,
            0xff, 0xfc, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x00, 0x07, 0x00, 0x00, 0x1f,
            0xfe, 0x00, 0x00, 0x00, 0x00, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xfc, 0x1f, 0xff, 0xff,
            0xff, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x03, 0xff, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x03, 0xff, 0xff, 0xff, 0xff, 0x80, 0x00, 0x7f, 0xe0, 0x1f, 0xff, 0xff, 0xff,
            0xe1, 0xff, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x7f, 0xff, 0xff, 0xe7, 0xff, 0xff, 0xff,
            0xff, 0xf0, 0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x07, 0xff, 0xff,
            0xff, 0xe0, 0x00, 0x00, 0x00,
        ];

        let mut test_context: Sha512Context = Sha512Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "8a006a0a1e2dd36ba57aa0325b2c9532db7649a4c3c6214ee0f004ecabcf1eef89a91b225ffc52a4f811791d20f6faddd900b863386da65daec18e00c48412d6");
    }

    #[test]
    fn test_incremental_update_and_finalize() {
        let test_data: [u8; 257] = [
            0x00, 0x00, 0x07, 0xff, 0xff, 0x00, 0x00, 0x07, 0xff, 0xff, 0xf0, 0x01, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x80, 0x00, 0x00, 0x00, 0x06, 0x00, 0x0f, 0x7f, 0xff, 0xff, 0xff,
            0xc0, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x1f, 0xf0, 0xff, 0xff, 0xff, 0xff,
            0x80, 0x00, 0x70, 0x00, 0x00, 0x00, 0x3f, 0xff, 0x80, 0x0f, 0xff, 0xff, 0xff, 0xf0,
            0x00, 0x7f, 0xff, 0xc0, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x7f,
            0xff, 0xff, 0xff, 0xfe, 0x1f, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x03, 0xff, 0xff, 0xff,
            0xfc, 0x00, 0x00, 0x00, 0x7f, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x03, 0xff, 0xff,
            0x03, 0xff, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x00, 0x01, 0xff,
            0xe0, 0x00, 0x00, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xf0, 0x00, 0x1f, 0xff, 0xff, 0xff,
            0x00, 0x00, 0x01, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x00, 0x00, 0x78, 0x07, 0xff, 0x00,
            0x00, 0x00, 0xff, 0x80, 0x1f, 0xff, 0xff, 0xff, 0xc0, 0x00, 0x00, 0x03, 0xff, 0xff,
            0xff, 0xff, 0xff, 0x80, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x07, 0xff, 0xff,
            0xff, 0xfc, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xe0, 0x00, 0x00, 0x07, 0x00, 0x00, 0x1f,
            0xfe, 0x00, 0x00, 0x00, 0x00, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xfc, 0x1f, 0xff, 0xff,
            0xff, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x00, 0x03, 0xff, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x03, 0xff, 0xff, 0xff, 0xff, 0x80, 0x00, 0x7f, 0xe0, 0x1f, 0xff, 0xff, 0xff,
            0xe1, 0xff, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x7f, 0xff, 0xff, 0xe7, 0xff, 0xff, 0xff,
            0xff, 0xf0, 0x00, 0x00, 0x3f, 0xff, 0xff, 0xff, 0xfe, 0x00, 0x00, 0x07, 0xff, 0xff,
            0xff, 0xe0, 0x00, 0x00, 0x00,
        ];

        let mut test_context: Sha512Context = Sha512Context::new();

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
        assert_eq!(test_hash_string, "8a006a0a1e2dd36ba57aa0325b2c9532db7649a4c3c6214ee0f004ecabcf1eef89a91b225ffc52a4f811791d20f6faddd900b863386da65daec18e00c48412d6");
    }
}
