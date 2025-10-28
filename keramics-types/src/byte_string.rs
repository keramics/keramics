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

use keramics_core::ErrorTrace;
use keramics_encodings::{
    CharacterDecoder, CharacterEncoder, CharacterEncoding, new_character_decoder,
    new_character_encoder,
};

/// String of 8-bit elements.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteString {
    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Elements.
    pub elements: Vec<u8>,
}

impl ByteString {
    /// Creates a new string.
    pub fn new() -> Self {
        Self {
            encoding: CharacterEncoding::Utf8,
            elements: Vec::new(),
        }
    }

    /// Creates a new string with a specified character encoding.
    pub fn new_with_encoding(encoding: &CharacterEncoding) -> Self {
        Self {
            encoding: encoding.clone(),
            elements: Vec::new(),
        }
    }

    /// Extends the string from another [`ByteString`].
    pub fn extend(&mut self, byte_string: &ByteString) -> Result<(), ErrorTrace> {
        if self.encoding == byte_string.encoding {
            self.elements.extend_from_slice(&byte_string.elements);
        } else {
            let mut character_decoder: CharacterDecoder =
                new_character_decoder(&byte_string.encoding, &byte_string.elements);

            let mut code_points: Vec<u32> = Vec::new();

            while let Some(result) = character_decoder.next() {
                match result {
                    Ok(code_point) => code_points.push(code_point),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode character");
                        return Err(error);
                    }
                }
            }
            match self.extend_from_codepoints(&code_points) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to extend from code points"
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Extends the string from code points.
    pub fn extend_from_codepoints(&mut self, code_points: &Vec<u32>) -> Result<(), ErrorTrace> {
        let mut character_encoder: CharacterEncoder =
            new_character_encoder(&self.encoding, code_points);

        while let Some(result) = character_encoder.next() {
            match result {
                Ok(slice) => self.elements.extend_from_slice(&slice),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to encode character");
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Retrieves a character decoder for the string.
    pub fn get_character_decoder(&self) -> CharacterDecoder<'_> {
        new_character_decoder(&self.encoding, &self.elements)
    }

    /// Determines if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads the string from a buffer.
    pub fn read_data(&mut self, data: &[u8]) {
        let slice: &[u8] = match data.iter().position(|value| *value == 0) {
            Some(data_index) => &data[0..data_index],
            None => &data,
        };
        self.elements.extend_from_slice(&slice);
    }

    /// Converts a [`ByteString`] to a [`String`].
    pub fn to_string(&self) -> String {
        let mut character_decoder: CharacterDecoder = self.get_character_decoder();

        let mut string_parts: Vec<String> = Vec::new();

        while let Some(result) = character_decoder.next() {
            match result {
                Ok(code_point) => {
                    let string: String = match char::from_u32(code_point as u32) {
                        Some(unicode_character) => {
                            if unicode_character == '\\' {
                                String::from("\\\\")
                            } else {
                                unicode_character.to_string()
                            }
                        }
                        None => format!("\\{{{:04x}}}", code_point),
                    };
                    string_parts.push(string);
                }
                Err(error) => todo!(),
            }
        }
        string_parts.join("")
    }
}

impl From<&[u8]> for ByteString {
    /// Converts a [`&[u8]`] into a [`ByteString`]
    fn from(slice: &[u8]) -> Self {
        let elements: &[u8] = match slice.iter().position(|value| *value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => &slice,
        };
        Self {
            encoding: CharacterEncoding::Utf8,
            elements: Vec::from(elements),
        }
    }
}

impl From<&str> for ByteString {
    /// Converts a [`&str`] into a [`ByteString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::from(string.as_bytes())
    }
}

impl From<&String> for ByteString {
    /// Converts a [`&String`] into a [`ByteString`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str().as_bytes())
    }
}

impl From<&Vec<u8>> for ByteString {
    /// Converts a [`&Vec<u8>`] into a [`ByteString`]
    #[inline(always)]
    fn from(vector: &Vec<u8>) -> Self {
        Self::from(vector.as_slice())
    }
}

impl PartialEq<&[u8]> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&[u8]`]
    #[inline(always)]
    fn eq(&self, slice: &&[u8]) -> bool {
        self.elements == *slice
    }
}

impl PartialEq<&str> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, string: &&str) -> bool {
        self.elements == *string.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add test for extend

    #[test]
    fn test_is_empty() {
        let byte_string: ByteString = ByteString::new();
        assert!(byte_string.is_empty());

        let byte_string: ByteString = ByteString {
            encoding: CharacterEncoding::Utf8,
            elements: vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ],
        };
        assert!(!byte_string.is_empty());
    }

    #[test]
    fn test_len() {
        let byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let byte_string: ByteString = ByteString {
            encoding: CharacterEncoding::Utf8,
            elements: vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ],
        };
        assert_eq!(byte_string.len(), 12);
    }

    #[test]
    fn test_read_data() {
        let mut byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        byte_string.read_data(&test_data);
        assert_eq!(byte_string.len(), 12);
    }

    // TODO: add test for to_string

    #[test]
    fn test_from_u8_slice() {
        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from(test_data.as_slice());

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_str() {
        let byte_string: ByteString = ByteString::from("ASCII string");

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("ASCII string");
        let byte_string: ByteString = ByteString::from(&test_string);

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }

    #[test]
    fn test_from_vector() {
        let test_vector: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from(&test_vector);

        let expected_elements: Vec<u8> = vec![
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert_eq!(byte_string.elements, expected_elements);
    }
}
