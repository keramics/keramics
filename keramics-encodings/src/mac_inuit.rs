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

//! MacInuit encoding.
//!
//! Provides support for encoding and decoding MacInuit.

use keramics_core::ErrorTrace;

/// MacInuit decoder.
pub struct DecoderMacInuit<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacInuit<'a> {
    const BASE_0X80: [u16; 128] = [
        0x1403, 0x1404, 0x1405, 0x1406, 0x140a, 0x140b, 0x1431, 0x1432, 0x1433, 0x1434, 0x1438,
        0x1439, 0x1449, 0x144e, 0x144f, 0x1450, 0x1451, 0x1455, 0x1456, 0x1466, 0x146d, 0x146e,
        0x146f, 0x1470, 0x1472, 0x1473, 0x1483, 0x148b, 0x148c, 0x148d, 0x148e, 0x1490, 0x1491,
        0x00b0, 0x14a1, 0x14a5, 0x14a6, 0x2022, 0x00b6, 0x14a7, 0x00ae, 0x00a9, 0x2122, 0x14a8,
        0x14aa, 0x14ab, 0x14bb, 0x14c2, 0x14c3, 0x14c4, 0x14c5, 0x14c7, 0x14c8, 0x14d0, 0x14ef,
        0x14f0, 0x14f1, 0x14f2, 0x14f4, 0x14f5, 0x1505, 0x14d5, 0x14d6, 0x14d7, 0x14d8, 0x14da,
        0x14db, 0x14ea, 0x1528, 0x1529, 0x152a, 0x152b, 0x152d, 0x2026, 0x00a0, 0x152e, 0x153e,
        0x1555, 0x1556, 0x1557, 0x2013, 0x2014, 0x201c, 0x201d, 0x2018, 0x2019, 0x1558, 0x1559,
        0x155a, 0x155d, 0x1546, 0x1547, 0x1548, 0x1549, 0x154b, 0x154c, 0x1550, 0x157f, 0x1580,
        0x1581, 0x1582, 0x1583, 0x1584, 0x1585, 0x158f, 0x1590, 0x1591, 0x1592, 0x1593, 0x1594,
        0x1595, 0x1671, 0x1672, 0x1673, 0x1674, 0x1675, 0x1676, 0x1596, 0x15a0, 0x15a1, 0x15a2,
        0x15a3, 0x15a4, 0x15a5, 0x15a6, 0x157c, 0x0141, 0x0142,
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderMacInuit<'a> {
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

/// MacInuit encoder.
pub struct EncoderMacInuit<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacInuit<'a> {
    const BASE_0X1430: [Option<&'static [u8]>; 216] = [
        None,
        Some(&[0x86]),
        Some(&[0x87]),
        Some(&[0x88]),
        Some(&[0x89]),
        None,
        None,
        None,
        Some(&[0x8a]),
        Some(&[0x8b]),
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
        Some(&[0x8c]),
        None,
        None,
        None,
        None,
        Some(&[0x8d]),
        Some(&[0x8e]),
        Some(&[0x8f]),
        Some(&[0x90]),
        None,
        None,
        None,
        Some(&[0x91]),
        Some(&[0x92]),
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
        Some(&[0x93]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x94]),
        Some(&[0x95]),
        Some(&[0x96]),
        Some(&[0x97]),
        None,
        Some(&[0x98]),
        Some(&[0x99]),
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
        Some(&[0x9a]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0x9b]),
        Some(&[0x9c]),
        Some(&[0x9d]),
        Some(&[0x9e]),
        None,
        Some(&[0x9f]),
        Some(&[0xa0]),
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
        Some(&[0xa2]),
        None,
        None,
        None,
        Some(&[0xa3]),
        Some(&[0xa4]),
        Some(&[0xa7]),
        Some(&[0xab]),
        None,
        Some(&[0xac]),
        Some(&[0xad]),
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
        Some(&[0xae]),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xaf]),
        Some(&[0xb0]),
        Some(&[0xb1]),
        Some(&[0xb2]),
        None,
        Some(&[0xb3]),
        Some(&[0xb4]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xb5]),
        None,
        None,
        None,
        None,
        Some(&[0xbd]),
        Some(&[0xbe]),
        Some(&[0xbf]),
        Some(&[0xc0]),
        None,
        Some(&[0xc1]),
        Some(&[0xc2]),
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
        Some(&[0xc3]),
        None,
        None,
        None,
        None,
        Some(&[0xb6]),
        Some(&[0xb7]),
        Some(&[0xb8]),
        Some(&[0xb9]),
        None,
        Some(&[0xba]),
        Some(&[0xbb]),
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
        Some(&[0xbc]),
        None,
        None,
    ];

    const BASE_0X1528: [Option<&'static [u8]>; 56] = [
        Some(&[0xc4]),
        Some(&[0xc5]),
        Some(&[0xc6]),
        Some(&[0xc7]),
        None,
        Some(&[0xc8]),
        Some(&[0xcb]),
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
        Some(&[0xcc]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xda]),
        Some(&[0xdb]),
        Some(&[0xdc]),
        Some(&[0xdd]),
        None,
        Some(&[0xde]),
        Some(&[0xdf]),
        None,
        None,
        None,
        Some(&[0xe0]),
        None,
        None,
        None,
        None,
        Some(&[0xcd]),
        Some(&[0xce]),
        Some(&[0xcf]),
        Some(&[0xd6]),
        Some(&[0xd7]),
        Some(&[0xd8]),
        None,
        None,
        Some(&[0xd9]),
        None,
        None,
    ];

    const BASE_0X1578: [Option<&'static [u8]>; 48] = [
        None,
        None,
        None,
        None,
        Some(&[0xfd]),
        None,
        None,
        Some(&[0xe1]),
        Some(&[0xe2]),
        Some(&[0xe3]),
        Some(&[0xe4]),
        Some(&[0xe5]),
        Some(&[0xe6]),
        Some(&[0xe7]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xe8]),
        Some(&[0xe9]),
        Some(&[0xea]),
        Some(&[0xeb]),
        Some(&[0xec]),
        Some(&[0xed]),
        Some(&[0xee]),
        Some(&[0xf5]),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&[0xf6]),
        Some(&[0xf7]),
        Some(&[0xf8]),
        Some(&[0xf9]),
        Some(&[0xfa]),
        Some(&[0xfb]),
        Some(&[0xfc]),
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
        None,
        None,
        None,
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

impl<'a> Iterator for EncoderMacInuit<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x0080 | 0x00a9 => Some(Ok(vec![*code_point as u8])),
                    0x1430..0x1508 => {
                        match Self::BASE_0X1430[(*code_point as u32 - 0x1430) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacInuit",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x1528..0x1560 => {
                        match Self::BASE_0X1528[(*code_point as u32 - 0x1528) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacInuit",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x1578..0x15a8 => {
                        match Self::BASE_0X1578[(*code_point as u32 - 0x1578) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacInuit",
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
                                    "Unable to encode code point: U+{:04x} as MacInuit",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a0 => Some(Ok(vec![0xca])),
                    0x00ae => Some(Ok(vec![0xa8])),
                    0x00b0 => Some(Ok(vec![0xa1])),
                    0x00b6 => Some(Ok(vec![0xa6])),
                    0x0141 => Some(Ok(vec![0xfe])),
                    0x0142 => Some(Ok(vec![0xff])),
                    0x1403 => Some(Ok(vec![0x80])),
                    0x1404 => Some(Ok(vec![0x81])),
                    0x1405 => Some(Ok(vec![0x82])),
                    0x1406 => Some(Ok(vec![0x83])),
                    0x140a => Some(Ok(vec![0x84])),
                    0x140b => Some(Ok(vec![0x85])),
                    0x1671 => Some(Ok(vec![0xef])),
                    0x1672 => Some(Ok(vec![0xf0])),
                    0x1673 => Some(Ok(vec![0xf1])),
                    0x1674 => Some(Ok(vec![0xf2])),
                    0x1675 => Some(Ok(vec![0xf3])),
                    0x1676 => Some(Ok(vec![0xf4])),
                    0x2122 => Some(Ok(vec![0xaa])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacInuit",
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

        let mut decoder: DecoderMacInuit = DecoderMacInuit::new(&byte_string);

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

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

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
        let code_points: [u32; 1] = [0x1430];

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x152c];

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x1578];

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0x2010];

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderMacInuit = EncoderMacInuit::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
