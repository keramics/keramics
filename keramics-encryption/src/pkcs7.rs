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

//! Public-Key Cryptography Standard (PKCS) 7
//!
//! Provides PKCS7 support (RFC 2315).

use keramics_core::ErrorTrace;

/// Context for PKCS7.
pub struct Pkcs7Context {}

impl Pkcs7Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {}
    }

    /// Adds padding.
    pub fn add_padding(&self, block_size: usize, data: &mut Vec<u8>) -> Result<(), ErrorTrace> {
        if block_size == 0 || block_size > 255 {
            return Err(keramics_core::error_trace_new!(
                "Invalid block size value out of bounds"
            ));
        }
        let data_size: usize = data.len();
        let padding_size: usize = block_size - (data_size % block_size);

        data.resize(data_size + padding_size, padding_size as u8);

        Ok(())
    }

    /// Removes padding.
    pub fn remove_padding<'a>(
        &self,
        block_size: usize,
        padded_data: &'a [u8],
    ) -> Result<&'a [u8], ErrorTrace> {
        if block_size == 0 || block_size > 255 {
            return Err(keramics_core::error_trace_new!(
                "Invalid block size value out of bounds"
            ));
        }
        let padded_data_size: usize = padded_data.len();

        if padded_data_size < block_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid padded data size value too small"
            ));
        }
        if padded_data_size % block_size != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid padded data size value not a multitude of block size: {}",
                block_size
            )));
        }
        let padding_size: usize = match padded_data.last() {
            Some(byte_value) => *byte_value as usize,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve padding size"
                ));
            }
        };
        if padding_size == 0 || padding_size > block_size {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid padding size value out of bounds: {}",
                padding_size
            )));
        }
        let padding_offset: usize = padded_data_size - padding_size;
        let padding_value: u8 = padding_size as u8;

        for byte_value in &padded_data[padding_offset..] {
            if *byte_value != padding_value {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid padding value at offset: {}",
                    padding_offset
                )));
            }
        }
        Ok(&padded_data[0..padding_offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector.
    struct TestVector {
        /// Block size.
        block_size: usize,

        /// Data.
        data: &'static [u8],

        /// Padded data.
        padded_data: &'static [u8],
    }

    const TEST_VECTORS: &'static [TestVector] = &[
        TestVector {
            block_size: 16,
            data: &[
                0x59, 0x45, 0x4c, 0x4c, 0x4f, 0x57, 0x20, 0x53, 0x55, 0x42, 0x4d, 0x41, 0x52, 0x49,
                0x4e, 0x45,
            ],
            padded_data: &[
                0x59, 0x45, 0x4c, 0x4c, 0x4f, 0x57, 0x20, 0x53, 0x55, 0x42, 0x4d, 0x41, 0x52, 0x49,
                0x4e, 0x45, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
                0x10, 0x10, 0x10, 0x10,
            ],
        },
        TestVector {
            block_size: 16,
            data: &[
                0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d,
            ],
            padded_data: &[
                0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x03,
                0x03, 0x03,
            ],
        },
        TestVector {
            block_size: 16,
            data: &[
                0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44,
                0x45,
            ],
            padded_data: &[
                0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x41, 0x42, 0x43, 0x44,
                0x45, 0x01,
            ],
        },
        TestVector {
            block_size: 8,
            data: &[0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67],
            padded_data: &[0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67, 0x01],
        },
        TestVector {
            block_size: 8,
            data: &[],
            padded_data: &[0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08],
        },
    ];

    #[test]
    fn test_add_padding() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

            let mut data: Vec<u8> = test_vector.data.to_vec();
            pkcs7_context.add_padding(test_vector.block_size, &mut data)?;

            assert_eq!(
                &data,
                &test_vector.padded_data,
                "PKCS7 padded data mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_remove_padding() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

            let data: &[u8] =
                pkcs7_context.remove_padding(test_vector.block_size, &test_vector.padded_data)?;

            assert_eq!(
                &data,
                &test_vector.data,
                "PKCS7 data mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_remove_padding_with_invalid_padding() {
        let test_data: Vec<u8> = vec![0x41, 0x42, 0x43, 0x00];

        let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

        let result: Result<&[u8], ErrorTrace> = pkcs7_context.remove_padding(4, &test_data);
        assert!(result.is_err());

        let test_data: Vec<u8> = vec![0x41, 0x42, 0x43, 0x05, 0x05, 0x05, 0x05];

        let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

        let result: Result<&[u8], ErrorTrace> = pkcs7_context.remove_padding(4, &test_data);
        assert!(result.is_err());

        let test_data: Vec<u8> = vec![0x41, 0x42, 0x43, 0x11];

        let pkcs7_context: Pkcs7Context = Pkcs7Context::new();

        let result: Result<&[u8], ErrorTrace> = pkcs7_context.remove_padding(4, &test_data);
        assert!(result.is_err());
    }
}
