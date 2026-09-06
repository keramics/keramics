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
use keramics_hashes::{DigestHashContext, Sha256Context};
use keramics_types::{ByteString, Ucs2String};

/// BitLocker Drive Encryption (BDE) password.
pub struct BdePassword {}

impl BdePassword {
    /// Calculates a password hash.
    pub fn calculate_hash(password: &[u8]) -> Result<Vec<u8>, ErrorTrace> {
        let byte_string: ByteString = ByteString::from(password);
        let ucs2_string: Ucs2String = match Ucs2String::from_byte_string(&byte_string) {
            Ok(ucs2_string) => ucs2_string,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable convert byte string to UCS-2 string",
                );
                return Err(error);
            }
        };
        let ucs2_bytes: Vec<u8> = ucs2_string
            .elements
            .iter()
            .flat_map(|element| element.to_le_bytes())
            .collect();

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&ucs2_bytes);
        let password_hash: Vec<u8> = sha256_context.finalize();

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&password_hash);
        Ok(sha256_context.finalize())
    }

    /// Calculates a password key.
    pub fn calculate_key(salt: &[u8], password_hash: &[u8]) -> Result<Vec<u8>, ErrorTrace> {
        if salt.len() != 16 {
            return Err(keramics_core::error_trace_new!("Unsupported salt size"));
        }
        if password_hash.len() != 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported password hash size"
            ));
        }
        let mut block_data: [u8; 88] = [0; 88];
        block_data[32..64].copy_from_slice(password_hash);
        block_data[64..80].copy_from_slice(salt);
        let mut iteration: u64 = 0;

        while iteration < 1048575 {
            block_data[80..88].copy_from_slice(&iteration.to_le_bytes());
            iteration += 1;

            let mut sha256_context: Sha256Context = Sha256Context::new();
            sha256_context.update(&block_data);
            let block_hash: Vec<u8> = sha256_context.finalize();

            block_data[0..32].copy_from_slice(&block_hash);
        }
        block_data[80..88].copy_from_slice(&iteration.to_le_bytes());

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&block_data);
        Ok(sha256_context.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() -> Result<(), ErrorTrace> {
        let password_hash: Vec<u8> = BdePassword::calculate_hash(b"KeRaMiCs")?;

        let expected_password_hash: [u8; 32] = [
            0x48, 0xc6, 0x06, 0xdd, 0x6d, 0x33, 0xce, 0x6f, 0x72, 0x81, 0x6b, 0x2f, 0x97, 0xbc,
            0x1c, 0xfc, 0x75, 0x0d, 0xa6, 0x44, 0x25, 0x14, 0x87, 0xa9, 0x16, 0x73, 0xc0, 0x57,
            0xe5, 0x8d, 0x76, 0x86,
        ];
        assert_eq!(&password_hash, &expected_password_hash);

        Ok(())
    }

    #[test]
    fn test_calculate_key() -> Result<(), ErrorTrace> {
        let salt: [u8; 16] = [
            0xfe, 0xdc, 0xfa, 0xa5, 0xe2, 0x6e, 0xe3, 0x88, 0x0d, 0x2b, 0xdb, 0x2e, 0xe4, 0xe4,
            0x42, 0x8f,
        ];
        let password_hash: [u8; 32] = [
            0x48, 0xc6, 0x06, 0xdd, 0x6d, 0x33, 0xce, 0x6f, 0x72, 0x81, 0x6b, 0x2f, 0x97, 0xbc,
            0x1c, 0xfc, 0x75, 0x0d, 0xa6, 0x44, 0x25, 0x14, 0x87, 0xa9, 0x16, 0x73, 0xc0, 0x57,
            0xe5, 0x8d, 0x76, 0x86,
        ];
        let key: Vec<u8> = BdePassword::calculate_key(&salt, &password_hash)?;

        let expected_key: [u8; 32] = [
            0x6e, 0x94, 0x2f, 0xed, 0x18, 0xaf, 0x4f, 0x09, 0xe1, 0xcf, 0x5d, 0x01, 0xb0, 0xb1,
            0x4a, 0xb9, 0xcd, 0x39, 0x29, 0x59, 0x49, 0xaa, 0x9b, 0x82, 0xbb, 0x90, 0xcf, 0x31,
            0x0e, 0x07, 0x75, 0x5b,
        ];
        assert_eq!(&key, &expected_key);

        Ok(())
    }
}
