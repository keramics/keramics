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

use std::fmt;

use super::errors::ParseError;
use super::{bytes_to_u16_be, bytes_to_u16_le, bytes_to_u32_be, bytes_to_u32_le};

/// Universally unique identifier (UUID).
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Uuid {
    pub part1: u32,
    pub part2: u16,
    pub part3: u16,
    pub part4: u16,
    pub part5: u64,
}

impl Uuid {
    /// Creates a new UUID.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads a big-endian UUID from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let part5_upper: u16 = bytes_to_u16_be!(data, 10);
        let part5_lower: u32 = bytes_to_u32_be!(data, 12);
        Self {
            part1: bytes_to_u32_be!(data, 0),
            part2: bytes_to_u16_be!(data, 4),
            part3: bytes_to_u16_be!(data, 6),
            part4: bytes_to_u16_be!(data, 8),
            part5: ((part5_upper as u64) << 32) | (part5_lower as u64),
        }
    }

    /// Reads a little-endian UUID from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let part5_upper: u16 = bytes_to_u16_be!(data, 10);
        let part5_lower: u32 = bytes_to_u32_be!(data, 12);
        Self {
            part1: bytes_to_u32_le!(data, 0),
            part2: bytes_to_u16_le!(data, 4),
            part3: bytes_to_u16_le!(data, 6),
            part4: bytes_to_u16_be!(data, 8),
            part5: ((part5_upper as u64) << 32) | (part5_lower as u64),
        }
    }

    /// Determines if the UUID is the Max (or Omni) UUID (ffffffff-ffff-ffff-ffff-ffffffffffff)
    pub fn is_max(&self) -> bool {
        self.part1 == 0xffffffff
            && self.part2 == 0xffff
            && self.part3 == 0xffff
            && self.part4 == 0xffff
            && self.part5 == 0xffffffffffff
    }

    /// Determines if the UUID is the Nil UUID (00000000-0000-0000-0000-000000000000)
    pub fn is_nil(&self) -> bool {
        self.part1 == 0 && self.part2 == 0 && self.part3 == 0 && self.part4 == 0 && self.part5 == 0
    }

    /// Reads an UUID from a string.
    pub fn from_string(&mut self, mut string: &str) -> Result<(), ParseError> {
        let mut string_length: usize = string.len();

        if string.starts_with("{") && string.ends_with("}") {
            string = &string[1..string_length - 1];
            string_length -= 2;
        }
        if string_length != 36 {
            return Err(ParseError::new(format!("Unsupported string length")));
        }
        if &string[8..9] != "-"
            || &string[13..14] != "-"
            || &string[18..19] != "-"
            || &string[23..24] != "-"
        {
            return Err(ParseError::new(format!("Unsupported string")));
        }
        self.part1 = match u32::from_str_radix(&string[0..8], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(ParseError::new(format!(
                    "Unable to parse part1: {}",
                    &string[0..8]
                )));
            }
        };
        self.part2 = match u16::from_str_radix(&string[9..13], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(ParseError::new(format!(
                    "Unable to parse part2: {}",
                    &string[9..13]
                )));
            }
        };
        self.part3 = match u16::from_str_radix(&string[14..18], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(ParseError::new(format!(
                    "Unable to parse part3: {}",
                    &string[14..18]
                )));
            }
        };
        self.part4 = match u16::from_str_radix(&string[19..23], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(ParseError::new(format!(
                    "Unable to parse part4: {}",
                    &string[19..24]
                )));
            }
        };
        self.part5 = match u64::from_str_radix(&string[24..36], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(ParseError::new(format!(
                    "Unable to parse part5: {}",
                    &string[24..36]
                )));
            }
        };
        Ok(())
    }

    /// Retrieves the string representation of an UUID.
    pub fn to_string(&self) -> String {
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.part1, self.part2, self.part3, self.part4, self.part5,
        )
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 16] = [
            0xb6, 0x1f, 0x53, 0xca, 0xa7, 0x86, 0x45, 0x28, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_be_bytes(&test_data);
        assert_eq!(uuid.part1, 0xb61f53ca);
        assert_eq!(uuid.part2, 0xa786);
        assert_eq!(uuid.part3, 0x4528);
        assert_eq!(uuid.part4, 0x90e2);
        assert_eq!(uuid.part5, 0x55ba791a1c4c);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 16] = [
            0xca, 0x53, 0x1f, 0xb6, 0x86, 0xa7, 0x28, 0x45, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        assert_eq!(uuid.part1, 0xb61f53ca);
        assert_eq!(uuid.part2, 0xa786);
        assert_eq!(uuid.part3, 0x4528);
        assert_eq!(uuid.part4, 0x90e2);
        assert_eq!(uuid.part5, 0x55ba791a1c4c);
    }

    #[test]
    fn test_is_max() {
        let test_data: [u8; 16] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        assert_eq!(uuid.is_max(), true);
    }

    #[test]
    fn test_is_not_max() {
        let test_data: [u8; 16] = [
            0xca, 0x53, 0x1f, 0xb6, 0x86, 0xa7, 0x28, 0x45, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        assert_eq!(uuid.is_max(), false);
    }

    #[test]
    fn test_is_nil() {
        let uuid: Uuid = Uuid::new();
        assert_eq!(uuid.is_nil(), true);
    }

    #[test]
    fn test_is_not_nil() {
        let test_data: [u8; 16] = [
            0xca, 0x53, 0x1f, 0xb6, 0x86, 0xa7, 0x28, 0x45, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        assert_eq!(uuid.is_nil(), false);
    }

    #[test]
    fn test_from_string() -> Result<(), ParseError> {
        let mut uuid: Uuid = Uuid::new();
        uuid.from_string("{b61f53ca-a786-4528-90e2-55ba791a1c4c}")?;

        assert_eq!(uuid.part1, 0xb61f53ca);
        assert_eq!(uuid.part2, 0xa786);
        assert_eq!(uuid.part3, 0x4528);
        assert_eq!(uuid.part4, 0x90e2);
        assert_eq!(uuid.part5, 0x55ba791a1c4c);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let test_data: [u8; 16] = [
            0xca, 0x53, 0x1f, 0xb6, 0x86, 0xa7, 0x28, 0x45, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        let uuid_string: String = uuid.to_string();
        assert_eq!(uuid_string, "b61f53ca-a786-4528-90e2-55ba791a1c4c");
    }
}
