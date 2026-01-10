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

//! Single or multi byte string.

use std::fmt;

use keramics_core::ErrorTrace;
use keramics_encodings::{
    CharacterDecoder, CharacterEncoding, new_character_decoder, new_character_encoder,
};

use super::ucs2_string::Ucs2String;
use super::utf16_string::Utf16String;

/// Byte string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteString {
    /// Character encoding.
    pub encoding: CharacterEncoding,

    /// Elements.
    pub elements: Vec<u8>,
}

impl ByteString {
    /// Creates a new byte string.
    pub fn new() -> Self {
        Self {
            encoding: CharacterEncoding::Utf8,
            elements: Vec::new(),
        }
    }

    /// Creates a new byte string with a specified character encoding.
    pub fn new_with_encoding(encoding: &CharacterEncoding) -> Self {
        Self {
            encoding: encoding.clone(),
            elements: Vec::new(),
        }
    }

    /// Decodes the byte string.
    pub fn decode(&self) -> Result<Vec<u32>, ErrorTrace> {
        let mut code_points: Vec<u32> = Vec::new();

        for result in new_character_decoder(&self.encoding, &self.elements) {
            match result {
                Ok(mut decoder_results) => code_points.append(&mut decoder_results),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                    return Err(error);
                }
            }
        }
        Ok(code_points)
    }

    /// Encodes the byte string with a specified character encoding.
    pub fn encode(&self, encoding: &CharacterEncoding) -> Result<Self, ErrorTrace> {
        let byte_string: ByteString = if self.encoding == *encoding {
            self.clone()
        } else {
            let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);

            match byte_string.extend(self) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to extend byte string");
                    return Err(error);
                }
            }
            byte_string
        };
        Ok(byte_string)
    }

    /// Extends the byte string from another byte string.
    pub fn extend(&mut self, byte_string: &ByteString) -> Result<(), ErrorTrace> {
        if self.encoding == byte_string.encoding {
            self.elements.extend_from_slice(&byte_string.elements);
        } else {
            let code_points: Vec<u32> = match byte_string.decode() {
                Ok(code_points) => code_points,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                    return Err(error);
                }
            };
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

    /// Extends the byte string from code points.
    pub fn extend_from_codepoints(&mut self, code_points: &[u32]) -> Result<(), ErrorTrace> {
        for result in new_character_encoder(&self.encoding, code_points) {
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

    /// Creates a new byte string with a specified character encoding from a string.
    pub fn from_string_with_encoding(
        encoding: &CharacterEncoding,
        string: &str,
    ) -> Result<Self, ErrorTrace> {
        if *encoding == CharacterEncoding::Utf8 {
            Ok(ByteString::from(string))
        } else {
            let code_points: Vec<u32> = string.chars().map(|character| character as u32).collect();

            let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);

            match byte_string.extend_from_codepoints(&code_points) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to encode byte string");
                    return Err(error);
                }
            }
            Ok(byte_string)
        }
    }

    /// Creates a new byte string with a specified character encoding from an UCS-2 string.
    pub fn from_ucs2_string_with_encoding(
        encoding: &CharacterEncoding,
        ucs2_string: &Ucs2String,
    ) -> Result<Self, ErrorTrace> {
        let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);

        let code_points: Vec<u32> = ucs2_string.decode();

        match byte_string.extend_from_codepoints(&code_points) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to encode byte string");
                return Err(error);
            }
        }
        Ok(byte_string)
    }

    /// Creates a new byte string with a specified character encoding from an UTF-16 string.
    pub fn from_utf16_string_with_encoding(
        encoding: &CharacterEncoding,
        utf16_string: &Utf16String,
    ) -> Result<Self, ErrorTrace> {
        let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);

        let code_points: Vec<u32> = match utf16_string.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode UTF-16 string");
                return Err(error);
            }
        };
        match byte_string.extend_from_codepoints(&code_points) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to encode byte string");
                return Err(error);
            }
        }
        Ok(byte_string)
    }

    /// Retrieves a character decoder for the byte string.
    pub fn get_character_decoder(&self) -> CharacterDecoder<'_> {
        new_character_decoder(&self.encoding, &self.elements)
    }

    /// Determines if the byte string is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Retrieves the length (or size) of the byte string.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Reads the byte string from a buffer.
    pub fn read_data(&mut self, data: &[u8]) {
        let slice: &[u8] = match data.iter().position(|value| *value == 0) {
            Some(data_index) => &data[0..data_index],
            None => data,
        };
        self.elements.extend_from_slice(slice);
    }
}

impl From<&[u8]> for ByteString {
    /// Converts a [`&[u8]`] into a [`ByteString`]
    fn from(slice: &[u8]) -> Self {
        let elements: &[u8] = match slice.iter().position(|value| *value == 0) {
            Some(slice_index) => &slice[0..slice_index],
            None => slice,
        };
        Self {
            encoding: CharacterEncoding::Utf8,
            elements: elements.to_vec(),
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

impl PartialEq<&[u8]> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&[u8]`]
    #[inline(always)]
    fn eq(&self, slice: &&[u8]) -> bool {
        self.elements == *slice
    }
}

impl PartialEq<str> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`str`]
    #[inline(always)]
    fn eq(&self, string: &str) -> bool {
        // TODO: handle encoding
        self.elements == string.as_bytes()
    }
}

impl PartialEq<&str> for ByteString {
    /// Detemines if a [`ByteString`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, string: &&str) -> bool {
        Self::eq(self, *string)
    }
}

impl fmt::Display for ByteString {
    /// Formats the byte string for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut string_parts: Vec<String> = Vec::new();

        for result in self.get_character_decoder() {
            match result {
                Ok(code_points) => {
                    for code_point in code_points {
                        let string: String = match char::from_u32(code_point) {
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
                }
                Err(error) => return write!(formatter, "{}", error),
            }
        }
        write!(formatter, "{}", string_parts.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() -> Result<(), ErrorTrace> {
        let byte_string: ByteString = ByteString::from("ASCII string");

        let code_points: Vec<u32> = byte_string.decode()?;
        assert_eq!(
            code_points,
            vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ]
        );
        Ok(())
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let byte_string: ByteString = ByteString::from("ASCII string");

        let test_byte_string: ByteString = byte_string.encode(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            test_byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn test_extend() -> Result<(), ErrorTrace> {
        let mut byte_string: ByteString = ByteString::from("ASCII");

        byte_string.extend(&ByteString::from(" string"))?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );

        let mut byte_string: ByteString =
            ByteString::from_string_with_encoding(&CharacterEncoding::Iso8859_1, "ASCII")?;

        byte_string.extend(&ByteString::from(" string"))?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn test_extend_from_codepoints() -> Result<(), ErrorTrace> {
        let mut byte_string: ByteString =
            ByteString::from_string_with_encoding(&CharacterEncoding::Iso8859_1, "ASCII")?;

        byte_string.extend_from_codepoints(&[0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67])?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_string_with_encoding() -> Result<(), ErrorTrace> {
        let byte_string: ByteString =
            ByteString::from_string_with_encoding(&CharacterEncoding::Utf8, "ASCII string")?;

        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );

        let byte_string: ByteString =
            ByteString::from_string_with_encoding(&CharacterEncoding::Iso8859_1, "ASCII string")?;

        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
        Ok(())
    }

    // TODO: add tests for from_ucs2_string_with_encoding
    // TODO: add tests for from_utf16_string_with_encoding

    #[test]
    fn test_get_character_decoder() -> Result<(), ErrorTrace> {
        let byte_string: ByteString =
            ByteString::from_string_with_encoding(&CharacterEncoding::Iso8859_1, "ASCII")?;

        let _ = byte_string.get_character_decoder();

        Ok(())
    }

    #[test]
    fn test_is_empty() {
        let byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.is_empty(), true);

        let byte_string: ByteString = ByteString::from("ASCII string");
        assert_eq!(byte_string.is_empty(), false);
    }

    #[test]
    fn test_len() {
        let byte_string: ByteString = ByteString::new();
        assert_eq!(byte_string.len(), 0);

        let byte_string: ByteString = ByteString::from("ASCII string");
        assert_eq!(byte_string.len(), 12);
    }

    #[test]
    fn test_read_data() {
        let mut byte_string: ByteString = ByteString::new();
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![],
            }
        );

        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        byte_string.read_data(&test_data);
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67
                ],
            }
        );
    }

    #[test]
    fn test_from_u8_slice() {
        let test_data: [u8; 14] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x00,
        ];
        let byte_string: ByteString = ByteString::from(test_data.as_slice());

        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
    }

    #[test]
    fn test_from_str() {
        let byte_string: ByteString = ByteString::from("ASCII string");

        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("ASCII string");
        let byte_string: ByteString = ByteString::from(&test_string);

        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Utf8,
                elements: vec![
                    0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
                ]
            }
        );
    }

    #[test]
    fn test_eq_u8_slice() {
        let byte_string: ByteString = ByteString::from("ASCII string");

        let test_array: [u8; 12] = [
            0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
        ];
        assert!(byte_string.eq(&test_array.as_slice()));
    }

    #[test]
    fn test_eq_str() {
        let byte_string: ByteString = ByteString::from("ASCII string");

        assert!(byte_string.eq("ASCII string"));
        assert!(byte_string.eq(&"ASCII string"));
    }

    #[test]
    fn test_to_string() {
        let byte_string: ByteString = ByteString {
            encoding: CharacterEncoding::Iso8859_1,
            elements: vec![
                0x41, 0x53, 0x43, 0x49, 0x49, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67,
            ],
        };

        let string: String = byte_string.to_string();
        assert_eq!(string, "ASCII string");
    }
}
