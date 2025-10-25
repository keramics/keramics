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

//! ISO-8859-14 encoding.
//!
//! Provides support for encoding and decoding ISO-8859-14.

use keramics_core::ErrorTrace;

/// ISO-8859-14 decoder.
pub struct DecoderIso8859_14<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderIso8859_14<'a> {
    const BASE_0XA0: [u16; 96] = [
        0x00a0, 0x1e02, 0x1e03, 0x00a3, 0x010a, 0x010b, 0x1e0a, 0x00a7, 0x1e80, 0x00a9, 0x1e82,
        0x1e0b, 0x1ef2, 0x00ad, 0x00ae, 0x0178, 0x1e1e, 0x1e1f, 0x0120, 0x0121, 0x1e40, 0x1e41,
        0x00b6, 0x1e56, 0x1e81, 0x1e57, 0x1e83, 0x1e60, 0x1ef3, 0x1e84, 0x1e85, 0x1e61, 0x00c0,
        0x00c1, 0x00c2, 0x00c3, 0x00c4, 0x00c5, 0x00c6, 0x00c7, 0x00c8, 0x00c9, 0x00ca, 0x00cb,
        0x00cc, 0x00cd, 0x00ce, 0x00cf, 0x0174, 0x00d1, 0x00d2, 0x00d3, 0x00d4, 0x00d5, 0x00d6,
        0x1e6a, 0x00d8, 0x00d9, 0x00da, 0x00db, 0x00dc, 0x00dd, 0x0176, 0x00df, 0x00e0, 0x00e1,
        0x00e2, 0x00e3, 0x00e4, 0x00e5, 0x00e6, 0x00e7, 0x00e8, 0x00e9, 0x00ea, 0x00eb, 0x00ec,
        0x00ed, 0x00ee, 0x00ef, 0x0175, 0x00f1, 0x00f2, 0x00f3, 0x00f4, 0x00f5, 0x00f6, 0x1e6b,
        0x00f8, 0x00f9, 0x00fa, 0x00fb, 0x00fc, 0x00fd, 0x0177, 0x00ff,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderIso8859_14<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                let code_point: u16 = if *byte_value < 0xa0 {
                    *byte_value as u16
                } else {
                    Self::BASE_0XA0[(*byte_value - 0xa0) as usize]
                };
                Some(Ok(code_point as u32))
            }
            None => None,
        }
    }
}

/// ISO-8859-14 encoder.
pub struct EncoderIso8859_14<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderIso8859_14<'a> {
    const BASE_0X00C0: [Option<&'static [u8]>; 64] = [
        Some(&[0xc0]),
        Some(&[0xc1]),
        Some(&[0xc2]),
        Some(&[0xc3]),
        Some(&[0xc4]),
        Some(&[0xc5]),
        Some(&[0xc6]),
        Some(&[0xc7]),
        Some(&[0xc8]),
        Some(&[0xc9]),
        Some(&[0xca]),
        Some(&[0xcb]),
        Some(&[0xcc]),
        Some(&[0xcd]),
        Some(&[0xce]),
        Some(&[0xcf]),
        None,
        Some(&[0xd1]),
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xd4]),
        Some(&[0xd5]),
        Some(&[0xd6]),
        None,
        Some(&[0xd8]),
        Some(&[0xd9]),
        Some(&[0xda]),
        Some(&[0xdb]),
        Some(&[0xdc]),
        Some(&[0xdd]),
        None,
        Some(&[0xdf]),
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
        None,
        Some(&[0xf1]),
        Some(&[0xf2]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf5]),
        Some(&[0xf6]),
        None,
        Some(&[0xf8]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        Some(&[0xfb]),
        Some(&[0xfc]),
        Some(&[0xfd]),
        None,
        Some(&[0xff]),
    ];

    const BASE_0X0170: [Option<&'static [u8]>; 8] = [
        None,
        None,
        None,
        None,
        Some(&[0xd0]),
        Some(&[0xf0]),
        Some(&[0xde]),
        Some(&[0xfe]),
    ];

    const BASE_0X1E80: [Option<&'static [u8]>; 8] = [
        Some(&[0xa8]),
        Some(&[0xb8]),
        Some(&[0xaa]),
        Some(&[0xba]),
        Some(&[0xbd]),
        Some(&[0xbe]),
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

impl<'a> Iterator for EncoderIso8859_14<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x00a1 => Some(Ok(vec![*code_point as u8])),
                    0x00c0..0x0100 => {
                        match Self::BASE_0X00C0[(*code_point as u32 - 0x00c0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-14",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0170..0x0178 => {
                        match Self::BASE_0X0170[(*code_point as u32 - 0x0170) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-14",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x1e80..0x1e88 => {
                        match Self::BASE_0X1E80[(*code_point as u32 - 0x1e80) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-14",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a3 => Some(Ok(vec![0xa3])),
                    0x00a7 => Some(Ok(vec![0xa7])),
                    0x00a9 => Some(Ok(vec![0xa9])),
                    0x00ad => Some(Ok(vec![0xad])),
                    0x00ae => Some(Ok(vec![0xae])),
                    0x00b6 => Some(Ok(vec![0xb6])),
                    0x010a => Some(Ok(vec![0xa4])),
                    0x010b => Some(Ok(vec![0xa5])),
                    0x0120 => Some(Ok(vec![0xb2])),
                    0x0121 => Some(Ok(vec![0xb3])),
                    0x0178 => Some(Ok(vec![0xaf])),
                    0x1e02 => Some(Ok(vec![0xa1])),
                    0x1e03 => Some(Ok(vec![0xa2])),
                    0x1e0a => Some(Ok(vec![0xa6])),
                    0x1e0b => Some(Ok(vec![0xab])),
                    0x1e1e => Some(Ok(vec![0xb0])),
                    0x1e1f => Some(Ok(vec![0xb1])),
                    0x1e40 => Some(Ok(vec![0xb4])),
                    0x1e41 => Some(Ok(vec![0xb5])),
                    0x1e56 => Some(Ok(vec![0xb7])),
                    0x1e57 => Some(Ok(vec![0xb9])),
                    0x1e60 => Some(Ok(vec![0xbb])),
                    0x1e61 => Some(Ok(vec![0xbf])),
                    0x1e6a => Some(Ok(vec![0xd7])),
                    0x1e6b => Some(Ok(vec![0xf7])),
                    0x1ef2 => Some(Ok(vec![0xac])),
                    0x1ef3 => Some(Ok(vec![0xbc])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as ISO-8859-14",
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

        let mut decoder: DecoderIso8859_14 = DecoderIso8859_14::new(&byte_string);

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
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderIso8859_14 = EncoderIso8859_14::new(&code_points);

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
        let code_points: [u32; 1] = [0x00d0];

        let mut encoder: EncoderIso8859_14 = EncoderIso8859_14::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x0170];

        let mut encoder: EncoderIso8859_14 = EncoderIso8859_14::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x1e86];

        let mut encoder: EncoderIso8859_14 = EncoderIso8859_14::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderIso8859_14 = EncoderIso8859_14::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
