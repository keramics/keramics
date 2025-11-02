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

//! MacSymbol encoding.
//!
//! Provides support for encoding and decoding MacSymbol.

use keramics_core::ErrorTrace;

/// MacSymbol decoder.
pub struct DecoderMacSymbol<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacSymbol<'a> {
    const BASE_0X20: [Option<u16>; 224] = [
        Some(0x0020),
        Some(0x0021),
        Some(0x2200),
        Some(0x0023),
        Some(0x2203),
        Some(0x0025),
        Some(0x0026),
        Some(0x220d),
        Some(0x0028),
        Some(0x0029),
        Some(0x2217),
        Some(0x002b),
        Some(0x002c),
        Some(0x2212),
        Some(0x002e),
        Some(0x002f),
        Some(0x0030),
        Some(0x0031),
        Some(0x0032),
        Some(0x0033),
        Some(0x0034),
        Some(0x0035),
        Some(0x0036),
        Some(0x0037),
        Some(0x0038),
        Some(0x0039),
        Some(0x003a),
        Some(0x003b),
        Some(0x003c),
        Some(0x003d),
        Some(0x003e),
        Some(0x003f),
        Some(0x2245),
        Some(0x0391),
        Some(0x0392),
        Some(0x03a7),
        Some(0x0394),
        Some(0x0395),
        Some(0x03a6),
        Some(0x0393),
        Some(0x0397),
        Some(0x0399),
        Some(0x03d1),
        Some(0x039a),
        Some(0x039b),
        Some(0x039c),
        Some(0x039d),
        Some(0x039f),
        Some(0x03a0),
        Some(0x0398),
        Some(0x03a1),
        Some(0x03a3),
        Some(0x03a4),
        Some(0x03a5),
        Some(0x03c2),
        Some(0x03a9),
        Some(0x039e),
        Some(0x03a8),
        Some(0x0396),
        Some(0x005b),
        Some(0x2234),
        Some(0x005d),
        Some(0x22a5),
        Some(0x005f),
        Some(0xf8e5),
        Some(0x03b1),
        Some(0x03b2),
        Some(0x03c7),
        Some(0x03b4),
        Some(0x03b5),
        Some(0x03c6),
        Some(0x03b3),
        Some(0x03b7),
        Some(0x03b9),
        Some(0x03d5),
        Some(0x03ba),
        Some(0x03bb),
        Some(0x03bc),
        Some(0x03bd),
        Some(0x03bf),
        Some(0x03c0),
        Some(0x03b8),
        Some(0x03c1),
        Some(0x03c3),
        Some(0x03c4),
        Some(0x03c5),
        Some(0x03d6),
        Some(0x03c9),
        Some(0x03be),
        Some(0x03c8),
        Some(0x03b6),
        Some(0x007b),
        Some(0x007c),
        Some(0x007d),
        Some(0x223c),
        Some(0x007f),
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
        None,
        None,
        Some(0x20ac),
        Some(0x03d2),
        Some(0x2032),
        Some(0x2264),
        Some(0x2044),
        Some(0x221e),
        Some(0x0192),
        Some(0x2663),
        Some(0x2666),
        Some(0x2665),
        Some(0x2660),
        Some(0x2194),
        Some(0x2190),
        Some(0x2191),
        Some(0x2192),
        Some(0x2193),
        Some(0x00b0),
        Some(0x00b1),
        Some(0x2033),
        Some(0x2265),
        Some(0x00d7),
        Some(0x221d),
        Some(0x2202),
        Some(0x2022),
        Some(0x00f7),
        Some(0x2260),
        Some(0x2261),
        Some(0x2248),
        Some(0x2026),
        Some(0x23d0),
        Some(0x23af),
        Some(0x21b5),
        Some(0x2135),
        Some(0x2111),
        Some(0x211c),
        Some(0x2118),
        Some(0x2297),
        Some(0x2295),
        Some(0x2205),
        Some(0x2229),
        Some(0x222a),
        Some(0x2283),
        Some(0x2287),
        Some(0x2284),
        Some(0x2282),
        Some(0x2286),
        Some(0x2208),
        Some(0x2209),
        Some(0x2220),
        Some(0x2207),
        Some(0x00ae),
        Some(0x00a9),
        Some(0x2122),
        Some(0x220f),
        Some(0x221a),
        Some(0x22c5),
        Some(0x00ac),
        Some(0x2227),
        Some(0x2228),
        Some(0x21d4),
        Some(0x21d0),
        Some(0x21d1),
        Some(0x21d2),
        Some(0x21d3),
        Some(0x25ca),
        Some(0x3008),
        Some(0x00ae),
        Some(0x00a9),
        Some(0x2122),
        Some(0x2211),
        Some(0x239b),
        Some(0x239c),
        Some(0x239d),
        Some(0x23a1),
        Some(0x23a2),
        Some(0x23a3),
        Some(0x23a7),
        Some(0x23a8),
        Some(0x23a9),
        Some(0x23aa),
        Some(0xf8ff),
        Some(0x3009),
        Some(0x222b),
        Some(0x2320),
        Some(0x23ae),
        Some(0x2321),
        Some(0x239e),
        Some(0x239f),
        Some(0x23a0),
        Some(0x23a4),
        Some(0x23a5),
        Some(0x23a6),
        Some(0x23ab),
        Some(0x23ac),
        Some(0x23ad),
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

impl<'a> Iterator for DecoderMacSymbol<'a> {
    type Item = Result<Vec<u32>, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                // Note MacSymbol pairs:
                // * 0xe2, 0xe3 and 0xe4 with code point U+F87F
                //
                // Variant tags are used as transcoding hints.
                // This implementation does not add the variant tag.

                let code_point: u16 = if *byte_value < 0x20 {
                    *byte_value as u16
                } else {
                    match Self::BASE_0X20[(*byte_value - 0x20) as usize] {
                        Some(code_point) => code_point,
                        None => {
                            return Some(Err(keramics_core::error_trace_new!(format!(
                                "Unable to decode MacSymbol: 0x{:02x} as Unicode",
                                *byte_value
                            ))));
                        }
                    }
                };
                Some(Ok(vec![code_point as u32]))
            }
            None => None,
        }
    }
}

/// MacSymbol encoder.
pub struct EncoderMacSymbol<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacSymbol<'a> {
    const BASE_0X0390: [Option<&'static [u8]>; 72] = [
        None,
        Some(&[0x41]),
        Some(&[0x42]),
        Some(&[0x47]),
        Some(&[0x44]),
        Some(&[0x45]),
        Some(&[0x5a]),
        Some(&[0x48]),
        Some(&[0x51]),
        Some(&[0x49]),
        Some(&[0x4b]),
        Some(&[0x4c]),
        Some(&[0x4d]),
        Some(&[0x4e]),
        Some(&[0x58]),
        Some(&[0x4f]),
        Some(&[0x50]),
        Some(&[0x52]),
        None,
        Some(&[0x53]),
        Some(&[0x54]),
        Some(&[0x55]),
        Some(&[0x46]),
        Some(&[0x43]),
        Some(&[0x59]),
        Some(&[0x57]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x61]),
        Some(&[0x62]),
        Some(&[0x67]),
        Some(&[0x64]),
        Some(&[0x65]),
        Some(&[0x7a]),
        Some(&[0x68]),
        Some(&[0x71]),
        Some(&[0x69]),
        Some(&[0x6b]),
        Some(&[0x6c]),
        Some(&[0x6d]),
        Some(&[0x6e]),
        Some(&[0x78]),
        Some(&[0x6f]),
        Some(&[0x70]),
        Some(&[0x72]),
        Some(&[0x56]),
        Some(&[0x73]),
        Some(&[0x74]),
        Some(&[0x75]),
        Some(&[0x66]),
        Some(&[0x63]),
        Some(&[0x79]),
        Some(&[0x77]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x4a]),
        Some(&[0xa1]),
        None,
        None,
        Some(&[0x6a]),
        Some(&[0x76]),
        None,
    ];

    const BASE_0X2200: [Option<&'static [u8]>; 80] = [
        Some(&[0x22]),
        None,
        Some(&[0xb6]),
        Some(&[0x24]),
        None,
        Some(&[0xc6]),
        None,
        Some(&[0xd1]),
        Some(&[0xce]),
        Some(&[0xcf]),
        None,
        None,
        None,
        Some(&[0x27]),
        None,
        Some(&[0xd5]),
        None,
        Some(&[0xe5]),
        Some(&[0x2d]),
        None,
        None,
        None,
        None,
        Some(&[0x2a]),
        None,
        None,
        Some(&[0xd6]),
        None,
        None,
        Some(&[0xb5]),
        Some(&[0xa5]),
        None,
        Some(&[0xd0]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xd9]),
        Some(&[0xda]),
        Some(&[0xc7]),
        Some(&[0xc8]),
        Some(&[0xf2]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x5c]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x7e]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x40]),
        None,
        None,
        Some(&[0xbb]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];

    const BASE_0X2280: [Option<&'static [u8]>; 40] = [
        None,
        None,
        Some(&[0xcc]),
        Some(&[0xc9]),
        Some(&[0xcb]),
        None,
        Some(&[0xcd]),
        Some(&[0xca]),
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
        Some(&[0xc5]),
        None,
        Some(&[0xc4]),
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
        Some(&[0x5e]),
        None,
        None,
    ];

    const BASE_0X2398: [Option<&'static [u8]>; 24] = [
        None,
        None,
        None,
        Some(&[0xe6]),
        Some(&[0xe7]),
        Some(&[0xe8]),
        Some(&[0xf6]),
        Some(&[0xf7]),
        Some(&[0xf8]),
        Some(&[0xe9]),
        Some(&[0xea]),
        Some(&[0xeb]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        Some(&[0xfb]),
        Some(&[0xec]),
        Some(&[0xed]),
        Some(&[0xee]),
        Some(&[0xef]),
        Some(&[0xfc]),
        Some(&[0xfd]),
        Some(&[0xfe]),
        Some(&[0xf4]),
        Some(&[0xbe]),
    ];

    /// Creates a new encoder.
    pub fn new(code_points: &'a [u32]) -> Self {
        Self {
            code_points: code_points,
            code_point_index: 0,
        }
    }
}

impl<'a> Iterator for EncoderMacSymbol<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..=0x0021
                    | 0x0023
                    | 0x0025..=0x0026
                    | 0x0028..=0x0029
                    | 0x002b..=0x002c
                    | 0x002e..=0x003f
                    | 0x005b
                    | 0x005d
                    | 0x005f
                    | 0x007b..=0x007d
                    | 0x007f
                    | 0x00b0..=0x00b1 => Some(Ok(vec![*code_point as u8])),
                    0x0390..0x03d8 => {
                        match Self::BASE_0X0390[(*code_point as u32 - 0x0390) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacSymbol",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2200..0x2250 => {
                        match Self::BASE_0X2200[(*code_point as u32 - 0x2200) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacSymbol",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2280..0x22a8 => {
                        match Self::BASE_0X2280[(*code_point as u32 - 0x2280) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacSymbol",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2398..0x23b0 => {
                        match Self::BASE_0X2398[(*code_point as u32 - 0x2398) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacSymbol",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a9 => Some(Ok(vec![0xd3])),
                    0x00ac => Some(Ok(vec![0xd8])),
                    0x00ae => Some(Ok(vec![0xd2])),
                    0x00d7 => Some(Ok(vec![0xb4])),
                    0x00f7 => Some(Ok(vec![0xb8])),
                    0x0192 => Some(Ok(vec![0xa6])),
                    0x2022 => Some(Ok(vec![0xb7])),
                    0x2026 => Some(Ok(vec![0xbc])),
                    0x2032 => Some(Ok(vec![0xa2])),
                    0x2033 => Some(Ok(vec![0xb2])),
                    0x2044 => Some(Ok(vec![0xa4])),
                    0x20ac => Some(Ok(vec![0xa0])),
                    0x2111 => Some(Ok(vec![0xc1])),
                    0x2118 => Some(Ok(vec![0xc3])),
                    0x211c => Some(Ok(vec![0xc2])),
                    0x2122 => Some(Ok(vec![0xd4])),
                    0x2135 => Some(Ok(vec![0xc0])),
                    0x2190 => Some(Ok(vec![0xac])),
                    0x2191 => Some(Ok(vec![0xad])),
                    0x2192 => Some(Ok(vec![0xae])),
                    0x2193 => Some(Ok(vec![0xaf])),
                    0x2194 => Some(Ok(vec![0xab])),
                    0x21b5 => Some(Ok(vec![0xbf])),
                    0x21d0 => Some(Ok(vec![0xdc])),
                    0x21d1 => Some(Ok(vec![0xdd])),
                    0x21d2 => Some(Ok(vec![0xde])),
                    0x21d3 => Some(Ok(vec![0xdf])),
                    0x21d4 => Some(Ok(vec![0xdb])),
                    0x2260 => Some(Ok(vec![0xb9])),
                    0x2261 => Some(Ok(vec![0xba])),
                    0x2264 => Some(Ok(vec![0xa3])),
                    0x2265 => Some(Ok(vec![0xb3])),
                    0x22c5 => Some(Ok(vec![0xd7])),
                    0x2320 => Some(Ok(vec![0xf3])),
                    0x2321 => Some(Ok(vec![0xf5])),
                    0x23d0 => Some(Ok(vec![0xbd])),
                    0x25ca => Some(Ok(vec![0xe0])),
                    0x2660 => Some(Ok(vec![0xaa])),
                    0x2663 => Some(Ok(vec![0xa7])),
                    0x2665 => Some(Ok(vec![0xa9])),
                    0x2666 => Some(Ok(vec![0xa8])),
                    0x3008 => Some(Ok(vec![0xe1])),
                    0x3009 => Some(Ok(vec![0xf1])),
                    0xf8e5 => Some(Ok(vec![0x60])),
                    0xf8ff => Some(Ok(vec![0xf0])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacSymbol",
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

        let mut decoder: DecoderMacSymbol = DecoderMacSymbol::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(vec![0x0000039a])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003b5])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003c1])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003b1])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003bc])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003b9])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003c7])));
        assert_eq!(decoder.next(), Some(Ok(vec![0x000003c3])));
        assert_eq!(decoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_decode_with_unsupported_bytes() {
        let byte_string: [u8; 1] = [0x80];

        let mut decoder: DecoderMacSymbol = DecoderMacSymbol::new(&byte_string);

        let result: Result<Vec<u32>, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [
            0x039a, 0x03b5, 0x03c1, 0x03b1, 0x03bc, 0x03b9, 0x03c7, 0x03c3,
        ];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

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
        let code_points: [u32; 1] = [0x0390];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2201];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2280];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2398];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
