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

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterEncoding;
use keramics_types::{
    ByteString, Ucs2CharacterMappings, Ucs2String, Utf16CharacterMappings, Utf16String,
};

use super::path_character_mappings::PathCharacterMappings;

/// Path component for file resolver.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PathComponent {
    ByteString(ByteString),
    Current,
    OsString(OsString),
    Parent,
    Root,
    String(String),
    Ucs2String(Ucs2String),
    Utf16String(Utf16String),
}

impl PathComponent {
    /// Creates a new path component with case folding applied.
    pub fn new_with_case_folding(
        &self,
        case_folding_mappings: &PathCharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => {
                match case_folding_mappings {
                    PathCharacterMappings::Ucs2(mappings) => {
                        let elements: Vec<u8> = byte_string
                            .elements
                            .iter()
                            .map(|element| mappings.get(&(*element as u16)) as u8)
                            .collect();
                        Ok(PathComponent::ByteString(ByteString {
                            encoding: byte_string.encoding.clone(),
                            elements,
                        }))
                    }
                    PathCharacterMappings::Utf16(_) => Ok(self.clone()),
                }
            }
            PathComponent::Current | PathComponent::Parent | PathComponent::Root => Ok(self.clone()),
            PathComponent::OsString(os_string) => Ok(PathComponent::OsString(os_string.clone())),
            PathComponent::String(string) => Ok(PathComponent::String(string.to_lowercase())),
            PathComponent::Ucs2String(ucs2_string) => match case_folding_mappings {
                PathCharacterMappings::Ucs2(mappings) => Ok(PathComponent::Ucs2String(
                    ucs2_string.new_with_case_folding(mappings),
                )),
                PathCharacterMappings::Utf16(_) => Ok(PathComponent::Ucs2String(ucs2_string.clone())),
            },
            PathComponent::Utf16String(utf16_string) => match case_folding_mappings {
                PathCharacterMappings::Ucs2(_) => Ok(PathComponent::Utf16String(utf16_string.clone())),
                PathCharacterMappings::Utf16(mappings) => Ok(PathComponent::Utf16String(
                    utf16_string.new_with_case_folding(mappings)?,
                )),
            },
        }
    }

    /// Retrieves the extension if available.
    pub fn extension(&self) -> Result<Option<PathComponent>, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => Self::extension_from_byte_string(byte_string),
            PathComponent::Current | PathComponent::Parent | PathComponent::Root => Ok(None),
            PathComponent::OsString(os_string) => {
                let path_buf: PathBuf = PathBuf::from(os_string);

                match path_buf.extension() {
                    Some(os_str) => Ok(Some(PathComponent::from(os_str))),
                    None => Ok(None),
                }
            }
            PathComponent::String(string) => Ok(Self::extension_from_string(string.as_str())),
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(Self::extension_from_ucs2_string(ucs2_string))
            }
            PathComponent::Utf16String(utf16_string) => {
                Ok(Self::extension_from_utf16_string(utf16_string))
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
            .rposition(|value| *value == 0x0000002e)
        {
            Some(value_index) => {
                let mut extension_string: String = String::new();

                for code_point in code_points[value_index + 2..].iter() {
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
            .rposition(|value| *value == 0x002e)
        {
            Some(value_index) => {
                let extension_string: Ucs2String =
                    Ucs2String::from(&ucs2_string.elements[value_index + 2..]);

                Some(PathComponent::Ucs2String(extension_string))
            }
            None => None,
        }
    }

    /// Retrieves the extension from a [`&Utf16String`] if available.
    #[inline(always)]
    fn extension_from_utf16_string(utf16_string: &Utf16String) -> Option<PathComponent> {
        if utf16_string.is_empty() {
            return None;
        }
        match utf16_string.elements[1..]
            .iter()
            .rposition(|value| *value == 0x002e)
        {
            Some(value_index) => {
                let extension_string: Utf16String =
                    Utf16String::from(&utf16_string.elements[value_index + 2..]);

                Some(PathComponent::Utf16String(extension_string))
            }
            None => None,
        }
    }

    /// Retrieves the file stem if available.
    pub fn file_stem(&self) -> Result<Option<PathComponent>, ErrorTrace> {
        match self {
            PathComponent::ByteString(byte_string) => Self::file_stem_from_byte_string(byte_string),
            PathComponent::Current | PathComponent::Parent | PathComponent::Root => Ok(None),
            PathComponent::OsString(os_string) => {
                let path_buf: PathBuf = PathBuf::from(os_string);

                match path_buf.file_stem() {
                    Some(os_str) => Ok(Some(PathComponent::from(os_str))),
                    None => Ok(None),
                }
            }
            PathComponent::String(string) => Ok(Self::file_stem_from_string(string.as_str())),
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(Self::file_stem_from_ucs2_string(ucs2_string))
            }
            PathComponent::Utf16String(utf16_string) => {
                Ok(Self::file_stem_from_utf16_string(utf16_string))
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
            .rposition(|value| *value == 0x0000002e)
        {
            Some(value_index) => {
                let mut string: String = String::new();

                for code_point in code_points[0..value_index + 1].iter() {
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
            .rposition(|value| *value == 0x002e)
        {
            Some(value_index) => Some(PathComponent::Ucs2String(Ucs2String::from(
                &ucs2_string.elements[0..value_index + 1],
            ))),
            None => Some(PathComponent::Ucs2String(ucs2_string.clone())),
        }
    }

    /// Retrieves the file stem from a [`&Utf16String`] if available.
    #[inline(always)]
    fn file_stem_from_utf16_string(utf16_string: &Utf16String) -> Option<PathComponent> {
        if utf16_string.is_empty() {
            return None;
        }
        match utf16_string.elements[1..]
            .iter()
            .rposition(|value| *value == 0x002e)
        {
            Some(value_index) => Some(PathComponent::Utf16String(Utf16String::from(
                &utf16_string.elements[0..value_index + 1],
            ))),
            None => Some(PathComponent::Utf16String(utf16_string.clone())),
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
            PathComponent::Current => ByteString::from_string_with_encoding(encoding, "."),
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                ByteString::from_string_with_encoding(encoding, string.as_str())
            }
            PathComponent::Parent => ByteString::from_string_with_encoding(encoding, ".."),
            PathComponent::Root => Ok(ByteString::new_with_encoding(encoding)),
            PathComponent::String(string) => {
                ByteString::from_string_with_encoding(encoding, string)
            }
            PathComponent::Ucs2String(ucs2_string) => {
                match ByteString::from_ucs2_string_with_encoding(encoding, ucs2_string) {
                    Ok(encoded_byte_string) => Ok(encoded_byte_string),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to create byte string from UCS-2 string"
                        );
                        Err(error)
                    }
                }
            }
            PathComponent::Utf16String(utf16_string) => {
                match ByteString::from_utf16_string_with_encoding(encoding, utf16_string) {
                    Ok(encoded_byte_string) => Ok(encoded_byte_string),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to create byte string from UTF-16 string"
                        );
                        Err(error)
                    }
                }
            }
        }
    }

    /// Converts the path component to a `Ucs2String`.
    pub fn to_ucs2_string(&self) -> Result<Ucs2String, ErrorTrace> {
        let result: Result<Ucs2String, ErrorTrace> = match &self {
            PathComponent::ByteString(byte_string) => Ucs2String::from_byte_string(byte_string),
            PathComponent::Current => Ok(Ucs2String::from(".")),
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ok(Ucs2String::from(string.as_str()))
            }
            PathComponent::Parent => Ok(Ucs2String::from("..")),
            PathComponent::Root => Ok(Ucs2String::new()),
            PathComponent::String(string) => Ok(Ucs2String::from(string)),
            PathComponent::Ucs2String(ucs2_string) => Ok(ucs2_string.clone()),
            PathComponent::Utf16String(utf16_string) => Ucs2String::from_utf16_string(utf16_string),
        };
        match result {
            Ok(ucs2_string) => Ok(ucs2_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create UCS-2 string");
                Err(error)
            }
        }
    }

    /// Converts the path component to a `Ucs2String` with case folding applied.
    pub fn to_ucs2_string_with_case_folding(
        &self,
        mappings: &Ucs2CharacterMappings,
    ) -> Result<Ucs2String, ErrorTrace> {
        let result: Result<Ucs2String, ErrorTrace> = match &self {
            PathComponent::ByteString(byte_string) => {
                Ucs2String::from_byte_string_with_case_folding(&byte_string, mappings)
            }
            PathComponent::Current => Ok(Ucs2String::from(".")),
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ucs2String::from_string_with_case_folding(string.as_str(), mappings)
            }
            PathComponent::Parent => Ok(Ucs2String::from("..")),
            PathComponent::Root => Ok(Ucs2String::new()),
            PathComponent::String(string) => {
                Ucs2String::from_string_with_case_folding(string.as_str(), mappings)
            }
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(ucs2_string.new_with_case_folding(mappings))
            }
            PathComponent::Utf16String(utf16_string) => {
                Ucs2String::from_utf16_string_with_case_folding(utf16_string, mappings)
            }
        };
        match result {
            Ok(ucs2_string) => Ok(ucs2_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to create UCS-2 string with case folding"
                );
                Err(error)
            }
        }
    }

    /// Converts the path component to a `Utf16String`.
    pub fn to_utf16_string(&self) -> Result<Utf16String, ErrorTrace> {
        let result: Result<Utf16String, ErrorTrace> = match &self {
            PathComponent::ByteString(byte_string) => Utf16String::from_byte_string(byte_string),
            PathComponent::Current => Ok(Utf16String::from(".")),
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ok(Utf16String::from(string.as_str()))
            }
            PathComponent::Parent => Ok(Utf16String::from("..")),
            PathComponent::Root => Ok(Utf16String::new()),
            PathComponent::String(string) => Ok(Utf16String::from(string)),
            PathComponent::Ucs2String(ucs2_string) => {
                Ok(Utf16String::from_ucs2_string(ucs2_string))
            }
            PathComponent::Utf16String(utf16_string) => Ok(utf16_string.clone()),
        };
        match result {
            Ok(utf16_string) => Ok(utf16_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create UTF-16 string");
                Err(error)
            }
        }
    }

    /// Converts the path component to a `Utf16String` with case folding applied.
    pub fn to_utf16_string_with_case_folding(
        &self,
        mappings: &Utf16CharacterMappings,
    ) -> Result<Utf16String, ErrorTrace> {
        let result: Result<Utf16String, ErrorTrace> = match &self {
            PathComponent::ByteString(byte_string) => {
                Utf16String::from_byte_string_with_case_folding(&byte_string, mappings)
            }
            PathComponent::Current => Ok(Utf16String::from(".")),
            PathComponent::OsString(os_string) => {
                let string: String = os_string.display().to_string();

                Ok(Utf16String::from_string_with_case_folding(
                    string.as_str(),
                    mappings,
                ))
            }
            PathComponent::Parent => Ok(Utf16String::from("..")),
            PathComponent::Root => Ok(Utf16String::new()),
            PathComponent::String(string) => Ok(Utf16String::from_string_with_case_folding(
                string.as_str(),
                mappings,
            )),
            PathComponent::Ucs2String(ucs2_string) => Ok(
                Utf16String::from_ucs2_string_with_case_folding(ucs2_string, mappings),
            ),
            PathComponent::Utf16String(utf16_string) => {
                utf16_string.new_with_case_folding(mappings)
            }
        };
        match result {
            Ok(utf16_string) => Ok(utf16_string),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to create UTF-16 string with case folding"
                );
                Err(error)
            }
        }
    }
}

impl From<ByteString> for PathComponent {
    /// Converts a [`ByteString`] into a [`PathComponent`]
    fn from(byte_string: ByteString) -> Self {
        match byte_string.elements.as_slice() {
            [0x2e] => Self::Current,
            [0x2e, 0x2e] => Self::Parent,
            _ => Self::ByteString(byte_string),
        }
    }
}

impl From<&ByteString> for PathComponent {
    /// Converts a [`&ByteString`] into a [`PathComponent`]
    fn from(byte_string: &ByteString) -> Self {
        match byte_string.elements.as_slice() {
            [0x2e] => Self::Current,
            [0x2e, 0x2e] => Self::Parent,
            _ => Self::ByteString(byte_string.clone()),
        }
    }
}

impl From<OsString> for PathComponent {
    /// Converts a [`OsString`] into a [`PathComponent`]
    fn from(os_string: OsString) -> Self {
        if os_string.eq(".") {
            Self::Current
        } else if os_string.eq("..") {
            Self::Parent
        } else {
            Self::OsString(os_string)
        }
    }
}

impl From<&OsStr> for PathComponent {
    /// Converts a [`&OsStr`] into a [`PathComponent`]
    fn from(os_str: &OsStr) -> Self {
        if os_str.eq(".") {
            Self::Current
        } else if os_str.eq("..") {
            Self::Parent
        } else {
            Self::OsString(os_str.to_os_string())
        }
    }
}

impl From<&str> for PathComponent {
    /// Converts a [`&str`] into a [`PathComponent`]
    fn from(string: &str) -> Self {
        match string {
            "." => Self::Current,
            ".." => Self::Parent,
            _ => Self::String(string.to_string()),
        }
    }
}

impl From<String> for PathComponent {
    /// Converts a [`String`] into a [`PathComponent`]
    fn from(string: String) -> Self {
        match string.as_str() {
            "." => Self::Current,
            ".." => Self::Parent,
            _ => Self::String(string),
        }
    }
}

impl From<&String> for PathComponent {
    /// Converts a [`&String`] into a [`PathComponent`]
    fn from(string: &String) -> Self {
        match string.as_str() {
            "." => Self::Current,
            ".." => Self::Parent,
            _ => Self::String(string.clone()),
        }
    }
}

impl From<Ucs2String> for PathComponent {
    /// Converts a [`Ucs2String`] into a [`PathComponent`]
    fn from(ucs2_string: Ucs2String) -> Self {
        match ucs2_string.elements.as_slice() {
            [0x002e] => Self::Current,
            [0x002e, 0x002e] => Self::Parent,
            _ => Self::Ucs2String(ucs2_string),
        }
    }
}

impl From<&Ucs2String> for PathComponent {
    /// Converts a [`&Ucs2String`] into a [`PathComponent`]
    fn from(ucs2_string: &Ucs2String) -> Self {
        match ucs2_string.elements.as_slice() {
            [0x002e] => Self::Current,
            [0x002e, 0x002e] => Self::Parent,
            _ => Self::Ucs2String(ucs2_string.clone()),
        }
    }
}

impl From<Utf16String> for PathComponent {
    /// Converts a [`Utf16String`] into a [`PathComponent`]
    fn from(utf16_string: Utf16String) -> Self {
        match utf16_string.elements.as_slice() {
            [0x002e] => Self::Current,
            [0x002e, 0x002e] => Self::Parent,
            _ => Self::Utf16String(utf16_string),
        }
    }
}

impl From<&Utf16String> for PathComponent {
    /// Converts a [`&Utf16String`] into a [`PathComponent`]
    fn from(utf16_string: &Utf16String) -> Self {
        match utf16_string.elements.as_slice() {
            [0x002e] => Self::Current,
            [0x002e, 0x002e] => Self::Parent,
            _ => Self::Utf16String(utf16_string.clone()),
        }
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
            PathComponent::Current => *other == ".",
            PathComponent::OsString(os_string) => OsString::eq(os_string, other),
            PathComponent::Parent => *other == "..",
            PathComponent::Root => other.is_empty(),
            PathComponent::String(string) => String::eq(string, other),
            PathComponent::Ucs2String(ucs2_string) => Ucs2String::eq(ucs2_string, other),
            PathComponent::Utf16String(utf16_string) => Utf16String::eq(utf16_string, other),
        }
    }
}

impl fmt::Display for PathComponent {
    /// Formats the path for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathComponent::ByteString(byte_string) => byte_string.fmt(formatter),
            PathComponent::Current => write!(formatter, "."),
            PathComponent::OsString(os_string) => write!(formatter, "{}", os_string.display()),
            PathComponent::Parent => write!(formatter, ".."),
            PathComponent::Root => write!(formatter, ""),
            PathComponent::String(string) => string.fmt(formatter),
            PathComponent::Ucs2String(ucs2_string) => ucs2_string.fmt(formatter),
            PathComponent::Utf16String(utf16_string) => utf16_string.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    #[test]
    fn test_new_with_case_folding() -> Result<(), ErrorTrace> {
        use std::sync::Arc;

        // Ucs2 mappings apply to Ucs2String and ByteString components.
        let mut ucs2_mappings: Ucs2CharacterMappings = Ucs2CharacterMappings::new();
        ucs2_mappings.add(0x0041, 0x0061); // 'A' -> 'a'
        let ucs2_mappings: PathCharacterMappings = PathCharacterMappings::Ucs2(Arc::new(ucs2_mappings));

        let component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("AB"));
        let folded: PathComponent = component.new_with_case_folding(&ucs2_mappings)?;
        assert_eq!(folded, PathComponent::Ucs2String(Ucs2String::from("aB")));

        let component: PathComponent = PathComponent::ByteString(ByteString::from("AB"));
        let folded: PathComponent = component.new_with_case_folding(&ucs2_mappings)?;
        assert_eq!(folded, PathComponent::ByteString(ByteString::from("aB")));

        // String components are folded to lower-case.
        let component: PathComponent = PathComponent::from("A1");
        let folded: PathComponent = component.new_with_case_folding(&ucs2_mappings)?;
        assert_eq!(folded, PathComponent::from("a1"));

        // Root / Parent / Current / OsString / Utf16String are unchanged for Ucs2 mappings.
        let unchanged: [PathComponent; 5] = [
            PathComponent::Root,
            PathComponent::Parent,
            PathComponent::Current,
            PathComponent::OsString(OsString::from("file")),
            PathComponent::Utf16String(Utf16String::from("file")),
        ];
        for component in &unchanged {
            assert_eq!(&component.new_with_case_folding(&ucs2_mappings)?, component);
        }

        // Ucs2 mappings are a no-op on Utf16String components.
        let component: PathComponent = PathComponent::Utf16String(Utf16String::from("aB"));
        let folded: PathComponent = component.new_with_case_folding(&ucs2_mappings)?;
        assert_eq!(folded, PathComponent::Utf16String(Utf16String::from("aB")));

        // Utf16 mappings apply to Utf16String components.
        let mut utf16_mappings: keramics_types::Utf16CharacterMappings =
            keramics_types::Utf16CharacterMappings::new(&[]);
        utf16_mappings.add(0x0061, 0x0041); // 'a' -> 'A'
        let utf16_mappings: PathCharacterMappings = PathCharacterMappings::Utf16(Arc::new(utf16_mappings));

        let component: PathComponent = PathComponent::Utf16String(Utf16String::from("aB"));
        let folded: PathComponent = component.new_with_case_folding(&utf16_mappings)?;
        assert_eq!(folded, PathComponent::Utf16String(Utf16String::from("AB")));

        // The shared UCS2_CASE_MAPPINGS table (lower -> upper) is also usable.
        let case_mappings: Ucs2CharacterMappings = Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());
        let case_mappings: PathCharacterMappings = PathCharacterMappings::Ucs2(Arc::new(case_mappings));
        let component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("aB"));
        let folded: PathComponent = component.new_with_case_folding(&case_mappings)?;
        assert_eq!(folded, PathComponent::Ucs2String(Ucs2String::from("AB")));

        Ok(())
    }

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
    fn test_extension_with_utf16_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from(""));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from("file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from(".file"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(result, None);

        let path_component: PathComponent =
            PathComponent::Utf16String(Utf16String::from("file.txt"));
        let result: Option<PathComponent> = path_component.extension()?;
        assert_eq!(
            result,
            Some(PathComponent::Utf16String(Utf16String::from("txt")))
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
    fn test_file_stem_with_utf16_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from(""));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from("file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Utf16String(Utf16String::from("file")))
        );

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from(".file"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Utf16String(Utf16String::from(".file")))
        );

        let path_component: PathComponent =
            PathComponent::Utf16String(Utf16String::from("file.txt"));
        let result: Option<PathComponent> = path_component.file_stem()?;
        assert_eq!(
            result,
            Some(PathComponent::Utf16String(Utf16String::from("file")))
        );
        Ok(())
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
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
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
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string()?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );
        Ok(())
    }

    #[test]
    fn test_to_ucs2_string_with_case_folding() -> Result<(), ErrorTrace> {
        let mappings: Ucs2CharacterMappings =
            Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice());

        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string_with_case_folding(&mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0054, 0x0045, 0x0053, 0x0054]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string_with_case_folding(&mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0054, 0x0045, 0x0053, 0x0054]
            }
        );

        let path_component: PathComponent = PathComponent::Root;
        let ucs2_string: Ucs2String = path_component.to_ucs2_string_with_case_folding(&mappings)?;
        assert_eq!(ucs2_string, Ucs2String { elements: vec![] });

        let path_component: PathComponent = PathComponent::from("test");
        let ucs2_string: Ucs2String = path_component.to_ucs2_string_with_case_folding(&mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0054, 0x0045, 0x0053, 0x0054]
            }
        );

        let path_component: PathComponent = PathComponent::Ucs2String(Ucs2String::from("test"));
        let ucs2_string: Ucs2String = path_component.to_ucs2_string_with_case_folding(&mappings)?;
        assert_eq!(
            ucs2_string,
            Ucs2String {
                elements: vec![0x0054, 0x0045, 0x0053, 0x0054]
            }
        );
        Ok(())
    }

    #[test]
    fn test_to_utf16_string() -> Result<(), ErrorTrace> {
        let path_component: PathComponent = PathComponent::ByteString(ByteString::from("test"));
        let utf16_string: Utf16String = path_component.to_utf16_string()?;
        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );

        let path_component: PathComponent = PathComponent::OsString(OsString::from("test"));
        let utf16_string: Utf16String = path_component.to_utf16_string()?;
        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );

        let path_component: PathComponent = PathComponent::Root;
        let utf16_string: Utf16String = path_component.to_utf16_string()?;
        assert_eq!(utf16_string, Utf16String { elements: vec![] });

        let path_component: PathComponent = PathComponent::from("test");
        let utf16_string: Utf16String = path_component.to_utf16_string()?;
        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from("test"));
        let utf16_string: Utf16String = path_component.to_utf16_string()?;
        assert_eq!(
            utf16_string,
            Utf16String {
                elements: vec![0x0074, 0x0065, 0x0073, 0x0074]
            }
        );
        Ok(())
    }

    // TODO: add tests for from byte string

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

    // TODO: add tests for from UCS-2 string
    // TODO: add tests for from UTF-16 string

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

        let path_component: PathComponent = PathComponent::Utf16String(Utf16String::from("test"));
        let string: String = path_component.to_string();
        assert_eq!(string, "test");

        Ok(())
    }
}
