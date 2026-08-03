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

//! Adler-32 checksum.
//!
//! Provides support for calculating Adler-32 checksums.

/// Context for calculating an Adler-32 checksum.
pub struct Adler32Context {
    /// The initial checksum value.
    initial_value: u32,

    /// The checksum value.
    checksum: u32,
}

impl Adler32Context {
    /// Creates a new context.
    pub fn new(initial_value: u32) -> Self {
        Self {
            initial_value,
            checksum: initial_value,
        }
    }

    /// Finalizes the checksum calculation.
    pub fn finalize(&mut self) -> u32 {
        let checksum: u32 = self.checksum;

        self.checksum = self.initial_value;

        checksum
    }

    /// Optimized modulus 65521 (0xfff1) calculation.
    #[inline(always)]
    fn mod_65521(mut value: u32) -> u32 {
        let value_32bit: u32 = value >> 16;
        value &= 0x0000ffff;
        value += (value_32bit << 4) - value_32bit;

        if value > 65521 {
            let value_32bit: u32 = value >> 16;
            value &= 0x0000ffff;
            value += (value_32bit << 4) - value_32bit;
        }
        if value >= 65521 {
            value -= 65521;
        }
        value
    }

    /// Calculates the checksum of the data.
    pub fn update(&mut self, data: &[u8]) {
        let mut lower_16bit: u32 = self.checksum & 0x0000ffff;
        let mut upper_16bit: u32 = self.checksum >> 16;

        let data_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut data_end_offset: usize = 5552;
        while data_end_offset < data_size {
            for byte_value in data[data_offset..data_end_offset].iter() {
                lower_16bit = lower_16bit.wrapping_add(*byte_value as u32);
                upper_16bit = upper_16bit.wrapping_add(lower_16bit);
            }
            data_offset = data_end_offset;
            data_end_offset += 5552;

            // The modulo calculation is needed per 5552 (0x15b0) bytes
            lower_16bit = Self::mod_65521(lower_16bit);
            upper_16bit = Self::mod_65521(upper_16bit);
        }
        if data_offset < data_size {
            for byte_value in data[data_offset..data_size].iter() {
                lower_16bit = lower_16bit.wrapping_add(*byte_value as u32);
                upper_16bit = upper_16bit.wrapping_add(lower_16bit);
            }
            lower_16bit = Self::mod_65521(lower_16bit);
            upper_16bit = Self::mod_65521(upper_16bit);
        }
        self.checksum = (upper_16bit << 16) | lower_16bit;
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
    fn test_mod_65521() {
        assert_eq!(Adler32Context::mod_65521(1), 1);
        assert_eq!(Adler32Context::mod_65521(65522), 1);
        assert_eq!(Adler32Context::mod_65521(0xffffff20), 1);
    }

    #[test]
    fn test_update_and_finalize() {
        let mut test_context: Adler32Context = Adler32Context::new(1);

        let test_data: Vec<u8> = get_test_data(5632);
        test_context.update(&test_data);

        let test_checksum: u32 = test_context.finalize();
        assert_eq!(test_checksum, 0x3222f597);
    }
}
