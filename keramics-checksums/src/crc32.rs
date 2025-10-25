/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

//! CRC-32 checksum.
//!
//! Provides support for calculating 32-bit Cyclic Redundancy Check (CRC-32) checksums.

/// Context for calculating a CRC-32 checksum.
pub struct Crc32Context {
    /// Polynomial.
    polynomial: u32,

    /// The initial checksum value.
    initial_value: u32,

    /// The checksum value.
    checksum: u32,

    /// Lookup table.
    table: [u32; 256],

    /// Value to indicate the lookup table has been initialized.
    table_initilized: bool,
}

impl Crc32Context {
    /// Creates a new context.
    pub fn new(polynomial: u32, initial_value: u32) -> Self {
        Self {
            polynomial: polynomial,
            initial_value: initial_value,
            checksum: initial_value ^ 0xffffffff,
            table: [0; 256],
            table_initilized: false,
        }
    }

    /// Initializes the lookup table.
    fn initialize_table(&mut self, polynomial: u32) {
        for table_index in 0..256 {
            let mut checksum: u32 = (table_index as u32) << 24;

            for _ in 0..8 {
                if checksum & 0x80000000 != 0 {
                    checksum = polynomial ^ (checksum << 1);
                } else {
                    checksum <<= 1;
                }
            }
            self.table[table_index] = checksum;
        }
        self.table_initilized = true
    }

    /// Finalizes the checksum calculation.
    pub fn finalize(&mut self) -> u32 {
        let checksum: u32 = self.checksum ^ 0xffffffff;

        self.checksum = self.initial_value ^ 0xffffffff;

        checksum
    }

    /// Calculates the checksum of the data.
    pub fn update(&mut self, data: &[u8]) {
        if !self.table_initilized {
            self.initialize_table(self.polynomial);
        }
        let data_size: usize = data.len();
        let mut checksum: u32 = self.checksum;

        for data_offset in 0..data_size {
            let table_index: u32 = ((checksum >> 24) ^ data[data_offset] as u32) & 0x000000ff;

            checksum = self.table[table_index as usize] ^ (checksum << 8);
        }
        self.checksum = checksum
    }
}

/// Context for calculating a reversed CRC-32 checksum.
pub struct ReversedCrc32Context {
    /// Polynomial.
    polynomial: u32,

    /// The initial checksum value.
    initial_value: u32,

    /// The checksum value.
    checksum: u32,

    /// Lookup table.
    table: [u32; 256],

    /// Value to indicate the lookup table has been initialized.
    table_initilized: bool,
}

impl ReversedCrc32Context {
    /// Creates a new context.
    pub fn new(polynomial: u32, initial_value: u32) -> Self {
        Self {
            polynomial: polynomial,
            initial_value: initial_value,
            checksum: initial_value ^ 0xffffffff,
            table: [0; 256],
            table_initilized: false,
        }
    }

    /// Initializes the lookup table.
    fn initialize_table(&mut self, polynomial: u32) {
        for table_index in 0..256 {
            let mut checksum: u32 = table_index as u32;

            for _ in 0..8 {
                if checksum & 1 != 0 {
                    checksum = polynomial ^ (checksum >> 1);
                } else {
                    checksum >>= 1;
                }
            }
            self.table[table_index] = checksum;
        }
        self.table_initilized = true
    }

    /// Finalizes the checksum calculation.
    pub fn finalize(&mut self) -> u32 {
        let checksum: u32 = self.checksum ^ 0xffffffff;

        self.checksum = self.initial_value ^ 0xffffffff;

        checksum
    }

    /// Calculates the checksum of the data.
    pub fn update(&mut self, data: &[u8]) {
        if !self.table_initilized {
            self.initialize_table(self.polynomial);
        }
        let data_size: usize = data.len();
        let mut checksum: u32 = self.checksum;

        for data_offset in 0..data_size {
            let table_index: u32 = (checksum ^ data[data_offset] as u32) & 0x000000ff;

            checksum = self.table[table_index as usize] ^ (checksum >> 8);
        }
        self.checksum = checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
            0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x60,
            0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60,
            0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70, 0x70,
            0x70, 0x70, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x80, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
            0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
            0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0,
            0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xb0, 0xc0, 0xc1, 0xc2, 0xc3,
            0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1,
            0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
            0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed,
            0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb,
            0xfc, 0xfd, 0xfe, 0xff,
        ];
    }

    #[test]
    fn test_update_and_finalize_with_context() {
        let mut test_context: Crc32Context = Crc32Context::new(0x04c11db7, 0);

        let test_data: Vec<u8> = get_test_data();
        test_context.update(&test_data);

        let test_checksum: u32 = test_context.finalize();
        assert_eq!(test_checksum, 0x33db01c5);
    }

    #[test]
    fn test_update_and_finalize_with_reversed_context() {
        let mut test_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

        let test_data: Vec<u8> = get_test_data();
        test_context.update(&test_data);

        let test_checksum: u32 = test_context.finalize();
        assert_eq!(test_checksum, 0x5737ac31);
    }
}
