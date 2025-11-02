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

//! UTF-8 encoding.
//!
//! Provides support for encoding and decoding UTF-8 (RFC 3629).

use keramics_core::ErrorTrace;

/// UTF-8 decoder.
pub struct DecoderUtf8<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderUtf8<'a> {
    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderUtf8<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        let mut byte_index: usize = self.byte_index;

        let byte_value1: u8 = match self.bytes.get(byte_index) {
            Some(byte_value) => {
                byte_index += 1;

                *byte_value
            }
            None => return None,
        };
        if (byte_value1 >= 0x80 && byte_value1 < 0xc0) || byte_value1 > 0xf4 {
            return Some(Err(keramics_core::error_trace_new!(format!(
                "Unable to decode UTF-8: 0x{:02x} as Unicode",
                byte_value1
            ))));
        }
        let byte_value2: u8 = if byte_value1 >= 0xc0 {
            let byte_value: u8 = match self.bytes.get(byte_index) {
                Some(byte_value) => {
                    byte_index += 1;

                    *byte_value
                }
                None => {
                    return Some(Err(keramics_core::error_trace_new!(format!(
                        "Unable to decode UTF-8: 0x{:02x} as Unicode",
                        byte_value1
                    ))));
                }
            };
            let is_invalid: bool = match byte_value1 {
                0xe0 => byte_value < 0xa0 || byte_value > 0xbf,
                0xed => byte_value < 0x80 || byte_value > 0x9f,
                0xf0 => byte_value < 0x90 || byte_value > 0xbf,
                _ => byte_value < 0x80 || byte_value > 0xbf,
            };
            if is_invalid {
                return Some(Err(keramics_core::error_trace_new!(format!(
                    "Unable to decode UTF-8: 0x{:02x}, 0x{:02x} as Unicode",
                    byte_value1, byte_value
                ))));
            }
            byte_value
        } else {
            0
        };
        let byte_value3: u8 = if byte_value1 >= 0xe0 {
            let byte_value: u8 = match self.bytes.get(byte_index) {
                Some(byte_value) => {
                    byte_index += 1;

                    *byte_value
                }
                None => {
                    return Some(Err(keramics_core::error_trace_new!(format!(
                        "Unable to decode UTF-8: 0x{:02x}, 0x{:02x} as Unicode",
                        byte_value1, byte_value2
                    ))));
                }
            };
            let is_invalid: bool = match byte_value2 {
                0xe0 => byte_value < 0xa0 || byte_value > 0xbf,
                0xed => byte_value < 0x80 || byte_value > 0x9f,
                _ => byte_value < 0x80 || byte_value > 0xbf,
            };
            if is_invalid {
                return Some(Err(keramics_core::error_trace_new!(format!(
                    "Unable to decode UTF-8: 0x{:02x}, 0x{:02x}, 0x{:02x} as Unicode",
                    byte_value1, byte_value2, byte_value
                ))));
            }
            byte_value
        } else {
            0
        };
        let byte_value4: u8 = if byte_value1 >= 0xf0 {
            let byte_value: u8 = match self.bytes.get(byte_index) {
                Some(byte_value) => {
                    byte_index += 1;

                    *byte_value
                }
                None => {
                    return Some(Err(keramics_core::error_trace_new!(format!(
                        "Unable to decode UTF-8: 0x{:02x}, 0x{:02x}, 0x{:02x} as Unicode",
                        byte_value1, byte_value2, byte_value3
                    ))));
                }
            };
            if byte_value < 0x80 || byte_value > 0xbf {
                return Some(Err(keramics_core::error_trace_new!(format!(
                    "Unable to decode UTF-8: 0x{:02x}, 0x{:02x}, 0x{:02x}, 0x{:02x} as Unicode",
                    byte_value1, byte_value2, byte_value3, byte_value
                ))));
            }
            byte_value
        } else {
            0
        };
        let code_point: u32 = if byte_value1 < 0x80 {
            byte_value1 as u32
        } else if byte_value1 < 0xe0 {
            (((byte_value1 & 0x1f) as u32) << 6) | ((byte_value2 & 0x3f) as u32)
        } else if byte_value1 < 0xf0 {
            (((byte_value1 & 0x0f) as u32) << 12)
                | (((byte_value2 & 0x3f) as u32) << 6)
                | ((byte_value3 & 0x3f) as u32)
        } else {
            (((byte_value1 & 0x07) as u32) << 18)
                | (((byte_value2 & 0x3f) as u32) << 12)
                | (((byte_value3 & 0x3f) as u32) << 6)
                | ((byte_value4 & 0x3f) as u32)
        };
        self.byte_index = byte_index;

        Some(Ok(code_point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() -> Result<(), ErrorTrace> {
        let byte_string: [u8; 8] = [b'K', b'e', b'r', b'a', b'm', b'i', b'c', b's'];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x4b)));
        assert_eq!(decoder.next(), Some(Ok(0x65)));
        assert_eq!(decoder.next(), Some(Ok(0x72)));
        assert_eq!(decoder.next(), Some(Ok(0x61)));
        assert_eq!(decoder.next(), Some(Ok(0x6d)));
        assert_eq!(decoder.next(), Some(Ok(0x69)));
        assert_eq!(decoder.next(), Some(Ok(0x63)));
        assert_eq!(decoder.next(), Some(Ok(0x73)));
        assert_eq!(decoder.next(), None);

        let byte_string: [u8; 2] = [0xc2, 0x80];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x80)));
        assert_eq!(decoder.next(), None);

        let byte_string: [u8; 3] = [0xe0, 0xa0, 0x80];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x800)));
        assert_eq!(decoder.next(), None);

        let byte_string: [u8; 4] = [0xf0, 0x90, 0x80, 0x80];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x10000)));
        assert_eq!(decoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_decode_with_unsupported_bytes() {
        let byte_string: [u8; 4] = [0xff, 0x90, 0x80, 0x80];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 1] = [0xc0];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 2] = [0xc0, 0xff];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 2] = [0xe0, 0xa0];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 3] = [0xe0, 0xa0, 0xff];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 3] = [0xed, 0xe0, 0xff];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 3] = [0xed, 0xed, 0xff];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 3] = [0xf0, 0x90, 0x80];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());

        let byte_string: [u8; 4] = [0xf0, 0x90, 0x80, 0xff];

        let mut decoder: DecoderUtf8 = DecoderUtf8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }
}
