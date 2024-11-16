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

//! Adler-32 checksum.
//!
//! Provides support for calculating Adler-32 checksums.

/// Context for calculating an Adler-32 checksum.
pub struct Adler32Context {
    initial_value: u32,
    checksum: u32,
}

impl Adler32Context {
    /// Creates a new context.
    pub fn new(initial_value: u32) -> Self {
        Self {
            initial_value: initial_value,
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
    fn mod_65521(&self, mut value: u32) -> u32 {
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
        let data_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut lower_word: u32 = self.checksum & 0x0000ffff;
        let mut upper_word: u32 = self.checksum >> 16;

        while data_offset + 5552 < data_size {
            // The modulo calculation is needed per 5552 (0x15b0) bytes
            for _ in 0..5552 {
                lower_word = lower_word.wrapping_add(data[data_offset] as u32);
                upper_word = upper_word.wrapping_add(lower_word);

                data_offset += 1;
            }
            lower_word = self.mod_65521(lower_word);
            upper_word = self.mod_65521(upper_word);
        }
        if data_offset < data_size {
            while data_offset < data_size {
                lower_word = lower_word.wrapping_add(data[data_offset] as u32);
                upper_word = upper_word.wrapping_add(lower_word);

                data_offset += 1;
            }
            lower_word = self.mod_65521(lower_word);
            upper_word = self.mod_65521(upper_word);
        }
        self.checksum = (upper_word << 16) | lower_word;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_and_finalize() {
        let test_data: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let mut test_context: Adler32Context = Adler32Context::new(1);
        test_context.update(&test_data);
        let test_checksum: u32 = test_context.finalize();

        assert_eq!(test_checksum, 0x02b80079);
    }
}
