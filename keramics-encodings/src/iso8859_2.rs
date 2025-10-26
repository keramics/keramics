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

//! ISO-8859-2 (Latin-2 or Central European) encoding.
//!
//! Provides support for encoding and decoding ISO-8859-2.

use keramics_core::ErrorTrace;

/// ISO-8859-2 decoder.
pub struct DecoderIso8859_2<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderIso8859_2<'a> {
    const BASE_0XA0: [Option<u16>; 96] = [
        Some(0x00a0),
        Some(0x0104),
        Some(0x02d8),
        Some(0x0141),
        Some(0x00a4),
        Some(0x013d),
        Some(0x015a),
        Some(0x00a7),
        Some(0x00a8),
        Some(0x0160),
        Some(0x015e),
        Some(0x0164),
        Some(0x0179),
        Some(0x00ad),
        Some(0x017d),
        Some(0x017b),
        Some(0x00b0),
        Some(0x0105),
        Some(0x02db),
        Some(0x0142),
        Some(0x00b4),
        Some(0x013e),
        Some(0x015b),
        Some(0x02c7),
        Some(0x00b8),
        Some(0x0161),
        Some(0x015f),
        Some(0x0165),
        Some(0x017a),
        Some(0x02dd),
        Some(0x017e),
        Some(0x017c),
        Some(0x0154),
        Some(0x00c1),
        Some(0x00c2),
        Some(0x0102),
        Some(0x00c4),
        Some(0x0139),
        Some(0x0106),
        Some(0x00c7),
        Some(0x010c),
        Some(0x00c9),
        Some(0x0118),
        Some(0x00cb),
        Some(0x011a),
        Some(0x00cd),
        Some(0x00ce),
        Some(0x010e),
        Some(0x0110),
        Some(0x0143),
        Some(0x0147),
        Some(0x00d3),
        Some(0x00d4),
        Some(0x0150),
        Some(0x00d6),
        Some(0x00d7),
        Some(0x0158),
        Some(0x016e),
        Some(0x00da),
        Some(0x0170),
        Some(0x00dc),
        Some(0x00dd),
        Some(0x0162),
        Some(0x00df),
        Some(0x0155),
        Some(0x00e1),
        Some(0x00e2),
        Some(0x0103),
        Some(0x00e4),
        Some(0x013a),
        Some(0x0107),
        Some(0x00e7),
        Some(0x010d),
        Some(0x00e9),
        Some(0x0119),
        Some(0x00eb),
        Some(0x011b),
        Some(0x00ed),
        Some(0x00ee),
        Some(0x010f),
        Some(0x0111),
        Some(0x0144),
        Some(0x0148),
        Some(0x00f3),
        Some(0x00f4),
        Some(0x0151),
        Some(0x00f6),
        Some(0x00f7),
        Some(0x0159),
        Some(0x016f),
        Some(0x00fa),
        Some(0x0171),
        Some(0x00fc),
        Some(0x00fd),
        Some(0x0163),
        Some(0x02d9),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderIso8859_2<'a> {
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
                            "Unable to decode ISO-8859-2: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// ISO-8859-2 encoder.
pub struct EncoderIso8859_2<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderIso8859_2<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 128] = [
        Some(&[0xa0]),
        None,
        None,
        None,
        Some(&[0xa4]),
        None,
        None,
        Some(&[0xa7]),
        Some(&[0xa8]),
        None,
        None,
        None,
        None,
        Some(&[0xad]),
        None,
        None,
        Some(&[0xb0]),
        None,
        None,
        None,
        Some(&[0xb4]),
        None,
        None,
        None,
        Some(&[0xb8]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xc1]),
        Some(&[0xc2]),
        None,
        Some(&[0xc4]),
        None,
        None,
        Some(&[0xc7]),
        None,
        Some(&[0xc9]),
        None,
        Some(&[0xcb]),
        None,
        Some(&[0xcd]),
        Some(&[0xce]),
        None,
        None,
        None,
        None,
        Some(&[0xd3]),
        Some(&[0xd4]),
        None,
        Some(&[0xd6]),
        Some(&[0xd7]),
        None,
        None,
        Some(&[0xda]),
        None,
        Some(&[0xdc]),
        Some(&[0xdd]),
        None,
        Some(&[0xdf]),
        None,
        Some(&[0xe1]),
        Some(&[0xe2]),
        None,
        Some(&[0xe4]),
        None,
        None,
        Some(&[0xe7]),
        None,
        Some(&[0xe9]),
        None,
        Some(&[0xeb]),
        None,
        Some(&[0xed]),
        Some(&[0xee]),
        None,
        None,
        None,
        None,
        Some(&[0xf3]),
        Some(&[0xf4]),
        None,
        Some(&[0xf6]),
        Some(&[0xf7]),
        None,
        None,
        Some(&[0xfa]),
        None,
        Some(&[0xfc]),
        Some(&[0xfd]),
        None,
        None,
        None,
        None,
        Some(&[0xc3]),
        Some(&[0xe3]),
        Some(&[0xa1]),
        Some(&[0xb1]),
        Some(&[0xc6]),
        Some(&[0xe6]),
        None,
        None,
        None,
        None,
        Some(&[0xc8]),
        Some(&[0xe8]),
        Some(&[0xcf]),
        Some(&[0xef]),
        Some(&[0xd0]),
        Some(&[0xf0]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xca]),
        Some(&[0xea]),
        Some(&[0xcc]),
        Some(&[0xec]),
        None,
        None,
        None,
        None,
    ];

    const BASE_0X0138: [Option<&'static [u8]>; 72] = [
        None,
        Some(&[0xc5]),
        Some(&[0xe5]),
        None,
        None,
        Some(&[0xa5]),
        Some(&[0xb5]),
        None,
        None,
        Some(&[0xa3]),
        Some(&[0xb3]),
        Some(&[0xd1]),
        Some(&[0xf1]),
        None,
        None,
        Some(&[0xd2]),
        Some(&[0xf2]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xd5]),
        Some(&[0xf5]),
        None,
        None,
        Some(&[0xc0]),
        Some(&[0xe0]),
        None,
        None,
        Some(&[0xd8]),
        Some(&[0xf8]),
        Some(&[0xa6]),
        Some(&[0xb6]),
        None,
        None,
        Some(&[0xaa]),
        Some(&[0xba]),
        Some(&[0xa9]),
        Some(&[0xb9]),
        Some(&[0xde]),
        Some(&[0xfe]),
        Some(&[0xab]),
        Some(&[0xbb]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xd9]),
        Some(&[0xf9]),
        Some(&[0xdb]),
        Some(&[0xfb]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xac]),
        Some(&[0xbc]),
        Some(&[0xaf]),
        Some(&[0xbf]),
        Some(&[0xae]),
        Some(&[0xbe]),
        None,
    ];

    const BASE_0X02D8: [Option<&'static [u8]>; 8] = [
        Some(&[0xa2]),
        Some(&[0xff]),
        Some(&[0x1a]),
        Some(&[0xb2]),
        Some(&[0x1a]),
        Some(&[0xbd]),
        Some(&[0x1a]),
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

impl<'a> Iterator for EncoderIso8859_2<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x00a0 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x0120 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-2",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0138..0x0180 => {
                        match Self::BASE_0X0138[(*code_point as u32 - 0x0138) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-2",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x02d8..0x02e0 => {
                        match Self::BASE_0X02D8[(*code_point as u32 - 0x02d8) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-2",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x02c7 => Some(Ok(vec![0xb7])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as ISO-8859-2",
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

        let mut decoder: DecoderIso8859_2 = DecoderIso8859_2::new(&byte_string);

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

        let mut encoder: EncoderIso8859_2 = EncoderIso8859_2::new(&code_points);

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

        let mut encoder: EncoderIso8859_2 = EncoderIso8859_2::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
