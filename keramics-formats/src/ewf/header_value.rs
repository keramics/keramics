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

use keramics_datetime::PosixTime32;
use keramics_types::{ByteString, Utf16String};

/// Expert Witness Compression Format (EWF) header value.
pub enum EwfHeaderValue {
    Byte(ByteString),
    PosixTime(PosixTime32),
    Utf16(Utf16String),
}

// TODO: add datetime values.

impl EwfHeaderValue {
    /// Creates a new EWF header value from a byte sequence.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        EwfHeaderValue::Byte(ByteString::from(bytes))
    }

    /// Creates a new EWF header value from a UTF-16 sequence.
    pub fn from_utf16(values_16bit: &[u16]) -> Self {
        EwfHeaderValue::Utf16(Utf16String {
            elements: values_16bit.to_vec(),
        })
    }
}

impl fmt::Display for EwfHeaderValue {
    /// Formats the header value for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EwfHeaderValue::Byte(byte_string) => byte_string.fmt(formatter),
            EwfHeaderValue::PosixTime(posix_time32) => {
                write!(formatter, "{}", posix_time32.to_iso8601_string())
            }
            EwfHeaderValue::Utf16(utf16_string) => utf16_string.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let test_bytes: [u8; 4] = [0x74, 0x65, 0x73, 0x74];

        let header_value: EwfHeaderValue = EwfHeaderValue::from_bytes(&test_bytes);

        match header_value {
            EwfHeaderValue::Byte(byte_string) => assert_eq!(
                byte_string.elements,
                vec![0x74, 0x65, 0x73, 0x74]
            ),
            _ => panic!("Unexpected header value"),
        }
    }

    #[test]
    fn test_from_utf16() {
        let test_elements: [u16; 4] = [0x0074, 0x0065, 0x0073, 0x0074];

        let header_value: EwfHeaderValue = EwfHeaderValue::from_utf16(&test_elements);

        match header_value {
            EwfHeaderValue::Utf16(utf16_string) => assert_eq!(
                utf16_string.elements,
                vec![0x0074, 0x0065, 0x0073, 0x0074]
            ),
            _ => panic!("Unexpected header value"),
        }
    }

    #[test]
    fn test_display() {
        let header_value: EwfHeaderValue = EwfHeaderValue::from_bytes(b"test");

        let string: String = header_value.to_string();
        assert_eq!(string.as_str(), "test");

        let header_value: EwfHeaderValue = EwfHeaderValue::from_utf16(&[0x0074, 0x0065, 0x0073, 0x0074]);

        let string: String = header_value.to_string();
        assert_eq!(string.as_str(), "test");

        let header_value: EwfHeaderValue =
            EwfHeaderValue::PosixTime(PosixTime32::new(1281643591));

        let string: String = header_value.to_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31");
    }
}
