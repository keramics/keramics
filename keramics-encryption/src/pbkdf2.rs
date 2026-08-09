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

//! Password-Based Key Derivation Function 2 (PBKDF2).
//!
//! Provides PBKDF2 key derivation support.

use std::slice::ChunksExactMut;

use keramics_core::ErrorTrace;
use keramics_hashes::{
    DigestHashContext, Sha1Context, Sha224Context, Sha256Context, Sha384Context, Sha512Context,
};

use super::hmac::HmacContext;

/// Context for PBKDF2-HMAC-SHA1.
pub type Pbkdf2HmacSha1Context = Pbkdf2Context<Sha1Context, 64, 20>;

/// Context for PBKDF2-HMAC-SHA-224.
pub type Pbkdf2HmacSha224Context = Pbkdf2Context<Sha224Context, 64, 28>;

/// Context for PBKDF2-HMAC-SHA-256.
pub type Pbkdf2HmacSha256Context = Pbkdf2Context<Sha256Context, 64, 32>;

/// Context for PBKDF2-HMAC-SHA-384.
pub type Pbkdf2HmacSha384Context = Pbkdf2Context<Sha384Context, 128, 48>;

/// Context for PBKDF2-HMAC-SHA-512.
pub type Pbkdf2HmacSha512Context = Pbkdf2Context<Sha512Context, 128, 64>;

/// Context for PBKDF2 key derivation.
pub struct Pbkdf2Context<H: DigestHashContext, const BLOCK_SIZE: usize, const HASH_SIZE: usize> {
    /// HMAC context.
    hmac_context: HmacContext<H, BLOCK_SIZE, HASH_SIZE>,

    /// Salt.
    salt: Vec<u8>,

    /// Number of iterations.
    number_of_iterations: usize,
}

impl<H: DigestHashContext, const BLOCK_SIZE: usize, const HASH_SIZE: usize>
    Pbkdf2Context<H, BLOCK_SIZE, HASH_SIZE>
{
    /// Creates a new context.
    pub fn new(salt: &[u8], number_of_iterations: usize) -> Self {
        Self {
            hmac_context: HmacContext::new(),
            salt: salt.to_vec(),
            number_of_iterations,
        }
    }

    /// Derives a key from the password.
    pub fn derive_key(&mut self, password: &[u8], key: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.number_of_iterations == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - number of iterations value out of bounds"
            ));
        }
        let mut hash_buffer1: [u8; HASH_SIZE] = [0; HASH_SIZE];
        let mut hash_buffer2: [u8; HASH_SIZE] = [0; HASH_SIZE];

        let salt_size: usize = self.salt.len();
        let data_buffer_size: usize = salt_size + 4;
        let mut data_buffer: Vec<u8> = vec![0; data_buffer_size];
        data_buffer[0..salt_size].copy_from_slice(&self.salt);

        let mut block_index: u32 = 1;
        let mut chunks: ChunksExactMut<'_, u8> = key.chunks_exact_mut(HASH_SIZE);

        for chunk in &mut chunks {
            data_buffer[salt_size..data_buffer_size].copy_from_slice(&block_index.to_be_bytes());
            block_index += 1;

            match self
                .hmac_context
                .calculate_hmac(password, &data_buffer, &mut hash_buffer1)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to calculate HMAC");
                    return Err(error);
                }
            }
            chunk.copy_from_slice(&hash_buffer1);

            for _ in 1..self.number_of_iterations {
                match self
                    .hmac_context
                    .calculate_hmac(password, &hash_buffer1, &mut hash_buffer2)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to calculate HMAC");
                        return Err(error);
                    }
                }
                for (key_byte, hash_byte) in chunk.iter_mut().zip(&hash_buffer2) {
                    *key_byte ^= hash_byte;
                }
                hash_buffer1.copy_from_slice(&hash_buffer2);
            }
        }
        let remainder: &mut [u8] = chunks.into_remainder();

        if !remainder.is_empty() {
            data_buffer[salt_size..data_buffer_size].copy_from_slice(&block_index.to_be_bytes());

            match self
                .hmac_context
                .calculate_hmac(password, &data_buffer, &mut hash_buffer1)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to calculate HMAC");
                    return Err(error);
                }
            }
            remainder.copy_from_slice(&hash_buffer1[0..remainder.len()]);

            for _ in 1..self.number_of_iterations {
                match self
                    .hmac_context
                    .calculate_hmac(password, &hash_buffer1, &mut hash_buffer2)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to calculate HMAC");
                        return Err(error);
                    }
                }
                for (key_byte, hash_byte) in remainder.iter_mut().zip(&hash_buffer2) {
                    *key_byte ^= hash_byte;
                }
                hash_buffer1.copy_from_slice(&hash_buffer2);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector.
    struct TestVector {
        /// Password.
        password: &'static [u8],

        /// Salt.
        salt: &'static [u8],

        /// Number of iterations.
        number_of_iterations: usize,

        /// Key PBKDF2-HMAC-SHA1.
        key_sha1: &'static [u8],

        /// Key PBKDF2-HMAC-SHA-224.
        key_sha224: &'static [u8],

        /// Key PBKDF2-HMAC-SHA-256.
        key_sha256: &'static [u8],

        /// Key PBKDF2-HMAC-SHA-384.
        key_sha384: &'static [u8],

        /// Key PBKDF2-HMAC-SHA-512.
        key_sha512: &'static [u8],
    }

    const TEST_VECTORS: &'static [TestVector] = &[
        // RFC 6070 test vectors.
        TestVector {
            password: b"password",
            salt: b"salt",
            number_of_iterations: 1,
            key_sha1: &[
                0x0c, 0x60, 0xc8, 0x0f, 0x96, 0x1f, 0x0e, 0x71, 0xf3, 0xa9, 0xb5, 0x24, 0xaf, 0x60,
                0x12, 0x06, 0x2f, 0xe0, 0x37, 0xa6,
            ],
            key_sha224: &[
                0x3c, 0x19, 0x8c, 0xbd, 0xb9, 0x46, 0x4b, 0x78, 0x57, 0x96, 0x6b, 0xd0, 0x5b, 0x7b,
                0xc9, 0x2b, 0xc1, 0xcc, 0x4e, 0x6e,
            ],
            key_sha256: &[
                0x12, 0x0f, 0xb6, 0xcf, 0xfc, 0xf8, 0xb3, 0x2c, 0x43, 0xe7, 0x22, 0x52, 0x56, 0xc4,
                0xf8, 0x37, 0xa8, 0x65, 0x48, 0xc9,
            ],
            key_sha384: &[
                0xc0, 0xe1, 0x4f, 0x06, 0xe4, 0x9e, 0x32, 0xd7, 0x3f, 0x9f, 0x52, 0xdd, 0xf1, 0xd0,
                0xc5, 0xc7, 0x19, 0x16, 0x09, 0x23,
            ],
            key_sha512: &[
                0x86, 0x7f, 0x70, 0xcf, 0x1a, 0xde, 0x02, 0xcf, 0xf3, 0x75, 0x25, 0x99, 0xa3, 0xa5,
                0x3d, 0xc4, 0xaf, 0x34, 0xc7, 0xa6,
            ],
        },
        TestVector {
            password: b"password",
            salt: b"salt",
            number_of_iterations: 2,
            key_sha1: &[
                0xea, 0x6c, 0x01, 0x4d, 0xc7, 0x2d, 0x6f, 0x8c, 0xcd, 0x1e, 0xd9, 0x2a, 0xce, 0x1d,
                0x41, 0xf0, 0xd8, 0xde, 0x89, 0x57,
            ],
            key_sha224: &[
                0x93, 0x20, 0x0f, 0xfa, 0x96, 0xc5, 0x77, 0x6d, 0x38, 0xfa, 0x10, 0xab, 0xdf, 0x8f,
                0x5b, 0xfc, 0x00, 0x54, 0xb9, 0x71,
            ],
            key_sha256: &[
                0xae, 0x4d, 0x0c, 0x95, 0xaf, 0x6b, 0x46, 0xd3, 0x2d, 0x0a, 0xdf, 0xf9, 0x28, 0xf0,
                0x6d, 0xd0, 0x2a, 0x30, 0x3f, 0x8e,
            ],
            key_sha384: &[
                0x54, 0xf7, 0x75, 0xc6, 0xd7, 0x90, 0xf2, 0x19, 0x30, 0x45, 0x91, 0x62, 0xfc, 0x53,
                0x5d, 0xbf, 0x04, 0xa9, 0x39, 0x18,
            ],
            key_sha512: &[
                0xe1, 0xd9, 0xc1, 0x6a, 0xa6, 0x81, 0x70, 0x8a, 0x45, 0xf5, 0xc7, 0xc4, 0xe2, 0x15,
                0xce, 0xb6, 0x6e, 0x01, 0x1a, 0x2e,
            ],
        },
        TestVector {
            password: b"password",
            salt: b"salt",
            number_of_iterations: 4096,
            key_sha1: &[
                0x4b, 0x00, 0x79, 0x01, 0xb7, 0x65, 0x48, 0x9a, 0xbe, 0xad, 0x49, 0xd9, 0x26, 0xf7,
                0x21, 0xd0, 0x65, 0xa4, 0x29, 0xc1,
            ],
            key_sha224: &[
                0x21, 0x8c, 0x45, 0x3b, 0xf9, 0x06, 0x35, 0xbd, 0x0a, 0x21, 0xa7, 0x5d, 0x17, 0x27,
                0x03, 0xff, 0x61, 0x08, 0xef, 0x60,
            ],
            key_sha256: &[
                0xc5, 0xe4, 0x78, 0xd5, 0x92, 0x88, 0xc8, 0x41, 0xaa, 0x53, 0x0d, 0xb6, 0x84, 0x5c,
                0x4c, 0x8d, 0x96, 0x28, 0x93, 0xa0,
            ],
            key_sha384: &[
                0x55, 0x97, 0x26, 0xbe, 0x38, 0xdb, 0x12, 0x5b, 0xc8, 0x5e, 0xd7, 0x89, 0x5f, 0x6e,
                0x3c, 0xf5, 0x74, 0xc7, 0xa0, 0x1c,
            ],
            key_sha512: &[
                0xd1, 0x97, 0xb1, 0xb3, 0x3d, 0xb0, 0x14, 0x3e, 0x01, 0x8b, 0x12, 0xf3, 0xd1, 0xd1,
                0x47, 0x9e, 0x6c, 0xde, 0xbd, 0xcc,
            ],
        },
        /* TODO move to separate test program
                TestVector {
                    password: b"password",
                    salt: b"salt",
                    number_of_iterations: 16777216,
                    key_sha1: &[
                        0xee, 0xfe, 0x3d, 0x61, 0xcd, 0x4d, 0xa4, 0xe4, 0xe9, 0x94, 0x5b, 0x3d, 0x6b, 0xa2,
                        0x15, 0x8c, 0x26, 0x34, 0xe9, 0x84,
                    ],
                    key_sha224: &[
                        0xb4, 0x99, 0x25, 0x18, 0x4c, 0xb4, 0xb5, 0x59, 0xf3, 0x65, 0xe9, 0x4f, 0xca, 0xfc,
                        0xd4, 0xcd, 0xb9, 0xf7, 0xae, 0xf4,
                    ],
                    key_sha256: &[
                        0xcf, 0x81, 0xc6, 0x6f, 0xe8, 0xcf, 0xc0, 0x4d, 0x1f, 0x31, 0xec, 0xb6, 0x5d, 0xab,
                        0x40, 0x89, 0xf7, 0xf1, 0x79, 0xe8,
                    ],
                    key_sha384: &[
                        0xa7, 0xfd, 0xb3, 0x49, 0xba, 0x2b, 0xfa, 0x6b, 0xf6, 0x47, 0xbb, 0x01, 0x61, 0xba,
                        0xe1, 0x32, 0x0d, 0xf2, 0x7e, 0x64,
                    ],
                    key_sha512: &[
                        0x61, 0x80, 0xa3, 0xce, 0xab, 0xab, 0x45, 0xcc, 0x39, 0x64, 0x11, 0x2c, 0x81, 0x1e,
                        0x01, 0x31, 0xbc, 0xa9, 0x3a, 0x35,
                    ],
                },
        */
        TestVector {
            password: b"passwordPASSWORDpassword",
            salt: b"saltSALTsaltSALTsaltSALTsaltSALTsalt",
            number_of_iterations: 4096,
            key_sha1: &[
                0x3d, 0x2e, 0xec, 0x4f, 0xe4, 0x1c, 0x84, 0x9b, 0x80, 0xc8, 0xd8, 0x36, 0x62, 0xc0,
                0xe4, 0x4a, 0x8b, 0x29, 0x1a, 0x96, 0x4c, 0xf2, 0xf0, 0x70, 0x38,
            ],
            key_sha224: &[
                0x05, 0x6c, 0x4b, 0xa4, 0x38, 0xde, 0xd9, 0x1f, 0xc1, 0x4e, 0x05, 0x94, 0xe6, 0xf5,
                0x2b, 0x87, 0xe1, 0xf3, 0x69, 0x0c, 0x0d, 0xc0, 0xfb, 0xc0, 0x57,
            ],
            key_sha256: &[
                0x34, 0x8c, 0x89, 0xdb, 0xcb, 0xd3, 0x2b, 0x2f, 0x32, 0xd8, 0x14, 0xb8, 0x11, 0x6e,
                0x84, 0xcf, 0x2b, 0x17, 0x34, 0x7e, 0xbc, 0x18, 0x00, 0x18, 0x1c,
            ],
            key_sha384: &[
                0x81, 0x91, 0x43, 0xad, 0x66, 0xdf, 0x9a, 0x55, 0x25, 0x59, 0xb9, 0xe1, 0x31, 0xc5,
                0x2a, 0xe6, 0xc5, 0xc1, 0xb0, 0xee, 0xd1, 0x8f, 0x4d, 0x28, 0x3b,
            ],
            key_sha512: &[
                0x8c, 0x05, 0x11, 0xf4, 0xc6, 0xe5, 0x97, 0xc6, 0xac, 0x63, 0x15, 0xd8, 0xf0, 0x36,
                0x2e, 0x22, 0x5f, 0x3c, 0x50, 0x14, 0x95, 0xba, 0x23, 0xb8, 0x68,
            ],
        },
        TestVector {
            password: b"pass\0word",
            salt: b"sa\0lt",
            number_of_iterations: 4096,
            key_sha1: &[
                0x56, 0xfa, 0x6a, 0xa7, 0x55, 0x48, 0x09, 0x9d, 0xcc, 0x37, 0xd7, 0xf0, 0x34, 0x25,
                0xe0, 0xc3,
            ],
            key_sha224: &[
                0x9b, 0x40, 0x11, 0xb6, 0x41, 0xf4, 0x0a, 0x2a, 0x50, 0x0a, 0x31, 0xd4, 0xa3, 0x92,
                0xd1, 0x5c,
            ],
            key_sha256: &[
                0x89, 0xb6, 0x9d, 0x05, 0x16, 0xf8, 0x29, 0x89, 0x3c, 0x69, 0x62, 0x26, 0x65, 0x0a,
                0x86, 0x87,
            ],
            key_sha384: &[
                0xa3, 0xf0, 0x0a, 0xc8, 0x65, 0x7e, 0x09, 0x5f, 0x8e, 0x08, 0x23, 0xd2, 0x32, 0xfc,
                0x60, 0xb3,
            ],
            key_sha512: &[
                0x9d, 0x9e, 0x9c, 0x4c, 0xd2, 0x1f, 0xe4, 0xbe, 0x24, 0xd5, 0xb8, 0x24, 0x4c, 0x75,
                0x96, 0x65,
            ],
        },
    ];

    #[test]
    fn test_derive_key_sha1() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let mut pbkdf2_context: Pbkdf2HmacSha1Context =
                Pbkdf2HmacSha1Context::new(test_vector.salt, test_vector.number_of_iterations);

            let mut key: Vec<u8> = vec![0; test_vector.key_sha1.len()];
            pbkdf2_context.derive_key(test_vector.password, &mut key)?;

            assert_eq!(
                &key,
                test_vector.key_sha1,
                "key mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_derive_key_sha224() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let mut pbkdf2_context: Pbkdf2HmacSha224Context =
                Pbkdf2HmacSha224Context::new(test_vector.salt, test_vector.number_of_iterations);

            let mut key: Vec<u8> = vec![0; test_vector.key_sha224.len()];
            pbkdf2_context.derive_key(test_vector.password, &mut key)?;

            assert_eq!(
                &key,
                test_vector.key_sha224,
                "key mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_derive_key_sha256() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let mut pbkdf2_context: Pbkdf2HmacSha256Context =
                Pbkdf2HmacSha256Context::new(test_vector.salt, test_vector.number_of_iterations);

            let mut key: Vec<u8> = vec![0; test_vector.key_sha256.len()];
            pbkdf2_context.derive_key(test_vector.password, &mut key)?;

            assert_eq!(
                &key,
                test_vector.key_sha256,
                "key mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_derive_key_sha384() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let mut pbkdf2_context: Pbkdf2HmacSha384Context =
                Pbkdf2HmacSha384Context::new(test_vector.salt, test_vector.number_of_iterations);

            let mut key: Vec<u8> = vec![0; test_vector.key_sha384.len()];
            pbkdf2_context.derive_key(test_vector.password, &mut key)?;

            assert_eq!(
                &key,
                test_vector.key_sha384,
                "key mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }

    #[test]
    fn test_derive_key_sha512() -> Result<(), ErrorTrace> {
        for (test_number, test_vector) in TEST_VECTORS.iter().enumerate() {
            let mut pbkdf2_context: Pbkdf2HmacSha512Context =
                Pbkdf2HmacSha512Context::new(test_vector.salt, test_vector.number_of_iterations);

            let mut key: Vec<u8> = vec![0; test_vector.key_sha512.len()];
            pbkdf2_context.derive_key(test_vector.password, &mut key)?;

            assert_eq!(
                &key,
                test_vector.key_sha512,
                "key mismatch for test vector: {}",
                test_number + 1
            );
        }
        Ok(())
    }
}
