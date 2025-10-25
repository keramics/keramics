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

//! ISO-8859-16 encoding.
//!
//! Provides support for encoding and decoding ISO-8859-16.

use keramics_core::ErrorTrace;

/// ISO-8859-16 decoder.
pub struct DecoderIso8859_16<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderIso8859_16<'a> {
    const BASE_0XA0: [u16; 96] = [
        0x00a0, 0x0104, 0x0105, 0x0141, 0x20ac, 0x201e, 0x0160, 0x00a7, 0x0161, 0x00a9, 0x0218,
        0x00ab, 0x0179, 0x00ad, 0x017a, 0x017b, 0x00b0, 0x00b1, 0x010c, 0x0142, 0x017d, 0x201d,
        0x00b6, 0x00b7, 0x017e, 0x010d, 0x0219, 0x00bb, 0x0152, 0x0153, 0x0178, 0x017c, 0x00c0,
        0x00c1, 0x00c2, 0x0102, 0x00c4, 0x0106, 0x00c6, 0x00c7, 0x00c8, 0x00c9, 0x00ca, 0x00cb,
        0x00cc, 0x00cd, 0x00ce, 0x00cf, 0x0110, 0x0143, 0x00d2, 0x00d3, 0x00d4, 0x0150, 0x00d6,
        0x015a, 0x0170, 0x00d9, 0x00da, 0x00db, 0x00dc, 0x0118, 0x021a, 0x00df, 0x00e0, 0x00e1,
        0x00e2, 0x0103, 0x00e4, 0x0107, 0x00e6, 0x00e7, 0x00e8, 0x00e9, 0x00ea, 0x00eb, 0x00ec,
        0x00ed, 0x00ee, 0x00ef, 0x0111, 0x0144, 0x00f2, 0x00f3, 0x00f4, 0x0151, 0x00f6, 0x015b,
        0x0171, 0x00f9, 0x00fa, 0x00fb, 0x00fc, 0x0119, 0x021b, 0x00ff,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderIso8859_16<'a> {
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

/// ISO-8859-16 encoder.
pub struct EncoderIso8859_16<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderIso8859_16<'a> {
    const BASE_0X00A8: [Option<&'static [u8]>; 96] = [
        None,
        Some(&[0xa9]),
        None,
        Some(&[0xab]),
        None,
        Some(&[0xad]),
        None,
        None,
        Some(&[0xb0]),
        Some(&[0xb1]),
        None,
        None,
        None,
        None,
        Some(&[0xb6]),
        Some(&[0xb7]),
        None,
        None,
        None,
        Some(&[0xbb]),
        None,
        None,
        None,
        None,
        Some(&[0xc0]),
        Some(&[0xc1]),
        Some(&[0xc2]),
        None,
        Some(&[0xc4]),
        None,
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
        None,
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xd4]),
        None,
        Some(&[0xd6]),
        None,
        None,
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
        None,
        Some(&[0xe4]),
        None,
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
        None,
        Some(&[0xf2]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        None,
        Some(&[0xf6]),
        None,
        None,
        Some(&[0xf9]),
        Some(&[0xfa]),
        Some(&[0xfb]),
        Some(&[0xfc]),
        None,
        None,
        Some(&[0xff]),
        None,
        None,
        Some(&[0xc3]),
        Some(&[0xe3]),
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xc5]),
        Some(&[0xe5]),
    ];

    const BASE_0X0140: [Option<&'static [u8]>; 24] = [
        None,
        Some(&[0xa3]),
        Some(&[0xb3]),
        Some(&[0xd1]),
        Some(&[0xf1]),
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
        Some(&[0xd5]),
        Some(&[0xf5]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        None,
        None,
        None,
        None,
    ];

    const BASE_0X0178: [Option<&'static [u8]>; 8] = [
        Some(&[0xbe]),
        Some(&[0xac]),
        Some(&[0xae]),
        Some(&[0xaf]),
        Some(&[0xbf]),
        Some(&[0xb4]),
        Some(&[0xb8]),
        None,
    ];

    const BASE_0X0218: [Option<&'static [u8]>; 8] = [
        Some(&[0xaa]),
        Some(&[0xba]),
        Some(&[0xde]),
        Some(&[0xfe]),
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

impl<'a> Iterator for EncoderIso8859_16<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x00a1 => Some(Ok(vec![*code_point as u8])),
                    0x00a8..0x0108 => {
                        match Self::BASE_0X00A8[(*code_point as u32 - 0x00a8) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-16",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0140..0x0158 => {
                        match Self::BASE_0X0140[(*code_point as u32 - 0x0140) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-16",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0178..0x0180 => {
                        match Self::BASE_0X0178[(*code_point as u32 - 0x0178) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-16",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0218..0x0220 => {
                        match Self::BASE_0X0218[(*code_point as u32 - 0x0218) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-16",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a7 => Some(Ok(vec![0xa7])),
                    0x010c => Some(Ok(vec![0xb2])),
                    0x010d => Some(Ok(vec![0xb9])),
                    0x0110 => Some(Ok(vec![0xd0])),
                    0x0111 => Some(Ok(vec![0xf0])),
                    0x0118 => Some(Ok(vec![0xdd])),
                    0x0119 => Some(Ok(vec![0xfd])),
                    0x015a => Some(Ok(vec![0xd7])),
                    0x015b => Some(Ok(vec![0xf7])),
                    0x0160 => Some(Ok(vec![0xa6])),
                    0x0161 => Some(Ok(vec![0xa8])),
                    0x0170 => Some(Ok(vec![0xd8])),
                    0x0171 => Some(Ok(vec![0xf8])),
                    0x201d => Some(Ok(vec![0xb5])),
                    0x201e => Some(Ok(vec![0xa5])),
                    0x20ac => Some(Ok(vec![0xa4])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as ISO-8859-16",
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

        let mut decoder: DecoderIso8859_16 = DecoderIso8859_16::new(&byte_string);

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

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

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
        let code_points: [u32; 1] = [0x00a8];

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x0140];

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x017f];

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x021c];

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderIso8859_16 = EncoderIso8859_16::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
