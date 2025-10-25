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

//! Windows 1254 (Turkish) encoding.
//!
//! Provides support for encoding and decoding Windows 1254.

use keramics_core::ErrorTrace;

/// Windows 1254 decoder.
pub struct DecoderWindows1254<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderWindows1254<'a> {
    const BASE_0X80: [Option<u16>; 32] = [
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
        Some(0x0160),
        Some(0x2039),
        Some(0x0152),
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
        Some(0x0161),
        Some(0x203a),
        Some(0x0153),
        None,
        None,
        Some(0x0178),
    ];

    const BASE_0XD0: [Option<u16>; 16] = [
        Some(0x011e),
        Some(0x00d1),
        Some(0x00d2),
        Some(0x00d3),
        Some(0x00d4),
        Some(0x00d5),
        Some(0x00d6),
        Some(0x00d7),
        Some(0x00d8),
        Some(0x00d9),
        Some(0x00da),
        Some(0x00db),
        Some(0x00dc),
        Some(0x0130),
        Some(0x015e),
        Some(0x00df),
    ];

    const BASE_0XF0: [Option<u16>; 16] = [
        Some(0x011f),
        Some(0x00f1),
        Some(0x00f2),
        Some(0x00f3),
        Some(0x00f4),
        Some(0x00f5),
        Some(0x00f6),
        Some(0x00f7),
        Some(0x00f8),
        Some(0x00f9),
        Some(0x00fa),
        Some(0x00fb),
        Some(0x00fc),
        Some(0x0131),
        Some(0x015f),
        Some(0x00ff),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderWindows1254<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                match *byte_value {
                    0x00..0x80 | 0xa0..0xd0 | 0xe0..0xf0 => Some(Ok(*byte_value as u32)),
                    0x80..0xa0 => match Self::BASE_0X80[(*byte_value - 0x80) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode Windows 1254: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    },
                    0xd0..0xe0 => match Self::BASE_0XD0[(*byte_value - 0xd0) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode Windows 1254: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    },
                    0xf0..=0xff => match Self::BASE_0XF0[(*byte_value - 0xf0) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode Windows 1254: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    },
                }
            }
            None => None,
        }
    }
}

/// Windows 1254 encoder.
pub struct EncoderWindows1254<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderWindows1254<'a> {
    const BASE_0X00D0: [Option<&'static [u8]>; 48] = [
        None,
        Some(&[0xd1]),
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xd4]),
        Some(&[0xd5]),
        Some(&[0xd6]),
        Some(&[0xd7]),
        Some(&[0xd8]),
        Some(&[0xd9]),
        Some(&[0xda]),
        Some(&[0xdb]),
        Some(&[0xdc]),
        None,
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
        Some(&[0xf7]),
        Some(&[0xf8]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        Some(&[0xfb]),
        Some(&[0xfc]),
        None,
        None,
        Some(&[0xff]),
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

impl<'a> Iterator for EncoderWindows1254<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 | 0x00a0..0x00d0 => Some(Ok(vec![*code_point as u8])),
                    0x00d0..0x0100 => {
                        match Self::BASE_0X00D0[(*code_point as u32 - 0x00d0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 1254",
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
                                    "Unable to encode code point: U+{:04x} as Windows 1254",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x011e => Some(Ok(vec![0xd0])),
                    0x011f => Some(Ok(vec![0xf0])),
                    0x0130 => Some(Ok(vec![0xdd])),
                    0x0131 => Some(Ok(vec![0xfd])),
                    0x0152 => Some(Ok(vec![0x8c])),
                    0x0153 => Some(Ok(vec![0x9c])),
                    0x015e => Some(Ok(vec![0xde])),
                    0x015f => Some(Ok(vec![0xfe])),
                    0x0160 => Some(Ok(vec![0x8a])),
                    0x0161 => Some(Ok(vec![0x9a])),
                    0x0178 => Some(Ok(vec![0x9f])),
                    0x0192 => Some(Ok(vec![0x83])),
                    0x02c6 => Some(Ok(vec![0x88])),
                    0x02dc => Some(Ok(vec![0x98])),
                    0x2030 => Some(Ok(vec![0x89])),
                    0x2039 => Some(Ok(vec![0x8b])),
                    0x203a => Some(Ok(vec![0x9b])),
                    0x20ac => Some(Ok(vec![0x80])),
                    0x2122 => Some(Ok(vec![0x99])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as Windows 1254",
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

        let mut decoder: DecoderWindows1254 = DecoderWindows1254::new(&byte_string);

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

        let mut encoder: EncoderWindows1254 = EncoderWindows1254::new(&code_points);

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

        let mut encoder: EncoderWindows1254 = EncoderWindows1254::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
