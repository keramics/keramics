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

use keramics_core::ErrorTrace;
use keramics_encodings::{DecoderMacHebrew, EncoderMacHebrew};

const MAC_HEBREW_TEST_VECTOR: [(&'static [u32], &'static [u8], bool); 256] = [
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
    (&[0x0022], &[0x22], false),
    (&[0x0023], &[0x23], false),
    (&[0x0024], &[0x24], false),
    (&[0x0025], &[0x25], false),
    (&[0x0026], &[0x26], false),
    (&[0x0027], &[0x27], false),
    (&[0x0028], &[0x28], false),
    (&[0x0029], &[0x29], false),
    (&[0x002a], &[0x2a], false),
    (&[0x002b], &[0x2b], false),
    (&[0x002c], &[0x2c], false),
    (&[0x002d], &[0x2d], false),
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
    (&[0x0040], &[0x40], false),
    (&[0x0041], &[0x41], false),
    (&[0x0042], &[0x42], false),
    (&[0x0043], &[0x43], false),
    (&[0x0044], &[0x44], false),
    (&[0x0045], &[0x45], false),
    (&[0x0046], &[0x46], false),
    (&[0x0047], &[0x47], false),
    (&[0x0048], &[0x48], false),
    (&[0x0049], &[0x49], false),
    (&[0x004a], &[0x4a], false),
    (&[0x004b], &[0x4b], false),
    (&[0x004c], &[0x4c], false),
    (&[0x004d], &[0x4d], false),
    (&[0x004e], &[0x4e], false),
    (&[0x004f], &[0x4f], false),
    (&[0x0050], &[0x50], false),
    (&[0x0051], &[0x51], false),
    (&[0x0052], &[0x52], false),
    (&[0x0053], &[0x53], false),
    (&[0x0054], &[0x54], false),
    (&[0x0055], &[0x55], false),
    (&[0x0056], &[0x56], false),
    (&[0x0057], &[0x57], false),
    (&[0x0058], &[0x58], false),
    (&[0x0059], &[0x59], false),
    (&[0x005a], &[0x5a], false),
    (&[0x005b], &[0x5b], false),
    (&[0x005c], &[0x5c], false),
    (&[0x005d], &[0x5d], false),
    (&[0x005e], &[0x5e], false),
    (&[0x005f], &[0x5f], false),
    (&[0x0060], &[0x60], false),
    (&[0x0061], &[0x61], false),
    (&[0x0062], &[0x62], false),
    (&[0x0063], &[0x63], false),
    (&[0x0064], &[0x64], false),
    (&[0x0065], &[0x65], false),
    (&[0x0066], &[0x66], false),
    (&[0x0067], &[0x67], false),
    (&[0x0068], &[0x68], false),
    (&[0x0069], &[0x69], false),
    (&[0x006a], &[0x6a], false),
    (&[0x006b], &[0x6b], false),
    (&[0x006c], &[0x6c], false),
    (&[0x006d], &[0x6d], false),
    (&[0x006e], &[0x6e], false),
    (&[0x006f], &[0x6f], false),
    (&[0x0070], &[0x70], false),
    (&[0x0071], &[0x71], false),
    (&[0x0072], &[0x72], false),
    (&[0x0073], &[0x73], false),
    (&[0x0074], &[0x74], false),
    (&[0x0075], &[0x75], false),
    (&[0x0076], &[0x76], false),
    (&[0x0077], &[0x77], false),
    (&[0x0078], &[0x78], false),
    (&[0x0079], &[0x79], false),
    (&[0x007a], &[0x7a], false),
    (&[0x007b], &[0x7b], false),
    (&[0x007c], &[0x7c], false),
    (&[0x007d], &[0x7d], false),
    (&[0x007e], &[0x7e], false),
    (&[0x007f], &[0x7f], false),
    (&[0x00c4], &[0x80], false),
    (&[0x05f2, 0x05b7], &[0x81], false),
    (&[0x00c7], &[0x82], false),
    (&[0x00c9], &[0x83], false),
    (&[0x00d1], &[0x84], false),
    (&[0x00d6], &[0x85], false),
    (&[0x00dc], &[0x86], false),
    (&[0x00e1], &[0x87], false),
    (&[0x00e0], &[0x88], false),
    (&[0x00e2], &[0x89], false),
    (&[0x00e4], &[0x8a], false),
    (&[0x00e3], &[0x8b], false),
    (&[0x00e5], &[0x8c], false),
    (&[0x00e7], &[0x8d], false),
    (&[0x00e9], &[0x8e], false),
    (&[0x00e8], &[0x8f], false),
    (&[0x00ea], &[0x90], false),
    (&[0x00eb], &[0x91], false),
    (&[0x00ed], &[0x92], false),
    (&[0x00ec], &[0x93], false),
    (&[0x00ee], &[0x94], false),
    (&[0x00ef], &[0x95], false),
    (&[0x00f1], &[0x96], false),
    (&[0x00f3], &[0x97], false),
    (&[0x00f2], &[0x98], false),
    (&[0x00f4], &[0x99], false),
    (&[0x00f6], &[0x9a], false),
    (&[0x00f5], &[0x9b], false),
    (&[0x00fa], &[0x9c], false),
    (&[0x00f9], &[0x9d], false),
    (&[0x00fb], &[0x9e], false),
    (&[0x00fc], &[0x9f], false),
    (&[0x0020], &[0xa0], true), // equivalent of 0x20
    (&[0x0021], &[0xa1], true), // equivalent of 0x21
    (&[0x0022], &[0xa2], true), // equivalent of 0x22
    (&[0x0023], &[0xa3], true), // equivalent of 0x23
    (&[0x0024], &[0xa4], true), // equivalent of 0x24
    (&[0x0025], &[0xa5], true), // equivalent of 0x25
    (&[0x20aa], &[0xa6], false),
    (&[0x0027], &[0xa7], true), // equivalent of 0x27
    (&[0x0029], &[0xa8], true), // equivalent of 0x28
    (&[0x0028], &[0xa9], true), // equivalent of 0x29
    (&[0x002a], &[0xaa], true), // equivalent of 0x2a
    (&[0x002b], &[0xab], true), // equivalent of 0x2b
    (&[0x002c], &[0xac], true), // equivalent of 0x2c
    (&[0x002d], &[0xad], true), // equivalent of 0x2d
    (&[0x002e], &[0xae], true), // equivalent of 0x2e
    (&[0x002f], &[0xaf], true), // equivalent of 0x2f
    (&[0x0030], &[0xb0], true), // equivalent of 0x30
    (&[0x0031], &[0xb1], true), // equivalent of 0x31
    (&[0x0032], &[0xb2], true), // equivalent of 0x32
    (&[0x0033], &[0xb3], true), // equivalent of 0x33
    (&[0x0034], &[0xb4], true), // equivalent of 0x34
    (&[0x0035], &[0xb5], true), // equivalent of 0x35
    (&[0x0036], &[0xb6], true), // equivalent of 0x36
    (&[0x0037], &[0xb7], true), // equivalent of 0x37
    (&[0x0038], &[0xb8], true), // equivalent of 0x38
    (&[0x0039], &[0xb9], true), // equivalent of 0x39
    (&[0x003a], &[0xba], true), // equivalent of 0x3a
    (&[0x003b], &[0xbb], true), // equivalent of 0x3b
    (&[0x003c], &[0xbc], true), // equivalent of 0x3c
    (&[0x003d], &[0xbd], true), // equivalent of 0x3d
    (&[0x003e], &[0xbe], true), // equivalent of 0x3e
    (&[0x003f], &[0xbf], true), // equivalent of 0x3f
    (&[0xf86a, 0x05dc, 0x05b9], &[0xc0], false),
    (&[0x201e], &[0xc1], false),
    (&[0xf89b], &[0xc2], false),
    (&[0xf89c], &[0xc3], false),
    (&[0xf89d], &[0xc4], false),
    (&[0xf89e], &[0xc5], false),
    (&[0x05bc], &[0xc6], false),
    (&[0xfb4b], &[0xc7], false),
    (&[0xfb35], &[0xc8], false),
    (&[0x2026], &[0xc9], false),
    (&[0x00a0], &[0xca], false),
    (&[0x05b8], &[0xcb], false),
    (&[0x05b7], &[0xcc], false),
    (&[0x05b5], &[0xcd], false),
    (&[0x05b6], &[0xce], false),
    (&[0x05b4], &[0xcf], false),
    (&[0x2013], &[0xd0], false),
    (&[0x2014], &[0xd1], false),
    (&[0x201c], &[0xd2], false),
    (&[0x201d], &[0xd3], false),
    (&[0x2018], &[0xd4], false),
    (&[0x2019], &[0xd5], false),
    (&[0xfb2a], &[0xd6], false),
    (&[0xfb2b], &[0xd7], false),
    (&[0x05bf], &[0xd8], false),
    (&[0x05b0], &[0xd9], false),
    (&[0x05b2], &[0xda], false),
    (&[0x05b1], &[0xdb], false),
    (&[0x05bb], &[0xdc], false),
    (&[0x05b9], &[0xdd], false),
    (&[0x05c7], &[0xde], false),
    (&[0x05b3], &[0xdf], false),
    (&[0x05d0], &[0xe0], false),
    (&[0x05d1], &[0xe1], false),
    (&[0x05d2], &[0xe2], false),
    (&[0x05d3], &[0xe3], false),
    (&[0x05d4], &[0xe4], false),
    (&[0x05d5], &[0xe5], false),
    (&[0x05d6], &[0xe6], false),
    (&[0x05d7], &[0xe7], false),
    (&[0x05d8], &[0xe8], false),
    (&[0x05d9], &[0xe9], false),
    (&[0x05da], &[0xea], false),
    (&[0x05db], &[0xeb], false),
    (&[0x05dc], &[0xec], false),
    (&[0x05dd], &[0xed], false),
    (&[0x05de], &[0xee], false),
    (&[0x05df], &[0xef], false),
    (&[0x05e0], &[0xf0], false),
    (&[0x05e1], &[0xf1], false),
    (&[0x05e2], &[0xf2], false),
    (&[0x05e3], &[0xf3], false),
    (&[0x05e4], &[0xf4], false),
    (&[0x05e5], &[0xf5], false),
    (&[0x05e6], &[0xf6], false),
    (&[0x05e7], &[0xf7], false),
    (&[0x05e8], &[0xf8], false),
    (&[0x05e9], &[0xf9], false),
    (&[0x05ea], &[0xfa], false),
    (&[0x007d], &[0xfb], true), // equivalent of 0x7d
    (&[0x005d], &[0xfc], true), // equivalent of 0x5d
    (&[0x007b], &[0xfd], true), // equivalent of 0x7d
    (&[0x005b], &[0xfe], true), // equivalent of 0x5b
    (&[0x007c], &[0xff], true), // equivalent of 0x7c
];

#[test]
fn decode() -> Result<(), ErrorTrace> {
    for (expected_code_points, test_byte_string, _) in MAC_HEBREW_TEST_VECTOR.iter() {
        let mut decoder: DecoderMacHebrew = DecoderMacHebrew::new(test_byte_string);

        let test_code_points: Vec<u32> = match decoder.next() {
            Some(Ok(code_points)) => code_points,
            Some(Err(error)) => return Err(error),
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Failed to decode MacHebrew as code_point: U+{:04x}",
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
    for (test_code_points, expected_byte_string, is_duplicate) in MAC_HEBREW_TEST_VECTOR.iter() {
        let mut encoder: EncoderMacHebrew = EncoderMacHebrew::new(test_code_points);

        let test_byte_string: Vec<u8> = match encoder.next() {
            Some(Ok(byte_string)) => byte_string,
            Some(Err(error)) => return Err(error),
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Failed to encode code point: U+{:04x} as MacHebrew",
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
