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

//! MD5 hash (or message-digest).
//!
//! Provides support for calculating a MD5 hash (RFC 1321).

use crate::bytes_to_u32_le;

use super::traits::DigestHashContext;

/// MD5 block size.
const MD5_BLOCK_SIZE: usize = 64;

/// MD5 initial hash values.
const MD5_HASH_VALUES: [u32; 4] = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];

/// The first 32-bits of the sines (in radians) of the integers [0, 63].
const MD5_SINES: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// MD5 bit shifts.
const MD5_BIT_SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

/// MD5 32-bit values indexes
/// [0, 15] => index
/// [16, 31] => ( ( 5 x index ) + 1 ) mod 16
/// [32, 47] => ( ( 3 x index ) + 5 ) mod 16
/// [48, 63] => ( 7 x index ) mod 16
const MD5_VALUES_32BIT_INDEX: [usize; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 1, 6, 11, 0, 5, 10, 15, 4, 9, 14, 3, 8,
    13, 2, 7, 12, 5, 8, 11, 14, 1, 4, 7, 10, 13, 0, 3, 6, 9, 12, 15, 2, 0, 7, 14, 5, 12, 3, 10, 1,
    8, 15, 6, 13, 4, 11, 2, 9,
];

/// Context for calculating a MD5 hash.
pub struct Md5Context {
    hash_values: [u32; 4],
    number_of_bytes_hashed: u64,
    block_offset: usize,
    block: [u8; MD5_BLOCK_SIZE * 2],
}

impl Md5Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            hash_values: MD5_HASH_VALUES,
            number_of_bytes_hashed: 0,
            block_offset: 0,
            block: [0; MD5_BLOCK_SIZE * 2],
        }
    }

    /// Calculates the hash of a block of data.
    #[inline(always)]
    fn transform_block(
        &self,
        hash_values: &[u32],
        data: &[u8],
        mut data_offset: usize,
    ) -> [u32; 4] {
        let mut values_32bit: [u32; 16] = [0; 16];

        // Break the block of data into 16 x 32-bit little-endian values
        for value_index in 0..16 {
            values_32bit[value_index] = bytes_to_u32_le!(data, data_offset);

            data_offset += 4;
        }
        // Calculate the hash values
        let mut block_hashes: [u32; 4] = [0; 4];
        block_hashes.copy_from_slice(hash_values);

        for value_index in 0..64 {
            let block_hash: u32 = if value_index < 16 {
                (block_hashes[1] & block_hashes[2]) | (!(block_hashes[1]) & block_hashes[3])
            } else if value_index < 32 {
                (block_hashes[1] & block_hashes[3]) | (block_hashes[2] & !(block_hashes[3]))
            } else if value_index < 48 {
                block_hashes[1] ^ block_hashes[2] ^ block_hashes[3]
            } else {
                block_hashes[2] ^ (block_hashes[1] | !(block_hashes[3]))
            }
            .wrapping_add(block_hashes[0])
            .wrapping_add(values_32bit[MD5_VALUES_32BIT_INDEX[value_index]])
            .wrapping_add(MD5_SINES[value_index])
            .rotate_left(MD5_BIT_SHIFTS[value_index]);

            block_hashes[0] = block_hashes[3];
            block_hashes[3] = block_hashes[2];
            block_hashes[2] = block_hashes[1];
            block_hashes[1] = block_hashes[1].wrapping_add(block_hash);
        }
        // Update the hash values
        for value_index in 0..4 {
            block_hashes[value_index] =
                block_hashes[value_index].wrapping_add(hash_values[value_index]);
        }
        block_hashes
    }
}

impl DigestHashContext for Md5Context {
    /// Finalizes the digest hash calculation.
    fn finalize(&mut self) -> Vec<u8> {
        let bit_size: u64 = (self.number_of_bytes_hashed + self.block_offset as u64) * 8;

        // Add padding with a size of 56 mod 64
        let padding_size: usize = MD5_BLOCK_SIZE * if self.block_offset >= 56 { 2 } else { 1 };

        let bit_size_block_offset: usize = padding_size - 8;

        // The first byte of the padding contains 0x80
        self.block[self.block_offset] = 0x80;
        self.block[self.block_offset + 1..bit_size_block_offset].fill(0);
        self.block[bit_size_block_offset..padding_size].copy_from_slice(&bit_size.to_le_bytes());

        for block_offset in (0..padding_size).step_by(MD5_BLOCK_SIZE) {
            let hash_values: [u32; 4] =
                self.transform_block(&self.hash_values, &self.block, block_offset);

            self.hash_values.copy_from_slice(&hash_values);
        }
        let hash: Vec<u8> = self
            .hash_values
            .iter()
            .map(|hash_value| hash_value.to_le_bytes())
            .flatten()
            .collect::<Vec<u8>>();

        self.hash_values = MD5_HASH_VALUES;
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
            while self.block_offset < MD5_BLOCK_SIZE {
                if data_offset >= data_size {
                    break;
                }
                self.block[self.block_offset] = data[data_offset];
                self.block_offset += 1;

                data_offset += 1;
            }
            if self.block_offset == MD5_BLOCK_SIZE {
                let hash_values: [u32; 4] = self.transform_block(&self.hash_values, &self.block, 0);

                self.hash_values.copy_from_slice(&hash_values);
                self.number_of_bytes_hashed += MD5_BLOCK_SIZE as u64;

                self.block_offset = 0;
            }
        }
        while data_offset + MD5_BLOCK_SIZE < data_size {
            let hash_values: [u32; 4] = self.transform_block(&self.hash_values, &data, data_offset);

            self.hash_values.copy_from_slice(&hash_values);
            self.number_of_bytes_hashed += MD5_BLOCK_SIZE as u64;

            data_offset += MD5_BLOCK_SIZE;
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

        let mut test_context: Md5Context = Md5Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "d41d8cd98f00b204e9800998ecf8427e");
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

        let mut test_context: Md5Context = Md5Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "4933898605b4a1b970b674a2dde92292");
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

        let mut test_context: Md5Context = Md5Context::new();
        test_context.update(&test_data);
        let test_hash: Vec<u8> = test_context.finalize();

        let test_hash_string: String = format_as_string(&test_hash);
        assert_eq!(test_hash_string, "fa354daecc45f14b82b0e7e567d24282");
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

        let mut test_context: Md5Context = Md5Context::new();

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
        assert_eq!(test_hash_string, "fa354daecc45f14b82b0e7e567d24282");
    }
}
