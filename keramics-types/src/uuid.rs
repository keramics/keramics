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

use keramics_core::ErrorTrace;

use super::{bytes_to_u16_be, bytes_to_u16_le, bytes_to_u32_be, bytes_to_u32_le};

/// Universally unique identifier (UUID).
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// Creates a new UUID from a big-endian byte sequence.
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

    /// Creates a new UUID from a little-endian byte sequence.
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

    /// Creates a new UUID from a string.
    pub fn from_string(mut string: &str) -> Result<Self, ErrorTrace> {
        let mut string_length: usize = string.len();

        if string.starts_with("{") && string.ends_with("}") {
            string = &string[1..string_length - 1];
            string_length -= 2;
        }
        if string_length != 36 {
            return Err(keramics_core::error_trace_new!("Unsupported string length"));
        }
        if &string[8..9] != "-"
            || &string[13..14] != "-"
            || &string[18..19] != "-"
            || &string[23..24] != "-"
        {
            return Err(keramics_core::error_trace_new!("Unsupported string"));
        }
        let part1: u32 = match u32::from_str_radix(&string[0..8], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to parse part1: {}",
                    &string[0..8]
                )));
            }
        };
        let part2: u16 = match u16::from_str_radix(&string[9..13], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to parse part2: {}",
                    &string[9..13]
                )));
            }
        };
        let part3: u16 = match u16::from_str_radix(&string[14..18], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to parse part3: {}",
                    &string[14..18]
                )));
            }
        };
        let part4: u16 = match u16::from_str_radix(&string[19..23], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to parse part4: {}",
                    &string[19..24]
                )));
            }
        };
        let part5: u64 = match u64::from_str_radix(&string[24..36], 16) {
            Ok(value) => value,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to parse part5: {}",
                    &string[24..36]
                )));
            }
        };
        Ok(Self {
            part1,
            part2,
            part3,
            part4,
            part5,
        })
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
}

impl fmt::Display for Uuid {
    /// Formats the UUID for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.part1, self.part2, self.part3, self.part4, self.part5,
        )
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
        assert_eq!(
            uuid,
            Uuid {
                part1: 0xb61f53ca,
                part2: 0xa786,
                part3: 0x4528,
                part4: 0x90e2,
                part5: 0x55ba791a1c4c,
            }
        );
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 16] = [
            0xca, 0x53, 0x1f, 0xb6, 0x86, 0xa7, 0x28, 0x45, 0x90, 0xe2, 0x55, 0xba, 0x79, 0x1a,
            0x1c, 0x4c,
        ];

        let uuid: Uuid = Uuid::from_le_bytes(&test_data);
        assert_eq!(
            uuid,
            Uuid {
                part1: 0xb61f53ca,
                part2: 0xa786,
                part3: 0x4528,
                part4: 0x90e2,
                part5: 0x55ba791a1c4c,
            }
        );
    }

    #[test]
    fn test_from_string() -> Result<(), ErrorTrace> {
        let uuid: Uuid = Uuid::from_string("{b61f53ca-a786-4528-90e2-55ba791a1c4c}")?;
        assert_eq!(
            uuid,
            Uuid {
                part1: 0xb61f53ca,
                part2: 0xa786,
                part3: 0x4528,
                part4: 0x90e2,
                part5: 0x55ba791a1c4c,
            }
        );
        Ok(())
    }

    #[test]
    fn test_is_max() {
        let uuid: Uuid = Uuid {
            part1: 0xffffffff,
            part2: 0xffff,
            part3: 0xffff,
            part4: 0xffff,
            part5: 0xffffffffffff,
        };
        assert_eq!(uuid.is_max(), true);
    }

    #[test]
    fn test_is_not_max() {
        let uuid: Uuid = Uuid {
            part1: 0xb61f53ca,
            part2: 0xa786,
            part3: 0x4528,
            part4: 0x90e2,
            part5: 0x55ba791a1c4c,
        };
        assert_eq!(uuid.is_max(), false);
    }

    #[test]
    fn test_is_nil() {
        let uuid: Uuid = Uuid::new();
        assert_eq!(uuid.is_nil(), true);
    }

    #[test]
    fn test_is_not_nil() {
        let uuid: Uuid = Uuid {
            part1: 0xb61f53ca,
            part2: 0xa786,
            part3: 0x4528,
            part4: 0x90e2,
            part5: 0x55ba791a1c4c,
        };
        assert_eq!(uuid.is_nil(), false);
    }

    #[test]
    fn test_to_string() {
        let uuid: Uuid = Uuid {
            part1: 0xb61f53ca,
            part2: 0xa786,
            part3: 0x4528,
            part4: 0x90e2,
            part5: 0x55ba791a1c4c,
        };

        let uuid_string: String = uuid.to_string();
        assert_eq!(uuid_string, "b61f53ca-a786-4528-90e2-55ba791a1c4c");
    }
}
