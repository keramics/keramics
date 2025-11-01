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
    const BASE_0X80: [u16; 128] = [
        0x0410, 0x0411, 0x0412, 0x0413, 0x0414, 0x0415, 0x0416, 0x0417, 0x0418, 0x0419, 0x041a,
        0x041b, 0x041c, 0x041d, 0x041e, 0x041f, 0x0420, 0x0421, 0x0422, 0x0423, 0x0424, 0x0425,
        0x0426, 0x0427, 0x0428, 0x0429, 0x042a, 0x042b, 0x042c, 0x042d, 0x042e, 0x042f, 0x2020,
        0x00b0, 0x0490, 0x00a3, 0x00a7, 0x2022, 0x00b6, 0x0406, 0x00ae, 0x00a9, 0x2122, 0x0402,
        0x0452, 0x2260, 0x0403, 0x0453, 0x221e, 0x00b1, 0x2264, 0x2265, 0x0456, 0x00b5, 0x0491,
        0x0408, 0x0404, 0x0454, 0x0407, 0x0457, 0x0409, 0x0459, 0x040a, 0x045a, 0x0458, 0x0405,
        0x00ac, 0x221a, 0x0192, 0x2248, 0x2206, 0x00ab, 0x00bb, 0x2026, 0x00a0, 0x040b, 0x045b,
        0x040c, 0x045c, 0x0455, 0x2013, 0x2014, 0x201c, 0x201d, 0x2018, 0x2019, 0x00f7, 0x201e,
        0x040e, 0x045e, 0x040f, 0x045f, 0x2116, 0x0401, 0x0451, 0x044f, 0x0430, 0x0431, 0x0432,
        0x0433, 0x0434, 0x0435, 0x0436, 0x0437, 0x0438, 0x0439, 0x043a, 0x043b, 0x043c, 0x043d,
        0x043e, 0x043f, 0x0440, 0x0441, 0x0442, 0x0443, 0x0444, 0x0445, 0x0446, 0x0447, 0x0448,
        0x0449, 0x044a, 0x044b, 0x044c, 0x044d, 0x044e, 0x20ac,
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
        let code_points: [u32; 1] = [0x00a1];

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x0400];

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacCyrillic = EncoderMacCyrillic::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
