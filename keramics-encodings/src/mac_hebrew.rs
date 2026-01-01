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

//! MacHebrew encoding.
//!
//! Provides support for encoding and decoding MacHebrew.

use keramics_core::ErrorTrace;

/// MacHebrew decoder.
pub struct DecoderMacHebrew<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacHebrew<'a> {
    const BASE_0X80: [&'static [u16]; 128] = [
        &[0x00c4],
        &[0x05f2, 0x05b7],
        &[0x00c7],
        &[0x00c9],
        &[0x00d1],
        &[0x00d6],
        &[0x00dc],
        &[0x00e1],
        &[0x00e0],
        &[0x00e2],
        &[0x00e4],
        &[0x00e3],
        &[0x00e5],
        &[0x00e7],
        &[0x00e9],
        &[0x00e8],
        &[0x00ea],
        &[0x00eb],
        &[0x00ed],
        &[0x00ec],
        &[0x00ee],
        &[0x00ef],
        &[0x00f1],
        &[0x00f3],
        &[0x00f2],
        &[0x00f4],
        &[0x00f6],
        &[0x00f5],
        &[0x00fa],
        &[0x00f9],
        &[0x00fb],
        &[0x00fc],
        &[0x0020],
        &[0x0021],
        &[0x0022],
        &[0x0023],
        &[0x0024],
        &[0x0025],
        &[0x20aa],
        &[0x0027],
        &[0x0029],
        &[0x0028],
        &[0x002a],
        &[0x002b],
        &[0x002c],
        &[0x002d],
        &[0x002e],
        &[0x002f],
        &[0x0030],
        &[0x0031],
        &[0x0032],
        &[0x0033],
        &[0x0034],
        &[0x0035],
        &[0x0036],
        &[0x0037],
        &[0x0038],
        &[0x0039],
        &[0x003a],
        &[0x003b],
        &[0x003c],
        &[0x003d],
        &[0x003e],
        &[0x003f],
        &[0xf86a, 0x05dc, 0x05b9],
        &[0x201e],
        &[0xf89b],
        &[0xf89c],
        &[0xf89d],
        &[0xf89e],
        &[0x05bc],
        &[0xfb4b],
        &[0xfb35],
        &[0x2026],
        &[0x00a0],
        &[0x05b8],
        &[0x05b7],
        &[0x05b5],
        &[0x05b6],
        &[0x05b4],
        &[0x2013],
        &[0x2014],
        &[0x201c],
        &[0x201d],
        &[0x2018],
        &[0x2019],
        &[0xfb2a],
        &[0xfb2b],
        &[0x05bf],
        &[0x05b0],
        &[0x05b2],
        &[0x05b1],
        &[0x05bb],
        &[0x05b9],
        &[0x05c7],
        &[0x05b3],
        &[0x05d0],
        &[0x05d1],
        &[0x05d2],
        &[0x05d3],
        &[0x05d4],
        &[0x05d5],
        &[0x05d6],
        &[0x05d7],
        &[0x05d8],
        &[0x05d9],
        &[0x05da],
        &[0x05db],
        &[0x05dc],
        &[0x05dd],
        &[0x05de],
        &[0x05df],
        &[0x05e0],
        &[0x05e1],
        &[0x05e2],
        &[0x05e3],
        &[0x05e4],
        &[0x05e5],
        &[0x05e6],
        &[0x05e7],
        &[0x05e8],
        &[0x05e9],
        &[0x05ea],
        &[0x007d],
        &[0x005d],
        &[0x007b],
        &[0x005b],
        &[0x007c],
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacHebrew<'a> {
    type Item = Result<Vec<u32>, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                let code_points: &[u16] = if *byte_value < 0x80 {
                    &[*byte_value as u16]
                } else {
                    Self::BASE_0X80[(*byte_value - 0x80) as usize]
                };
                Some(Ok(code_points
                    .iter()
                    .map(|code_point| *code_point as u32)
                    .collect()))
            }
            None => None,
        }
    }
}

/// MacHebrew encoder.
pub struct EncoderMacHebrew<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacHebrew<'a> {
    const BASE_0X00C0: [Option<&'static [u8]>; 64] = [
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
        Some(&[0x8b]),
        Some(&[0x8a]),
        Some(&[0x8c]),
        None,
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
        None,
        None,
        Some(&[0x9d]),
        Some(&[0x9c]),
        Some(&[0x9e]),
        Some(&[0x9f]),
        None,
        None,
        None,
    ];

    const BASE_0X05B0: [Option<&'static [u8]>; 64] = [
        Some(&[0xd9]),
        Some(&[0xdb]),
        Some(&[0xda]),
        Some(&[0xdf]),
        Some(&[0xcf]),
        Some(&[0xcd]),
        Some(&[0xce]),
        Some(&[0xcc]),
        Some(&[0xcb]),
        Some(&[0xdd]),
        None,
        Some(&[0xdc]),
        Some(&[0xc6]),
        None,
        None,
        Some(&[0xd8]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xde]),
        None,
        None,
        None,
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
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf5]),
        Some(&[0xf6]),
        Some(&[0xf7]),
        Some(&[0xf8]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        None,
        None,
        None,
        None,
        None,
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
        Some(&[0xc1]),
        None,
        None,
        None,
        None,
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

impl<'a> Iterator for EncoderMacHebrew<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                let first_additional_code_point: u32 = match *code_point {
                    0x05f2 | 0xf86a => match self.code_points.get(self.code_point_index) {
                        Some(value) => {
                            self.code_point_index += 1;

                            *value
                        }
                        None => {
                            return Some(Err(keramics_core::error_trace_new!(format!(
                                "Unable to encode code point: U+{:04x} as MacHebrew - missing additional code point",
                                *code_point
                            ))));
                        }
                    },
                    _ => 0,
                };
                let second_additional_code_point: u32 = match *code_point {
                    0xf86a => match self.code_points.get(self.code_point_index) {
                        Some(value) => {
                            self.code_point_index += 1;

                            *value
                        }
                        None => {
                            return Some(Err(keramics_core::error_trace_new!(format!(
                                "Unable to encode code point: U+{:04x} U+{:04x} as MacHebrew - missing additional code point",
                                *code_point, first_additional_code_point
                            ))));
                        }
                    },
                    _ => 0,
                };
                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x00c0..0x0100 => match Self::BASE_0X00C0[(*code_point - 0x00c0) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacHebrew",
                            *code_point
                        )))),
                    },
                    0x05b0..0x05f0 => match Self::BASE_0X05B0[(*code_point - 0x05b0) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacHebrew",
                            *code_point
                        )))),
                    },
                    0x2010..0x2028 => match Self::BASE_0X2010[(*code_point - 0x2010) as usize] {
                        Some(bytes) => Some(Ok(bytes.to_vec())),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacHebrew",
                            *code_point
                        )))),
                    },
                    0x00a0 => Some(Ok(vec![0xca])),
                    0x05f2 => {
                        if first_additional_code_point == 0x05b7 {
                            Some(Ok(vec![0x81]))
                        } else {
                            Some(Err(keramics_core::error_trace_new!(format!(
                                "Unable to encode code point: U+{:04x} U+{:04x} as MacHebrew",
                                *code_point, first_additional_code_point
                            ))))
                        }
                    }
                    0x20aa => Some(Ok(vec![0xa6])),
                    0xf89b => Some(Ok(vec![0xc2])),
                    0xf89c => Some(Ok(vec![0xc3])),
                    0xf89d => Some(Ok(vec![0xc4])),
                    0xf89e => Some(Ok(vec![0xc5])),
                    0xf86a => {
                        if first_additional_code_point == 0x05dc
                            && second_additional_code_point == 0x05b9
                        {
                            Some(Ok(vec![0xc0]))
                        } else {
                            Some(Err(keramics_core::error_trace_new!(format!(
                                "Unable to encode code point: U+{:04x} U+{:04x} U+{:04x} as MacHebrew",
                                *code_point,
                                first_additional_code_point,
                                second_additional_code_point
                            ))))
                        }
                    }
                    0xfb2a => Some(Ok(vec![0xd6])),
                    0xfb2b => Some(Ok(vec![0xd7])),
                    0xfb35 => Some(Ok(vec![0xc8])),
                    0xfb4b => Some(Ok(vec![0xc7])),
                    _ => Some(Err(keramics_core::error_trace_new!(format!(
                        "Unable to encode code point: U+{:04x} as MacHebrew",
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

        let mut decoder: DecoderMacHebrew = DecoderMacHebrew::new(&byte_string);

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

        let mut encoder: EncoderMacHebrew = EncoderMacHebrew::new(&code_points);

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
        let code_points: [u32; 1] = [0x00a4];

        let mut encoder: EncoderMacHebrew = EncoderMacHebrew::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderMacHebrew = EncoderMacHebrew::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacHebrew = EncoderMacHebrew::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
