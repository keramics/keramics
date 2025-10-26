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

//! MacCeltic encoding.
//!
//! Provides support for encoding and decoding MacCeltic.

use keramics_core::ErrorTrace;

/// MacCeltic decoder.
pub struct DecoderMacCeltic<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacCeltic<'a> {
    const BASE_0X80: [u16; 128] = [
        0x00c4, 0x00c5, 0x00c7, 0x00c9, 0x00d1, 0x00d6, 0x00dc, 0x00e1, 0x00e0, 0x00e2, 0x00e4,
        0x00e3, 0x00e5, 0x00e7, 0x00e9, 0x00e8, 0x00ea, 0x00eb, 0x00ed, 0x00ec, 0x00ee, 0x00ef,
        0x00f1, 0x00f3, 0x00f2, 0x00f4, 0x00f6, 0x00f5, 0x00fa, 0x00f9, 0x00fb, 0x00fc, 0x2020,
        0x00b0, 0x00a2, 0x00a3, 0x00a7, 0x2022, 0x00b6, 0x00df, 0x00ae, 0x00a9, 0x2122, 0x00b4,
        0x00a8, 0x2260, 0x00c6, 0x00d8, 0x221e, 0x00b1, 0x2264, 0x2265, 0x00a5, 0x00b5, 0x2202,
        0x2211, 0x220f, 0x03c0, 0x222b, 0x00aa, 0x00ba, 0x03a9, 0x00e6, 0x00f8, 0x00bf, 0x00a1,
        0x00ac, 0x221a, 0x0192, 0x2248, 0x2206, 0x00ab, 0x00bb, 0x2026, 0x00a0, 0x00c0, 0x00c3,
        0x00d5, 0x0152, 0x0153, 0x2013, 0x2014, 0x201c, 0x201d, 0x2018, 0x2019, 0x00f7, 0x25ca,
        0x00ff, 0x0178, 0x2044, 0x20ac, 0x2039, 0x203a, 0x0176, 0x0177, 0x2021, 0x00b7, 0x1ef2,
        0x1ef3, 0x2030, 0x00c2, 0x00ca, 0x00c1, 0x00cb, 0x00c8, 0x00cd, 0x00ce, 0x00cf, 0x00cc,
        0x00d3, 0x00d4, 0x2663, 0x00d2, 0x00da, 0x00db, 0x00d9, 0x0131, 0x00dd, 0x00fd, 0x0174,
        0x0175, 0x1e84, 0x1e85, 0x1e80, 0x1e81, 0x1e82, 0x1e83,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacCeltic<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                let code_point: u16 = if *byte_value < 0x80 {
                    *byte_value as u16
                } else {
                    Self::BASE_0X80[(*byte_value - 0x80) as usize]
                };
                Some(Ok(code_point as u32))
            }
            None => None,
        }
    }
}

/// MacCeltic encoder.
pub struct EncoderMacCeltic<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacCeltic<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 96] = [
        Some(&[0xca]),
        Some(&[0xc1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        None,
        Some(&[0xb4]),
        None,
        Some(&[0xa4]),
        Some(&[0xac]),
        Some(&[0xa9]),
        Some(&[0xbb]),
        Some(&[0xc7]),
        Some(&[0xc2]),
        None,
        Some(&[0xa8]),
        None,
        Some(&[0xa1]),
        Some(&[0xb1]),
        None,
        None,
        Some(&[0xab]),
        Some(&[0xb5]),
        Some(&[0xa6]),
        Some(&[0xe1]),
        None,
        None,
        Some(&[0xbc]),
        Some(&[0xc8]),
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
        Some(&[0xae]),
        Some(&[0x82]),
        Some(&[0xe9]),
        Some(&[0x83]),
        Some(&[0xe6]),
        Some(&[0xe8]),
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
        Some(&[0xf6]),
        None,
        Some(&[0xa7]),
        Some(&[0x88]),
        Some(&[0x87]),
        Some(&[0x89]),
        Some(&[0x8b]),
        Some(&[0x8a]),
        Some(&[0x8c]),
        Some(&[0xbe]),
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
        Some(&[0xf7]),
        None,
        Some(&[0xd8]),
    ];

    const BASE_0X2010: [Option<&'static [u8]>; 56] = [
        None,
        None,
        None,
        Some(&[0xd0]),
        Some(&[0xd1]),
        None,
        None,
        None,
        Some(&[0xd4]),
        Some(&[0xd5]),
        None,
        None,
        Some(&[0xd2]),
        Some(&[0xd3]),
        None,
        None,
        Some(&[0xa0]),
        Some(&[0xe0]),
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

impl<'a> Iterator for EncoderMacCeltic<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x0100 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacCeltic",
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
                                    "Unable to encode code point: U+{:04x} as MacCeltic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0131 => Some(Ok(vec![0xf5])),
                    0x0152 => Some(Ok(vec![0xce])),
                    0x0153 => Some(Ok(vec![0xcf])),
                    0x0174 => Some(Ok(vec![0xf8])),
                    0x0175 => Some(Ok(vec![0xf9])),
                    0x0176 => Some(Ok(vec![0xde])),
                    0x0177 => Some(Ok(vec![0xdf])),
                    0x0178 => Some(Ok(vec![0xd9])),
                    0x0192 => Some(Ok(vec![0xc4])),
                    0x03a9 => Some(Ok(vec![0xbd])),
                    0x03c0 => Some(Ok(vec![0xb9])),
                    0x1e80 => Some(Ok(vec![0xfc])),
                    0x1e81 => Some(Ok(vec![0xfd])),
                    0x1e82 => Some(Ok(vec![0xfe])),
                    0x1e83 => Some(Ok(vec![0xff])),
                    0x1e84 => Some(Ok(vec![0xfa])),
                    0x1e85 => Some(Ok(vec![0xfb])),
                    0x1ef2 => Some(Ok(vec![0xe2])),
                    0x1ef3 => Some(Ok(vec![0xe3])),
                    0x20ac => Some(Ok(vec![0xdb])),
                    0x2122 => Some(Ok(vec![0xaa])),
                    0x2202 => Some(Ok(vec![0xb6])),
                    0x2206 => Some(Ok(vec![0xc6])),
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
                    0x2663 => Some(Ok(vec![0xf0])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacCeltic",
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

        let mut decoder: DecoderMacCeltic = DecoderMacCeltic::new(&byte_string);

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

        let mut encoder: EncoderMacCeltic = EncoderMacCeltic::new(&code_points);

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

        let mut encoder: EncoderMacCeltic = EncoderMacCeltic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
