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

//! ISO-8859-8 (Hebrew) encoding.
//!
//! Provides support for encoding and decoding ISO-8859-8.

use keramics_core::ErrorTrace;

/// ISO-8859-8 decoder.
pub struct DecoderIso8859_8<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderIso8859_8<'a> {
    const BASE_0XA0: [Option<u16>; 96] = [
        Some(0x00a0),
        None,
        Some(0x00a2),
        Some(0x00a3),
        Some(0x00a4),
        Some(0x00a5),
        Some(0x00a6),
        Some(0x00a7),
        Some(0x00a8),
        Some(0x00a9),
        Some(0x00d7),
        Some(0x00ab),
        Some(0x00ac),
        Some(0x00ad),
        Some(0x00ae),
        Some(0x00af),
        Some(0x00b0),
        Some(0x00b1),
        Some(0x00b2),
        Some(0x00b3),
        Some(0x00b4),
        Some(0x00b5),
        Some(0x00b6),
        Some(0x00b7),
        Some(0x00b8),
        Some(0x00b9),
        Some(0x00f7),
        Some(0x00bb),
        Some(0x00bc),
        Some(0x00bd),
        Some(0x00be),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(0x2017),
        Some(0x05d0),
        Some(0x05d1),
        Some(0x05d2),
        Some(0x05d3),
        Some(0x05d4),
        Some(0x05d5),
        Some(0x05d6),
        Some(0x05d7),
        Some(0x05d8),
        Some(0x05d9),
        Some(0x05da),
        Some(0x05db),
        Some(0x05dc),
        Some(0x05dd),
        Some(0x05de),
        Some(0x05df),
        Some(0x05e0),
        Some(0x05e1),
        Some(0x05e2),
        Some(0x05e3),
        Some(0x05e4),
        Some(0x05e5),
        Some(0x05e6),
        Some(0x05e7),
        Some(0x05e8),
        Some(0x05e9),
        Some(0x05ea),
        None,
        None,
        Some(0x200e),
        Some(0x200f),
        None,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderIso8859_8<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                if *byte_value < 0xa0 {
                    Some(Ok(*byte_value as u32))
                } else {
                    match Self::BASE_0XA0[(*byte_value - 0xa0) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode ISO-8859-8: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// ISO-8859-8 encoder.
pub struct EncoderIso8859_8<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderIso8859_8<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 32] = [
        Some(&[0xa0]),
        None,
        Some(&[0xa2]),
        Some(&[0xa3]),
        Some(&[0xa4]),
        Some(&[0xa5]),
        Some(&[0xa6]),
        Some(&[0xa7]),
        Some(&[0xa8]),
        Some(&[0xa9]),
        None,
        Some(&[0xab]),
        Some(&[0xac]),
        Some(&[0xad]),
        Some(&[0xae]),
        Some(&[0xaf]),
        Some(&[0xb0]),
        Some(&[0xb1]),
        Some(&[0xb2]),
        Some(&[0xb3]),
        Some(&[0xb4]),
        Some(&[0xb5]),
        Some(&[0xb6]),
        Some(&[0xb7]),
        Some(&[0xb8]),
        Some(&[0xb9]),
        None,
        Some(&[0xbb]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        Some(&[0xbe]),
        None,
    ];

    const BASE_0X05D0: [Option<&'static [u8]>; 32] = [
        Some(&[0xe0]),
        Some(&[0xe1]),
        Some(&[0xe2]),
        Some(&[0xe3]),
        Some(&[0xe4]),
        Some(&[0xe5]),
        Some(&[0xe6]),
        Some(&[0xe7]),
        Some(&[0xe8]),
        Some(&[0xe9]),
        Some(&[0xea]),
        Some(&[0xeb]),
        Some(&[0xec]),
        Some(&[0xed]),
        Some(&[0xee]),
        Some(&[0xef]),
        Some(&[0xf0]),
        Some(&[0xf1]),
        Some(&[0xf2]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf5]),
        Some(&[0xf6]),
        Some(&[0xf7]),
        Some(&[0xf8]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        None,
        None,
        None,
        None,
        None,
    ];

    /// Creates a new encoder.
    pub fn new(code_points: &'a [u32]) -> Self {
        Self {
            code_points: code_points,
            code_point_index: 0,
        }
    }
}

impl<'a> Iterator for EncoderIso8859_8<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x00a0 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x00c0 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-8",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x05d0..0x05f0 => {
                        match Self::BASE_0X05D0[(*code_point as u32 - 0x05d0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-8",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00d7 => Some(Ok(vec![0xaa])),
                    0x00f7 => Some(Ok(vec![0xba])),
                    0x200e => Some(Ok(vec![0xfd])),
                    0x200f => Some(Ok(vec![0xfe])),
                    0x2017 => Some(Ok(vec![0xdf])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as ISO-8859-8",
                            *code_point as u32
                        ))));
                    }
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() -> Result<(), ErrorTrace> {
        let byte_string: [u8; 8] = [b'K', b'e', b'r', b'a', b'm', b'i', b'c', b's'];

        let mut decoder: DecoderIso8859_8 = DecoderIso8859_8::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x4b)));
        assert_eq!(decoder.next(), Some(Ok(0x65)));
        assert_eq!(decoder.next(), Some(Ok(0x72)));
        assert_eq!(decoder.next(), Some(Ok(0x61)));
        assert_eq!(decoder.next(), Some(Ok(0x6d)));
        assert_eq!(decoder.next(), Some(Ok(0x69)));
        assert_eq!(decoder.next(), Some(Ok(0x63)));
        assert_eq!(decoder.next(), Some(Ok(0x73)));
        assert_eq!(decoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_decode_with_unsupported_bytes() {
        let byte_string: [u8; 1] = [0xa1];

        let mut decoder: DecoderIso8859_8 = DecoderIso8859_8::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderIso8859_8 = EncoderIso8859_8::new(&code_points);

        assert_eq!(encoder.next(), Some(Ok(vec![b'K'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'e'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'r'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'a'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'm'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'i'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b'c'])));
        assert_eq!(encoder.next(), Some(Ok(vec![b's'])));
        assert_eq!(encoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_encode_with_unsupported_code_point() {
        let code_points: [u32; 1] = [0x00a1];

        let mut encoder: EncoderIso8859_8 = EncoderIso8859_8::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x05eb];

        let mut encoder: EncoderIso8859_8 = EncoderIso8859_8::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderIso8859_8 = EncoderIso8859_8::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
