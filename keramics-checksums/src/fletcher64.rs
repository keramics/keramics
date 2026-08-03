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

//! Fletcher-64 checksum.
//!
//! Provides support for calculating Fletcher-64 checksums.

use std::slice::ChunksExact;

/// Context for calculating an Fletcher-64 checksum.
pub struct Fletcher64Context {
    /// The initial checksum value.
    initial_value: u64,

    /// The checksum value.
    checksum: u64,
}

impl Fletcher64Context {
    /// Creates a new context.
    pub fn new(initial_value: u64) -> Self {
        Self {
            initial_value,
            checksum: initial_value,
        }
    }

    /// Finalizes the checksum calculation.
    pub fn finalize(&mut self) -> u64 {
        let checksum: u64 = self.checksum;

        self.checksum = self.initial_value;

        checksum
    }

    /// Calculates the checksum of the data.
    /// Expects data to be a multitude of 4 bytes.
    pub fn update(&mut self, data: &[u8]) {
        let mut lower_32bit: u64 = self.checksum & 0xffffffff;
        let mut upper_32bit: u64 = self.checksum >> 32;

        let mut chunks: ChunksExact<'_, u8> = data.chunks_exact(4);

        for chunk in &mut chunks {
            let value_32bit: u64 = u32::from_le_bytes(chunk.try_into().unwrap()) as u64;

            lower_32bit = (lower_32bit + value_32bit) % 0xffffffff;
            upper_32bit = (upper_32bit + lower_32bit) % 0xffffffff;
        }
        let remainder: &[u8] = chunks.remainder();

        if !remainder.is_empty() {
            let mut chunk: [u8; 4] = [0; 4];
            chunk[0..remainder.len()].copy_from_slice(remainder);

            let value_32bit: u64 = u32::from_le_bytes(chunk.try_into().unwrap()) as u64;

            lower_32bit = (lower_32bit + value_32bit) % 0xffffffff;
            upper_32bit = (upper_32bit + lower_32bit) % 0xffffffff;
        }
        self.checksum = (upper_32bit << 32) | lower_32bit;
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
    fn test_update_and_finalize() {
        let mut test_context: Fletcher64Context = Fletcher64Context::new(0);

        let test_data: Vec<u8> = get_test_data(256);
        test_context.update(&test_data);

        let test_checksum: u64 = test_context.finalize();
        assert_eq!(test_checksum, 0x9d754d45601fdfa0);
    }
}
