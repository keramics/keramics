/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

//! MacGreek encoding.
//!
//! Provides support for encoding and decoding MacGreek.

use keramics_core::ErrorTrace;

/// MacGreek decoder.
pub struct DecoderMacGreek<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacGreek<'a> {
    const BASE_0X80: [u16; 128] = [
        0x00c4, 0x00b9, 0x00b2, 0x00c9, 0x00b3, 0x00d6, 0x00dc, 0x0385, 0x00e0, 0x00e2, 0x00e4,
        0x0384, 0x00a8, 0x00e7, 0x00e9, 0x00e8, 0x00ea, 0x00eb, 0x00a3, 0x2122, 0x00ee, 0x00ef,
        0x2022, 0x00bd, 0x2030, 0x00f4, 0x00f6, 0x00a6, 0x20ac, 0x00f9, 0x00fb, 0x00fc, 0x2020,
        0x0393, 0x0394, 0x0398, 0x039b, 0x039e, 0x03a0, 0x00df, 0x00ae, 0x00a9, 0x03a3, 0x03aa,
        0x00a7, 0x2260, 0x00b0, 0x00b7, 0x0391, 0x00b1, 0x2264, 0x2265, 0x00a5, 0x0392, 0x0395,
        0x0396, 0x0397, 0x0399, 0x039a, 0x039c, 0x03a6, 0x03ab, 0x03a8, 0x03a9, 0x03ac, 0x039d,
        0x00ac, 0x039f, 0x03a1, 0x2248, 0x03a4, 0x00ab, 0x00bb, 0x2026, 0x00a0, 0x03a5, 0x03a7,
        0x0386, 0x0388, 0x0153, 0x2013, 0x2015, 0x201c, 0x201d, 0x2018, 0x2019, 0x00f7, 0x0389,
        0x038a, 0x038c, 0x038e, 0x03ad, 0x03ae, 0x03af, 0x03cc, 0x038f, 0x03cd, 0x03b1, 0x03b2,
        0x03c8, 0x03b4, 0x03b5, 0x03c6, 0x03b3, 0x03b7, 0x03b9, 0x03be, 0x03ba, 0x03bb, 0x03bc,
        0x03bd, 0x03bf, 0x03c0, 0x03ce, 0x03c1, 0x03c3, 0x03c4, 0x03b8, 0x03c9, 0x03c2, 0x03c7,
        0x03c5, 0x03b6, 0x03ca, 0x03cb, 0x0390, 0x03b0, 0x00ad,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacGreek<'a> {
    type Item = Result<Vec<u32>, ErrorTrace>;

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
                Some(Ok(vec![code_point as u32]))
            }
            None => None,
        }
    }
}

/// MacGreek encoder.
pub struct EncoderMacGreek<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacGreek<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 96] = [
        Some(&[0xca]),
        None,
        None,
        Some(&[0x92]),
        None,
        Some(&[0xb4]),
        Some(&[0x9b]),
        Some(&[0xac]),
        Some(&[0x8c]),
        Some(&[0xa9]),
        None,
        Some(&[0xc7]),
        Some(&[0xc2]),
        Some(&[0xff]),
        Some(&[0xa8]),
        None,
        Some(&[0xae]),
        Some(&[0xb1]),
        Some(&[0x82]),
        Some(&[0x84]),
        None,
        None,
        None,
        Some(&[0xaf]),
        None,
        Some(&[0x81]),
        None,
        Some(&[0xc8]),
        None,
        Some(&[0x97]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x80]),
        None,
        None,
        None,
        None,
        Some(&[0x83]),
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
        Some(&[0x85]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0x86]),
        None,
        None,
        Some(&[0xa7]),
        Some(&[0x88]),
        None,
        Some(&[0x89]),
        None,
        Some(&[0x8a]),
        None,
        None,
        Some(&[0x8d]),
        Some(&[0x8f]),
        Some(&[0x8e]),
        Some(&[0x90]),
        Some(&[0x91]),
        None,
        None,
        Some(&[0x94]),
        Some(&[0x95]),
        None,
        None,
        None,
        None,
        Some(&[0x99]),
        None,
        Some(&[0x9a]),
        Some(&[0xd6]),
        None,
        Some(&[0x9d]),
        None,
        Some(&[0x9e]),
        Some(&[0x9f]),
        None,
        None,
        None,
    ];

    const BASE_0X0380: [Option<&'static [u8]>; 80] = [
        None,
        None,
        None,
        None,
        Some(&[0x8b]),
        Some(&[0x87]),
        Some(&[0xcd]),
        None,
        Some(&[0xce]),
        Some(&[0xd7]),
        Some(&[0xd8]),
        None,
        Some(&[0xd9]),
        None,
        Some(&[0xda]),
        Some(&[0xdf]),
        Some(&[0xfd]),
        Some(&[0xb0]),
        Some(&[0xb5]),
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xb6]),
        Some(&[0xb7]),
        Some(&[0xb8]),
        Some(&[0xa3]),
        Some(&[0xb9]),
        Some(&[0xba]),
        Some(&[0xa4]),
        Some(&[0xbb]),
        Some(&[0xc1]),
        Some(&[0xa5]),
        Some(&[0xc3]),
        Some(&[0xa6]),
        Some(&[0xc4]),
        None,
        Some(&[0xaa]),
        Some(&[0xc6]),
        Some(&[0xcb]),
        Some(&[0xbc]),
        Some(&[0xcc]),
        Some(&[0xbe]),
        Some(&[0xbf]),
        Some(&[0xab]),
        Some(&[0xbd]),
        Some(&[0xc0]),
        Some(&[0xdb]),
        Some(&[0xdc]),
        Some(&[0xdd]),
        Some(&[0xfe]),
        Some(&[0xe1]),
        Some(&[0xe2]),
        Some(&[0xe7]),
        Some(&[0xe4]),
        Some(&[0xe5]),
        Some(&[0xfa]),
        Some(&[0xe8]),
        Some(&[0xf5]),
        Some(&[0xe9]),
        Some(&[0xeb]),
        Some(&[0xec]),
        Some(&[0xed]),
        Some(&[0xee]),
        Some(&[0xea]),
        Some(&[0xef]),
        Some(&[0xf0]),
        Some(&[0xf2]),
        Some(&[0xf7]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf9]),
        Some(&[0xe6]),
        Some(&[0xf8]),
        Some(&[0xe3]),
        Some(&[0xf6]),
        Some(&[0xfb]),
        Some(&[0xfc]),
        Some(&[0xde]),
        Some(&[0xe0]),
        Some(&[0xf1]),
        None,
    ];

    const BASE_0X2010: [Option<&'static [u8]>; 24] = [
        None,
        None,
        None,
        Some(&[0xd0]),
        None,
        Some(&[0xd1]),
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
        None,
        Some(&[0x96]),
        None,
        None,
        None,
        Some(&[0xc9]),
        None,
    ];

    /// Creates a new encoder.
    pub fn new(code_points: &'a [u32]) -> Self {
        Self {
            code_points,
            code_point_index: 0,
        }
    }
}

impl<'a> Iterator for EncoderMacGreek<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00a0..0x0100 => match Self::BASE_0X00A0[(*code_point - 0x00a0) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacGreek",
                            *code_point
                        )))),
                    },
                    0x0380..0x03d0 => match Self::BASE_0X0380[(*code_point - 0x0380) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacGreek",
                            *code_point
                        )))),
                    },
                    0x2010..0x2028 => match Self::BASE_0X2010[(*code_point - 0x2010) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacGreek",
                            *code_point
                        )))),
                    },
                    0x0153 => Some(Ok(vec![0xcf])),
                    0x20ac => Some(Ok(vec![0x9c])),
                    0x2122 => Some(Ok(vec![0x93])),
                    0x2248 => Some(Ok(vec![0xc5])),
                    0x2260 => Some(Ok(vec![0xad])),
                    0x2264 => Some(Ok(vec![0xb2])),
                    0x2265 => Some(Ok(vec![0xb3])),
                    0x2030 => Some(Ok(vec![0x98])),
                    _ => Some(Err(keramics_core::error_trace_new!(format!(
                        "Unable to encode code point: U+{:04x} as MacGreek",
                        *code_point
                    )))),
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

        let mut decoder: DecoderMacGreek = DecoderMacGreek::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(vec![0x0000004b])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000065])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000072])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000061])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x0000006d])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000069])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000063])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x00000073])));
        assert_eq!(decoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderMacGreek = EncoderMacGreek::new(&code_points);

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

        let mut encoder: EncoderMacGreek = EncoderMacGreek::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x0380];

        let mut encoder: EncoderMacGreek = EncoderMacGreek::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderMacGreek = EncoderMacGreek::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacGreek = EncoderMacGreek::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
