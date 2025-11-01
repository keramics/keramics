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

//! Windows 1257 (Baltic) encoding.
//!
//! Provides support for encoding and decoding Windows 1257.

use keramics_core::ErrorTrace;

/// Windows 1257 decoder.
pub struct DecoderWindows1257<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderWindows1257<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x20ac),
        None,
        Some(0x201a),
        None,
        Some(0x201e),
        Some(0x2026),
        Some(0x2020),
        Some(0x2021),
        None,
        Some(0x2030),
        None,
        Some(0x2039),
        None,
        Some(0x00a8),
        Some(0x02c7),
        Some(0x00b8),
        None,
        Some(0x2018),
        Some(0x2019),
        Some(0x201c),
        Some(0x201d),
        Some(0x2022),
        Some(0x2013),
        Some(0x2014),
        None,
        Some(0x2122),
        None,
        Some(0x203a),
        None,
        Some(0x00af),
        Some(0x02db),
        None,
        Some(0x00a0),
        None,
        Some(0x00a2),
        Some(0x00a3),
        Some(0x00a4),
        None,
        Some(0x00a6),
        Some(0x00a7),
        Some(0x00d8),
        Some(0x00a9),
        Some(0x0156),
        Some(0x00ab),
        Some(0x00ac),
        Some(0x00ad),
        Some(0x00ae),
        Some(0x00c6),
        Some(0x00b0),
        Some(0x00b1),
        Some(0x00b2),
        Some(0x00b3),
        Some(0x00b4),
        Some(0x00b5),
        Some(0x00b6),
        Some(0x00b7),
        Some(0x00f8),
        Some(0x00b9),
        Some(0x0157),
        Some(0x00bb),
        Some(0x00bc),
        Some(0x00bd),
        Some(0x00be),
        Some(0x00e6),
        Some(0x0104),
        Some(0x012e),
        Some(0x0100),
        Some(0x0106),
        Some(0x00c4),
        Some(0x00c5),
        Some(0x0118),
        Some(0x0112),
        Some(0x010c),
        Some(0x00c9),
        Some(0x0179),
        Some(0x0116),
        Some(0x0122),
        Some(0x0136),
        Some(0x012a),
        Some(0x013b),
        Some(0x0160),
        Some(0x0143),
        Some(0x0145),
        Some(0x00d3),
        Some(0x014c),
        Some(0x00d5),
        Some(0x00d6),
        Some(0x00d7),
        Some(0x0172),
        Some(0x0141),
        Some(0x015a),
        Some(0x016a),
        Some(0x00dc),
        Some(0x017b),
        Some(0x017d),
        Some(0x00df),
        Some(0x0105),
        Some(0x012f),
        Some(0x0101),
        Some(0x0107),
        Some(0x00e4),
        Some(0x00e5),
        Some(0x0119),
        Some(0x0113),
        Some(0x010d),
        Some(0x00e9),
        Some(0x017a),
        Some(0x0117),
        Some(0x0123),
        Some(0x0137),
        Some(0x012b),
        Some(0x013c),
        Some(0x0161),
        Some(0x0144),
        Some(0x0146),
        Some(0x00f3),
        Some(0x014d),
        Some(0x00f5),
        Some(0x00f6),
        Some(0x00f7),
        Some(0x0173),
        Some(0x0142),
        Some(0x015b),
        Some(0x016b),
        Some(0x00fc),
        Some(0x017c),
        Some(0x017e),
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

impl<'a> Iterator for DecoderWindows1257<'a> {
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
                            "Unable to decode Windows 1257: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// Windows 1257 encoder.
pub struct EncoderWindows1257<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderWindows1257<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 224] = [
        Some(&[0xa0]),
        None,
        Some(&[0xa2]),
        Some(&[0xa3]),
        Some(&[0xa4]),
        None,
        Some(&[0xa6]),
        Some(&[0xa7]),
        Some(&[0x8d]),
        Some(&[0xa9]),
        None,
        Some(&[0xab]),
        Some(&[0xac]),
        Some(&[0xad]),
        Some(&[0xae]),
        Some(&[0x9d]),
        Some(&[0xb0]),
        Some(&[0xb1]),
        Some(&[0xb2]),
        Some(&[0xb3]),
        Some(&[0xb4]),
        Some(&[0xb5]),
        Some(&[0xb6]),
        Some(&[0xb7]),
        Some(&[0x8f]),
        Some(&[0xb9]),
        None,
        Some(&[0xbb]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        Some(&[0xbe]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0xc4]),
        Some(&[0xc5]),
        Some(&[0xaf]),
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
        Some(&[0xd3]),
        None,
        Some(&[0xd5]),
        Some(&[0xd6]),
        Some(&[0xd7]),
        Some(&[0xa8]),
        None,
        None,
        None,
        Some(&[0xdc]),
        None,
        None,
        Some(&[0xdf]),
        None,
        None,
        None,
        None,
        Some(&[0xe4]),
        Some(&[0xe5]),
        Some(&[0xbf]),
        None,
        None,
        Some(&[0xe9]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xf3]),
        None,
        Some(&[0xf5]),
        Some(&[0xf6]),
        Some(&[0xf7]),
        Some(&[0xb8]),
        None,
        None,
        None,
        Some(&[0xfc]),
        None,
        None,
        None,
        Some(&[0xc2]),
        Some(&[0xe2]),
        None,
        None,
        Some(&[0xc0]),
        Some(&[0xe0]),
        Some(&[0xc3]),
        Some(&[0xe3]),
        None,
        None,
        None,
        None,
        Some(&[0xc8]),
        Some(&[0xe8]),
        None,
        None,
        None,
        None,
        Some(&[0xc7]),
        Some(&[0xe7]),
        None,
        None,
        Some(&[0xcb]),
        Some(&[0xeb]),
        Some(&[0xc6]),
        Some(&[0xe6]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xcc]),
        Some(&[0xec]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xce]),
        Some(&[0xee]),
        None,
        None,
        Some(&[0xc1]),
        Some(&[0xe1]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xcd]),
        Some(&[0xed]),
        None,
        None,
        None,
        Some(&[0xcf]),
        Some(&[0xef]),
        None,
        None,
        None,
        None,
        Some(&[0xd9]),
        Some(&[0xf9]),
        Some(&[0xd1]),
        Some(&[0xf1]),
        Some(&[0xd2]),
        Some(&[0xf2]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0xd4]),
        Some(&[0xf4]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xaa]),
        Some(&[0xba]),
        None,
        None,
        Some(&[0xda]),
        Some(&[0xfa]),
        None,
        None,
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
        None,
        None,
        Some(&[0xdb]),
        Some(&[0xfb]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xd8]),
        Some(&[0xf8]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0xca]),
        Some(&[0xea]),
        Some(&[0xdd]),
        Some(&[0xfd]),
        Some(&[0xde]),
        Some(&[0xfe]),
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

impl<'a> Iterator for EncoderWindows1257<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x0180 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 1257",
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
                                    "Unable to encode code point: U+{:04x} as Windows 1257",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x02c7 => Some(Ok(vec![0x8e])),
                    0x02d9 => Some(Ok(vec![0xff])),
                    0x02db => Some(Ok(vec![0x9e])),
                    0x2030 => Some(Ok(vec![0x89])),
                    0x2039 => Some(Ok(vec![0x8b])),
                    0x203a => Some(Ok(vec![0x9b])),
                    0x20ac => Some(Ok(vec![0x80])),
                    0x2122 => Some(Ok(vec![0x99])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as Windows 1257",
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

        let mut decoder: DecoderWindows1257 = DecoderWindows1257::new(&byte_string);

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

        let mut decoder: DecoderWindows1257 = DecoderWindows1257::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderWindows1257 = EncoderWindows1257::new(&code_points);

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

        let mut encoder: EncoderWindows1257 = EncoderWindows1257::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderWindows1257 = EncoderWindows1257::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderWindows1257 = EncoderWindows1257::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
