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

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_encodings::{CharacterDecoder, CharacterEncoding};
use keramics_types::{ByteString, Ucs2String};

/// Path component for file resolver.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PathComponent {
    ByteString(ByteString),
    OsString(OsString),
    String(String),
    Root,
    Ucs2String(Ucs2String),
    // TODO: add Utf16String support.
}

impl PathComponent {
    /// Retrieves the extension if available.
    pub fn extension(&self) -> Result<Option<PathComponent>, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => Self::extension_from_byte_string(byte_string),
            PathComponent::OsString(os_string) => {
                let path_buf: PathBuf = PathBuf::from(os_string);

                match path_buf.extension() {
                    Some(os_str) => Ok(Some(PathComponent::from(os_str))),
                    None => Ok(None),
                }
            }
            PathComponent::Root => Ok(None),
            PathComponent::String(string) => Ok(Self::extension_from_string(string.as_str())),
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(Self::extension_from_ucs2_string(ucs2_string))
            }
        }
    }

    /// Retrieves the extension from a [`&ByteString`] if available.
    #[inline(always)]
    fn extension_from_byte_string(
        byte_string: &ByteString,
    ) -> Result<Option<PathComponent>, ErrorTrace> {
        if byte_string.is_empty() {
            return Ok(None);
        }
        let code_points: Vec<u32> = match byte_string.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                return Err(error);
            }
        };
        match code_points[1..]
            .iter()
            .rev()
            .position(|value| *value == 0x0000002e)
        {
            Some(value_index) => {
                // Note that value_index is relative to end of the code points.
                let extention_index: usize = code_points.len() - value_index;

                let mut extension_string: String = String::new();

                for code_point in code_points[extention_index..].iter() {
                    match char::from_u32(*code_point) {
                        Some(character) => extension_string.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                Ok(Some(PathComponent::String(extension_string)))
            }
            None => Ok(None),
        }
    }

    /// Retrieves the extension from a [`&str`] if available.
    #[inline(always)]
    fn extension_from_string(string: &str) -> Option<PathComponent> {
        if string.is_empty() {
            return None;
        }
        match string[1..].chars().rev().position(|value| value == '.') {
            Some(value_index) => {
                // Note that value_index is relative to end of the string.
                let extention_index: usize = string.len() - value_index;

                let extension_string: String = string[extention_index..].to_string();

                Some(PathComponent::String(extension_string))
            }
            None => None,
        }
    }

    /// Retrieves the extension from a [`&Ucs2String`] if available.
    #[inline(always)]
    fn extension_from_ucs2_string(ucs2_string: &Ucs2String) -> Option<PathComponent> {
        if ucs2_string.is_empty() {
            return None;
        }
        match ucs2_string.elements[1..]
            .iter()
            .rev()
            .position(|value| *value == 0x002e)
        {
            Some(value_index) => {
                // Note that value_index is relative to end of the string.
                let extention_index: usize = ucs2_string.len() - value_index;

                let extension_string: Ucs2String =
                    Ucs2String::from(&ucs2_string.elements[extention_index..]);

                Some(PathComponent::Ucs2String(extension_string))
            }
            None => None,
        }
    }

    /// Retrieves the file stem if available.
    pub fn file_stem(&self) -> Result<Option<PathComponent>, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => Self::file_stem_from_byte_string(byte_string),
            PathComponent::OsString(os_string) => {
                let path_buf: PathBuf = PathBuf::from(os_string);

                match path_buf.file_stem() {
                    Some(os_str) => Ok(Some(PathComponent::from(os_str))),
                    None => Ok(None),
                }
            }
            PathComponent::Root => Ok(None),
            PathComponent::String(string) => Ok(Self::file_stem_from_string(string.as_str())),
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(Self::file_stem_from_ucs2_string(ucs2_string))
            }
        }
    }

    /// Retrieves the file stem from a [`&ByteString`] if available.
    #[inline(always)]
    fn file_stem_from_byte_string(
        byte_string: &ByteString,
    ) -> Result<Option<PathComponent>, ErrorTrace> {
        if byte_string.is_empty() {
            return Ok(None);
        }
        let code_points: Vec<u32> = match byte_string.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode byte string");
                return Err(error);
            }
        };
        match code_points[1..]
            .iter()
            .rev()
            .position(|value| *value == 0x0000002e)
        {
            Some(value_index) => {
                // Note that value_index is relative to end of the code points.
                let string_size: usize = code_points.len() - value_index - 1;

                let mut string: String = String::new();

                for code_point in code_points[0..string_size].iter() {
                    match char::from_u32(*code_point) {
                        Some(character) => string.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                Ok(Some(PathComponent::String(string)))
            }
            None => Ok(Some(PathComponent::ByteString(byte_string.clone()))),
        }
    }

    /// Retrieves the file stem from a [`&str`] if available.
    #[inline(always)]
    fn file_stem_from_string(string: &str) -> Option<PathComponent> {
        if string.is_empty() {
            return None;
        }
        match string[1..].chars().rev().position(|value| value == '.') {
            Some(value_index) => {
                // Note that value_index is relative to end of the string.
                let string_size: usize = string.len() - value_index - 1;

                Some(PathComponent::String(string[0..string_size].to_string()))
            }
            None => Some(PathComponent::String(string.to_string())),
        }
    }

    /// Retrieves the file stem from a [`&Ucs2String`] if available.
    #[inline(always)]
    fn file_stem_from_ucs2_string(ucs2_string: &Ucs2String) -> Option<PathComponent> {
        if ucs2_string.is_empty() {
            return None;
        }
        match ucs2_string.elements[1..]
            .iter()
            .rev()
            .position(|value| *value == 0x002e)
        {
            Some(value_index) => {
                // Note that value_index is relative to end of the string.
                let string_size: usize = ucs2_string.len() - value_index - 1;

                Some(PathComponent::Ucs2String(Ucs2String::from(
                    &ucs2_string.elements[0..string_size],
                )))
            }
            None => Some(PathComponent::Ucs2String(ucs2_string.clone())),
        }
    }

    /// Determines if the path component is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            PathComponent::ByteString(byte_string) => byte_string.is_empty(),
            PathComponent::OsString(os_string) => os_string.is_empty(),
            PathComponent::Root => true,
            PathComponent::String(string) => string.is_empty(),
            PathComponent::Ucs2String(ucs2_string) => ucs2_string.is_empty(),
        }
    }

    /// Converts the path component to a `ByteString` with a specific encoding.
    pub fn to_byte_string(&self, encoding: &CharacterEncoding) -> Result<ByteString, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => match byte_string.encode(encoding) {
                Ok(encoded_byte_string) => Ok(encoded_byte_string),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to encode byte string");
                    Err(error)
                }
            },
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                ByteString::from_string_with_encoding(string.as_str(), encoding)
            }
            PathComponent::Root => Ok(ByteString::new_with_encoding(encoding)),
            PathComponent::String(string) => {
                ByteString::from_string_with_encoding(string, encoding)
            }
            PathComponent::Ucs2String(ucs2_string) => {
                let code_points: Vec<u32> = ucs2_string
                    .elements
                    .iter()
                    .map(|element| *element as u32)
                    .collect();

                let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);

                match byte_string.extend_from_codepoints(&code_points) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to encode byte string"
                        );
                        return Err(error);
                    }
                }
                Ok(byte_string)
            }
        }
    }

    /// Converts the path component to a `Ucs2String`.
    pub fn to_ucs2_string(&self) -> Result<Ucs2String, ErrorTrace> {
        let ucs2_string: Ucs2String = match &self {
            PathComponent::ByteString(byte_string) => {
                let mut ucs2_string: Ucs2String = Ucs2String::new();

                let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

                while let Some(result) = character_decoder.next() {
                    match result {
                        Ok(code_points) => {
                            for mut code_point in code_points {
                                if code_point > 0xffff {
                                    code_point -= 0x10000;
                                    ucs2_string
                                        .elements
                                        .push(0xd800 + (code_point >> 10) as u16);
                                    ucs2_string
                                        .elements
                                        .push(0xdc00 + (code_point & 0x03ff) as u16);
                                } else {
                                    ucs2_string.elements.push(code_point as u16);
                                }
                            }
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to decode byte string"
                            );
                            return Err(error);
                        }
                    }
                }
                ucs2_string
            }
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ucs2String::from(string.as_str())
            }
            PathComponent::Root => Ucs2String::new(),
            PathComponent::String(string) => Ucs2String::from(string),
            PathComponent::Ucs2String(ucs2_string) => ucs2_string.clone(),
        };
        Ok(ucs2_string)
    }

    /// Converts the path component to a `Ucs2String` with case folding applied.
    pub fn to_ucs2_string_with_case_folding(
        &self,
        case_folding_mappings: &HashMap<u16, u16>,
    ) -> Result<Ucs2String, ErrorTrace> {
        let ucs2_string: Ucs2String = match &self {
            PathComponent::ByteString(byte_string) => {
                let mut ucs2_string: Ucs2String = Ucs2String::new();

                let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

                while let Some(result) = character_decoder.next() {
                    match result {
                        Ok(code_points) => {
                            for code_point in code_points {
                                if code_point > 0xffff {
                                    return Err(keramics_core::error_trace_new!(
                                        "Unable to encode UCS-2 string - code point outside of supported range"
                                    ));
                                }
                                let folded_code_point: u16 =
                                    match case_folding_mappings.get(&(code_point as u16)) {
                                        Some(folded_code_point) => *folded_code_point,
                                        None => code_point as u16,
                                    };
                                ucs2_string.elements.push(folded_code_point);
                            }
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to decode byte string"
                            );
                            return Err(error);
                        }
                    }
                }
                ucs2_string
            }
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ucs2String::from_string_with_case_folding(string.as_str(), case_folding_mappings)
            }
            PathComponent::Root => Ucs2String::new(),
            PathComponent::String(string) => {
                Ucs2String::from_string_with_case_folding(string.as_str(), case_folding_mappings)
            }
            PathComponent::Ucs2String(ucs2_string) => {
                ucs2_string.new_with_case_folding(case_folding_mappings)
            }
        };
        Ok(ucs2_string)
    }
}

impl From<ByteString> for PathComponent {
    /// Converts a [`ByteString`] into a [`PathComponent`]
    fn from(byte_string: ByteString) -> Self {
        Self::ByteString(byte_string)
    }
}

impl From<&ByteString> for PathComponent {
    /// Converts a [`&ByteString`] into a [`PathComponent`]
    fn from(byte_string: &ByteString) -> Self {
        Self::ByteString(byte_string.clone())
    }
}

impl From<OsString> for PathComponent {
    /// Converts a [`OsString`] into a [`PathComponent`]
    fn from(os_string: OsString) -> Self {
        Self::OsString(os_string)
    }
}

impl From<&OsStr> for PathComponent {
    /// Converts a [`&OsStr`] into a [`PathComponent`]
    fn from(os_str: &OsStr) -> Self {
        Self::OsString(os_str.to_os_string())
    }
}

impl From<&str> for PathComponent {
    /// Converts a [`&str`] into a [`PathComponent`]
    fn from(string: &str) -> Self {
        Self::String(string.to_string())
    }
}

impl From<String> for PathComponent {
    /// Converts a [`String`] into a [`PathComponent`]
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<&String> for PathComponent {
    /// Converts a [`&String`] into a [`PathComponent`]
    fn from(string: &String) -> Self {
        Self::String(string.clone())
    }
}

impl From<Ucs2String> for PathComponent {
    /// Converts a [`Ucs2String`] into a [`PathComponent`]
    fn from(ucs2_string: Ucs2String) -> Self {
        Self::Ucs2String(ucs2_string)
    }
}

impl From<&Ucs2String> for PathComponent {
    /// Converts a [`&Ucs2String`] into a [`PathComponent`]
    fn from(ucs2_string: &Ucs2String) -> Self {
        Self::Ucs2String(ucs2_string.clone())
    }
}

impl PartialEq<str> for PathComponent {
    /// Detemines if a [`PathComponent`] is equal to a [`str`]
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        Self::eq(self, &other)
    }
}

impl PartialEq<&str> for PathComponent {
    /// Detemines if a [`PathComponent`] is equal to a [`&str`]
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        match self {
            PathComponent::ByteString(byte_string) => ByteString::eq(byte_string, other),
            PathComponent::OsString(os_string) => OsString::eq(os_string, other),
            PathComponent::Root => other.is_empty(),
            PathComponent::String(string) => String::eq(string, other),
            PathComponent::Ucs2String(ucs2_string) => Ucs2String::eq(ucs2_string, other),
        }
    }
}

impl fmt::Display for PathComponent {
    /// Formats the path for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathComponent::ByteString(byte_string) => byte_string.fmt(formatter),
            PathComponent::OsString(os_string) => write!(formatter, "{}", os_string.display()),
            PathComponent::Root => write!(formatter, ""),
            PathComponent::String(string) => string.fmt(formatter),
            PathComponent::Ucs2String(ucs2_string) => ucs2_string.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    #[test]
    fn test_extension_with_byte_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from(""));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from(".file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("file.txt"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, Some(PathComponent::from("txt")));

        Ok(())
    }

    #[test]
    fn test_extension_with_os_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::OsString(OsString::from(""));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::OsString(OsString::from("file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::OsString(OsString::from(".file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::OsString(OsString::from("file.txt"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, Some(PathComponent::OsString(OsString::from("txt"))));

        Ok(())
    }

    #[test]
    fn test_extension_with_root() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Root;
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_extension_with_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::from("");
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file");
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from(".file");
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file.txt");
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, Some(PathComponent::from("txt")));

        Ok(())
    }

    #[test]
    fn test_extension_with_ucs2_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from(""));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from(".file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("file.txt"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from("txt")))
        );
        Ok(())
    }

    #[test]
    fn test_file_stem_with_byte_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from(""));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::ByteString(ByteString::from("file")))
        );

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from(".file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::ByteString(ByteString::from(".file")))
        );

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("file.txt"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, Some(PathComponent::from("file")));

        Ok(())
    }

    #[test]
    fn test_file_stem_with_os_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::OsString(OsString::from(""));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::OsString(OsString::from("file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::OsString(OsString::from("file")))
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from(".file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::OsString(OsString::from(".file")))
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("file.txt"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::OsString(OsString::from("file")))
        );

        Ok(())
    }

    #[test]
    fn test_file_stem_with_root() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Root;
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_file_stem_with_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::from("");
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file");
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, Some(PathComponent::from("file")));

        let path_component: PathComponent = PathComponent::from(".file");
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, Some(PathComponent::from(".file")));

        let path_component: PathComponent = PathComponent::from("file.txt");
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, Some(PathComponent::from("file")));

        Ok(())
    }

    #[test]
    fn test_file_stem_with_ucs2_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from(""));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from("file")))
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from(".file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from(".file")))
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("file.txt"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from("file")))
        );
        Ok(())
    }

    #[test]
    fn test_is_empty() {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        assert_eq!(path_component.is_empty(), false);

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        assert_eq!(path_component.is_empty(), false);

        let path_component: PathComponent = PathComponent::Root;
        assert_eq!(path_component.is_empty(), true);

        let path_component: PathComponent = PathComponent::from("test");
        assert_eq!(path_component.is_empty(), false);

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        assert_eq!(path_component.is_empty(), false);
    }

    #[test]
    fn test_to_byte_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let byte_string: ByteString =
            path_component.to_byte_string(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let byte_string: ByteString =
            path_component.to_byte_string(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::Root;
        let byte_string: ByteString =
            path_component.to_byte_string(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![]
            }
        );

        let path_component: PathComponent = PathComponent::from("test");
        let byte_string: ByteString =
            path_component.to_byte_string(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let byte_string: ByteString =
            path_component.to_byte_string(&CharacterEncoding::Iso8859_1)?;
        assert_eq!(
            byte_string,
            ByteString {
                encoding: CharacterEncoding::Iso8859_1,
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        Ok(())
    }

    #[test]
    fn test_to_ucs2_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::Root;
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(ucs2_string, Ucs2String { elements: vec![] });

        let path_component: PathComponent = PathComponent::from("test");
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x74, 0x65, 0x73, 0x74]
            }
        );
        Ok(())
    }

    #[test]
    fn test_to_ucs2_string_with_case_folding() -> Result<(), ErrorTrace> {
        let case_folding_mappings: HashMap<u16, u16> = UCS2_CASE_MAPPINGS
            .into_iter()
            .collect::<HashMap<u16, u16>>();

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let ucs2_string: Ucs2String =
            path_component.to_ucs2_string_with_case_folding(&case_folding_mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x54, 0x45, 0x53, 0x54]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let ucs2_string: Ucs2String =
            path_component.to_ucs2_string_with_case_folding(&case_folding_mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x54, 0x45, 0x53, 0x54]
            }
        );

        let path_component: PathComponent = PathComponent::Root;
        let ucs2_string: Ucs2String =
            path_component.to_ucs2_string_with_case_folding(&case_folding_mappings)?;
        assert_eq!(ucs2_string, Ucs2String { elements: vec![] });

        let path_component: PathComponent = PathComponent::from("test");
        let ucs2_string: Ucs2String =
            path_component.to_ucs2_string_with_case_folding(&case_folding_mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x54, 0x45, 0x53, 0x54]
            }
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let ucs2_string: Ucs2String =
            path_component.to_ucs2_string_with_case_folding(&case_folding_mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x54, 0x45, 0x53, 0x54]
            }
        );
        Ok(())
    }

    #[test]
    fn test_from_os_str() {
        let os_str: &OsStr = OsStr::new("test");
        let path_component: PathComponent = PathComponent::from(os_str);

        assert_eq!(
            path_component,
            PathComponent::OsString(OsString::from("test"))
        );
    }

    #[test]
    fn test_from_str() {
        let path_component: PathComponent = PathComponent::from("test");

        assert_eq!(path_component, PathComponent::String(String::from("test")));
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("test");
        let path_component: PathComponent = PathComponent::from(&string);

        assert_eq!(path_component, PathComponent::String(String::from("test")));
    }

    // TODO: test eq str
    // TODO: test eq &str

    #[test]
    fn test_to_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let string: String = path_component.to_string();
        assert_eq!(string, "test");

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let string: String = path_component.to_string();
        assert_eq!(string, "test");

        let path_component: PathComponent = PathComponent::Root;
        let string: String = path_component.to_string();
        assert_eq!(string, "");

        let path_component: PathComponent = PathComponent::from("test");
        let string: String = path_component.to_string();
        assert_eq!(string, "test");

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let string: String = path_component.to_string();
        assert_eq!(string, "test");

        Ok(())
    }
}
