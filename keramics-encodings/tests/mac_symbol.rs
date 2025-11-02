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

use keramics_core::ErrorTrace;
use keramics_encodings::{DecoderMacSymbol, EncoderMacSymbol};

const MAC_SYMBOL_TEST_VECTOR: [(&'static [u32], &'static [u8], bool); 223] = [
    (&[0x0000], &[0x00], false),
    (&[0x0001], &[0x01], false),
    (&[0x0002], &[0x02], false),
    (&[0x0003], &[0x03], false),
    (&[0x0004], &[0x04], false),
    (&[0x0005], &[0x05], false),
    (&[0x0006], &[0x06], false),
    (&[0x0007], &[0x07], false),
    (&[0x0008], &[0x08], false),
    (&[0x0009], &[0x09], false),
    (&[0x000a], &[0x0a], false),
    (&[0x000b], &[0x0b], false),
    (&[0x000c], &[0x0c], false),
    (&[0x000d], &[0x0d], false),
    (&[0x000e], &[0x0e], false),
    (&[0x000f], &[0x0f], false),
    (&[0x0010], &[0x10], false),
    (&[0x0011], &[0x11], false),
    (&[0x0012], &[0x12], false),
    (&[0x0013], &[0x13], false),
    (&[0x0014], &[0x14], false),
    (&[0x0015], &[0x15], false),
    (&[0x0016], &[0x16], false),
    (&[0x0017], &[0x17], false),
    (&[0x0018], &[0x18], false),
    (&[0x0019], &[0x19], false),
    (&[0x001a], &[0x1a], false),
    (&[0x001b], &[0x1b], false),
    (&[0x001c], &[0x1c], false),
    (&[0x001d], &[0x1d], false),
    (&[0x001e], &[0x1e], false),
    (&[0x001f], &[0x1f], false),
    (&[0x0020], &[0x20], false),
    (&[0x0021], &[0x21], false),
    (&[0x2200], &[0x22], false),
    (&[0x0023], &[0x23], false),
    (&[0x2203], &[0x24], false),
    (&[0x0025], &[0x25], false),
    (&[0x0026], &[0x26], false),
    (&[0x220d], &[0x27], false),
    (&[0x0028], &[0x28], false),
    (&[0x0029], &[0x29], false),
    (&[0x2217], &[0x2a], false),
    (&[0x002b], &[0x2b], false),
    (&[0x002c], &[0x2c], false),
    (&[0x2212], &[0x2d], false),
    (&[0x002e], &[0x2e], false),
    (&[0x002f], &[0x2f], false),
    (&[0x0030], &[0x30], false),
    (&[0x0031], &[0x31], false),
    (&[0x0032], &[0x32], false),
    (&[0x0033], &[0x33], false),
    (&[0x0034], &[0x34], false),
    (&[0x0035], &[0x35], false),
    (&[0x0036], &[0x36], false),
    (&[0x0037], &[0x37], false),
    (&[0x0038], &[0x38], false),
    (&[0x0039], &[0x39], false),
    (&[0x003a], &[0x3a], false),
    (&[0x003b], &[0x3b], false),
    (&[0x003c], &[0x3c], false),
    (&[0x003d], &[0x3d], false),
    (&[0x003e], &[0x3e], false),
    (&[0x003f], &[0x3f], false),
    (&[0x2245], &[0x40], false),
    (&[0x0391], &[0x41], false),
    (&[0x0392], &[0x42], false),
    (&[0x03a7], &[0x43], false),
    (&[0x0394], &[0x44], false),
    (&[0x0395], &[0x45], false),
    (&[0x03a6], &[0x46], false),
    (&[0x0393], &[0x47], false),
    (&[0x0397], &[0x48], false),
    (&[0x0399], &[0x49], false),
    (&[0x03d1], &[0x4a], false),
    (&[0x039a], &[0x4b], false),
    (&[0x039b], &[0x4c], false),
    (&[0x039c], &[0x4d], false),
    (&[0x039d], &[0x4e], false),
    (&[0x039f], &[0x4f], false),
    (&[0x03a0], &[0x50], false),
    (&[0x0398], &[0x51], false),
    (&[0x03a1], &[0x52], false),
    (&[0x03a3], &[0x53], false),
    (&[0x03a4], &[0x54], false),
    (&[0x03a5], &[0x55], false),
    (&[0x03c2], &[0x56], false),
    (&[0x03a9], &[0x57], false),
    (&[0x039e], &[0x58], false),
    (&[0x03a8], &[0x59], false),
    (&[0x0396], &[0x5a], false),
    (&[0x005b], &[0x5b], false),
    (&[0x2234], &[0x5c], false),
    (&[0x005d], &[0x5d], false),
    (&[0x22a5], &[0x5e], false),
    (&[0x005f], &[0x5f], false),
    (&[0xf8e5], &[0x60], false),
    (&[0x03b1], &[0x61], false),
    (&[0x03b2], &[0x62], false),
    (&[0x03c7], &[0x63], false),
    (&[0x03b4], &[0x64], false),
    (&[0x03b5], &[0x65], false),
    (&[0x03c6], &[0x66], false),
    (&[0x03b3], &[0x67], false),
    (&[0x03b7], &[0x68], false),
    (&[0x03b9], &[0x69], false),
    (&[0x03d5], &[0x6a], false),
    (&[0x03ba], &[0x6b], false),
    (&[0x03bb], &[0x6c], false),
    (&[0x03bc], &[0x6d], false),
    (&[0x03bd], &[0x6e], false),
    (&[0x03bf], &[0x6f], false),
    (&[0x03c0], &[0x70], false),
    (&[0x03b8], &[0x71], false),
    (&[0x03c1], &[0x72], false),
    (&[0x03c3], &[0x73], false),
    (&[0x03c4], &[0x74], false),
    (&[0x03c5], &[0x75], false),
    (&[0x03d6], &[0x76], false),
    (&[0x03c9], &[0x77], false),
    (&[0x03be], &[0x78], false),
    (&[0x03c8], &[0x79], false),
    (&[0x03b6], &[0x7a], false),
    (&[0x007b], &[0x7b], false),
    (&[0x007c], &[0x7c], false),
    (&[0x007d], &[0x7d], false),
    (&[0x223c], &[0x7e], false),
    (&[0x007f], &[0x7f], false),
    (&[0x20ac], &[0xa0], false),
    (&[0x03d2], &[0xa1], false),
    (&[0x2032], &[0xa2], false),
    (&[0x2264], &[0xa3], false),
    (&[0x2044], &[0xa4], false),
    (&[0x221e], &[0xa5], false),
    (&[0x0192], &[0xa6], false),
    (&[0x2663], &[0xa7], false),
    (&[0x2666], &[0xa8], false),
    (&[0x2665], &[0xa9], false),
    (&[0x2660], &[0xaa], false),
    (&[0x2194], &[0xab], false),
    (&[0x2190], &[0xac], false),
    (&[0x2191], &[0xad], false),
    (&[0x2192], &[0xae], false),
    (&[0x2193], &[0xaf], false),
    (&[0x00b0], &[0xb0], false),
    (&[0x00b1], &[0xb1], false),
    (&[0x2033], &[0xb2], false),
    (&[0x2265], &[0xb3], false),
    (&[0x00d7], &[0xb4], false),
    (&[0x221d], &[0xb5], false),
    (&[0x2202], &[0xb6], false),
    (&[0x2022], &[0xb7], false),
    (&[0x00f7], &[0xb8], false),
    (&[0x2260], &[0xb9], false),
    (&[0x2261], &[0xba], false),
    (&[0x2248], &[0xbb], false),
    (&[0x2026], &[0xbc], false),
    (&[0x23d0], &[0xbd], false),
    (&[0x23af], &[0xbe], false),
    (&[0x21b5], &[0xbf], false),
    (&[0x2135], &[0xc0], false),
    (&[0x2111], &[0xc1], false),
    (&[0x211c], &[0xc2], false),
    (&[0x2118], &[0xc3], false),
    (&[0x2297], &[0xc4], false),
    (&[0x2295], &[0xc5], false),
    (&[0x2205], &[0xc6], false),
    (&[0x2229], &[0xc7], false),
    (&[0x222a], &[0xc8], false),
    (&[0x2283], &[0xc9], false),
    (&[0x2287], &[0xca], false),
    (&[0x2284], &[0xcb], false),
    (&[0x2282], &[0xcc], false),
    (&[0x2286], &[0xcd], false),
    (&[0x2208], &[0xce], false),
    (&[0x2209], &[0xcf], false),
    (&[0x2220], &[0xd0], false),
    (&[0x2207], &[0xd1], false),
    (&[0x00ae], &[0xd2], false),
    (&[0x00a9], &[0xd3], false),
    (&[0x2122], &[0xd4], false),
    (&[0x220f], &[0xd5], false),
    (&[0x221a], &[0xd6], false),
    (&[0x22c5], &[0xd7], false),
    (&[0x00ac], &[0xd8], false),
    (&[0x2227], &[0xd9], false),
    (&[0x2228], &[0xda], false),
    (&[0x21d4], &[0xdb], false),
    (&[0x21d0], &[0xdc], false),
    (&[0x21d1], &[0xdd], false),
    (&[0x21d2], &[0xde], false),
    (&[0x21d3], &[0xdf], false),
    (&[0x25ca], &[0xe0], false),
    (&[0x3008], &[0xe1], false),
    (&[0x00ae], &[0xe2], true), // equivalent of 0xd2
    (&[0x00a9], &[0xe3], true), // equivalent of 0xd3
    (&[0x2122], &[0xe4], true), // equivalent of 0xd4
    (&[0x2211], &[0xe5], false),
    (&[0x239b], &[0xe6], false),
    (&[0x239c], &[0xe7], false),
    (&[0x239d], &[0xe8], false),
    (&[0x23a1], &[0xe9], false),
    (&[0x23a2], &[0xea], false),
    (&[0x23a3], &[0xeb], false),
    (&[0x23a7], &[0xec], false),
    (&[0x23a8], &[0xed], false),
    (&[0x23a9], &[0xee], false),
    (&[0x23aa], &[0xef], false),
    (&[0xf8ff], &[0xf0], false),
    (&[0x3009], &[0xf1], false),
    (&[0x222b], &[0xf2], false),
    (&[0x2320], &[0xf3], false),
    (&[0x23ae], &[0xf4], false),
    (&[0x2321], &[0xf5], false),
    (&[0x239e], &[0xf6], false),
    (&[0x239f], &[0xf7], false),
    (&[0x23a0], &[0xf8], false),
    (&[0x23a4], &[0xf9], false),
    (&[0x23a5], &[0xfa], false),
    (&[0x23a6], &[0xfb], false),
    (&[0x23ab], &[0xfc], false),
    (&[0x23ac], &[0xfd], false),
    (&[0x23ad], &[0xfe], false),
];

#[test]
fn decode() -> Result<(), ErrorTrace> {
    for (expected_code_points, test_byte_string, _) in MAC_SYMBOL_TEST_VECTOR.iter() {
        let mut decoder: DecoderMacSymbol = DecoderMacSymbol::new(test_byte_string);

        let test_code_points: Vec<u32> = match decoder.next() {
            Some(Ok(code_points)) => code_points,
            Some(Err(error)) => return Err(error),
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Failed to decode MacSymbol as code_point: U+{:04x}",
                    expected_code_points[0]
                )));
            }
        };
        assert_eq!(&test_code_points, expected_code_points);
    }
    Ok(())
}

#[test]
fn encode() -> Result<(), ErrorTrace> {
    for (test_code_points, expected_byte_string, is_duplicate) in MAC_SYMBOL_TEST_VECTOR.iter() {
        let mut encoder: EncoderMacSymbol = EncoderMacSymbol::new(test_code_points);

        let test_byte_string: Vec<u8> = match encoder.next() {
            Some(Ok(byte_string)) => byte_string,
            Some(Err(error)) => return Err(error),
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Failed to encode code point: U+{:04x} as MacSymbol",
                    test_code_points[0]
                )));
            }
        };
        if !is_duplicate {
            assert_eq!(&test_byte_string, expected_byte_string);
        }
    }
    Ok(())
}
