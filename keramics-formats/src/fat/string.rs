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
use std::sync::Arc;

use keramics_core::ErrorTrace;
use keramics_encodings::CharacterDecoder;
use keramics_types::{ByteString, Ucs2String};

/// File Allocation Table (FAT) string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FatString {
    /// Byte string, used by short name.
    ByteString(ByteString),

    /// UCS-2 string, used by long name.
    Ucs2String(Ucs2String),
}

impl FatString {
    /// Creates a new string.
    pub fn new() -> Self {
        Self::Ucs2String(Ucs2String::new())
    }

    /// Retrieves the lookup name.
    pub(super) fn get_lookup_name(
        &self,
        case_folding_mappings: &Arc<HashMap<u16, u16>>,
    ) -> Result<Ucs2String, ErrorTrace> {
        let lookup_name: Ucs2String = match &self {
            FatString::ByteString(byte_string) => {
                let mut character_decoder: CharacterDecoder = byte_string.get_character_decoder();

                let mut lookup_name: Ucs2String = Ucs2String::new();

                while let Some(result) = character_decoder.next() {
                    match result {
                        Ok(code_point) => {
                            if code_point > 0xffff {
                                return Err(keramics_core::error_trace_new!(
                                    "Unable to encode string - code point outside of UCS-2 range"
                                ));
                            }
                            lookup_name.elements.push(code_point as u16);
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
                lookup_name
            }
            FatString::Ucs2String(ucs2_string) => {
                Ucs2String::new_with_case_folding(ucs2_string, case_folding_mappings)
            }
        };
        Ok(lookup_name)
    }

    /// Determines if the string is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            FatString::ByteString(byte_string) => byte_string.is_empty(),
            FatString::Ucs2String(ucs2_string) => ucs2_string.is_empty(),
        }
    }

    /// Retrieves the length (or size) of the string.
    pub fn len(&self) -> usize {
        match self {
            FatString::ByteString(byte_string) => byte_string.len(),
            FatString::Ucs2String(ucs2_string) => ucs2_string.len(),
        }
    }

    /// Converts the `FatString` to `String`.
    pub fn to_string(&self) -> String {
        match self {
            FatString::ByteString(byte_string) => byte_string.to_string(),
            FatString::Ucs2String(ucs2_string) => ucs2_string.to_string(),
        }
    }
}

impl From<&str> for FatString {
    /// Converts a [`&str`] into a [`FatString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::Ucs2String(Ucs2String::from(string))
    }
}

impl From<&String> for FatString {
    /// Converts a [`&String`] into a [`FatString`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::Ucs2String(Ucs2String::from(string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests.
}
