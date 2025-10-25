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

//! MacArabic encoding.
//!
//! Provides support for encoding and decoding MacArabic.

use keramics_core::ErrorTrace;

/// MacArabic decoder.
pub struct DecoderMacArabic<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacArabic<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x00c4),
        Some(0x00a0),
        Some(0x00c7),
        Some(0x00c9),
        Some(0x00d1),
        Some(0x00d6),
        Some(0x00dc),
        Some(0x00e1),
        Some(0x00e0),
        Some(0x00e2),
        Some(0x00e4),
        Some(0x06ba),
        Some(0x00ab),
        Some(0x00e7),
        Some(0x00e9),
        Some(0x00e8),
        Some(0x00ea),
        Some(0x00eb),
        Some(0x00ed),
        Some(0x2026),
        Some(0x00ee),
        Some(0x00ef),
        Some(0x00f1),
        Some(0x00f3),
        Some(0x00bb),
        Some(0x00f4),
        Some(0x00f6),
        Some(0x00f7),
        Some(0x00fa),
        Some(0x00f9),
        Some(0x00fb),
        Some(0x00fc),
        Some(0x0020),
        Some(0x0021),
        Some(0x0022),
        Some(0x0023),
        Some(0x0024),
        Some(0x066a),
        Some(0x0026),
        Some(0x0027),
        Some(0x0028),
        Some(0x0029),
        Some(0x002a),
        Some(0x002b),
        Some(0x060c),
        Some(0x002d),
        Some(0x002e),
        Some(0x002f),
        Some(0x0660),
        Some(0x0661),
        Some(0x0662),
        Some(0x0663),
        Some(0x0664),
        Some(0x0665),
        Some(0x0666),
        Some(0x0667),
        Some(0x0668),
        Some(0x0669),
        Some(0x003a),
        Some(0x061b),
        Some(0x003c),
        Some(0x003d),
        Some(0x003e),
        Some(0x061f),
        Some(0x274a),
        Some(0x0621),
        Some(0x0622),
        Some(0x0623),
        Some(0x0624),
        Some(0x0625),
        Some(0x0626),
        Some(0x0627),
        Some(0x0628),
        Some(0x0629),
        Some(0x062a),
        Some(0x062b),
        Some(0x062c),
        Some(0x062d),
        Some(0x062e),
        Some(0x062f),
        Some(0x0630),
        Some(0x0631),
        Some(0x0632),
        Some(0x0633),
        Some(0x0634),
        Some(0x0635),
        Some(0x0636),
        Some(0x0637),
        Some(0x0638),
        Some(0x0639),
        Some(0x063a),
        Some(0x005b),
        Some(0x005c),
        Some(0x005d),
        Some(0x005e),
        Some(0x005f),
        Some(0x0640),
        Some(0x0641),
        Some(0x0642),
        Some(0x0643),
        Some(0x0644),
        Some(0x0645),
        Some(0x0646),
        Some(0x0647),
        Some(0x0648),
        Some(0x0649),
        Some(0x064a),
        Some(0x064b),
        Some(0x064c),
        Some(0x064d),
        Some(0x064e),
        Some(0x064f),
        Some(0x0650),
        Some(0x0651),
        Some(0x0652),
        Some(0x067e),
        Some(0x0679),
        Some(0x0686),
        Some(0x06d5),
        Some(0x06a4),
        Some(0x06af),
        Some(0x0688),
        Some(0x0691),
        Some(0x007b),
        Some(0x007c),
        Some(0x007d),
        Some(0x0698),
        Some(0x06d2),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacArabic<'a> {
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
                            "Unable to decode MacArabic: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// MacArabic encoder.
pub struct EncoderMacArabic<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacArabic<'a> {
    const BASE_0X0020: [Option<&'static [u8]>; 32] = [
        Some(&[0xa0]),
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        Some(&[0xa4]),
        Some(&[0x25]),
        Some(&[0xa6]),
        Some(&[0xa7]),
        Some(&[0xa8]),
        Some(&[0xa9]),
        Some(&[0xaa]),
        Some(&[0xab]),
        Some(&[0x2c]),
        Some(&[0xad]),
        Some(&[0xae]),
        Some(&[0xaf]),
        Some(&[0x30]),
        Some(&[0x31]),
        Some(&[0x32]),
        Some(&[0x33]),
        Some(&[0x34]),
        Some(&[0x35]),
        Some(&[0x36]),
        Some(&[0x37]),
        Some(&[0x38]),
        Some(&[0x39]),
        Some(&[0xba]),
        Some(&[0x3b]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        Some(&[0xbe]),
        Some(&[0x3f]),
    ];

    const BASE_0X00A0: [Option<&'static [u8]>; 96] = [
        Some(&[0x81]),
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
        Some(&[0x8c]),
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
        None,
        None,
        None,
        Some(&[0x98]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x80]),
        None,
        None,
        Some(&[0x82]),
        None,
        Some(&[0x83]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x84]),
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
        None,
        Some(&[0x88]),
        Some(&[0x87]),
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
        Some(&[0x92]),
        Some(&[0x94]),
        Some(&[0x95]),
        None,
        Some(&[0x96]),
        None,
        Some(&[0x97]),
        Some(&[0x99]),
        None,
        Some(&[0x9a]),
        Some(&[0x9b]),
        None,
        Some(&[0x9d]),
        Some(&[0x9c]),
        Some(&[0x9e]),
        Some(&[0x9f]),
        None,
        None,
        None,
    ];

    const BASE_0X0608: [Option<&'static [u8]>; 184] = [
        None,
        None,
        None,
        None,
        Some(&[0xac]),
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
        None,
        None,
        Some(&[0xbb]),
        None,
        None,
        None,
        Some(&[0xbf]),
        None,
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
        None,
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
        Some(&[0xa5]),
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
        None,
        None,
        Some(&[0xf4]),
        None,
        None,
        None,
        None,
        Some(&[0xf3]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xf5]),
        None,
        Some(&[0xf9]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xfa]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xfe]),
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
        Some(&[0xf7]),
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
        Some(&[0xf8]),
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
        Some(&[0x8b]),
        None,
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

impl<'a> Iterator for EncoderMacArabic<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0020 | 0x0040..0x005b | 0x0060..0x007b | 0x007e..0x0080 => {
                        Some(Ok(vec![*code_point as u8]))
                    }
                    0x0020..0x0040 => {
                        match Self::BASE_0X0020[(*code_point as u32 - 0x0020) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacArabic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a0..0x0100 => {
                        match Self::BASE_0X00A0[(*code_point as u32 - 0x00a0) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacArabic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x0608..0x06c0 => {
                        match Self::BASE_0X0608[(*code_point as u32 - 0x0608) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacArabic",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x005b => Some(Ok(vec![0xdb])),
                    0x005c => Some(Ok(vec![0xdc])),
                    0x005d => Some(Ok(vec![0xdd])),
                    0x005e => Some(Ok(vec![0xde])),
                    0x005f => Some(Ok(vec![0xdf])),
                    0x007b => Some(Ok(vec![0xfb])),
                    0x007c => Some(Ok(vec![0xfc])),
                    0x007d => Some(Ok(vec![0xfd])),
                    0x06d2 => Some(Ok(vec![0xff])),
                    0x06d5 => Some(Ok(vec![0xf6])),
                    0x2026 => Some(Ok(vec![0x93])),
                    0x274a => Some(Ok(vec![0xc0])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacArabic",
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

        let mut decoder: DecoderMacArabic = DecoderMacArabic::new(&byte_string);

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

        let mut encoder: EncoderMacArabic = EncoderMacArabic::new(&code_points);

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

        let mut encoder: EncoderMacArabic = EncoderMacArabic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
