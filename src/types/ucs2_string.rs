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

use crate::{bytes_to_u16_be, bytes_to_u16_le};

/// 16-bit Universal Coded Character Set (UCS-2) string.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Ucs2String {
    pub string: Vec<u16>,
}

impl Ucs2String {
    /// Creates a new UCS-2 string.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads a big-endian UCS-2 string from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len() / 2;
        let mut string: Vec<u16> = Vec::new();

        for string_index in 0..data_size {
            let value_16bit = bytes_to_u16_be!(data, string_index * 2);
            if value_16bit == 0 {
                break;
            }
            string.push(value_16bit);
        }
        Self { string: string }
    }

    /// Reads a little-endian UCS-2 string from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len() / 2;
        let mut string: Vec<u16> = Vec::new();

        for string_index in 0..data_size {
            let value_16bit = bytes_to_u16_le!(data, string_index * 2);
            if value_16bit == 0 {
                break;
            }
            string.push(value_16bit);
        }
        Self { string: string }
    }

    /// Retrieves the string representation of an UCS-2 string.
    pub fn to_string(&self) -> String {
        // TODO: add support for non-Unicode characters.
        String::from_utf16(&self.string).unwrap()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 24] = [
            0x00, 0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67,
        ];

        let ucs2_string: Ucs2String = Ucs2String::from_be_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), "UCS-2 string".to_string(),);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 24] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];

        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), "UCS-2 string".to_string(),);
    }
}
