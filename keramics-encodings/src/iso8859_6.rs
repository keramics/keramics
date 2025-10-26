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

//! ISO-8859-6 (Arabic) encoding.
//!
//! Provides support for encoding and decoding ISO-8859-6.

use keramics_core::ErrorTrace;

/// ISO-8859-6 decoder.
pub struct DecoderIso8859_6<'a> {
    /// Encoded byte sequence.
    bytes: &'a [u8],

    /// Encoded byte sequence index.
    byte_index: usize,
}

impl<'a> DecoderIso8859_6<'a> {
    const BASE_0XA0: [Option<u16>; 96] = [
        Some(0x00a0),
        None,
        None,
        None,
        Some(0x00a4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(0x060c),
        Some(0x00ad),
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
        Some(0x061b),
        None,
        None,
        None,
        Some(0x061f),
        None,
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
        None,
        None,
        None,
        None,
        None,
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
    ];

    /// Creates a new decoder.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes,
            byte_index: 0,
        }
    }
}

impl<'a> Iterator for DecoderIso8859_6<'a> {
    type Item = Result<u32, ErrorTrace>;

    /// Retrieves the next next decoded code point.
    fn next(&mut self) -> Option<Self::Item> {
        match self.bytes.get(self.byte_index) {
            Some(byte_value) => {
                self.byte_index += 1;

                if *byte_value < 0xa0 {
                    Some(Ok(*byte_value as u32))
                } else {
                    match Self::BASE_0XA0[(*byte_value - 0xa0) as usize] {
                        Some(code_point) => Some(Ok(code_point as u32)),
                        None => Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to decode ISO-8859-6: 0x{:02x} as Unicode",
                            *byte_value
                        )))),
                    }
                }
            }
            None => None,
        }
    }
}

/// ISO-8859-6 encoder.
pub struct EncoderIso8859_6<'a> {
    /// Code points.
    code_points: &'a [u32],

    /// Code point index.
    code_point_index: usize,
}

impl<'a> EncoderIso8859_6<'a> {
    const BASE_0X0618: [Option<&'static [u8]>; 64] = [
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
    ];

    /// Creates a new encoder.
    pub fn new(code_points: &'a [u32]) -> Self {
        Self {
            code_points: code_points,
            code_point_index: 0,
        }
    }
}

impl<'a> Iterator for EncoderIso8859_6<'a> {
    type Item = Result<Vec<u8>, ErrorTrace>;

    /// Retrieves the next encoded byte sequence.
    fn next(&mut self) -> Option<Self::Item> {
        match self.code_points.get(self.code_point_index) {
            Some(code_point) => {
                self.code_point_index += 1;

                match *code_point {
                    0x0000..0x00a1 => Some(Ok(vec![*code_point as u8])),
                    0x0618..0x0658 => {
                        match Self::BASE_0X0618[(*code_point as u32 - 0x0618) as usize] {
                            Some(bytes) => Some(Ok(bytes.to_vec())),
                            None => {
                                return Some(Err(keramics_core::error_trace_new!(format!(
                                    "Unable to encode code point: U+{:04x} as ISO-8859-6",
                                    *code_point as u32
                                ))));
                            }
                        }
                    }
                    0x00a4 => Some(Ok(vec![0xa4])),
                    0x00ad => Some(Ok(vec![0xad])),
                    0x060c => Some(Ok(vec![0xac])),
                    _ => {
                        return Some(Err(keramics_core::error_trace_new!(format!(
                            "Unable to encode code point: U+{:04x} as ISO-8859-6",
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

        let mut decoder: DecoderIso8859_6 = DecoderIso8859_6::new(&byte_string);

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
    fn test_decode_with_unsupported_bytes() {
        let byte_string: [u8; 1] = [0xa1];

        let mut decoder: DecoderIso8859_6 = DecoderIso8859_6::new(&byte_string);

        let result: Result<u32, ErrorTrace> = decoder.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_encode() -> Result<(), ErrorTrace> {
        let code_points: [u32; 8] = [0x4b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73];

        let mut encoder: EncoderIso8859_6 = EncoderIso8859_6::new(&code_points);

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
        let code_points: [u32; 1] = [0x0618];

        let mut encoder: EncoderIso8859_6 = EncoderIso8859_6::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());

        let code_points: [u32; 1] = [0xd800];

        let mut encoder: EncoderIso8859_6 = EncoderIso8859_6::new(&code_points);

        let result: Result<Vec<u8>, ErrorTrace> = encoder.next().unwrap();
        assert!(result.is_err());
    }
}
