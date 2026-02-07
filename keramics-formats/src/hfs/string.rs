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

use std::fmt;

use keramics_core::ErrorTrace;
use keramics_types::{ByteString, Utf16CharacterMappings, Utf16String};

/// Hierarchical File System (HFS) string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HfsString {
    /// Byte string, used by HFS standard.
    ByteString(ByteString),

    /// UTF-16 string, used by HFS extended.
    Utf16String(Utf16String),
}

impl HfsString {
    /// Creates a new string.
    pub(super) fn new() -> Self {
        Self::Utf16String(Utf16String::new())
    }

    /// Creates a new string with case folding applied.
    pub(super) fn new_with_case_folding(
        &self,
        mappings: &Utf16CharacterMappings,
    ) -> Result<Self, ErrorTrace> {
        match self {
            Self::ByteString(byte_string) => {
                // TODO: improve
                let elements: Vec<u8> = byte_string
                    .elements
                    .iter()
                    .map(|element| {
                        if *element >= b'a' && *element <= b'z' {
                            *element - 32
                        } else {
                            *element
                        }
                    })
                    .collect();

                Ok(HfsString::ByteString(ByteString {
                    encoding: byte_string.encoding.clone(),
                    elements,
                }))
            }
            Self::Utf16String(utf16_string) => match utf16_string.new_with_case_folding(mappings) {
                Ok(utf16_string) => Ok(HfsString::Utf16String(utf16_string)),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert to UTF-16 string with case folding"
                    );
                    Err(error)
                }
            },
        }
    }

    /// Determines if the string is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            HfsString::ByteString(byte_string) => byte_string.is_empty(),
            HfsString::Utf16String(utf16_string) => utf16_string.is_empty(),
        }
    }
}

impl From<&str> for HfsString {
    /// Converts a [`&str`] into a [`HfsString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::Utf16String(Utf16String::from(string))
    }
}

impl fmt::Display for HfsString {
    /// Formats the byte string for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HfsString::ByteString(byte_string) => byte_string.fmt(formatter),
            HfsString::Utf16String(utf16_string) => utf16_string.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::hfs::constants::HFS_UTF16_CASE_MAPPINGS;

    #[test]
    fn test_new_with_case_folding() -> Result<(), ErrorTrace> {
        let hfs_string: HfsString = HfsString::from("HFS string");

        let mappings: Utf16CharacterMappings =
            Utf16CharacterMappings::from(HFS_UTF16_CASE_MAPPINGS.as_slice());
        let test_string: HfsString = hfs_string.new_with_case_folding(&mappings)?;

        assert_eq!(
            test_string,
            HfsString::Utf16String(Utf16String {
                elements: vec![
                    0x0068, 0x0066, 0x0073, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067,
                ]
            })
        );
        Ok(())
    }

    // TODO: add tests for is_empty

    #[test]
    fn test_from_str() {
        let hfs_string: HfsString = HfsString::from("HFS string");

        assert_eq!(
            hfs_string,
            HfsString::Utf16String(Utf16String {
                elements: vec![
                    0x0048, 0x0046, 0x0053, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067,
                ]
            })
        );
    }

    #[test]
    fn test_to_string() {
        let hfs_string: HfsString = HfsString::Utf16String(Utf16String {
            elements: vec![
                0x0048, 0x0046, 0x0053, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067,
            ],
        });

        let string: String = hfs_string.to_string();
        assert_eq!(string, "HFS string");
    }
}
