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
use keramics_layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "nonce_time", data_type = "Filetime"),
        field(name = "nonce_counter", data_type = "u32"),
        field(name = "nonce", data_type = "[u8; 12]", format = "hex"),
        field(name = "tag", data_type = "[u8; 16]", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) AES-CCM encrypted key.
pub struct BdeAesCcmEncryptedKey {
    /// Nonce.
    pub nonce: Vec<u8>,

    /// Tag.
    pub tag: Vec<u8>,

    /// Encrypted data.
    pub encrypted_data: Vec<u8>,
}

impl BdeAesCcmEncryptedKey {
    /// Creates a new AES-CCM encrypted key.
    pub fn new() -> Self {
        Self {
            nonce: Vec::new(),
            tag: Vec::new(),
            encrypted_data: Vec::new(),
        }
    }

    /// Reads the AES-CCM encrypted key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 28 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.nonce = data[0..12].to_vec();
        self.tag = data[12..28].to_vec();
        self.encrypted_data = data[28..].to_vec();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x03, 0x00, 0x00, 0x00, 0x61, 0x99,
            0x52, 0x91, 0xa6, 0x7a, 0xf5, 0xb7, 0x7d, 0x49, 0x43, 0x15, 0xae, 0x32, 0x2e, 0x1b,
            0xed, 0x99, 0x40, 0x76, 0xcc, 0xd0, 0x22, 0x54, 0xd2, 0xcf, 0x82, 0xfd, 0x2e, 0x92,
            0x36, 0x53, 0xbb, 0x6a, 0xab, 0x0f, 0xd2, 0x50, 0x91, 0xff, 0x7e, 0xa9, 0xe1, 0x0b,
            0x61, 0xc6, 0x12, 0x52, 0xe3, 0xc7, 0x94, 0xd7, 0xe3, 0x92, 0x04, 0x47, 0xbf, 0x42,
            0x26, 0x7a,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeAesCcmEncryptedKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.nonce, &test_data[0..12]);
        assert_eq!(test_struct.tag, &test_data[12..28]);
        assert_eq!(test_struct.encrypted_data, &test_data[28..]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeAesCcmEncryptedKey::new();
        let result = test_struct.read_data(&test_data[0..27]);
        assert!(result.is_err());
    }
}
