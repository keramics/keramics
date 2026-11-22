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

/// BitLocker Drive Encryption (BDE) encryption type.
#[derive(Clone, Debug, PartialEq)]
pub struct BdeEncryptionType {
    /// Encryption method.
    pub(super) method: u16,
}

impl BdeEncryptionType {
    /// Creates a new encryption type.
    pub(super) fn new(method: u16) -> Self {
        Self { method }
    }

    /// Retrieves the FVEK size.
    pub(super) fn get_fvek_size(&self) -> usize {
        match self.method {
            0x8000..=0x8001 | 0x8005 => 12 + 64,
            0x8002 => 12 + 16,
            0x8003..=0x8004 => 12 + 32,
            _ => 0,
        }
    }

    /// Retrieves the key size.
    pub(super) fn get_key_size(&self) -> usize {
        match self.method {
            0x8000 | 0x8002 => 16,
            0x8001 | 0x8003..=0x8004 => 32,
            0x8005 => 64,
            _ => 0,
        }
    }
}

impl fmt::Display for BdeEncryptionType {
    /// Formats encryption type for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.method {
            0x8000 => write!(formatter, "AES-128-CBC with Elephant Diffuser"),
            0x8001 => write!(formatter, "AES-256-CBC with Elephant Diffuser"),
            0x8002 => write!(formatter, "AES-128-CBC"),
            0x8003 => write!(formatter, "AES-256-CBC"),
            0x8004 => write!(formatter, "AES-128-XTS"),
            0x8005 => write!(formatter, "AES-256-XTS"),
            _ => write!(formatter, "Unknown: 0x{:08x}", self.method),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fvek_size_key_size_and_fmt() {
        let cases: [(u16, usize, usize, &str); 7] = [
            (0x8000, 76, 16, "AES-128-CBC with Elephant Diffuser"),
            (0x8001, 76, 32, "AES-256-CBC with Elephant Diffuser"),
            (0x8002, 28, 16, "AES-128-CBC"),
            (0x8003, 44, 32, "AES-256-CBC"),
            (0x8004, 44, 32, "AES-128-XTS"),
            (0x8005, 76, 64, "AES-256-XTS"),
            (0x9999, 0, 0, "Unknown: 0x00009999"),
        ];
        for (method, expected_fvek_size, expected_key_size, expected_display) in cases {
            let encryption_type: BdeEncryptionType = BdeEncryptionType::new(method);
            assert_eq!(encryption_type.get_fvek_size(), expected_fvek_size);
            assert_eq!(encryption_type.get_key_size(), expected_key_size);
            assert_eq!(encryption_type.to_string(), expected_display);
        }
    }
}
