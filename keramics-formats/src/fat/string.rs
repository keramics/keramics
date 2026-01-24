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

use keramics_types::{ByteString, Ucs2String};

/// File Allocation Table (FAT) string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FatString {
    /// Byte string, used by short name.
    ByteString(ByteString),

    /// UCS-2 string, used by long name.
    Ucs2String(Ucs2String),
}

impl From<&str> for FatString {
    /// Converts a [`&str`] into a [`FatString`]
    #[inline(always)]
    fn from(string: &str) -> Self {
        Self::Ucs2String(Ucs2String::from(string))
    }
}

impl fmt::Display for FatString {
    /// Formats the byte string for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FatString::ByteString(byte_string) => byte_string.fmt(formatter),
            FatString::Ucs2String(ucs2_string) => ucs2_string.fmt(formatter),
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
            FatString::Ucs2String(Ucs2String {
                elements: vec![
                    0x0046, 0x0041, 0x0054, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067,
                ]
            })
        );
    }

    #[test]
    fn test_to_string() {
        let fat_string: FatString = FatString::Ucs2String(Ucs2String {
            elements: vec![
                0x0046, 0x0041, 0x0054, 0x0020, 0x0073, 0x0074, 0x0072, 0x0069, 0x006e, 0x0067,
            ],
        });

        let string: String = fat_string.to_string();
        assert_eq!(string, "FAT string");
    }
}
