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

    fn get_test_data(data_size: usize) -> Vec<u8> {
        (0..data_size)
            .map(|value| (value % 256) as u8)
            .collect::<Vec<u8>>()
    }

    #[test]
    fn test_update_and_finalize_with_context() {
        let mut test_context: Crc32Context = Crc32Context::new(0x04c11db7, 0);

        let test_data: Vec<u8> = get_test_data(256);
        test_context.update(&test_data);

        let test_checksum: u32 = test_context.finalize();
        assert_eq!(test_checksum, 0xb6b5ee95);
    }

    #[test]
    fn test_update_and_finalize_with_reversed_context() {
        let mut test_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

        let test_data: Vec<u8> = get_test_data(256);
        test_context.update(&test_data);

        let test_checksum: u32 = test_context.finalize();
        assert_eq!(test_checksum, 0x29058c73);
    }
}
