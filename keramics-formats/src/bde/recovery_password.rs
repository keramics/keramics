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

/// BitLocker Drive Encryption (BDE) recovery password.
pub struct BdeRecoveryPassword {}

impl BdeRecoveryPassword {
    /// Calculates a recovery password hash.
    pub fn calculate_hash(password: &[u8]) -> Result<Vec<u8>, ErrorTrace> {
        let password_string: &str = match str::from_utf8(password) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to insert block range into block tree",
                    error
                ));
            }
        };
        let mut segment_index: usize = 0;
        let mut segments: [u16; 8] = [0; 8];
        let mut is_valid: bool = true;

        for segment in password_string.split("-") {
            if segment_index >= 8 {
                is_valid = false;
                break;
            }
            match u32::from_str_radix(segment, 10) {
                Ok(mut integer) => {
                    if integer % 11 != 0 {
                        is_valid = false;
                        break;
                    }
                    integer /= 11;

                    if integer > 0xffff {
                        is_valid = false;
                        break;
                    }
                    segments[segment_index] = integer as u16;
                    segment_index += 1;
                }
                Err(_) => {
                    is_valid = false;
                    break;
                }
            }
        }
        if !is_valid || segment_index != 8 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported recovery password"
            ));
        }
        let password_data: Vec<u8> = segments
            .into_iter()
            .flat_map(|integer| integer.to_le_bytes())
            .collect();

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&password_data);
        Ok(sha256_context.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() -> Result<(), ErrorTrace> {
        let password_hash: Vec<u8> = BdeRecoveryPassword::calculate_hash(
            b"471207-278498-422125-177177-561902-537405-468006-693451",
        )?;

        let expected_password_hash: [u8; 32] = [
            0xa4, 0xc9, 0x24, 0x44, 0x78, 0x11, 0x08, 0x05, 0x01, 0x5c, 0x94, 0xd1, 0xb0, 0x01,
            0xa9, 0x33, 0xad, 0xa2, 0xab, 0x0b, 0x71, 0x6b, 0xac, 0x34, 0x2b, 0xb8, 0x6c, 0x84,
            0x4b, 0x63, 0x49, 0x1f,
        ];
        assert_eq!(&password_hash, &expected_password_hash);

        Ok(())
    }
}
