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

use crate::{bytes_to_u16_be, bytes_to_u16_le};

/// 16-bit Universal Coded Character Set (UCS-2) string.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Ucs2String {
    /// Elements.
    pub elements: Vec<u16>,
}

impl Ucs2String {
    /// Creates a new UCS-2 string.
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Reads a big-endian UCS-2 string from a byte sequence.
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

    /// Reads a little-endian UCS-2 string from a byte sequence.
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

    /// Creates a new UCS-2 string from a string.
    pub fn from_string(string: &str) -> Self {
        // TODO: add support for escaped non UTF-16 characters.
        let elements: Vec<u16> = string.encode_utf16().collect();

        Self { elements: elements }
    }

    /// Determines if the UCS-2 string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the UCS-2 string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Retrieves the string representation of an UCS-2 string.
    pub fn to_string(&self) -> String {
        // TODO: escape the escape character (\)
        self.elements
            .iter()
            .map(|element| match char::from_u32(*element as u32) {
                Some(unicode_character) => unicode_character.to_string(),
                None => format!("\\{{{:04x}}}", element),
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 28] = [
            0x00, 0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_be_bytes(&test_data);

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(&ucs2_string.elements, &expected_elements);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 28] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(&ucs2_string.elements, &expected_elements);
    }

    #[test]
    fn test_is_empty() {
        let ucs2_string: Ucs2String = Ucs2String::new();
        assert!(ucs2_string.is_empty());

        let test_data: [u8; 28] = [
            0x00, 0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);
        assert!(!ucs2_string.is_empty());
    }

    #[test]
    fn test_len() {
        let ucs2_string: Ucs2String = Ucs2String::new();
        assert_eq!(ucs2_string.len(), 0);

        let test_data: [u8; 28] = [
            0x00, 0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);
        assert_eq!(ucs2_string.len(), 12);
    }

    #[test]
    fn test_to_string() {
        let test_data: [u8; 24] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];

        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);

        assert_eq!(ucs2_string.to_string(), "UCS-2 string".to_string());
        let test_data: [u8; 24] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x00, 0xd8, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];

        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), "UCS-\\{d800} string".to_string());
    }
}
