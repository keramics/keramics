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

//! Rivest Cipher 4 (RC4) encryption.
//!
//! Provides RC4 encryption and decryption support.

use keramics_core::ErrorTrace;

/// Context for RC4 encryption.
pub struct Rc4Context {
    /// Permutation values.
    permutation_values: Vec<u8>,

    /// Permutation indexes.
    permutation_indexes: [u8; 2],
}

impl Rc4Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            permutation_values: Vec::new(),
            permutation_indexes: [0; 2],
        }
    }

    /// Encrypts or decrypts data.
    pub fn crypt(&mut self, input_data: &[u8], output_data: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.permutation_values.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        if input_data.len() > output_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid output data value too small"
            ));
        }
        let mut data_offset: usize = 0;
        let mut permutation_index1: u8 = self.permutation_indexes[0];
        let mut permutation_index2: u8 = self.permutation_indexes[1];

        for byte_value in input_data.iter() {
            permutation_index1 = permutation_index1.wrapping_add(1);
            let permutation_value1: u8 = self.permutation_values[permutation_index1 as usize];

            permutation_index2 = permutation_index2.wrapping_add(permutation_value1);
            let permutation_value2: u8 = self.permutation_values[permutation_index2 as usize];

            self.permutation_values[permutation_index1 as usize] = permutation_value2;
            self.permutation_values[permutation_index2 as usize] = permutation_value1;

            let permutation_index: u8 = permutation_value1.wrapping_add(permutation_value2);
            let permutation_value: u8 = self.permutation_values[permutation_index as usize];

            output_data[data_offset] = *byte_value ^ permutation_value;
            data_offset += 1;
        }
        self.permutation_indexes[0] = permutation_index1;
        self.permutation_indexes[1] = permutation_index2;

        Ok(())
    }

    /// Sets the key.
    pub fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        let key_size: usize = key.len();

        if key_size < 1 || key_size > 256 {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        let mut permutation_index: u8 = 0;

        self.permutation_values = (0..=255).collect();

        for byte_value in 0..256 {
            let key_index: usize = byte_value % key_size;
            let permutation_value: u8 = self.permutation_values[byte_value];

            permutation_index = permutation_index.wrapping_add(permutation_value);
            permutation_index = permutation_index.wrapping_add(key[key_index]);

            self.permutation_values[byte_value] =
                self.permutation_values[permutation_index as usize];
            self.permutation_values[permutation_index as usize] = permutation_value;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypt() -> Result<(), ErrorTrace> {
        let mut rc4_context: Rc4Context = Rc4Context::new();

        rc4_context.set_key(b"test1")?;

        let mut data: Vec<u8> = vec![0; 59];
        rc4_context.crypt(
            b"012345678ABCDEFGHIJKLMNOPQRSTUVWXYabcdefghijklmnopqrstuvwxy",
            &mut data,
        )?;

        let expected_data: [u8; 59] = [
            0xde, 0x05, 0xfc, 0x23, 0xcf, 0xc6, 0x1a, 0x46, 0x70, 0x7c, 0x4c, 0xc3, 0x5b, 0xfb,
            0x2c, 0xdb, 0x1d, 0xfc, 0xb6, 0xd2, 0x0d, 0x71, 0xf2, 0x2c, 0xf6, 0xfd, 0xad, 0x12,
            0xd0, 0x8d, 0xa6, 0x74, 0xd5, 0x6a, 0x75, 0x33, 0x7d, 0xff, 0xed, 0xd2, 0xd6, 0x3b,
            0x17, 0x16, 0x5a, 0x78, 0xb8, 0x05, 0x5b, 0xf4, 0xbc, 0xe4, 0x61, 0x44, 0x2e, 0xaf,
            0x23, 0xa7, 0xab,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }
}
