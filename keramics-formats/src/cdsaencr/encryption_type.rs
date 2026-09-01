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

use std::fmt;

/// Mac OS Encrypted Encoding (cdsaencr) encryption type.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsaEncrEncryptionType {
    /// Method.
    pub(crate) method: u32,

    /// Mode.
    pub(crate) mode: u32,

    /// Key size.
    pub(crate) key_size: usize,
}

impl CdsaEncrEncryptionType {
    /// Creates a new encryption type.
    pub(crate) fn new() -> Self {
        Self {
            method: 0,
            mode: 0,
            key_size: 0,
        }
    }
}

impl fmt::Display for CdsaEncrEncryptionType {
    /// Formats encryption type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.method == 0 {
            write!(formatter, "N/A (not set)")
        } else {
            let method_string: String = match &self.method {
                0x00000011 => String::from("DES3"),
                0x80000001 => String::from("AES"),
                _ => format!("0x{:08x}", self.method),
            };
            let mode_string: String = match &self.mode {
                2 | 3 => String::from("ECB"),
                4 | 5 | 6 => String::from("CBC"),
                7 | 8 | 9 => String::from("CFB"),
                10 | 11 | 12 => String::from("OFB"),
                _ => format!("0x{:08x}", self.method),
            };
            let suffix: Option<String> = match &self.mode {
                5 | 8 | 11 => Some(String::from("IV8")),
                6 | 9 | 12 => Some(String::from("PadIV8")),
                _ => None,
            };
            match suffix {
                Some(suffix) => write!(
                    formatter,
                    "{}-{}-{}-{}",
                    method_string,
                    self.key_size * 8,
                    mode_string,
                    suffix
                ),
                None => write!(
                    formatter,
                    "{}-{}-{}",
                    method_string,
                    self.key_size * 8,
                    mode_string
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_not_set() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType::new();

        assert_eq!(encryption_type.to_string(), "N/A (not set)");
    }

    #[test]
    fn test_display_with_des3_cbc_iv8() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x00000011,
            mode: 5,
            key_size: 24,
        };
        assert_eq!(encryption_type.to_string(), "DES3-192-CBC-IV8");
    }

    #[test]
    fn test_display_with_aes_cbc_pad_iv8() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x80000001,
            mode: 6,
            key_size: 16,
        };
        assert_eq!(encryption_type.to_string(), "AES-128-CBC-PadIV8");
    }

    #[test]
    fn test_display_with_aes_ecb() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x80000001,
            mode: 2,
            key_size: 32,
        };
        assert_eq!(encryption_type.to_string(), "AES-256-ECB");
    }

    #[test]
    fn test_display_with_unknown_method_and_mode() {
        let encryption_type: CdsaEncrEncryptionType = CdsaEncrEncryptionType {
            method: 0x00012345,
            mode: 1,
            key_size: 16,
        };
        assert_eq!(encryption_type.to_string(), "0x00012345-128-0x00012345");
    }
}
