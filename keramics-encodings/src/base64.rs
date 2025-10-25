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

//! Base64 encoding.
//!
//! Provides support for decoding Base64 data.

use keramics_core::ErrorTrace;

/// Base64 encoded data stream.
pub struct Base64Stream<'a> {
    /// Encoded data.
    data: &'a [u8],

    /// Current offset in the encoded data.
    pub data_offset: usize,

    /// Size of the encoded data.
    pub data_size: usize,

    /// Bits buffer.
    bits: u32,

    /// Number of bits in the bits buffer.
    pub number_of_bits: usize,

    /// Value to indicate white space code_points should be skipped.
    skip_white_space: bool,

    /// Value to indicate if padding was found.
    pub found_padding: bool,
}

impl<'a> Base64Stream<'a> {
    /// Creates a new encoded data stream.
    pub fn new(data: &'a [u8], data_offset: usize, skip_white_space: bool) -> Self {
        let data_size: usize = data.len();
        Self {
            data: data,
            data_offset: data_offset,
            data_size: data_size,
            bits: 0,
            number_of_bits: 0,
            skip_white_space: skip_white_space,
            found_padding: false,
        }
    }

    /// Retrieves a byte value.
    pub fn get_value(&mut self) -> Result<Option<u8>, ErrorTrace> {
        while self.number_of_bits < 8 {
            if self.data_offset >= self.data_size || self.found_padding {
                break;
            }
            let byte_value: u8 = self.data[self.data_offset];
            let sixtet: u8 = match byte_value {
                0x41..0x5b => byte_value - 0x41,        // A-Z
                0x61..0x7b => 26 + (byte_value - 0x61), // a-z
                0x30..0x3a => 52 + (byte_value - 0x30), // 0-9
                0x2b => 62,                             // +
                0x2f => 63,                             // /
                0x3d => {
                    self.found_padding = true;
                    break;
                }
                0x09 | 0x0a | 0x0b | 0x0d | 0x20 => {
                    if !self.skip_white_space {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid base64 code_point value: 0x{:02x}",
                            byte_value
                        )));
                    }
                    self.data_offset += 1;

                    continue;
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid base64 code_point value: 0x{:02x}",
                        byte_value
                    )));
                }
            };
            self.bits = (self.bits << 6) | (sixtet as u32);
            self.number_of_bits += 6;

            self.data_offset += 1;
        }
        if self.number_of_bits < 8 {
            return Ok(None);
        }
        self.number_of_bits -= 8;

        let value_32bit: u32 = self.bits >> self.number_of_bits;

        if self.number_of_bits == 0 {
            self.bits = 0;
        } else {
            self.bits &= 0xffffffff >> (32 - self.number_of_bits);
        }
        Ok(Some(value_32bit as u8))
    }
}

/// Context for decoding Base64 data.
pub struct Base64Context {
    /// Data size.
    pub data_size: usize,
}

impl Base64Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self { data_size: 0 }
    }

    /// Decode data.
    pub fn decode(&mut self, encoded_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        let mut base64_stream: Base64Stream = Base64Stream::new(&encoded_data, 0, false);
        let mut data_offset: usize = 0;

        while let Some(byte_value) = base64_stream.get_value()? {
            data[data_offset] = byte_value;
            data_offset += 1;
        }
        self.data_size = data_offset;

        if base64_stream.found_padding {
            let mut padding_size: usize = base64_stream.data_offset % 4;
            if padding_size > 0 {
                padding_size = 4 - padding_size;
            }
            if padding_size > base64_stream.data_size - base64_stream.data_offset {
                return Err(keramics_core::error_trace_new!(
                    "Invalid encoded data value too small"
                ));
            }
            let mut padding_offset: usize = base64_stream.data_offset;
            for _ in 0..padding_size {
                let byte_value: u8 = base64_stream.data[padding_offset];
                if byte_value != 0x3d {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid base64 padding code_point value: 0x{:02x}",
                        byte_value
                    )));
                }
                padding_offset += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value() -> Result<(), ErrorTrace> {
        let test_encoded_data: [u8; 22] = [
            0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67,
        ];
        let mut test_stream: Base64Stream = Base64Stream::new(&test_encoded_data, 0, false);

        let byte_value: u8 = test_stream.get_value()?.unwrap();
        assert_eq!(byte_value, 0x54);

        Ok(())
    }

    #[test]
    fn test_get_value_with_unsupported_character() {
        let test_encoded_data: [u8; 22] = [
            0xff, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67,
        ];
        let mut test_stream: Base64Stream = Base64Stream::new(&test_encoded_data, 0, false);

        let result: Result<Option<u8>, ErrorTrace> = test_stream.get_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_value_with_unsupported_white_space() {
        let test_encoded_data: [u8; 23] = [
            0x09, 0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f,
            0x53, 0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67,
        ];
        let mut test_stream: Base64Stream = Base64Stream::new(&test_encoded_data, 0, false);

        let result: Result<Option<u8>, ErrorTrace> = test_stream.get_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_decode() -> Result<(), ErrorTrace> {
        let mut test_context: Base64Context = Base64Context::new();

        let test_encoded_data: [u8; 22] = [
            0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        test_context.decode(&test_encoded_data, &mut data)?;

        assert_eq!(test_context.data_size, 16);

        let expected_data: [u8; 16] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0xc3, 0xa1, 0x20, 0x74, 0x65, 0x73,
            0x74, 0x2e,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_decode_with_padding() -> Result<(), ErrorTrace> {
        let mut test_context: Base64Context = Base64Context::new();

        let test_encoded_data: [u8; 25] = [
            0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67, 0x3d, 0x3d, 0x0a,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        test_context.decode(&test_encoded_data, &mut data)?;

        assert_eq!(test_context.data_size, 16);

        let expected_data: [u8; 16] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0xc3, 0xa1, 0x20, 0x74, 0x65, 0x73,
            0x74, 0x2e,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_decode_with_unsupported_padding_character() {
        let mut test_context: Base64Context = Base64Context::new();

        let test_encoded_data: [u8; 25] = [
            0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67, 0x3d, 0xff, 0x0a,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        let result: Result<(), ErrorTrace> = test_context.decode(&test_encoded_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_with_unsupported_padding_size() {
        let mut test_context: Base64Context = Base64Context::new();

        let test_encoded_data: [u8; 23] = [
            0x56, 0x47, 0x68, 0x70, 0x63, 0x79, 0x42, 0x70, 0x63, 0x79, 0x44, 0x44, 0x6f, 0x53,
            0x42, 0x30, 0x5a, 0x58, 0x4e, 0x30, 0x4c, 0x67, 0x3d,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        let result: Result<(), ErrorTrace> = test_context.decode(&test_encoded_data, &mut data);
        assert!(result.is_err());
    }
}
