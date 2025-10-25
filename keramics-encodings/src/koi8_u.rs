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

//! KOI8-U encoding.
//!
//! Provides support for encoding and decoding KOI8-U.

use keramics_core::ErrorTrace;

/// KOI8-U decoder.
pub struct DecoderKoi8U<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderKoi8U<'a> {
    const BASE_0X80: [Option<u16>; 128] = [
        Some(0x2500),
        Some(0x2502),
        Some(0x250c),
        Some(0x2510),
        Some(0x2514),
        Some(0x2518),
        Some(0x251c),
        Some(0x2524),
        Some(0x252c),
        Some(0x2534),
        Some(0x253c),
        Some(0x2580),
        Some(0x2584),
        Some(0x2588),
        Some(0x258c),
        Some(0x2590),
        Some(0x2591),
        Some(0x2592),
        Some(0x2593),
        Some(0x2320),
        Some(0x25a0),
        Some(0x2219),
        Some(0x221a),
        Some(0x2248),
        Some(0x2264),
        Some(0x2265),
        Some(0x00a0),
        Some(0x2321),
        Some(0x00b0),
        Some(0x00b2),
        Some(0x00b7),
        Some(0x00f7),
        Some(0x2550),
        Some(0x2551),
        Some(0x2552),
        Some(0x0451),
        Some(0x0454),
        Some(0x2554),
        Some(0x0456),
        Some(0x0457),
        Some(0x2557),
        Some(0x2558),
        Some(0x2559),
        Some(0x255a),
        Some(0x255b),
        Some(0x0491),
        Some(0x255d),
        Some(0x255e),
        Some(0x255f),
        Some(0x2560),
        Some(0x2561),
        Some(0x0401),
        Some(0x0404),
        Some(0x2563),
        Some(0x0406),
        Some(0x0407),
        Some(0x2566),
        Some(0x2567),
        Some(0x2568),
        Some(0x2569),
        Some(0x256a),
        Some(0x0490),
        Some(0x256c),
        Some(0x00a9),
        Some(0x044e),
        Some(0x0430),
        Some(0x0431),
        Some(0x0446),
        Some(0x0434),
        Some(0x0435),
        Some(0x0444),
        Some(0x0433),
        Some(0x0445),
        Some(0x0438),
        Some(0x0439),
        Some(0x043a),
        Some(0x043b),
        Some(0x043c),
        Some(0x043d),
        Some(0x043e),
        Some(0x043f),
        Some(0x044f),
        Some(0x0440),
        Some(0x0441),
        Some(0x0442),
        Some(0x0443),
        Some(0x0436),
        Some(0x0432),
        Some(0x044c),
        Some(0x044b),
        Some(0x0437),
        Some(0x0448),
        Some(0x044d),
        Some(0x0449),
        Some(0x0447),
        Some(0x044a),
        Some(0x042e),
        Some(0x0410),
        Some(0x0411),
        Some(0x0426),
        Some(0x0414),
        Some(0x0415),
        Some(0x0424),
        Some(0x0413),
        Some(0x0425),
        Some(0x0418),
        Some(0x0419),
        Some(0x041a),
        Some(0x041b),
        Some(0x041c),
        Some(0x041d),
        Some(0x041e),
        Some(0x041f),
        Some(0x042f),
        Some(0x0420),
        Some(0x0421),
        Some(0x0422),
        Some(0x0423),
        Some(0x0416),
        Some(0x0412),
        Some(0x042c),
        Some(0x042b),
        Some(0x0417),
        Some(0x0428),
        Some(0x042d),
        Some(0x0429),
        Some(0x0427),
        Some(0x042a),
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderKoi8U<'a> {
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
                            "Unable to decode KOI8-U: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// KOI8-U encoder.
pub struct EncoderKoi8U<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderKoi8U<'a> {
    const BASE_0X0410: [Option<&'static [u8]>; 72] = [
        Some(&[0xe1]),
        Some(&[0xe2]),
        Some(&[0xf7]),
        Some(&[0xe7]),
        Some(&[0xe4]),
        Some(&[0xe5]),
        Some(&[0xf6]),
        Some(&[0xfa]),
        Some(&[0xe9]),
        Some(&[0xea]),
        Some(&[0xeb]),
        Some(&[0xec]),
        Some(&[0xed]),
        Some(&[0xee]),
        Some(&[0xef]),
        Some(&[0xf0]),
        Some(&[0xf2]),
        Some(&[0xf3]),
        Some(&[0xf4]),
        Some(&[0xf5]),
        Some(&[0xe6]),
        Some(&[0xe8]),
        Some(&[0xe3]),
        Some(&[0xfe]),
        Some(&[0xfb]),
        Some(&[0xfd]),
        Some(&[0xff]),
        Some(&[0xf9]),
        Some(&[0xf8]),
        Some(&[0xfc]),
        Some(&[0xe0]),
        Some(&[0xf1]),
        Some(&[0xc1]),
        Some(&[0xc2]),
        Some(&[0xd7]),
        Some(&[0xc7]),
        Some(&[0xc4]),
        Some(&[0xc5]),
        Some(&[0xd6]),
        Some(&[0xda]),
        Some(&[0xc9]),
        Some(&[0xca]),
        Some(&[0xcb]),
        Some(&[0xcc]),
        Some(&[0xcd]),
        Some(&[0xce]),
        Some(&[0xcf]),
        Some(&[0xd0]),
        Some(&[0xd2]),
        Some(&[0xd3]),
        Some(&[0xd4]),
        Some(&[0xd5]),
        Some(&[0xc6]),
        Some(&[0xc8]),
        Some(&[0xc3]),
        Some(&[0xde]),
        Some(&[0xdb]),
        Some(&[0xdd]),
        Some(&[0xdf]),
        Some(&[0xd9]),
        Some(&[0xd8]),
        Some(&[0xdc]),
        Some(&[0xc0]),
        Some(&[0xd1]),
        None,
        Some(&[0xa3]),
        None,
        None,
        Some(&[0xa4]),
        None,
        Some(&[0xa6]),
        Some(&[0xa7]),
    ];

    const BASE_0X2550: [Option<&'static [u8]>; 32] = [
        Some(&[0xa0]),
        Some(&[0xa1]),
        Some(&[0xa2]),
        None,
        Some(&[0xa5]),
        None,
        None,
        Some(&[0xa8]),
        Some(&[0xa9]),
        Some(&[0xaa]),
        Some(&[0xab]),
        Some(&[0xac]),
        None,
        Some(&[0xae]),
        Some(&[0xaf]),
        Some(&[0xb0]),
        Some(&[0xb1]),
        Some(&[0xb2]),
        None,
        Some(&[0xb5]),
        None,
        None,
        Some(&[0xb8]),
        Some(&[0xb9]),
        Some(&[0xba]),
        Some(&[0xbb]),
        Some(&[0xbc]),
        None,
        Some(&[0xbe]),
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

impl<'a> Iterator for EncoderKoi8U<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 => Some(Ok(vec![*code_point as u8])),
                    0x0410..0x0458 => {
                        match Self::BASE_0X0410[(*code_point as u32 - 0x0410) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as KOI8-U",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2550..0x2570 => {
                        match Self::BASE_0X2550[(*code_point as u32 - 0x2550) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as KOI8-U",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a0 => Some(Ok(vec![0x9a])),
                    0x00a9 => Some(Ok(vec![0xbf])),
                    0x00b0 => Some(Ok(vec![0x9c])),
                    0x00b2 => Some(Ok(vec![0x9d])),
                    0x00b7 => Some(Ok(vec![0x9e])),
                    0x00f7 => Some(Ok(vec![0x9f])),
                    0x0401 => Some(Ok(vec![0xb3])),
                    0x0404 => Some(Ok(vec![0xb4])),
                    0x0406 => Some(Ok(vec![0xb6])),
                    0x0407 => Some(Ok(vec![0xb7])),
                    0x0490 => Some(Ok(vec![0xbd])),
                    0x0491 => Some(Ok(vec![0xad])),
                    0x2219 => Some(Ok(vec![0x95])),
                    0x221a => Some(Ok(vec![0x96])),
                    0x2248 => Some(Ok(vec![0x97])),
                    0x2264 => Some(Ok(vec![0x98])),
                    0x2265 => Some(Ok(vec![0x99])),
                    0x2320 => Some(Ok(vec![0x93])),
                    0x2321 => Some(Ok(vec![0x9b])),
                    0x2500 => Some(Ok(vec![0x80])),
                    0x2502 => Some(Ok(vec![0x81])),
                    0x250c => Some(Ok(vec![0x82])),
                    0x2510 => Some(Ok(vec![0x83])),
                    0x2514 => Some(Ok(vec![0x84])),
                    0x2518 => Some(Ok(vec![0x85])),
                    0x251c => Some(Ok(vec![0x86])),
                    0x2524 => Some(Ok(vec![0x87])),
                    0x252c => Some(Ok(vec![0x88])),
                    0x2534 => Some(Ok(vec![0x89])),
                    0x253c => Some(Ok(vec![0x8a])),
                    0x2580 => Some(Ok(vec![0x8b])),
                    0x2584 => Some(Ok(vec![0x8c])),
                    0x2588 => Some(Ok(vec![0x8d])),
                    0x258c => Some(Ok(vec![0x8e])),
                    0x2590 => Some(Ok(vec![0x8f])),
                    0x2591 => Some(Ok(vec![0x90])),
                    0x2592 => Some(Ok(vec![0x91])),
                    0x2593 => Some(Ok(vec![0x92])),
                    0x25a0 => Some(Ok(vec![0x94])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as KOI8-U",
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

        let mut decoder: DecoderKoi8U = DecoderKoi8U::new(&byte_string);

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

        let mut encoder: EncoderKoi8U = EncoderKoi8U::new(&code_points);

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

        let mut encoder: EncoderKoi8U = EncoderKoi8U::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
