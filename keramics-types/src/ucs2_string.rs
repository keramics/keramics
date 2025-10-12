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

use std::cmp::Ordering;

use super::{bytes_to_u16_be, bytes_to_u16_le};

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
        let data_size: usize = data.len();
        let mut elements: Vec<u16> = Vec::new();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_be!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
        Self { elements: elements }
    }

    /// Reads a little-endian UCS-2 string from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let mut elements: Vec<u16> = Vec::new();
        Ucs2String::read_elements_le(&mut elements, data);

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

    // TODO: add read_elements_be

    /// Reads a little-endian UCS-2 string from a byte sequence.
    pub fn read_elements_le(elements: &mut Vec<u16>, data: &[u8]) {
        let data_size: usize = data.len();

        for data_offset in (0..data_size).step_by(2) {
            let value_16bit: u16 = bytes_to_u16_le!(data, data_offset);
            if value_16bit == 0 {
                break;
            }
            elements.push(value_16bit);
        }
    }

    /// Compares two UCS-2 strings.
    pub fn compare(&self, other: &Self) -> Ordering {
        let self_size: usize = self.elements.len();
        let other_size: usize = other.len();

        let mut element_index: usize = 0;
        while element_index < self_size && element_index < other_size {
            let self_element: u16 = self.elements[element_index];
            let other_element: u16 = other.elements[element_index];

            if self_element < other_element {
                return Ordering::Less;
            }
            if self_element > other_element {
                return Ordering::Greater;
            }
            element_index += 1;
        }
        if element_index < other_size {
            return Ordering::Less;
        }
        if element_index < self_size {
            return Ordering::Greater;
        }
        Ordering::Equal
    }

    /// Converts the UCS-2 string to a `String`.
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

impl From<&[u16]> for Ucs2String {
    /// Converts a [`&[u16]`] into a [`Ucs2String`]
    fn from(slice: &[u16]) -> Self {
        let elements: &[u16] = match slice.iter().position(|ucs2_value| *ucs2_value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => &slice,
        };
        Self {
            elements: Vec::from(elements),
        }
    }
}

impl From<&str> for Ucs2String {
    /// Converts a [`&str`] into a [`Ucs2String`]
    fn from(string: &str) -> Self {
        // TODO: add support for escaped non UTF-16 characters.
        Self {
            elements: string.encode_utf16().collect(),
        }
    }
}

impl From<&String> for Ucs2String {
    /// Converts a [`&String`] into a [`Ucs2String`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&Vec<u16>> for Ucs2String {
    /// Converts a [`&Vec<u16>`] into a [`Ucs2String`]
    #[inline(always)]
    fn from(vector: &Vec<u16>) -> Self {
        Self::from(vector.as_slice())
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

    // TODO: add tests for read_elements_be
    // TODO: add tests for read_elements_le

    #[test]
    fn test_compare() {
        let ucs2_string: Ucs2String = Ucs2String::from("string1");

        let compare_ucs2_string: Ucs2String = Ucs2String::from("STRING1");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Greater);

        let compare_ucs2_string: Ucs2String = Ucs2String::from("string0");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Greater);

        let compare_ucs2_string: Ucs2String = Ucs2String::from("string1");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Equal);

        let compare_ucs2_string: Ucs2String = Ucs2String::from("string2");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Less);

        let compare_ucs2_string: Ucs2String = Ucs2String::from("string");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Greater);

        let compare_ucs2_string: Ucs2String = Ucs2String::from("string10");
        assert_eq!(ucs2_string.compare(&compare_ucs2_string), Ordering::Less);
    }

    #[test]
    fn test_to_string() {
        let test_data: [u8; 24] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);

        assert_eq!(ucs2_string.to_string(), String::from("UCS-2 string"));
        let test_data: [u8; 24] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x00, 0xd8, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00,
        ];

        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);
        assert_eq!(ucs2_string.to_string(), String::from("UCS-\\{d800} string"));
    }

    #[test]
    fn test_from_slice() {
        let test_data: [u16; 14] = [
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067, 0x0000, 0x0000,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from(test_data.as_slice());

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(ucs2_string.elements, expected_elements);
    }

    #[test]
    fn test_from_str() {
        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(ucs2_string.elements, expected_elements);
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("UCS-2 string");
        let ucs2_string: Ucs2String = Ucs2String::from(&test_string);

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(ucs2_string.elements, expected_elements);
    }

    #[test]
    fn test_from_vector() {
        let test_vector: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067, 0x0000, 0x0000,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from(&test_vector);

        let expected_elements: Vec<u16> = vec![
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert_eq!(ucs2_string.elements, expected_elements);
    }
}
