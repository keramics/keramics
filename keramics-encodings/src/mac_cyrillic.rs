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

//! MacCyrillic encoding.
//!
//! Provides support for encoding and decoding MacCyrillic.

use keramics_core::ErrorTrace;

/// MacCyrillic decoder.
pub struct DecoderMacCyrillic<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacCyrillic<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x0410),
        Some(0x0411),
        Some(0x0412),
        Some(0x0413),
        Some(0x0414),
        Some(0x0415),
        Some(0x0416),
        Some(0x0417),
        Some(0x0418),
        Some(0x0419),
        Some(0x041a),
        Some(0x041b),
        Some(0x041c),
        Some(0x041d),
        Some(0x041e),
        Some(0x041f),
        Some(0x0420),
        Some(0x0421),
        Some(0x0422),
        Some(0x0423),
        Some(0x0424),
        Some(0x0425),
        Some(0x0426),
        Some(0x0427),
        Some(0x0428),
        Some(0x0429),
        Some(0x042a),
        Some(0x042b),
        Some(0x042c),
        Some(0x042d),
        Some(0x042e),
        Some(0x042f),
        Some(0x2020),
        Some(0x00b0),
        Some(0x0490),
        Some(0x00a3),
        Some(0x00a7),
        Some(0x2022),
        Some(0x00b6),
        Some(0x0406),
        Some(0x00ae),
        Some(0x00a9),
        Some(0x2122),
        Some(0x0402),
        Some(0x0452),
        Some(0x2260),
        Some(0x0403),
        Some(0x0453),
        Some(0x221e),
        Some(0x00b1),
        Some(0x2264),
        Some(0x2265),
        Some(0x0456),
        Some(0x00b5),
        Some(0x0491),
        Some(0x0408),
        Some(0x0404),
        Some(0x0454),
        Some(0x0407),
        Some(0x0457),
        Some(0x0409),
        Some(0x0459),
        Some(0x040a),
        Some(0x045a),
        Some(0x0458),
        Some(0x0405),
        Some(0x00ac),
        Some(0x221a),
        Some(0x0192),
        Some(0x2248),
        Some(0x2206),
        Some(0x00ab),
        Some(0x00bb),
        Some(0x2026),
        Some(0x00a0),
        Some(0x040b),
        Some(0x045b),
        Some(0x040c),
        Some(0x045c),
        Some(0x0455),
        Some(0x2013),
        Some(0x2014),
        Some(0x201c),
        Some(0x201d),
        Some(0x2018),
        Some(0x2019),
        Some(0x00f7),
        Some(0x201e),
        Some(0x040e),
        Some(0x045e),
        Some(0x040f),
        Some(0x045f),
        Some(0x2116),
        Some(0x0401),
        Some(0x0451),
        Some(0x044f),
        Some(0x0430),
        Some(0x0431),
        Some(0x0432),
        Some(0x0433),
        Some(0x0434),
        Some(0x0435),
        Some(0x0436),
        Some(0x0437),
        Some(0x0438),
        Some(0x0439),
        Some(0x043a),
        Some(0x043b),
        Some(0x043c),
        Some(0x043d),
        Some(0x043e),
        Some(0x043f),
        Some(0x0440),
        Some(0x0441),
        Some(0x0442),
        Some(0x0443),
        Some(0x0444),
        Some(0x0445),
        Some(0x0446),
        Some(0x0447),
        Some(0x0448),
        Some(0x0449),
        Some(0x044a),
        Some(0x044b),
        Some(0x044c),
        Some(0x044d),
        Some(0x044e),
        Some(0x20ac),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacCyrillic<'a> {
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
                            "Unable to decode MacCyrillic: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// MacCyrillic encoder.
pub struct EncoderMacCyrillic<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacCyrillic<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 32] = [
        Some(&[0xca]),
        None,
        None,
        Some(&[0xa3]),
        None,
        None,
        None,
        Some(&[0xa4]),
        None,
        Some(&[0xa9]),
        None,
        Some(&[0xc7]),
        Some(&[0xc2]),
        None,
        Some(&[0xa8]),
        None,
        Some(&[0xa1]),
        Some(&[0xb1]),
        None,
        None,
        None,
        Some(&[0xb5]),
        Some(&[0xa6]),
        None,
        None,
        None,
        None,
        Some(&[0xc8]),
        None,
        None,
        None,
        None,
    ];

    const BASE_0X0400: [Option<&'static [u8]>; 96] = [
        None,
        Some(&[0xdd]),
        Some(&[0xab]),
        Some(&[0xae]),
        Some(&[0xb8]),
        Some(&[0xc1]),
        Some(&[0xa7]),
        Some(&[0xba]),
        Some(&[0xb7]),
        Some(&[0xbc]),
        Some(&[0xbe]),
        Some(&[0xcb]),
        Some(&[0xcd]),
        None,
        Some(&[0xd8]),
        Some(&[0xda]),
        Some(&[0x80]),
        Some(&[0x81]),
        Some(&[0x82]),
        Some(&[0x83]),
        Some(&[0x84]),
        Some(&[0x85]),
        Some(&[0x86]),
        Some(&[0x87]),
        Some(&[0x88]),
        Some(&[0x89]),
        Some(&[0x8a]),
        Some(&[0x8b]),
        Some(&[0x8c]),
        Some(&[0x8d]),
        Some(&[0x8e]),
        Some(&[0x8f]),
        Some(&[0x90]),
        Some(&[0x91]),
        Some(&[0x92]),
        Some(&[0x93]),
        Some(&[0x94]),
        Some(&[0x95]),
        Some(&[0x96]),
        Some(&[0x97]),
        Some(&[0x98]),
        Some(&[0x99]),
        Some(&[0x9a]),
        Some(&[0x9b]),
        Some(&[0x9c]),
        Some(&[0x9d]),
        Some(&[0x9e]),
        Some(&[0x9f]),
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
        Some(&[0xfc]),
        Some(&[0xfd]),
        Some(&[0xfe]),
        Some(&[0xdf]),
        None,
        Some(&[0xde]),
        Some(&[0xac]),
        Some(&[0xaf]),
        Some(&[0xb9]),
        Some(&[0xcf]),
        Some(&[0xb4]),
        Some(&[0xbb]),
        Some(&[0xc0]),
        Some(&[0xbd]),
        Some(&[0xbf]),
        Some(&[0xcc]),
        Some(&[0xce]),
        None,
        Some(&[0xd9]),
        Some(&[0xdb]),
    ];

    const BASE_0X2010: [Option<&'static [u8]>; 24] = [
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
        Some(&[0xd7]),
        None,
        Some(&[0xa0]),
        None,
        Some(&[0xa5]),
        None,
        None,
        None,
        Some(&[0xc9]),
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

impl<'a> Iterator for EncoderMacCyrillic<'a> {
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
                                    "Unable to encode code point: U+{:04x} as MacCyrillic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0400..0x0460 => {
                        match Self::BASE_0X0400[(*code_point as u32 - 0x0400) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacCyrillic",
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
                                    "Unable to encode code point: U+{:04x} as MacCyrillic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00f7 => Some(Ok(vec![0xd6])),
                    0x0192 => Some(Ok(vec![0xc4])),
                    0x0490 => Some(Ok(vec![0xa2])),
                    0x0491 => Some(Ok(vec![0xb6])),
                    0x20ac => Some(Ok(vec![0xff])),
                    0x2116 => Some(Ok(vec![0xdc])),
                    0x2122 => Some(Ok(vec![0xaa])),
                    0x2206 => Some(Ok(vec![0xc6])),
                    0x221a => Some(Ok(vec![0xc3])),
                    0x221e => Some(Ok(vec![0xb0])),
                    0x2248 => Some(Ok(vec![0xc5])),
                    0x2260 => Some(Ok(vec![0xad])),
                    0x2264 => Some(Ok(vec![0xb2])),
                    0x2265 => Some(Ok(vec![0xb3])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacCyrillic",
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

        let mut decoder: DecoderMacCyrillic = DecoderMacCyrillic::new(&byte_string);

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

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

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
        let code_points: [u32; 1] = [0x9676];

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
