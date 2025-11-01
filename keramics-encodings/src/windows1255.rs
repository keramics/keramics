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

//! Windows 1255 (Hebrew) encoding.
//!
//! Provides support for encoding and decoding Windows 1255.

use keramics_core::ErrorTrace;

/// Windows 1255 decoder.
pub struct DecoderWindows1255<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderWindows1255<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x20ac),
        None,
        Some(0x201a),
        Some(0x0192),
        Some(0x201e),
        Some(0x2026),
        Some(0x2020),
        Some(0x2021),
        Some(0x02c6),
        Some(0x2030),
        None,
        Some(0x2039),
        None,
        None,
        None,
        None,
        None,
        Some(0x2018),
        Some(0x2019),
        Some(0x201c),
        Some(0x201d),
        Some(0x2022),
        Some(0x2013),
        Some(0x2014),
        Some(0x02dc),
        Some(0x2122),
        None,
        Some(0x203a),
        None,
        None,
        None,
        None,
        Some(0x00a0),
        Some(0x00a1),
        Some(0x00a2),
        Some(0x00a3),
        Some(0x20aa),
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
        Some(0x00bf),
        Some(0x05b0),
        Some(0x05b1),
        Some(0x05b2),
        Some(0x05b3),
        Some(0x05b4),
        Some(0x05b5),
        Some(0x05b6),
        Some(0x05b7),
        Some(0x05b8),
        Some(0x05b9),
        None,
        Some(0x05bb),
        Some(0x05bc),
        Some(0x05bd),
        Some(0x05be),
        Some(0x05bf),
        Some(0x05c0),
        Some(0x05c1),
        Some(0x05c2),
        Some(0x05c3),
        Some(0x05f0),
        Some(0x05f1),
        Some(0x05f2),
        Some(0x05f3),
        Some(0x05f4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
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

impl<'a> Iterator for DecoderWindows1255<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                if *byte_value < 0x80 {
                    Some(Ok(*byte_value as u32))
                } else {
                    match Self::BASE_0X80[(*byte_value - 0x80) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode Windows 1255: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// Windows 1255 encoder.
pub struct EncoderWindows1255<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderWindows1255<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 32] = [
        Some(&[0xa0]),
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        None,
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
        Some(&[0xbf]),
    ];

    const BASE_0X05B0: [Option<&'static [u8]>; 72] = [
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
        None,
        Some(&[0xcb]),
        Some(&[0xcc]),
        Some(&[0xcd]),
        Some(&[0xce]),
        Some(&[0xcf]),
        Some(&[0xd0]),
        Some(&[0xd1]),
        Some(&[0xd2]),
        Some(&[0xd3]),
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
        Some(&[0xd4]),
        Some(&[0xd5]),
        Some(&[0xd6]),
        Some(&[0xd7]),
        Some(&[0xd8]),
        None,
        None,
        None,
    ];

    const BASE_0X2010: [Option<&'static [u8]>; 24] = [
        None,
        None,
        None,
        Some(&[0x96]),
        Some(&[0x97]),
        None,
        None,
        None,
        Some(&[0x91]),
        Some(&[0x92]),
        Some(&[0x82]),
        None,
        Some(&[0x93]),
        Some(&[0x94]),
        Some(&[0x84]),
        None,
        Some(&[0x86]),
        Some(&[0x87]),
        Some(&[0x95]),
        None,
        None,
        None,
        Some(&[0x85]),
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

impl<'a> Iterator for EncoderWindows1255<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x00c0 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 1255",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x05b0..0x05f8 => {
                        match Self::BASE_0X05B0[(*code_point as u32 - 0x05b0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 1255",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2010..0x2028 => {
                        match Self::BASE_0X2010[(*code_point as u32 - 0x2010) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 1255",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00d7 => Some(Ok(vec![0xaa])),
                    0x00f7 => Some(Ok(vec![0xba])),
                    0x0192 => Some(Ok(vec![0x83])),
                    0x02c6 => Some(Ok(vec![0x88])),
                    0x02dc => Some(Ok(vec![0x98])),
                    0x200e => Some(Ok(vec![0xfd])),
                    0x200f => Some(Ok(vec![0xfe])),
                    0x2030 => Some(Ok(vec![0x89])),
                    0x2039 => Some(Ok(vec![0x8b])),
                    0x203a => Some(Ok(vec![0x9b])),
                    0x20aa => Some(Ok(vec![0xa4])),
                    0x20ac => Some(Ok(vec![0x80])),
                    0x2122 => Some(Ok(vec![0x99])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as Windows 1255",
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

        let mut decoder: DecoderWindows1255 = DecoderWindows1255::new(&byte_string);

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
        let byte_string: [u8; 1] = [0x81];

        let mut decoder: DecoderWindows1255 = DecoderWindows1255::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderWindows1255 = EncoderWindows1255::new(&code_points);

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
        let code_points: [u32; 1] = [0x00a4];

        let mut encoder: EncoderWindows1255 = EncoderWindows1255::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x05ba];

        let mut encoder: EncoderWindows1255 = EncoderWindows1255::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderWindows1255 = EncoderWindows1255::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderWindows1255 = EncoderWindows1255::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
