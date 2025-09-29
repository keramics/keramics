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
        EwfHeaderValue::Byte(ByteString::from_bytes(bytes))
    }

    /// Creates a new EWF header value from a UTF-16 sequence.
    pub fn from_utf16(values_16bit: &[u16]) -> Self {
        EwfHeaderValue::Utf16(Utf16String {
            elements: values_16bit.to_vec(),
        })
    }

    /// Converts the EWF header value to `String`.
    pub fn to_string(&self) -> String {
        match self {
            EwfHeaderValue::Byte(byte_string) => byte_string.to_string(),
            EwfHeaderValue::PosixTime(posix_time32) => posix_time32.to_iso8601_string(),
            EwfHeaderValue::Utf16(utf16_string) => utf16_string.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests.
}
