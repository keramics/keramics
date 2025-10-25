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

//! MacDingbats encoding.
//!
//! Provides support for encoding and decoding MacDingbats.

use keramics_core::ErrorTrace;

/// MacDingbats decoder.
pub struct DecoderMacDingbats<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderMacDingbats<'a> {
    const BASE_0X20: [Option<u16>; 224] = [
        Some(0x0020),
        Some(0x2701),
        Some(0x2702),
        Some(0x2703),
        Some(0x2704),
        Some(0x260e),
        Some(0x2706),
        Some(0x2707),
        Some(0x2708),
        Some(0x2709),
        Some(0x261b),
        Some(0x261e),
        Some(0x270c),
        Some(0x270d),
        Some(0x270e),
        Some(0x270f),
        Some(0x2710),
        Some(0x2711),
        Some(0x2712),
        Some(0x2713),
        Some(0x2714),
        Some(0x2715),
        Some(0x2716),
        Some(0x2717),
        Some(0x2718),
        Some(0x2719),
        Some(0x271a),
        Some(0x271b),
        Some(0x271c),
        Some(0x271d),
        Some(0x271e),
        Some(0x271f),
        Some(0x2720),
        Some(0x2721),
        Some(0x2722),
        Some(0x2723),
        Some(0x2724),
        Some(0x2725),
        Some(0x2726),
        Some(0x2727),
        Some(0x2605),
        Some(0x2729),
        Some(0x272a),
        Some(0x272b),
        Some(0x272c),
        Some(0x272d),
        Some(0x272e),
        Some(0x272f),
        Some(0x2730),
        Some(0x2731),
        Some(0x2732),
        Some(0x2733),
        Some(0x2734),
        Some(0x2735),
        Some(0x2736),
        Some(0x2737),
        Some(0x2738),
        Some(0x2739),
        Some(0x273a),
        Some(0x273b),
        Some(0x273c),
        Some(0x273d),
        Some(0x273e),
        Some(0x273f),
        Some(0x2740),
        Some(0x2741),
        Some(0x2742),
        Some(0x2743),
        Some(0x2744),
        Some(0x2745),
        Some(0x2746),
        Some(0x2747),
        Some(0x2748),
        Some(0x2749),
        Some(0x274a),
        Some(0x274b),
        Some(0x25cf),
        Some(0x274d),
        Some(0x25a0),
        Some(0x274f),
        Some(0x2750),
        Some(0x2751),
        Some(0x2752),
        Some(0x25b2),
        Some(0x25bc),
        Some(0x25c6),
        Some(0x2756),
        Some(0x25d7),
        Some(0x2758),
        Some(0x2759),
        Some(0x275a),
        Some(0x275b),
        Some(0x275c),
        Some(0x275d),
        Some(0x275e),
        Some(0x007f),
        Some(0x2768),
        Some(0x2769),
        Some(0x276a),
        Some(0x276b),
        Some(0x276c),
        Some(0x276d),
        Some(0x276e),
        Some(0x276f),
        Some(0x2770),
        Some(0x2771),
        Some(0x2772),
        Some(0x2773),
        Some(0x2774),
        Some(0x2775),
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
        Some(0x2761),
        Some(0x2762),
        Some(0x2763),
        Some(0x2764),
        Some(0x2765),
        Some(0x2766),
        Some(0x2767),
        Some(0x2663),
        Some(0x2666),
        Some(0x2665),
        Some(0x2660),
        Some(0x2460),
        Some(0x2461),
        Some(0x2462),
        Some(0x2463),
        Some(0x2464),
        Some(0x2465),
        Some(0x2466),
        Some(0x2467),
        Some(0x2468),
        Some(0x2469),
        Some(0x2776),
        Some(0x2777),
        Some(0x2778),
        Some(0x2779),
        Some(0x277a),
        Some(0x277b),
        Some(0x277c),
        Some(0x277d),
        Some(0x277e),
        Some(0x277f),
        Some(0x2780),
        Some(0x2781),
        Some(0x2782),
        Some(0x2783),
        Some(0x2784),
        Some(0x2785),
        Some(0x2786),
        Some(0x2787),
        Some(0x2788),
        Some(0x2789),
        Some(0x278a),
        Some(0x278b),
        Some(0x278c),
        Some(0x278d),
        Some(0x278e),
        Some(0x278f),
        Some(0x2790),
        Some(0x2791),
        Some(0x2792),
        Some(0x2793),
        Some(0x2794),
        Some(0x2192),
        Some(0x2194),
        Some(0x2195),
        Some(0x2798),
        Some(0x2799),
        Some(0x279a),
        Some(0x279b),
        Some(0x279c),
        Some(0x279d),
        Some(0x279e),
        Some(0x279f),
        Some(0x27a0),
        Some(0x27a1),
        Some(0x27a2),
        Some(0x27a3),
        Some(0x27a4),
        Some(0x27a5),
        Some(0x27a6),
        Some(0x27a7),
        Some(0x27a8),
        Some(0x27a9),
        Some(0x27aa),
        Some(0x27ab),
        Some(0x27ac),
        Some(0x27ad),
        Some(0x27ae),
        Some(0x27af),
        None,
        Some(0x27b1),
        Some(0x27b2),
        Some(0x27b3),
        Some(0x27b4),
        Some(0x27b5),
        Some(0x27b6),
        Some(0x27b7),
        Some(0x27b8),
        Some(0x27b9),
        Some(0x27ba),
        Some(0x27bb),
        Some(0x27bc),
        Some(0x27bd),
        Some(0x27be),
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

impl<'a> Iterator for DecoderMacDingbats<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                if *byte_value < 0x20 {
                    Some(Ok(*byte_value as u32))
                } else {
                    match Self::BASE_0X20[(*byte_value - 0x20) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode MacDingbats: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// MacDingbats encoder.
pub struct EncoderMacDingbats<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderMacDingbats<'a> {
    const BASE_0X2460: [Option<&'static [u8]>; 16] = [
        Some(&[0xac]),
        Some(&[0xad]),
        Some(&[0xae]),
        Some(&[0xaf]),
        Some(&[0xb0]),
        Some(&[0xb1]),
        Some(&[0xb2]),
        Some(&[0xb3]),
        Some(&[0xb4]),
        Some(&[0xb5]),
        None,
        None,
        None,
        None,
        None,
        None,
    ];

    const BASE_0X2700: [Option<&'static [u8]>; 192] = [
        None,
        Some(&[0x21]),
        Some(&[0x22]),
        Some(&[0x23]),
        Some(&[0x24]),
        None,
        Some(&[0x26]),
        Some(&[0x27]),
        Some(&[0x28]),
        Some(&[0x29]),
        None,
        None,
        Some(&[0x2c]),
        Some(&[0x2d]),
        Some(&[0x2e]),
        Some(&[0x2f]),
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
        Some(&[0x3a]),
        Some(&[0x3b]),
        Some(&[0x3c]),
        Some(&[0x3d]),
        Some(&[0x3e]),
        Some(&[0x3f]),
        Some(&[0x40]),
        Some(&[0x41]),
        Some(&[0x42]),
        Some(&[0x43]),
        Some(&[0x44]),
        Some(&[0x45]),
        Some(&[0x46]),
        Some(&[0x47]),
        None,
        Some(&[0x49]),
        Some(&[0x4a]),
        Some(&[0x4b]),
        Some(&[0x4c]),
        Some(&[0x4d]),
        Some(&[0x4e]),
        Some(&[0x4f]),
        Some(&[0x50]),
        Some(&[0x51]),
        Some(&[0x52]),
        Some(&[0x53]),
        Some(&[0x54]),
        Some(&[0x55]),
        Some(&[0x56]),
        Some(&[0x57]),
        Some(&[0x58]),
        Some(&[0x59]),
        Some(&[0x5a]),
        Some(&[0x5b]),
        Some(&[0x5c]),
        Some(&[0x5d]),
        Some(&[0x5e]),
        Some(&[0x5f]),
        Some(&[0x60]),
        Some(&[0x61]),
        Some(&[0x62]),
        Some(&[0x63]),
        Some(&[0x64]),
        Some(&[0x65]),
        Some(&[0x66]),
        Some(&[0x67]),
        Some(&[0x68]),
        Some(&[0x69]),
        Some(&[0x6a]),
        Some(&[0x6b]),
        None,
        Some(&[0x6d]),
        None,
        Some(&[0x6f]),
        Some(&[0x70]),
        Some(&[0x71]),
        Some(&[0x72]),
        None,
        None,
        None,
        Some(&[0x76]),
        None,
        Some(&[0x78]),
        Some(&[0x79]),
        Some(&[0x7a]),
        Some(&[0x7b]),
        Some(&[0x7c]),
        Some(&[0x7d]),
        Some(&[0x7e]),
        None,
        None,
        Some(&[0xa1]),
        Some(&[0xa2]),
        Some(&[0xa3]),
        Some(&[0xa4]),
        Some(&[0xa5]),
        Some(&[0xa6]),
        Some(&[0xa7]),
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
        Some(&[0xb6]),
        Some(&[0xb7]),
        Some(&[0xb8]),
        Some(&[0xb9]),
        Some(&[0xba]),
        Some(&[0xbb]),
        Some(&[0xbc]),
        Some(&[0xbd]),
        Some(&[0xbe]),
        Some(&[0xbf]),
        Some(&[0xc0]),
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
        None,
        None,
        None,
        Some(&[0xd8]),
        Some(&[0xd9]),
        Some(&[0xda]),
        Some(&[0xdb]),
        Some(&[0xdc]),
        Some(&[0xdd]),
        Some(&[0xde]),
        Some(&[0xdf]),
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
        None,
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

impl<'a> Iterator for EncoderMacDingbats<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..=0x0020 | 0x007f => Some(Ok(vec![*code_point as u8])),
                    0x2460..0x2470 => {
                        match Self::BASE_0X2460[(*code_point as u32 - 0x2460) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacDingbats",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2700..0x27c0 => {
                        match Self::BASE_0X2700[(*code_point as u32 - 0x2700) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as MacDingbats",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x2192 => Some(Ok(vec![0xd5])),
                    0x2194 => Some(Ok(vec![0xd6])),
                    0x2195 => Some(Ok(vec![0xd7])),
                    0x25a0 => Some(Ok(vec![0x6e])),
                    0x25b2 => Some(Ok(vec![0x73])),
                    0x25bc => Some(Ok(vec![0x74])),
                    0x25c6 => Some(Ok(vec![0x75])),
                    0x25cf => Some(Ok(vec![0x6c])),
                    0x25d7 => Some(Ok(vec![0x77])),
                    0x2605 => Some(Ok(vec![0x48])),
                    0x260e => Some(Ok(vec![0x25])),
                    0x261b => Some(Ok(vec![0x2a])),
                    0x261e => Some(Ok(vec![0x2b])),
                    0x2660 => Some(Ok(vec![0xab])),
                    0x2663 => Some(Ok(vec![0xa8])),
                    0x2665 => Some(Ok(vec![0xaa])),
                    0x2666 => Some(Ok(vec![0xa9])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as MacDingbats",
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

        let mut decoder: DecoderMacDingbats = DecoderMacDingbats::new(&byte_string);

        assert_eq!(decoder.next(), Some(Ok(0x272b)));
        assert_eq!(decoder.next(), Some(Ok(0x2745)));
        assert_eq!(decoder.next(), Some(Ok(0x2752)));
        assert_eq!(decoder.next(), Some(Ok(0x2741)));
        assert_eq!(decoder.next(), Some(Ok(0x274d)));
        assert_eq!(decoder.next(), Some(Ok(0x2749)));
        assert_eq!(decoder.next(), Some(Ok(0x2743)));
        assert_eq!(decoder.next(), Some(Ok(0x25b2)));
        assert_eq!(decoder.next(), None);

        Ok(())
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [
            0x272b, 0x2745, 0x2752, 0x2741, 0x274d, 0x2749, 0x2743, 0x25b2,
        ];

        let mut encoder: EncoderMacDingbats = EncoderMacDingbats::new(&code_points);

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

        let mut encoder: EncoderMacDingbats = EncoderMacDingbats::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();

        assert!(result.is_err());
    }
}
