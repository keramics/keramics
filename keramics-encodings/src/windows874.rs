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

//! Windows 874 (Thai) encoding.
//!
//! Provides support for encoding and decoding Windows 874.

use keramics_core::ErrorTrace;

/// Windows 874 decoder.
pub struct DecoderWindows874<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderWindows874<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x20ac),
        None,
        None,
        None,
        None,
        Some(0x2026),
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
        Some(0x2018),
        Some(0x2019),
        Some(0x201c),
        Some(0x201d),
        Some(0x2022),
        Some(0x2013),
        Some(0x2014),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(0x00a0),
        Some(0x0e01),
        Some(0x0e02),
        Some(0x0e03),
        Some(0x0e04),
        Some(0x0e05),
        Some(0x0e06),
        Some(0x0e07),
        Some(0x0e08),
        Some(0x0e09),
        Some(0x0e0a),
        Some(0x0e0b),
        Some(0x0e0c),
        Some(0x0e0d),
        Some(0x0e0e),
        Some(0x0e0f),
        Some(0x0e10),
        Some(0x0e11),
        Some(0x0e12),
        Some(0x0e13),
        Some(0x0e14),
        Some(0x0e15),
        Some(0x0e16),
        Some(0x0e17),
        Some(0x0e18),
        Some(0x0e19),
        Some(0x0e1a),
        Some(0x0e1b),
        Some(0x0e1c),
        Some(0x0e1d),
        Some(0x0e1e),
        Some(0x0e1f),
        Some(0x0e20),
        Some(0x0e21),
        Some(0x0e22),
        Some(0x0e23),
        Some(0x0e24),
        Some(0x0e25),
        Some(0x0e26),
        Some(0x0e27),
        Some(0x0e28),
        Some(0x0e29),
        Some(0x0e2a),
        Some(0x0e2b),
        Some(0x0e2c),
        Some(0x0e2d),
        Some(0x0e2e),
        Some(0x0e2f),
        Some(0x0e30),
        Some(0x0e31),
        Some(0x0e32),
        Some(0x0e33),
        Some(0x0e34),
        Some(0x0e35),
        Some(0x0e36),
        Some(0x0e37),
        Some(0x0e38),
        Some(0x0e39),
        Some(0x0e3a),
        None,
        None,
        None,
        None,
        Some(0x0e3f),
        Some(0x0e40),
        Some(0x0e41),
        Some(0x0e42),
        Some(0x0e43),
        Some(0x0e44),
        Some(0x0e45),
        Some(0x0e46),
        Some(0x0e47),
        Some(0x0e48),
        Some(0x0e49),
        Some(0x0e4a),
        Some(0x0e4b),
        Some(0x0e4c),
        Some(0x0e4d),
        Some(0x0e4e),
        Some(0x0e4f),
        Some(0x0e50),
        Some(0x0e51),
        Some(0x0e52),
        Some(0x0e53),
        Some(0x0e54),
        Some(0x0e55),
        Some(0x0e56),
        Some(0x0e57),
        Some(0x0e58),
        Some(0x0e59),
        Some(0x0e5a),
        Some(0x0e5b),
        None,
        None,
        None,
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

impl<'a> Iterator for DecoderWindows874<'a> {
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
                            "Unable to decode Windows 874: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// Windows 874 encoder.
pub struct EncoderWindows874<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderWindows874<'a> {
    const BASE_0X0E00: [Option<&'static [u8]>; 96] = [
        None,
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        Some(&[0xa4]),
        Some(&[0xa5]),
        Some(&[0xa6]),
        Some(&[0xa7]),
        Some(&[0xa8]),
        Some(&[0xa9]),
        Some(&[0xaa]),
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
        Some(&[0xba]),
        Some(&[0xbb]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        Some(&[0xbe]),
        Some(&[0xbf]),
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
        Some(&[0xd0]),
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
        None,
        None,
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
        Some(&[0xfb]),
        None,
        None,
        None,
        None,
    ];

    const BASE_0X2018: [Option<&'static [u8]>; 8] = [
        Some(&[0x91]),
        Some(&[0x92]),
        None,
        None,
        Some(&[0x93]),
        Some(&[0x94]),
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

impl<'a> Iterator for EncoderWindows874<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 | 0x00a0 => Some(Ok(vec![*code_point as u8])),
                    0x0e00..0x0e60 => {
                        match Self::BASE_0X0E00[(*code_point as u32 - 0x0e00) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 874",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2018..0x2020 => {
                        match Self::BASE_0X2018[(*code_point as u32 - 0x2018) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as Windows 874",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2013 => Some(Ok(vec![0x96])),
                    0x2014 => Some(Ok(vec![0x97])),
                    0x2022 => Some(Ok(vec![0x95])),
                    0x2026 => Some(Ok(vec![0x85])),
                    0x20ac => Some(Ok(vec![0x80])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as Windows 874",
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

        let mut decoder: DecoderWindows874 = DecoderWindows874::new(&byte_string);

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

        let mut decoder: DecoderWindows874 = DecoderWindows874::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderWindows874 = EncoderWindows874::new(&code_points);

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
        let code_points: [u32; 1] = [0x0e00];

        let mut encoder: EncoderWindows874 = EncoderWindows874::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x201a];

        let mut encoder: EncoderWindows874 = EncoderWindows874::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderWindows874 = EncoderWindows874::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
