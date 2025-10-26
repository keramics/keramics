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

//! MacCentralEurRoman encoding.
//!
//! Provides support for encoding and decoding MacCentralEurRoman.

use keramics_core::ErrorTrace;

/// MacCentralEurRoman decoder.
pub struct DecoderMacCentralEurRoman<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacCentralEurRoman<'a> {
    const BASE_0X80: [u16; 128] = [
        0x00c4, 0x0100, 0x0101, 0x00c9, 0x0104, 0x00d6, 0x00dc, 0x00e1, 0x0105, 0x010c, 0x00e4,
        0x010d, 0x0106, 0x0107, 0x00e9, 0x0179, 0x017a, 0x010e, 0x00ed, 0x010f, 0x0112, 0x0113,
        0x0116, 0x00f3, 0x0117, 0x00f4, 0x00f6, 0x00f5, 0x00fa, 0x011a, 0x011b, 0x00fc, 0x2020,
        0x00b0, 0x0118, 0x00a3, 0x00a7, 0x2022, 0x00b6, 0x00df, 0x00ae, 0x00a9, 0x2122, 0x0119,
        0x00a8, 0x2260, 0x0123, 0x012e, 0x012f, 0x012a, 0x2264, 0x2265, 0x012b, 0x0136, 0x2202,
        0x2211, 0x0142, 0x013b, 0x013c, 0x013d, 0x013e, 0x0139, 0x013a, 0x0145, 0x0146, 0x0143,
        0x00ac, 0x221a, 0x0144, 0x0147, 0x2206, 0x00ab, 0x00bb, 0x2026, 0x00a0, 0x0148, 0x0150,
        0x00d5, 0x0151, 0x014c, 0x2013, 0x2014, 0x201c, 0x201d, 0x2018, 0x2019, 0x00f7, 0x25ca,
        0x014d, 0x0154, 0x0155, 0x0158, 0x2039, 0x203a, 0x0159, 0x0156, 0x0157, 0x0160, 0x201a,
        0x201e, 0x0161, 0x015a, 0x015b, 0x00c1, 0x0164, 0x0165, 0x00cd, 0x017d, 0x017e, 0x016a,
        0x00d3, 0x00d4, 0x016b, 0x016e, 0x00da, 0x016f, 0x0170, 0x0171, 0x0172, 0x0173, 0x00dd,
        0x00fd, 0x0137, 0x017b, 0x0141, 0x017c, 0x0122, 0x02c7,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacCentralEurRoman<'a> {
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

/// MacCentralEurRoman encoder.
pub struct EncoderMacCentralEurRoman<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacCentralEurRoman<'a> {
    const BASE_0X00A0: [Option<&'static [u8]>; 224] = [
        Some(&[0xca]),
        None,
        None,
        Some(&[0xa3]),
        None,
        None,
        None,
        Some(&[0xa4]),
        Some(&[0xac]),
        Some(&[0xa9]),
        None,
        Some(&[0xc7]),
        Some(&[0xc2]),
        None,
        Some(&[0xa8]),
        None,
        Some(&[0xa1]),
        None,
        None,
        None,
        None,
        None,
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
        None,
        Some(&[0xe7]),
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
        Some(&[0xea]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0xee]),
        Some(&[0xef]),
        Some(&[0xcd]),
        Some(&[0x85]),
        None,
        None,
        None,
        Some(&[0xf2]),
        None,
        Some(&[0x86]),
        Some(&[0xf8]),
        None,
        Some(&[0xa7]),
        None,
        Some(&[0x87]),
        None,
        None,
        Some(&[0x8a]),
        None,
        None,
        None,
        None,
        Some(&[0x8e]),
        None,
        None,
        None,
        Some(&[0x92]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0x97]),
        Some(&[0x99]),
        Some(&[0x9b]),
        Some(&[0x9a]),
        Some(&[0xd6]),
        None,
        None,
        Some(&[0x9c]),
        None,
        Some(&[0x9f]),
        Some(&[0xf9]),
        None,
        None,
        Some(&[0x81]),
        Some(&[0x82]),
        None,
        None,
        Some(&[0x84]),
        Some(&[0x88]),
        Some(&[0x8c]),
        Some(&[0x8d]),
        None,
        None,
        None,
        None,
        Some(&[0x89]),
        Some(&[0x8b]),
        Some(&[0x91]),
        Some(&[0x93]),
        None,
        None,
        Some(&[0x94]),
        Some(&[0x95]),
        None,
        None,
        Some(&[0x96]),
        Some(&[0x98]),
        Some(&[0xa2]),
        Some(&[0xab]),
        Some(&[0x9d]),
        Some(&[0x9e]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xfe]),
        Some(&[0xae]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xb1]),
        Some(&[0xb4]),
        None,
        None,
        Some(&[0xaf]),
        Some(&[0xb0]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xb5]),
        Some(&[0xfa]),
        None,
        Some(&[0xbd]),
        Some(&[0xbe]),
        Some(&[0xb9]),
        Some(&[0xba]),
        Some(&[0xbb]),
        Some(&[0xbc]),
        None,
        None,
        Some(&[0xfc]),
        Some(&[0xb8]),
        Some(&[0xc1]),
        Some(&[0xc4]),
        Some(&[0xbf]),
        Some(&[0xc0]),
        Some(&[0xc5]),
        Some(&[0xcb]),
        None,
        None,
        None,
        Some(&[0xcf]),
        Some(&[0xd8]),
        None,
        None,
        Some(&[0xcc]),
        Some(&[0xce]),
        None,
        None,
        Some(&[0xd9]),
        Some(&[0xda]),
        Some(&[0xdf]),
        Some(&[0xe0]),
        Some(&[0xdb]),
        Some(&[0xde]),
        Some(&[0xe5]),
        Some(&[0xe6]),
        None,
        None,
        None,
        None,
        Some(&[0xe1]),
        Some(&[0xe4]),
        None,
        None,
        Some(&[0xe8]),
        Some(&[0xe9]),
        None,
        None,
        None,
        None,
        Some(&[0xed]),
        Some(&[0xf0]),
        None,
        None,
        Some(&[0xf1]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf5]),
        Some(&[0xf6]),
        Some(&[0xf7]),
        None,
        None,
        None,
        None,
        None,
        Some(&[0x8f]),
        Some(&[0x90]),
        Some(&[0xfb]),
        Some(&[0xfd]),
        Some(&[0xeb]),
        Some(&[0xec]),
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
        Some(&[0xe2]),
        None,
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xe3]),
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

impl<'a> Iterator for EncoderMacCentralEurRoman<'a> {
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
                                    "Unable to encode code point: U+{:04x} as MacCentralEurRoman",
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
                                    "Unable to encode code point: U+{:04x} as MacCentralEurRoman",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x02c7 => Some(Ok(vec![0xff])),
                    0x2039 => Some(Ok(vec![0xdc])),
                    0x203a => Some(Ok(vec![0xdd])),
                    0x2122 => Some(Ok(vec![0xaa])),
                    0x2202 => Some(Ok(vec![0xb6])),
                    0x2206 => Some(Ok(vec![0xc6])),
                    0x2211 => Some(Ok(vec![0xb7])),
                    0x221a => Some(Ok(vec![0xc3])),
                    0x2260 => Some(Ok(vec![0xad])),
                    0x2264 => Some(Ok(vec![0xb2])),
                    0x2265 => Some(Ok(vec![0xb3])),
                    0x25ca => Some(Ok(vec![0xd7])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacCentralEurRoman",
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

        let mut decoder: DecoderMacCentralEurRoman = DecoderMacCentralEurRoman::new(&byte_string);

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

        let mut encoder: EncoderMacCentralEurRoman = EncoderMacCentralEurRoman::new(&code_points);

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

        let mut encoder: EncoderMacCentralEurRoman = EncoderMacCentralEurRoman::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
