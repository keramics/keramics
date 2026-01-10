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

//! 16-bit Unicode Transformation Format (UTF-16) string.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::slice::Iter;

use keramics_core::ErrorTrace;

use super::{bytes_to_u16_be, bytes_to_u16_le};

/// UTF-16 character mappings.
pub struct Utf16CharacterMappings {
    /// Mappings.
    mappings: HashMap<u32, u32>,
}

impl Utf16CharacterMappings {
    /// Creates a new UCS-2 character mappings.
    pub fn new(mappings: &[(u32, u32)]) -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Adds a mapping.
    pub fn add(&mut self, code_point: u32, mapped_code_point: u32) {
        self.mappings.insert(code_point, mapped_code_point);
    }

    /// Retrieves a mapped code point.
    pub(super) fn get_code_point(&self, code_point: &u32) -> u32 {
        match self.mappings.get(code_point) {
            Some(mapped_code_point) => *mapped_code_point,
            None => *code_point,
        }
    }

    /// Retrieves the number of mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

impl From<&[(u32, u32)]> for Utf16CharacterMappings {
    /// Converts a [`&[(u32, u32)]`] into a [`Utf16CharacterMappings`]
    #[inline(always)]
    fn from(mappings: &[(u32, u32)]) -> Self {
        Self {
            mappings: mappings
                .iter()
                .map(|(character, mapped_character)| (*character, *mapped_character))
                .collect::<HashMap<u32, u32>>(),
        }
    }
}

/// UTF-16 string.
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

    /// Creates a new UTF-16 string with case folding applied.
    pub fn new_with_case_folding(
        &self,
        mappings: &Utf16CharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        let code_points: Vec<u32> = match self.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode UTF-16 string");
                return Err(error);
            }
        };
        let mut elements: Vec<u16> = Vec::new();

        for code_point in code_points.iter() {
            let mut mapped_code_point: u32 = mappings.get_code_point(code_point);

            if mapped_code_point > 0x0000ffff {
                mapped_code_point -= 0x00010000;
                elements.push(0xd800 + (mapped_code_point >> 10) as u16);
                elements.push(0xdc00 + (mapped_code_point & 0x000003ff) as u16);
            } else {
                elements.push(mapped_code_point as u16);
            }
        }
        Ok(Self { elements })
    }

    /// Decodes the UTF-16 string.
    pub fn decode(&self) -> Result<Vec<u32>, ErrorTrace> {
        let mut code_points: Vec<u32> = Vec::new();

        let mut elements_iterator: Iter<u16> = self.elements.iter();

        while let Some(element) = elements_iterator.next() {
            let mut code_point: u32 = *element as u32;

            if code_point >= 0x0000d800 && code_point <= 0x0000dbff {
                let low_surrogate: u32 = match elements_iterator.next() {
                    Some(next_element) => *next_element as u32,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid UTF-16 missing low surrogate"
                        ));
                    }
                };
                if low_surrogate < 0x0000dc00 || low_surrogate > 0x0000dfff {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid UTF-16 low surrogate value out of bounds"
                    ));
                }
                code_point =
                    0x00010000 + ((code_point - 0x0000d800) << 10) + (low_surrogate - 0x0000dc00);
            }
            code_points.push(code_point);
        }
        Ok(code_points)
    }

    /// Reads a big-endian UTF-16 string from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len();
        let mut elements: Vec<u16> = Vec::new();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_be!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
        Self { elements }
    }

    /// Reads a little-endian UTF-16 string from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let data_size: usize = data.len();
        let mut elements: Vec<u16> = Vec::new();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_le!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
        Self { elements }
    }

    /// Creates a new UTF-16 string from a string with case folding applied.
    pub fn from_string_with_case_folding(string: &str, mappings: &Utf16CharacterMappings) -> Self {
        // TODO: implement
        todo!();
    }

    /// Determines if the UTF-16 string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the UTF-16 string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads a UTF-16 string from a buffer in big-endian.
    pub fn read_data_be(&mut self, data: &[u8]) {
        let data_size: usize = data.len();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_be!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            self.elements.push(value_16bit);
        }
    }

    /// Reads a UTF-16 string from a buffer in little-endian.
    pub fn read_data_le(&mut self, data: &[u8]) {
        let data_size: usize = data.len();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_le!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            self.elements.push(value_16bit);
        }
    }
}

impl From<&[u16]> for Utf16String {
    /// Converts a [`&[u16]`] into a [`Utf16String`]
    fn from(slice: &[u16]) -> Self {
        let elements: &[u16] = match slice.iter().position(|utf16_value| *utf16_value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => slice,
        };
        Self {
            elements: elements.to_vec(),
        }
    }
}

impl From<&str> for Utf16String {
    /// Converts a [`&str`] into a [`Utf16String`]
    fn from(string: &str) -> Self {
        Self {
            elements: string.encode_utf16().collect::<Vec<u16>>(),
        }
    }
}

impl From<&String> for Utf16String {
    /// Converts a [`&String`] into a [`Utf16String`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl PartialEq<&[u16]> for Utf16String {
    /// Detemines if a [`Utf16String`] is equal to a [`&[u16]`]
    #[inline(always)]
    fn eq(&self, slice: &&[u16]) -> bool {
        self.elements == *slice
    }
}

impl PartialEq<str> for Utf16String {
    /// Detemines if a [`Utf16String`] is equal to a [`str`]
    #[inline(always)]
    fn eq(&self, string: &str) -> bool {
        self.elements == string.encode_utf16().collect::<Vec<u16>>()
    }
}

impl PartialEq<&str> for Utf16String {
    /// Detemines if a [`Utf16String`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, string: &&str) -> bool {
        Self::eq(self, *string)
    }
}

impl fmt::Display for Utf16String {
    /// Formats the UTF-16 string for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: String = String::from_utf16(&self.elements).unwrap();

        write!(formatter, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::UNICODE_CASE_MAPPINGS;

    #[test]
    fn test_new_with_case_folding() -> Result<(), ErrorTrace> {
        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");

        let mappings: Utf16CharacterMappings =
            Utf16CharacterMappings::from(UNICODE_CASE_MAPPINGS.as_slice());

        let test_string: Utf16String = utf16_string.new_with_case_folding(&mappings)?;

        assert_eq!(
            test_string,
            Utf16String {
                elements: vec![
                    0x0075, 0x0074, 0x0066, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ],
            }
        );
        Ok(())
    }

    // TODO: add tests for decode

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 26] = [
            0x00, 0x55, 0x00, 0x54, 0x00, 0x46, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x36, 0x00, 0x20,
            0x00, 0x73, 0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67,
        ];
        let utf16_string: Utf16String = Utf16String::from_be_bytes(&test_data);

        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![
                    0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ]
            }
        );
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 26] = [
            0x55, 0x00, 0x54, 0x00, 0x46, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x36, 0x00, 0x20, 0x00,
            0x73, 0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];
        let utf16_string: Utf16String = Utf16String::from_le_bytes(&test_data);

        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![
                    0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ]
            }
        );
    }

    // TODO: add test_from_string_with_case_folding

    #[test]
    fn test_is_empty() {
        let utf16_string: Utf16String = Utf16String::new();
        assert!(utf16_string.is_empty());

        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");
        assert!(!utf16_string.is_empty());
    }

    #[test]
    fn test_len() {
        let utf16_string: Utf16String = Utf16String::new();
        assert_eq!(utf16_string.len(), 0);

        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");
        assert_eq!(utf16_string.len(), 13);
    }

    // TODO: add tests for read_data_be
    // TODO: add tests for read_data_le

    #[test]
    fn test_from_u16_slice() {
        let test_data: [u16; 15] = [
            0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
            0x006e, 0x0067, 0x0000, 0x0000,
        ];
        let utf16_string: Utf16String = Utf16String::from(test_data.as_slice());

        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![
                    0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_str() {
        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");

        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![
                    0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("UTF-16 string");
        let utf16_string: Utf16String = Utf16String::from(&test_string);

        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![
                    0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                    0x0069, 0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_eq_u16_slice() {
        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");

        let test_array: [u16; 13] = [
            0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
            0x006e, 0x0067,
        ];
        assert!(utf16_string.eq(&test_array.as_slice()));
    }

    #[test]
    fn test_eq_str() {
        let utf16_string: Utf16String = Utf16String::from("UTF-16 string");

        assert!(utf16_string.eq("UTF-16 string"));
        assert!(utf16_string.eq(&"UTF-16 string"));
    }

    #[test]
    fn test_to_string() {
        let utf16_string: Utf16String = Utf16String {
            elements: vec![
                0x0055, 0x0054, 0x0046, 0x002d, 0x0031, 0x0036, 0x0020, 0x0073, 0x0074, 0x0072,
                0x0069, 0x006e, 0x0067,
            ],
        };

        let string: String = utf16_string.to_string();
        assert_eq!(string, "UTF-16 string");
    }
}
