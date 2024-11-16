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

//! ADC decompression.
//!
//! Provides decompression support for ADC compressed data.

use std::io;

use crate::bytes_to_u16_be;
use crate::mediator::{Mediator, MediatorReference};

/// Context for decompressing ADC compressed data.
pub struct AdcContext {
    /// Mediator.
    mediator: MediatorReference,

    /// Uncompressed data size.
    pub uncompressed_data_size: usize,
}

impl AdcContext {
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
            self.mediator.debug_print(format!("AdcContext {{\n",));
        }
        while compressed_data_offset < compressed_data_size {
            if uncompressed_data_offset >= uncompressed_data_size {
                break;
            }
            if compressed_data_offset >= compressed_data_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid compressed data value too small",
                ));
            }
            let oppcode: u8 = compressed_data[compressed_data_offset];
            compressed_data_offset += 1;

            if self.mediator.debug_output {
                self.mediator
                    .debug_print(format!("    oppcode: {}\n", oppcode));
            }
            if (oppcode & 0x80) != 0 {
                let literal_size: u8 = (oppcode & 0x7f) + 1;

                if literal_size as usize > compressed_data_size - compressed_data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Literal size value exceeds compressed data size",
                    ));
                }
                if literal_size as usize > uncompressed_data_size - uncompressed_data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Literal size value exceeds uncompressed data size",
                    ));
                }
                let compressed_data_end_offset: usize =
                    compressed_data_offset + literal_size as usize;
                let uncompressed_data_end_offset: usize =
                    uncompressed_data_offset + literal_size as usize;

                if self.mediator.debug_output {
                    self.mediator.debug_print(format!("    literal data:\n"));
                    self.mediator.debug_print_data(
                        &compressed_data[compressed_data_offset..compressed_data_end_offset],
                        true,
                    );
                }
                uncompressed_data[uncompressed_data_offset..uncompressed_data_end_offset]
                    .copy_from_slice(
                        &compressed_data[compressed_data_offset..compressed_data_end_offset],
                    );

                compressed_data_offset = compressed_data_end_offset;
                uncompressed_data_offset = uncompressed_data_end_offset;
            } else {
                let match_size: u8;
                let distance: u16;

                if (oppcode & 0x40) != 0 {
                    if 2 > compressed_data_size - compressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    match_size = (oppcode & 0x3f) + 4;
                    distance = bytes_to_u16_be!(compressed_data, compressed_data_offset);

                    compressed_data_offset += 2;
                } else {
                    if compressed_data_offset >= compressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    match_size = ((oppcode & 0x3f) >> 2) + 3;
                    distance = ((oppcode as u16 & 0x03) << 8)
                        | compressed_data[compressed_data_offset] as u16;

                    compressed_data_offset += 1;
                }
                if uncompressed_data_offset < 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid uncompressed data offset value out of bounds",
                    ));
                }
                if distance as usize >= uncompressed_data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid distance value exceeds uncompressed data offset",
                    ));
                }
                if match_size as usize > uncompressed_data_size - uncompressed_data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid match size value exceeds uncompressed data size",
                    ));
                }
                let match_offset: usize = uncompressed_data_offset - distance as usize - 1;
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
                    self.mediator
                        .debug_print_data(&uncompressed_data[match_offset..match_end_offset], true);
                }
            }
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("}}\n\n",));
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
        let test_data: [u8; 10] = [0x83, 0xfe, 0xed, 0xfa, 0xce, 0x00, 0x00, 0x40, 0x00, 0x06];
        let mut test_context: AdcContext = AdcContext::new();

        let expected_data: [u8; 11] = [
            0xfe, 0xed, 0xfa, 0xce, 0xce, 0xce, 0xce, 0xfe, 0xed, 0xfa, 0xce,
        ];
        let mut uncompressed_data: Vec<u8> = vec![0; 11];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(uncompressed_data, expected_data);

        Ok(())
    }
}
