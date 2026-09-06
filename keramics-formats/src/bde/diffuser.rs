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

use keramics_core::ErrorTrace;

/// BitLocker Drive Encryption (BDE) diffuser.
pub struct BdeDiffuser {}

impl BdeDiffuser {
    /// Decrypts the data using Diffuser-A and B.
    pub fn decrypt(data: &mut [u8]) -> Result<(), ErrorTrace> {
        let data_size = data.len();

        if data_size < 32 || (data_size % 4) != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let mut values_32bit: Vec<u32> = data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        Self::decrypt_b(&mut values_32bit);
        Self::decrypt_a(&mut values_32bit);

        data.chunks_exact_mut(4)
            .zip(values_32bit.iter())
            .for_each(|(chunk, &value_32bit)| chunk.copy_from_slice(&value_32bit.to_le_bytes()));

        Ok(())
    }

    /// Decrypts the data using Diffuser-A.
    #[inline(always)]
    fn decrypt_a(values_32bit: &mut [u32]) {
        let number_of_values: usize = values_32bit.len();

        for _ in 0..5 {
            let mut index1: usize = 0;
            let mut index2: usize = number_of_values - 2;
            let mut index3: usize = number_of_values - 5;

            while index1 < (number_of_values - 1) {
                values_32bit[index1] = values_32bit[index1]
                    .wrapping_add(values_32bit[index2] ^ values_32bit[index3].rotate_left(9));

                index1 += 1;
                index2 += 1;
                index3 += 1;

                if index3 >= number_of_values {
                    index3 -= number_of_values;
                }
                values_32bit[index1] =
                    values_32bit[index1].wrapping_add(values_32bit[index2] ^ values_32bit[index3]);

                index1 += 1;
                index2 += 1;
                index3 += 1;

                if index2 >= number_of_values {
                    index2 -= number_of_values;
                }
                values_32bit[index1] = values_32bit[index1]
                    .wrapping_add(values_32bit[index2] ^ values_32bit[index3].rotate_left(13));

                index1 += 1;
                index2 += 1;
                index3 += 1;

                values_32bit[index1] =
                    values_32bit[index1].wrapping_add(values_32bit[index2] ^ values_32bit[index3]);

                index1 += 1;
                index2 += 1;
                index3 += 1;
            }
        }
    }

    /// Decrypts the data using Diffuser-B.
    #[inline(always)]
    fn decrypt_b(values_32bit: &mut [u32]) {
        let number_of_values: usize = values_32bit.len();

        for _ in 0..3 {
            let mut index1: usize = 0;
            let mut index2: usize = 2;
            let mut index3: usize = 5;

            while index1 < (number_of_values - 1) {
                values_32bit[index1] =
                    values_32bit[index1].wrapping_add(values_32bit[index2] ^ values_32bit[index3]);

                index1 += 1;
                index2 += 1;
                index3 += 1;

                values_32bit[index1] = values_32bit[index1]
                    .wrapping_add(values_32bit[index2] ^ values_32bit[index3].rotate_left(10));

                index1 += 1;
                index2 += 1;
                index3 += 1;

                if index2 >= number_of_values {
                    index2 -= number_of_values;
                }
                values_32bit[index1] =
                    values_32bit[index1].wrapping_add(values_32bit[index2] ^ values_32bit[index3]);

                index1 += 1;
                index2 += 1;
                index3 += 1;

                if index3 >= number_of_values {
                    index3 -= number_of_values;
                }
                values_32bit[index1] = values_32bit[index1]
                    .wrapping_add(values_32bit[index2] ^ values_32bit[index3].rotate_left(25));

                index1 += 1;
                index2 += 1;
                index3 += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt() -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        BdeDiffuser::decrypt(&mut data)?;

        let expected_data: [u8; 32] = [
            0x34, 0x96, 0x3f, 0x17, 0x74, 0x30, 0x4f, 0xfe, 0xa4, 0x9e, 0x27, 0x2b, 0xcc, 0xef,
            0xf8, 0xd0, 0xb5, 0x71, 0xec, 0x7b, 0x95, 0x43, 0x72, 0xb6, 0xdd, 0x1b, 0x39, 0x74,
            0xa8, 0xb5, 0x80, 0xe2,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }
}
