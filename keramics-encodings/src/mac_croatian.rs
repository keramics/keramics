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

//! MacCroatian encoding.
//!
//! Provides support for encoding and decoding MacCroatian.

use keramics_core::ErrorTrace;

/// MacCroatian decoder.
pub struct DecoderMacCroatian<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacCroatian<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x00c4),
        Some(0x00c5),
        Some(0x00c7),
        Some(0x00c9),
        Some(0x00d1),
        Some(0x00d6),
        Some(0x00dc),
        Some(0x00e1),
        Some(0x00e0),
        Some(0x00e2),
        Some(0x00e4),
        Some(0x00e3),
        Some(0x00e5),
        Some(0x00e7),
        Some(0x00e9),
        Some(0x00e8),
        Some(0x00ea),
        Some(0x00eb),
        Some(0x00ed),
        Some(0x00ec),
        Some(0x00ee),
        Some(0x00ef),
        Some(0x00f1),
        Some(0x00f3),
        Some(0x00f2),
        Some(0x00f4),
        Some(0x00f6),
        Some(0x00f5),
        Some(0x00fa),
        Some(0x00f9),
        Some(0x00fb),
        Some(0x00fc),
        Some(0x2020),
        Some(0x00b0),
        Some(0x00a2),
        Some(0x00a3),
        Some(0x00a7),
        Some(0x2022),
        Some(0x00b6),
        Some(0x00df),
        Some(0x00ae),
        Some(0x0160),
        Some(0x2122),
        Some(0x00b4),
        Some(0x00a8),
        Some(0x2260),
        Some(0x017d),
        Some(0x00d8),
        Some(0x221e),
        Some(0x00b1),
        Some(0x2264),
        Some(0x2265),
        Some(0x2206),
        Some(0x00b5),
        Some(0x2202),
        Some(0x2211),
        Some(0x220f),
        Some(0x0161),
        Some(0x222b),
        Some(0x00aa),
        Some(0x00ba),
        Some(0x03a9),
        Some(0x017e),
        Some(0x00f8),
        Some(0x00bf),
        Some(0x00a1),
        Some(0x00ac),
        Some(0x221a),
        Some(0x0192),
        Some(0x2248),
        Some(0x0106),
        Some(0x00ab),
        Some(0x010c),
        Some(0x2026),
        Some(0x00a0),
        Some(0x00c0),
        Some(0x00c3),
        Some(0x00d5),
        Some(0x0152),
        Some(0x0153),
        Some(0x0110),
        Some(0x2014),
        Some(0x201c),
        Some(0x201d),
        Some(0x2018),
        Some(0x2019),
        Some(0x00f7),
        Some(0x25ca),
        Some(0xf8ff),
        Some(0x00a9),
        Some(0x2044),
        Some(0x20ac),
        Some(0x2039),
        Some(0x203a),
        Some(0x00c6),
        Some(0x00bb),
        Some(0x2013),
        Some(0x00b7),
        Some(0x201a),
        Some(0x201e),
        Some(0x2030),
        Some(0x00c2),
        Some(0x0107),
        Some(0x00c1),
        Some(0x010d),
        Some(0x00c8),
        Some(0x00cd),
        Some(0x00ce),
        Some(0x00cf),
        Some(0x00cc),
        Some(0x00d3),
        Some(0x00d4),
        Some(0x0111),
        Some(0x00d2),
        Some(0x00da),
        Some(0x00db),
        Some(0x00d9),
        Some(0x0131),
        Some(0x02c6),
        Some(0x02dc),
        Some(0x00af),
        Some(0x03c0),
        Some(0x00cb),
        Some(0x02da),
        Some(0x00b8),
        Some(0x00ca),
        Some(0x00e6),
        Some(0x02c7),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacCroatian<'a> {
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
                            "Unable to decode MacCroatian: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// MacCroatian encoder.
pub struct EncoderMacCroatian<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacCroatian<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 120] = [
        Some(&[0xca]),
        Some(&[0xc1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        None,
        None,
        None,
        Some(&[0xa4]),
        Some(&[0xac]),
        Some(&[0xd9]),
        Some(&[0xbb]),
        Some(&[0xc7]),
        Some(&[0xc2]),
        None,
        Some(&[0xa8]),
        Some(&[0xf8]),
        Some(&[0xa1]),
        Some(&[0xb1]),
        None,
        None,
        Some(&[0xab]),
        Some(&[0xb5]),
        Some(&[0xa6]),
        Some(&[0xe1]),
        Some(&[0xfc]),
        None,
        Some(&[0xbc]),
        Some(&[0xdf]),
        None,
        None,
        None,
        Some(&[0xc0]),
        Some(&[0xcb]),
        Some(&[0xe7]),
        Some(&[0xe5]),
        Some(&[0xcc]),
        Some(&[0x80]),
        Some(&[0x81]),
        Some(&[0xde]),
        Some(&[0x82]),
        Some(&[0xe9]),
        Some(&[0x83]),
        Some(&[0xfd]),
        Some(&[0xfa]),
        Some(&[0xed]),
        Some(&[0xea]),
        Some(&[0xeb]),
        Some(&[0xec]),
        None,
        Some(&[0x84]),
        Some(&[0xf1]),
        Some(&[0xee]),
        Some(&[0xef]),
        Some(&[0xcd]),
        Some(&[0x85]),
        None,
        Some(&[0xaf]),
        Some(&[0xf4]),
        Some(&[0xf2]),
        Some(&[0xf3]),
        Some(&[0x86]),
        None,
        None,
        Some(&[0xa7]),
        Some(&[0x88]),
        Some(&[0x87]),
        Some(&[0x89]),
        Some(&[0x8b]),
        Some(&[0x8a]),
        Some(&[0x8c]),
        Some(&[0xfe]),
        Some(&[0x8d]),
        Some(&[0x8f]),
        Some(&[0x8e]),
        Some(&[0x90]),
        Some(&[0x91]),
        Some(&[0x93]),
        Some(&[0x92]),
        Some(&[0x94]),
        Some(&[0x95]),
        None,
        Some(&[0x96]),
        Some(&[0x98]),
        Some(&[0x97]),
        Some(&[0x99]),
        Some(&[0x9b]),
        Some(&[0x9a]),
        Some(&[0xd6]),
        Some(&[0xbf]),
        Some(&[0x9d]),
        Some(&[0x9c]),
        Some(&[0x9e]),
        Some(&[0x9f]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xc6]),
        Some(&[0xe6]),
        None,
        None,
        None,
        None,
        Some(&[0xc8]),
        Some(&[0xe8]),
        None,
        None,
        Some(&[0xd0]),
        Some(&[0xf0]),
        None,
        None,
        None,
        None,
        None,
        None,
    ];

    const BASE_0X2010: [Option<&'static [u8]>; 56] = [
        None,
        None,
        None,
        Some(&[0xe0]),
        Some(&[0xd1]),
        None,
        None,
        None,
        Some(&[0xd4]),
        Some(&[0xd5]),
        Some(&[0xe2]),
        None,
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xe3]),
        None,
        Some(&[0xa0]),
        None,
        Some(&[0xa5]),
        None,
        None,
        None,
        Some(&[0xc9]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xe4]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xdc]),
        Some(&[0xdd]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xda]),
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

impl<'a> Iterator for EncoderMacCroatian<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x0118 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacCroatian",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2010..0x2048 => {
                        match Self::BASE_0X2010[(*code_point as u32 - 0x2010) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacCroatian",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0131 => Some(Ok(vec![0xf5])),
                    0x0152 => Some(Ok(vec![0xce])),
                    0x0153 => Some(Ok(vec![0xcf])),
                    0x0160 => Some(Ok(vec![0xa9])),
                    0x0161 => Some(Ok(vec![0xb9])),
                    0x017d => Some(Ok(vec![0xae])),
                    0x017e => Some(Ok(vec![0xbe])),
                    0x0192 => Some(Ok(vec![0xc4])),
                    0x02c6 => Some(Ok(vec![0xf6])),
                    0x02c7 => Some(Ok(vec![0xff])),
                    0x02da => Some(Ok(vec![0xfb])),
                    0x02dc => Some(Ok(vec![0xf7])),
                    0x03a9 => Some(Ok(vec![0xbd])),
                    0x03c0 => Some(Ok(vec![0xf9])),
                    0x20ac => Some(Ok(vec![0xdb])),
                    0x2122 => Some(Ok(vec![0xaa])),
                    0x2202 => Some(Ok(vec![0xb6])),
                    0x2206 => Some(Ok(vec![0xb4])),
                    0x220f => Some(Ok(vec![0xb8])),
                    0x2211 => Some(Ok(vec![0xb7])),
                    0x221a => Some(Ok(vec![0xc3])),
                    0x221e => Some(Ok(vec![0xb0])),
                    0x222b => Some(Ok(vec![0xba])),
                    0x2248 => Some(Ok(vec![0xc5])),
                    0x2260 => Some(Ok(vec![0xad])),
                    0x2264 => Some(Ok(vec![0xb2])),
                    0x2265 => Some(Ok(vec![0xb3])),
                    0x25ca => Some(Ok(vec![0xd7])),
                    0xf8ff => Some(Ok(vec![0xd8])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacCroatian",
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

        let mut decoder: DecoderMacCroatian = DecoderMacCroatian::new(&byte_string);

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

        let mut encoder: EncoderMacCroatian = EncoderMacCroatian::new(&code_points);

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
        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacCroatian = EncoderMacCroatian::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
