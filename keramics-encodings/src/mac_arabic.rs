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
    const BASE_0X80: [u16; 128] = [
        0x00c4, 0x00a0, 0x00c7, 0x00c9, 0x00d1, 0x00d6, 0x00dc, 0x00e1, 0x00e0, 0x00e2, 0x00e4,
        0x06ba, 0x00ab, 0x00e7, 0x00e9, 0x00e8, 0x00ea, 0x00eb, 0x00ed, 0x2026, 0x00ee, 0x00ef,
        0x00f1, 0x00f3, 0x00bb, 0x00f4, 0x00f6, 0x00f7, 0x00fa, 0x00f9, 0x00fb, 0x00fc, 0x0020,
        0x0021, 0x0022, 0x0023, 0x0024, 0x066a, 0x0026, 0x0027, 0x0028, 0x0029, 0x002a, 0x002b,
        0x060c, 0x002d, 0x002e, 0x002f, 0x0660, 0x0661, 0x0662, 0x0663, 0x0664, 0x0665, 0x0666,
        0x0667, 0x0668, 0x0669, 0x003a, 0x061b, 0x003c, 0x003d, 0x003e, 0x061f, 0x274a, 0x0621,
        0x0622, 0x0623, 0x0624, 0x0625, 0x0626, 0x0627, 0x0628, 0x0629, 0x062a, 0x062b, 0x062c,
        0x062d, 0x062e, 0x062f, 0x0630, 0x0631, 0x0632, 0x0633, 0x0634, 0x0635, 0x0636, 0x0637,
        0x0638, 0x0639, 0x063a, 0x005b, 0x005c, 0x005d, 0x005e, 0x005f, 0x0640, 0x0641, 0x0642,
        0x0643, 0x0644, 0x0645, 0x0646, 0x0647, 0x0648, 0x0649, 0x064a, 0x064b, 0x064c, 0x064d,
        0x064e, 0x064f, 0x0650, 0x0651, 0x0652, 0x067e, 0x0679, 0x0686, 0x06d5, 0x06a4, 0x06af,
        0x0688, 0x0691, 0x007b, 0x007c, 0x007d, 0x0698, 0x06d2,
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

/// MacArabic encoder.
pub struct EncoderMacArabic<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacArabic<'a> {
    const BASE_0X0020: [&'static [u8]; 32] = [
        &[0xa0],
        &[0xa1],
        &[0xa2],
        &[0xa3],
        &[0xa4],
        &[0x25],
        &[0xa6],
        &[0xa7],
        &[0xa8],
        &[0xa9],
        &[0xaa],
        &[0xab],
        &[0x2c],
        &[0xad],
        &[0xae],
        &[0xaf],
        &[0x30],
        &[0x31],
        &[0x32],
        &[0x33],
        &[0x34],
        &[0x35],
        &[0x36],
        &[0x37],
        &[0x38],
        &[0x39],
        &[0xba],
        &[0x3b],
        &[0xbc],
        &[0xbd],
        &[0xbe],
        &[0x3f],
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
                        let bytes: &[u8] =
                            Self::BASE_0X0020[(*code_point as u32 - 0x0020) as usize];

                        Some(Ok(bytes.to_vec()))
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
        let code_points: [u32; 1] = [0x00a1];

        let mut encoder: EncoderMacArabic = EncoderMacArabic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x0608];

        let mut encoder: EncoderMacArabic = EncoderMacArabic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacArabic = EncoderMacArabic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
