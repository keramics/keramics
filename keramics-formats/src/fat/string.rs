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

use keramics_types::{ByteString, Ucs2String};

use crate::path_component::PathComponent;

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

impl From<&PathComponent> for FatString {
    /// Converts a [`&PathComponent`] into a [`FatString`]
    #[inline(always)]
    fn from(path_component: &PathComponent) -> Self {
        match path_component {
            PathComponent::ByteString(byte_string) => Self::ByteString(byte_string.clone()),
            PathComponent::Ucs2String(ucs2_string) => Self::Ucs2String(ucs2_string.clone()),
            PathComponent::String(string) => Self::Ucs2String(Ucs2String::from(string)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let fat_string: FatString = FatString::from("FAT string");

        assert_eq!(
            fat_string,
            FatString::Ucs2String(Ucs2String::from("FAT string"))
        );
    }

    #[test]
    fn test_from_string() {
        let test_string: String = String::from("FAT string");
        let fat_string: FatString = FatString::from(&test_string);

        assert_eq!(
            fat_string,
            FatString::Ucs2String(Ucs2String::from("FAT string"))
        );
    }

    // TODO: add tests.
}
