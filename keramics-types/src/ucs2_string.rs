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

//! 16-bit Universal Coded Character Set (UCS-2) string.

use std::collections::HashMap;
use std::fmt;

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterDecoder;

use super::byte_string::ByteString;
use super::utf16_string::Utf16String;
use super::{bytes_to_u16_be, bytes_to_u16_le};

/// UCS-2 character mappings.
pub struct Ucs2CharacterMappings {
    /// Mappings.
    mappings: HashMap<u16, u16>,
}

impl Ucs2CharacterMappings {
    /// Creates a new UCS-2 character mappings.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Adds a mapping.
    pub fn add(&mut self, element: u16, mapped_element: u16) {
        self.mappings.insert(element, mapped_element);
    }

    /// Retrieves a mapped element.
    pub fn get(&self, element: &u16) -> u16 {
        match self.mappings.get(element) {
            Some(mapped_element) => *mapped_element,
            None => *element,
        }
    }

    /// Retrieves the number of mappings.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

impl From<&[(u16, u16)]> for Ucs2CharacterMappings {
    /// Converts a [`&[(u16, u16)]`] into a [`Ucs2CharacterMappings`]
    #[inline(always)]
    fn from(mappings: &[(u16, u16)]) -> Self {
        Self {
            mappings: mappings
                .iter()
                .map(|(character, mapped_character)| (*character, *mapped_character))
                .collect::<HashMap<u16, u16>>(),
        }
    }
}

/// UCS-2 string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// Creates a new UCS-2 string with case folding applied.
    pub fn new_with_case_folding(&self, mappings: &Ucs2CharacterMappings) -> Self {
        let elements: Vec<u16> = self
            .elements
            .iter()
            .map(|element| mappings.get(element))
            .collect::<Vec<u16>>();

        Self { elements }
    }

    /// Decodes the UCS-2 string.
    pub fn decode(&self) -> Vec<u32> {
        self.elements
            .iter()
            .map(|element| *element as u32)
            .collect::<Vec<u32>>()
    }

    /// Creates a new UCS-2 string from a byte sequence in big-endian.
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

    /// Creates a new UCS-2 string from a `ByteString`.
    pub fn from_byte_string(byte_string: &ByteString) -> Result<Self, ErrorTrace> {
        let mut elements: Vec<u16> = Vec::new();

        let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

        while let Some(result) = character_decoder.next() {
            match result {
                Ok(code_points) => {
                    for code_point in code_points {
                        if code_point > 0x0000ffff {
                            return Err(keramics_core::error_trace_new!(
                                "Code point outside of UCS-2 range"
                            ));
                        }
                        elements.push(code_point as u16);
                    }
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                    return Err(error);
                }
            }
        }
        Ok(Self { elements })
    }

    /// Creates a new UCS-2 string from a `ByteString` with case folding applied.
    pub fn from_byte_string_with_case_folding(
        byte_string: &ByteString,
        mappings: &Ucs2CharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        let mut elements: Vec<u16> = Vec::new();

        let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

        while let Some(result) = character_decoder.next() {
            match result {
                Ok(code_points) => {
                    for code_point in code_points {
                        if code_point > 0x0000ffff {
                            return Err(keramics_core::error_trace_new!(
                                "Code point outside of UCS-2 range"
                            ));
                        }
                        elements.push(mappings.get(&(code_point as u16)));
                    }
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                    return Err(error);
                }
            }
        }
        Ok(Self { elements })
    }

    /// Creates a new UCS-2 string from a byte sequence in little-endian.
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

    /// Creates a new UCS-2 string from a string with case folding applied.
    pub fn from_string_with_case_folding(
        string: &str,
        mappings: &Ucs2CharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        let mut elements: Vec<u16> = Vec::new();

        for character in string.chars() {
            let code_point: u32 = character as u32;
            if code_point > 0x0000ffff {
                return Err(keramics_core::error_trace_new!(
                    "Code point outside of UCS-2 range"
                ));
            }
            elements.push(mappings.get(&(code_point as u16)));
        }
        Ok(Self { elements })
    }

    /// Creates a new UCS-2 string from an `Utf16String`.
    pub fn from_utf16_string(utf16_string: &Utf16String) -> Result<Self, ErrorTrace> {
        let code_points: Vec<u32> = match utf16_string.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode UTF-16 string");
                return Err(error);
            }
        };
        let mut elements: Vec<u16> = Vec::new();

        for code_point in code_points {
            if code_point > 0x0000ffff {
                return Err(keramics_core::error_trace_new!(
                    "Code point outside of UCS-2 range"
                ));
            }
            elements.push(code_point as u16);
        }
        Ok(Self { elements })
    }

    /// Creates a new UCS-2 string from an `Utf16String` with case folding applied.
    pub fn from_utf16_string_with_case_folding(
        utf16_string: &Utf16String,
        mappings: &Ucs2CharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        let code_points: Vec<u32> = match utf16_string.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode UTF-16 string");
                return Err(error);
            }
        };
        let mut elements: Vec<u16> = Vec::new();

        for code_point in code_points {
            if code_point > 0x0000ffff {
                return Err(keramics_core::error_trace_new!(
                    "Code point outside of UCS-2 range"
                ));
            }
            elements.push(mappings.get(&(code_point as u16)));
        }
        Ok(Self { elements })
    }

    /// Determines if the UCS-2 string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the UCS-2 string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads a UCS-2 string from a buffer in big-endian.
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

    /// Reads a UCS-2 string from a buffer in little-endian.
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

impl From<&[u16]> for Ucs2String {
    /// Converts a [`&[u16]`] into a [`Ucs2String`]
    fn from(slice: &[u16]) -> Self {
        let elements: &[u16] = match slice.iter().position(|ucs2_value| *ucs2_value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => slice,
        };
        Self {
            elements: elements.to_vec(),
        }
    }
}

impl From<&str> for Ucs2String {
    /// Converts a [`&str`] into a [`Ucs2String`]
    fn from(string: &str) -> Self {
        Self {
            elements: string.encode_utf16().collect::<Vec<u16>>(),
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

impl PartialEq<&[u16]> for Ucs2String {
    /// Detemines if a [`Ucs2String`] is equal to a [`&[u16]`]
    #[inline(always)]
    fn eq(&self, slice: &&[u16]) -> bool {
        self.elements == *slice
    }
}

impl PartialEq<str> for Ucs2String {
    /// Detemines if a [`Ucs2String`] is equal to a [`str`]
    #[inline(always)]
    fn eq(&self, string: &str) -> bool {
        self.elements == string.encode_utf16().collect::<Vec<u16>>()
    }
}

impl PartialEq<&str> for Ucs2String {
    /// Detemines if a [`Ucs2String`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, string: &&str) -> bool {
        Self::eq(self, *string)
    }
}

impl fmt::Display for Ucs2String {
    /// Formats the UCS-2 string for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string_parts: Vec<String> = self
            .elements
            .iter()
            .map(|element| match char::from_u32(*element as u32) {
                Some(unicode_character) => {
                    if unicode_character == '\\' {
                        String::from("\\\\")
                    } else {
                        unicode_character.to_string()
                    }
                }
                None => format!("\\{{{:04x}}}", element),
            })
            .collect::<Vec<String>>();

        write!(formatter, "{}", string_parts.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::UCS2_CASE_MAPPINGS;

    #[test]
    fn test_new_with_case_folding() {
        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");

        let mappings: Ucs2CharacterMappings =
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());

        let test_string: Ucs2String = ucs2_string.new_with_case_folding(&mappings);

        assert_eq!(
            test_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0053, 0x0054, 0x0052, 0x0049,
                    0x004e, 0x0047,
                ],
            }
        );
    }

    // TODO: add tests for decode

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 28] = [
            0x00, 0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73,
            0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_be_bytes(&test_data);

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_byte_string() -> Result<(), ErrorTrace> {
        let byte_string: ByteString = ByteString::from("UCS-2 string");
        let ucs2_string: Ucs2String = Ucs2String::from_byte_string(&byte_string)?;

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_byte_string_with_case_folding() -> Result<(), ErrorTrace> {
        let mappings: Ucs2CharacterMappings =
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());

        let byte_string: ByteString = ByteString::from("UCS-2 string");
        let ucs2_string: Ucs2String =
            Ucs2String::from_byte_string_with_case_folding(&byte_string, &mappings)?;

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0053, 0x0054, 0x0052, 0x0049,
                    0x004e, 0x0047,
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 28] = [
            0x55, 0x00, 0x43, 0x00, 0x53, 0x00, 0x2d, 0x00, 0x32, 0x00, 0x20, 0x00, 0x73, 0x00,
            0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from_le_bytes(&test_data);

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_string_with_case_folding() -> Result<(), ErrorTrace> {
        let mappings: Ucs2CharacterMappings =
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());

        let ucs2_string: Ucs2String =
            Ucs2String::from_string_with_case_folding("UCS-2 string", &mappings)?;

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0053, 0x0054, 0x0052, 0x0049,
                    0x004e, 0x0047,
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_utf16_string() -> Result<(), ErrorTrace> {
        let utf16_string: Utf16String = Utf16String::from("UCS-2 string");
        let ucs2_string: Ucs2String = Ucs2String::from_utf16_string(&utf16_string)?;

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_utf16_string_with_case_folding() -> Result<(), ErrorTrace> {
        let mappings: Ucs2CharacterMappings =
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());

        let utf16_string: Utf16String = Utf16String::from("UCS-2 string");
        let ucs2_string: Ucs2String =
            Ucs2String::from_utf16_string_with_case_folding(&utf16_string, &mappings)?;

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0053, 0x0054, 0x0052, 0x0049,
                    0x004e, 0x0047,
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn test_is_empty() {
        let ucs2_string: Ucs2String = Ucs2String::new();
        assert!(ucs2_string.is_empty());

        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");
        assert!(!ucs2_string.is_empty());
    }

    #[test]
    fn test_len() {
        let ucs2_string: Ucs2String = Ucs2String::new();
        assert_eq!(ucs2_string.len(), 0);

        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");
        assert_eq!(ucs2_string.len(), 12);
    }

    // TODO: add tests for read_data_be
    // TODO: add tests for read_data_le

    #[test]
    fn test_from_u16_slice() {
        let test_data: [u16; 14] = [
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067, 0x0000, 0x0000,
        ];
        let ucs2_string: Ucs2String = Ucs2String::from(test_data.as_slice());

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_str() {
        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("UCS-2 string");
        let ucs2_string: Ucs2String = Ucs2String::from(&test_string);

        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![
                    0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                    0x006e, 0x0067,
                ],
            }
        );
    }

    #[test]
    fn test_eq_u16_slice() {
        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");

        let test_array: [u16; 12] = [
            0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e,
            0x0067,
        ];
        assert!(ucs2_string.eq(&test_array.as_slice()));
    }

    #[test]
    fn test_eq_str() {
        let ucs2_string: Ucs2String = Ucs2String::from("UCS-2 string");

        assert!(ucs2_string.eq("UCS-2 string"));
        assert!(ucs2_string.eq(&"UCS-2 string"));
    }

    #[test]
    fn test_to_string() {
        let ucs2_string: Ucs2String = Ucs2String {
            elements: vec![
                0x0055, 0x0043, 0x0053, 0x002d, 0x0032, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                0x006e, 0x0067,
            ],
        };

        let string: String = ucs2_string.to_string();
        assert_eq!(string, "UCS-2 string");

        let ucs2_string: Ucs2String = Ucs2String {
            elements: vec![
                0x0055, 0x0043, 0x0053, 0x002d, 0xd800, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069,
                0x006e, 0x0067,
            ],
        };

        let string: String = ucs2_string.to_string();
        assert_eq!(string, "UCS-\\{d800} string");
    }
}
