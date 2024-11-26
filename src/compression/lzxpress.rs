/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

//! LZXPRESS (LZ77 + DIRECT2) decompression.
//!
//! Provides decompression support for LZXPRESS compressed data.

use std::io;

use crate::mediator::{Mediator, MediatorReference};
use crate::{bytes_to_u16_le, bytes_to_u32_le};

/// Context for decompressing LZXPRESS compressed data.
pub struct LzxpressContext {
    /// Mediator.
    mediator: MediatorReference,

    /// Uncompressed data size.
    pub uncompressed_data_size: usize,
}

impl LzxpressContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            uncompressed_data_size: 0,
        }
    }

    /// Decompress data.
    pub fn decompress(
        &mut self,
        compressed_data: &[u8],
        uncompressed_data: &mut [u8],
    ) -> io::Result<()> {
        let mut compressed_data_offset: usize = 0;
        let compressed_data_size: usize = compressed_data.len();

        let mut uncompressed_data_offset: usize = 0;
        let uncompressed_data_size: usize = uncompressed_data.len();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(format!("LzxpressContext::decompress {{\n",));
        }
        let mut shared_compression_byte_offset: usize = 0;

        while compressed_data_offset < compressed_data_size {
            if uncompressed_data_offset >= uncompressed_data_size {
                break;
            }
            if 4 > compressed_data_size - compressed_data_offset {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid compressed data value too small",
                ));
            }
            let compression_flags: u32 = bytes_to_u32_le!(compressed_data, compressed_data_offset);

            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "    compressed_data_offset: {} (0x{:08x}),\n",
                    compressed_data_offset, compressed_data_offset
                ));
                self.mediator.debug_print(format!(
                    "    compression_flags: 0x{:08x},\n",
                    compression_flags
                ));
            }
            compressed_data_offset += 4;

            let mut compression_flags_mask = 0x80000000;

            for _ in 0..32 {
                if compressed_data_offset >= compressed_data_size {
                    break;
                }
                if uncompressed_data_offset >= uncompressed_data_size {
                    break;
                }
                // If a compression flags bit is 0 the data is uncompressed or 1 if the data is compressed
                if compression_flags & compression_flags_mask != 0 {
                    if 2 > compressed_data_size - compressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    let compression_tuple: u16 =
                        bytes_to_u16_le!(compressed_data, compressed_data_offset);
                    compressed_data_offset += 2;

                    let distance: u16 = (compression_tuple >> 3) + 1;
                    let mut match_size: u16 = compression_tuple & 0x0007;

                    // Check for a first level extended match size, which is stored in the 4-bits of a shared compression byte.
                    if match_size == 0x07 {
                        if shared_compression_byte_offset == 0 {
                            if compressed_data_offset >= compressed_data_size {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Invalid compressed data value too small",
                                ));
                            }
                            shared_compression_byte_offset = compressed_data_offset;
                            compressed_data_offset += 1;

                            match_size +=
                                (compressed_data[shared_compression_byte_offset] & 0x0f) as u16;
                        } else {
                            match_size +=
                                (compressed_data[shared_compression_byte_offset] >> 4) as u16;

                            shared_compression_byte_offset = 0;
                        }
                    }
                    // Check for a second level extended match size, which is stored in the next 8-bits.
                    if match_size == 0x07 + 0x0f {
                        if compressed_data_offset >= compressed_data_size {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid compressed data value too small",
                            ));
                        }
                        match_size += compressed_data[compressed_data_offset] as u16;
                        compressed_data_offset += 1;
                    }
                    // Check for a third level extended match size, which is stored in the next 16-bits.
                    if match_size == 0x07 + 0x0f + 0xff {
                        if 2 > compressed_data_size - compressed_data_offset {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Invalid compressed data value too small",
                            ));
                        }
                        match_size = bytes_to_u16_le!(compressed_data, compressed_data_offset);
                        compressed_data_offset += 2;
                    }
                    // The match size value is stored as size - 3.
                    match_size += 3;

                    if self.mediator.debug_output {
                        self.mediator.debug_print(format!(
                            "    compressed_data_offset: {} (0x{:08x}),\n",
                            compressed_data_offset, compressed_data_offset
                        ));
                        self.mediator.debug_print(format!(
                            "    compression_tuple: 0x{:04x},\n",
                            compression_tuple,
                        ));
                        self.mediator
                            .debug_print(format!("    distance: {},\n", distance));
                        self.mediator
                            .debug_print(format!("    match_size: {},\n", match_size));
                        self.mediator.debug_print(format!(
                            "    uncompressed_data_offset: {},\n",
                            uncompressed_data_offset
                        ));
                    }
                    if distance as usize > uncompressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid distance value exceeds uncompressed data offset",
                        ));
                    }
                    if match_size > 32771 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid match size value out of bounds",
                        ));
                    }
                    if match_size as usize > uncompressed_data_size - uncompressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid match size value exceeds uncompressed data size",
                        ));
                    }
                    let match_offset: usize = uncompressed_data_offset - distance as usize;
                    let mut match_end_offset: usize = match_offset;

                    for _ in 0..match_size {
                        uncompressed_data[uncompressed_data_offset] =
                            uncompressed_data[match_end_offset];

                        match_end_offset += 1;
                        uncompressed_data_offset += 1;
                    }
                    if self.mediator.debug_output {
                        self.mediator
                            .debug_print(format!("    match offset: {}\n", match_offset));
                        self.mediator.debug_print(format!("    match data:\n"));
                        self.mediator.debug_print_data(
                            &uncompressed_data[match_offset..match_end_offset],
                            true,
                        );
                    }
                } else {
                    if compressed_data_offset >= compressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    if uncompressed_data_offset >= uncompressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid uncompressed data value too small",
                        ));
                    }
                    uncompressed_data[uncompressed_data_offset] =
                        compressed_data[compressed_data_offset];

                    compressed_data_offset += 1;
                    uncompressed_data_offset += 1;
                }
                compression_flags_mask >>= 1;
            }
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("}}\n\n"));
        }
        self.uncompressed_data_size = uncompressed_data_offset;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress() -> io::Result<()> {
        let test_data: [u8; 30] = [
            0x3f, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
            0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78,
            0x79, 0x7a,
        ];
        let mut test_context: LzxpressContext = LzxpressContext::new();

        let mut uncompressed_data: Vec<u8> = vec![0; 26];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, 26);

        let expected_data: [u8; 26] = [
            0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e,
            0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a,
        ];
        assert_eq!(uncompressed_data, expected_data);

        Ok(())
    }

    #[test]
    fn test2_decompress() -> io::Result<()> {
        Mediator { debug_output: true }.make_current();

        let test_data: [u8; 13] = [
            0xff, 0xff, 0xff, 0x1f, 0x61, 0x62, 0x63, 0x17, 0x00, 0x0f, 0xff, 0x26, 0x01,
        ];
        let mut test_context: LzxpressContext = LzxpressContext::new();

        let mut uncompressed_data: Vec<u8> = vec![0; 300];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, 300);

        let expected_data: [u8; 300] = [
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62,
            0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61,
            0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
            0x61, 0x62, 0x63, 0x61, 0x62, 0x63,
        ];
        assert_eq!(uncompressed_data, expected_data);

        Ok(())
    }
}
