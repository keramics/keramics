/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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

use super::{bytes_to_u16_be, bytes_to_u16_le};

/// 16-bit Unicode Transformation Format (UTF-16) string.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Utf16String {
    /// Elements.
    pub elements: Vec<u16>,
}

impl Utf16String {
    /// Creates a new UTF-16 string.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Reads a big-endian UTF-16 string from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len() / 2;
        let mut elements: Vec<u16> = Vec::new();

        for data_offset in 0..data_size {
            let value_16bit = bytes_to_u16_be!(data, data_offset * 2);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
        Self { elements: elements }
    }

    /// Reads a little-endian UTF-16 string from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len() / 2;
        let mut elements: Vec<u16> = Vec::new();

        for data_offset in 0..data_size {
            let value_16bit = bytes_to_u16_le!(data, data_offset * 2);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
        Self { elements: elements }
    }

    /// Retrieves the string representation of an UTF-16 string.
    pub fn to_string(&self) -> String {
        String::from_utf16(&self.elements).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 26] = [
            0x00, 0x55, 0x00, 0x54, 0x00, 0x46, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x36, 0x00, 0x20,
            0x00, 0x73, 0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67,
        ];

        let ucs2_string: Utf16String = Utf16String::from_be_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), "UTF-16 string".to_string(),);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 26] = [
            0x55, 0x00, 0x54, 0x00, 0x46, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x36, 0x00, 0x20, 0x00,
            0x73, 0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];

        let ucs2_string: Utf16String = Utf16String::from_le_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), "UTF-16 string".to_string(),);
    }
}
