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

//! LZVN decompression.
//!
//! Provides decompression support for LZVN compressed data.

use std::io;

use crate::mediator::{Mediator, MediatorReference};

/// LZVN oppcode type.
#[derive(Clone, PartialEq)]
enum LzvnOppcodeType {
    DistanceLarge,
    DistanceMedium,
    DistancePrevious,
    DistanceSmall,
    EndOfStream,
    Invalid,
    LiteralLarge,
    LiteralSmall,
    MatchLarge,
    MatchSmall,
    None,
}

/// Lookup table to map a LZCN oppcode to its type.
const LZVN_OPPCODE_TYPES: [LzvnOppcodeType; 256] = [
    LzvnOppcodeType::DistanceSmall,    // 0x00
    LzvnOppcodeType::DistanceSmall,    // 0x01
    LzvnOppcodeType::DistanceSmall,    // 0x02
    LzvnOppcodeType::DistanceSmall,    // 0x03
    LzvnOppcodeType::DistanceSmall,    // 0x04
    LzvnOppcodeType::DistanceSmall,    // 0x05
    LzvnOppcodeType::EndOfStream,      // 0x06
    LzvnOppcodeType::DistanceLarge,    // 0x07
    LzvnOppcodeType::DistanceSmall,    // 0x08
    LzvnOppcodeType::DistanceSmall,    // 0x09
    LzvnOppcodeType::DistanceSmall,    // 0x0a
    LzvnOppcodeType::DistanceSmall,    // 0x0b
    LzvnOppcodeType::DistanceSmall,    // 0x0c
    LzvnOppcodeType::DistanceSmall,    // 0x0d
    LzvnOppcodeType::None,             // 0x0e
    LzvnOppcodeType::DistanceLarge,    // 0x0f
    LzvnOppcodeType::DistanceSmall,    // 0x10
    LzvnOppcodeType::DistanceSmall,    // 0x11
    LzvnOppcodeType::DistanceSmall,    // 0x12
    LzvnOppcodeType::DistanceSmall,    // 0x13
    LzvnOppcodeType::DistanceSmall,    // 0x14
    LzvnOppcodeType::DistanceSmall,    // 0x15
    LzvnOppcodeType::None,             // 0x16
    LzvnOppcodeType::DistanceLarge,    // 0x17
    LzvnOppcodeType::DistanceSmall,    // 0x18
    LzvnOppcodeType::DistanceSmall,    // 0x19
    LzvnOppcodeType::DistanceSmall,    // 0x1a
    LzvnOppcodeType::DistanceSmall,    // 0x1b
    LzvnOppcodeType::DistanceSmall,    // 0x1c
    LzvnOppcodeType::DistanceSmall,    // 0x1d
    LzvnOppcodeType::Invalid,          // 0x1e
    LzvnOppcodeType::DistanceLarge,    // 0x1f
    LzvnOppcodeType::DistanceSmall,    // 0x20
    LzvnOppcodeType::DistanceSmall,    // 0x21
    LzvnOppcodeType::DistanceSmall,    // 0x22
    LzvnOppcodeType::DistanceSmall,    // 0x23
    LzvnOppcodeType::DistanceSmall,    // 0x24
    LzvnOppcodeType::DistanceSmall,    // 0x25
    LzvnOppcodeType::Invalid,          // 0x26
    LzvnOppcodeType::DistanceLarge,    // 0x27
    LzvnOppcodeType::DistanceSmall,    // 0x28
    LzvnOppcodeType::DistanceSmall,    // 0x29
    LzvnOppcodeType::DistanceSmall,    // 0x2a
    LzvnOppcodeType::DistanceSmall,    // 0x2b
    LzvnOppcodeType::DistanceSmall,    // 0x2c
    LzvnOppcodeType::DistanceSmall,    // 0x2d
    LzvnOppcodeType::Invalid,          // 0x2e
    LzvnOppcodeType::DistanceLarge,    // 0x2f
    LzvnOppcodeType::DistanceSmall,    // 0x30
    LzvnOppcodeType::DistanceSmall,    // 0x31
    LzvnOppcodeType::DistanceSmall,    // 0x32
    LzvnOppcodeType::DistanceSmall,    // 0x33
    LzvnOppcodeType::DistanceSmall,    // 0x34
    LzvnOppcodeType::DistanceSmall,    // 0x35
    LzvnOppcodeType::Invalid,          // 0x36
    LzvnOppcodeType::DistanceLarge,    // 0x37
    LzvnOppcodeType::DistanceSmall,    // 0x38
    LzvnOppcodeType::DistanceSmall,    // 0x39
    LzvnOppcodeType::DistanceSmall,    // 0x3a
    LzvnOppcodeType::DistanceSmall,    // 0x3b
    LzvnOppcodeType::DistanceSmall,    // 0x3c
    LzvnOppcodeType::DistanceSmall,    // 0x3d
    LzvnOppcodeType::Invalid,          // 0x3e
    LzvnOppcodeType::DistanceLarge,    // 0x3f
    LzvnOppcodeType::DistanceSmall,    // 0x40
    LzvnOppcodeType::DistanceSmall,    // 0x41
    LzvnOppcodeType::DistanceSmall,    // 0x42
    LzvnOppcodeType::DistanceSmall,    // 0x43
    LzvnOppcodeType::DistanceSmall,    // 0x44
    LzvnOppcodeType::DistanceSmall,    // 0x45
    LzvnOppcodeType::DistancePrevious, // 0x46
    LzvnOppcodeType::DistanceLarge,    // 0x47
    LzvnOppcodeType::DistanceSmall,    // 0x48
    LzvnOppcodeType::DistanceSmall,    // 0x49
    LzvnOppcodeType::DistanceSmall,    // 0x4a
    LzvnOppcodeType::DistanceSmall,    // 0x4b
    LzvnOppcodeType::DistanceSmall,    // 0x4c
    LzvnOppcodeType::DistanceSmall,    // 0x4d
    LzvnOppcodeType::DistancePrevious, // 0x4e
    LzvnOppcodeType::DistanceLarge,    // 0x4f
    LzvnOppcodeType::DistanceSmall,    // 0x50
    LzvnOppcodeType::DistanceSmall,    // 0x51
    LzvnOppcodeType::DistanceSmall,    // 0x52
    LzvnOppcodeType::DistanceSmall,    // 0x53
    LzvnOppcodeType::DistanceSmall,    // 0x54
    LzvnOppcodeType::DistanceSmall,    // 0x55
    LzvnOppcodeType::DistancePrevious, // 0x56
    LzvnOppcodeType::DistanceLarge,    // 0x57
    LzvnOppcodeType::DistanceSmall,    // 0x58
    LzvnOppcodeType::DistanceSmall,    // 0x59
    LzvnOppcodeType::DistanceSmall,    // 0x5a
    LzvnOppcodeType::DistanceSmall,    // 0x5b
    LzvnOppcodeType::DistanceSmall,    // 0x5c
    LzvnOppcodeType::DistanceSmall,    // 0x5d
    LzvnOppcodeType::DistancePrevious, // 0x5e
    LzvnOppcodeType::DistanceLarge,    // 0x5f
    LzvnOppcodeType::DistanceSmall,    // 0x60
    LzvnOppcodeType::DistanceSmall,    // 0x61
    LzvnOppcodeType::DistanceSmall,    // 0x62
    LzvnOppcodeType::DistanceSmall,    // 0x63
    LzvnOppcodeType::DistanceSmall,    // 0x64
    LzvnOppcodeType::DistanceSmall,    // 0x65
    LzvnOppcodeType::DistancePrevious, // 0x66
    LzvnOppcodeType::DistanceLarge,    // 0x67
    LzvnOppcodeType::DistanceSmall,    // 0x68
    LzvnOppcodeType::DistanceSmall,    // 0x69
    LzvnOppcodeType::DistanceSmall,    // 0x6a
    LzvnOppcodeType::DistanceSmall,    // 0x6b
    LzvnOppcodeType::DistanceSmall,    // 0x6c
    LzvnOppcodeType::DistanceSmall,    // 0x6d
    LzvnOppcodeType::DistancePrevious, // 0x6e
    LzvnOppcodeType::DistanceLarge,    // 0x6f
    LzvnOppcodeType::Invalid,          // 0x70
    LzvnOppcodeType::Invalid,          // 0x71
    LzvnOppcodeType::Invalid,          // 0x72
    LzvnOppcodeType::Invalid,          // 0x73
    LzvnOppcodeType::Invalid,          // 0x74
    LzvnOppcodeType::Invalid,          // 0x75
    LzvnOppcodeType::Invalid,          // 0x76
    LzvnOppcodeType::Invalid,          // 0x77
    LzvnOppcodeType::Invalid,          // 0x78
    LzvnOppcodeType::Invalid,          // 0x79
    LzvnOppcodeType::Invalid,          // 0x7a
    LzvnOppcodeType::Invalid,          // 0x7b
    LzvnOppcodeType::Invalid,          // 0x7c
    LzvnOppcodeType::Invalid,          // 0x7d
    LzvnOppcodeType::Invalid,          // 0x7e
    LzvnOppcodeType::Invalid,          // 0x7f
    LzvnOppcodeType::DistanceSmall,    // 0x80
    LzvnOppcodeType::DistanceSmall,    // 0x81
    LzvnOppcodeType::DistanceSmall,    // 0x82
    LzvnOppcodeType::DistanceSmall,    // 0x83
    LzvnOppcodeType::DistanceSmall,    // 0x84
    LzvnOppcodeType::DistanceSmall,    // 0x85
    LzvnOppcodeType::DistancePrevious, // 0x86
    LzvnOppcodeType::DistanceLarge,    // 0x87
    LzvnOppcodeType::DistanceSmall,    // 0x88
    LzvnOppcodeType::DistanceSmall,    // 0x89
    LzvnOppcodeType::DistanceSmall,    // 0x8a
    LzvnOppcodeType::DistanceSmall,    // 0x8b
    LzvnOppcodeType::DistanceSmall,    // 0x8c
    LzvnOppcodeType::DistanceSmall,    // 0x8d
    LzvnOppcodeType::DistancePrevious, // 0x8e
    LzvnOppcodeType::DistanceLarge,    // 0x8f
    LzvnOppcodeType::DistanceSmall,    // 0x90
    LzvnOppcodeType::DistanceSmall,    // 0x91
    LzvnOppcodeType::DistanceSmall,    // 0x92
    LzvnOppcodeType::DistanceSmall,    // 0x93
    LzvnOppcodeType::DistanceSmall,    // 0x94
    LzvnOppcodeType::DistanceSmall,    // 0x95
    LzvnOppcodeType::DistancePrevious, // 0x96
    LzvnOppcodeType::DistanceLarge,    // 0x97
    LzvnOppcodeType::DistanceSmall,    // 0x98
    LzvnOppcodeType::DistanceSmall,    // 0x99
    LzvnOppcodeType::DistanceSmall,    // 0x9a
    LzvnOppcodeType::DistanceSmall,    // 0x9b
    LzvnOppcodeType::DistanceSmall,    // 0x9c
    LzvnOppcodeType::DistanceSmall,    // 0x9d
    LzvnOppcodeType::DistancePrevious, // 0x9e
    LzvnOppcodeType::DistanceLarge,    // 0x9f
    LzvnOppcodeType::DistanceMedium,   // 0xa0
    LzvnOppcodeType::DistanceMedium,   // 0xa1
    LzvnOppcodeType::DistanceMedium,   // 0xa2
    LzvnOppcodeType::DistanceMedium,   // 0xa3
    LzvnOppcodeType::DistanceMedium,   // 0xa4
    LzvnOppcodeType::DistanceMedium,   // 0xa5
    LzvnOppcodeType::DistanceMedium,   // 0xa6
    LzvnOppcodeType::DistanceMedium,   // 0xa7
    LzvnOppcodeType::DistanceMedium,   // 0xa8
    LzvnOppcodeType::DistanceMedium,   // 0xa9
    LzvnOppcodeType::DistanceMedium,   // 0xaa
    LzvnOppcodeType::DistanceMedium,   // 0xab
    LzvnOppcodeType::DistanceMedium,   // 0xac
    LzvnOppcodeType::DistanceMedium,   // 0xad
    LzvnOppcodeType::DistanceMedium,   // 0xae
    LzvnOppcodeType::DistanceMedium,   // 0xaf
    LzvnOppcodeType::DistanceMedium,   // 0xb0
    LzvnOppcodeType::DistanceMedium,   // 0xb1
    LzvnOppcodeType::DistanceMedium,   // 0xb2
    LzvnOppcodeType::DistanceMedium,   // 0xb3
    LzvnOppcodeType::DistanceMedium,   // 0xb4
    LzvnOppcodeType::DistanceMedium,   // 0xb5
    LzvnOppcodeType::DistanceMedium,   // 0xb6
    LzvnOppcodeType::DistanceMedium,   // 0xb7
    LzvnOppcodeType::DistanceMedium,   // 0xb8
    LzvnOppcodeType::DistanceMedium,   // 0xb9
    LzvnOppcodeType::DistanceMedium,   // 0xba
    LzvnOppcodeType::DistanceMedium,   // 0xbb
    LzvnOppcodeType::DistanceMedium,   // 0xbc
    LzvnOppcodeType::DistanceMedium,   // 0xbd
    LzvnOppcodeType::DistanceMedium,   // 0xbe
    LzvnOppcodeType::DistanceMedium,   // 0xbf
    LzvnOppcodeType::DistanceSmall,    // 0xc0
    LzvnOppcodeType::DistanceSmall,    // 0xc1
    LzvnOppcodeType::DistanceSmall,    // 0xc2
    LzvnOppcodeType::DistanceSmall,    // 0xc3
    LzvnOppcodeType::DistanceSmall,    // 0xc4
    LzvnOppcodeType::DistanceSmall,    // 0xc5
    LzvnOppcodeType::DistancePrevious, // 0xc6
    LzvnOppcodeType::DistanceLarge,    // 0xc7
    LzvnOppcodeType::DistanceSmall,    // 0xc8
    LzvnOppcodeType::DistanceSmall,    // 0xc9
    LzvnOppcodeType::DistanceSmall,    // 0xca
    LzvnOppcodeType::DistanceSmall,    // 0xcb
    LzvnOppcodeType::DistanceSmall,    // 0xcc
    LzvnOppcodeType::DistanceSmall,    // 0xcd
    LzvnOppcodeType::DistancePrevious, // 0xce
    LzvnOppcodeType::DistanceLarge,    // 0xcf
    LzvnOppcodeType::Invalid,          // 0xd0
    LzvnOppcodeType::Invalid,          // 0xd1
    LzvnOppcodeType::Invalid,          // 0xd2
    LzvnOppcodeType::Invalid,          // 0xd3
    LzvnOppcodeType::Invalid,          // 0xd4
    LzvnOppcodeType::Invalid,          // 0xd5
    LzvnOppcodeType::Invalid,          // 0xd6
    LzvnOppcodeType::Invalid,          // 0xd7
    LzvnOppcodeType::Invalid,          // 0xd8
    LzvnOppcodeType::Invalid,          // 0xd9
    LzvnOppcodeType::Invalid,          // 0xda
    LzvnOppcodeType::Invalid,          // 0xdb
    LzvnOppcodeType::Invalid,          // 0xdc
    LzvnOppcodeType::Invalid,          // 0xdd
    LzvnOppcodeType::Invalid,          // 0xde
    LzvnOppcodeType::Invalid,          // 0xdf
    LzvnOppcodeType::LiteralLarge,     // 0xe0
    LzvnOppcodeType::LiteralSmall,     // 0xe1
    LzvnOppcodeType::LiteralSmall,     // 0xe2
    LzvnOppcodeType::LiteralSmall,     // 0xe3
    LzvnOppcodeType::LiteralSmall,     // 0xe4
    LzvnOppcodeType::LiteralSmall,     // 0xe5
    LzvnOppcodeType::LiteralSmall,     // 0xe6
    LzvnOppcodeType::LiteralSmall,     // 0xe7
    LzvnOppcodeType::LiteralSmall,     // 0xe8
    LzvnOppcodeType::LiteralSmall,     // 0xe9
    LzvnOppcodeType::LiteralSmall,     // 0xea
    LzvnOppcodeType::LiteralSmall,     // 0xeb
    LzvnOppcodeType::LiteralSmall,     // 0xec
    LzvnOppcodeType::LiteralSmall,     // 0xed
    LzvnOppcodeType::LiteralSmall,     // 0xee
    LzvnOppcodeType::LiteralSmall,     // 0xef
    LzvnOppcodeType::MatchLarge,       // 0xf0
    LzvnOppcodeType::MatchSmall,       // 0xf1
    LzvnOppcodeType::MatchSmall,       // 0xf2
    LzvnOppcodeType::MatchSmall,       // 0xf3
    LzvnOppcodeType::MatchSmall,       // 0xf4
    LzvnOppcodeType::MatchSmall,       // 0xf5
    LzvnOppcodeType::MatchSmall,       // 0xf6
    LzvnOppcodeType::MatchSmall,       // 0xf7
    LzvnOppcodeType::MatchSmall,       // 0xf8
    LzvnOppcodeType::MatchSmall,       // 0xf9
    LzvnOppcodeType::MatchSmall,       // 0xfa
    LzvnOppcodeType::MatchSmall,       // 0xfb
    LzvnOppcodeType::MatchSmall,       // 0xfc
    LzvnOppcodeType::MatchSmall,       // 0xfd
    LzvnOppcodeType::MatchSmall,       // 0xfe
    LzvnOppcodeType::MatchSmall,       // 0xff
];

/// Context for decompressing LZVN compressed data.
pub struct LzvnContext {
    /// Mediator.
    mediator: MediatorReference,

    /// Uncompressed data size.
    pub uncompressed_data_size: usize,
}

impl LzvnContext {
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
            self.mediator.debug_print(format!("LzvnContext {{\n",));
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
            let mut literal_size: u16 = 0;
            let mut match_size: u16 = 0;
            let mut distance: u16 = 0;

            match &LZVN_OPPCODE_TYPES[oppcode as usize] {
                LzvnOppcodeType::DistanceLarge => {
                    if 2 > compressed_data_size - compressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    let oppcode_value: u8 = compressed_data[compressed_data_offset];
                    compressed_data_offset += 1;

                    literal_size = (oppcode as u16 & 0xc0) >> 6;
                    match_size = ((oppcode as u16 & 0x38) >> 3) + 3;
                    distance = ((compressed_data[compressed_data_offset] as u16) << 8)
                        | oppcode_value as u16;

                    compressed_data_offset += 1;
                }
                LzvnOppcodeType::DistanceMedium => {
                    if 2 > compressed_data_size - compressed_data_offset {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    let oppcode_value: u8 = compressed_data[compressed_data_offset];
                    compressed_data_offset += 1;

                    literal_size = (oppcode as u16 & 0x18) >> 3;
                    match_size =
                        (((oppcode as u16 & 0x07) << 2) | (oppcode_value as u16 & 0x03)) + 3;
                    distance = ((compressed_data[compressed_data_offset] as u16) << 6)
                        | ((oppcode_value as u16 & 0xfc) >> 2);

                    compressed_data_offset += 1;
                }
                LzvnOppcodeType::DistancePrevious => {
                    literal_size = (oppcode as u16 & 0xc0) >> 6;
                    match_size = ((oppcode as u16 & 0x38) >> 3) + 3;
                }
                LzvnOppcodeType::DistanceSmall => {
                    if compressed_data_offset >= compressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    literal_size = (oppcode as u16 & 0xc0) >> 6;
                    match_size = ((oppcode as u16 & 0x38) >> 3) + 3;
                    distance = ((oppcode as u16 & 0x07) << 8)
                        | compressed_data[compressed_data_offset] as u16;

                    compressed_data_offset += 1;
                }
                LzvnOppcodeType::LiteralLarge => {
                    if compressed_data_offset >= compressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    literal_size = compressed_data[compressed_data_offset] as u16 + 16;

                    compressed_data_offset += 1;
                }
                LzvnOppcodeType::LiteralSmall => {
                    literal_size = oppcode as u16 & 0x0f;
                }
                LzvnOppcodeType::MatchLarge => {
                    if compressed_data_offset >= compressed_data_size {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid compressed data value too small",
                        ));
                    }
                    match_size = compressed_data[compressed_data_offset] as u16 + 16;

                    compressed_data_offset += 1;
                }
                LzvnOppcodeType::MatchSmall => {
                    match_size = oppcode as u16 & 0x0f;
                }
                LzvnOppcodeType::EndOfStream => {
                    break;
                }
                LzvnOppcodeType::None => {}
                LzvnOppcodeType::Invalid => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid oppcode: {}", oppcode),
                    ));
                }
            };
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(format!("    literal_size: {}\n", literal_size));
                self.mediator
                    .debug_print(format!("    match_size: {}\n", match_size));
                self.mediator
                    .debug_print(format!("    distance: {}\n", distance));
            }
            if literal_size > 0 {
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
            }
            if match_size > 0 {
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
        let test_data: [u8; 29] = [
            0xe0, 0x03, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65,
            0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00,
        ];
        let mut test_context: LzvnContext = LzvnContext::new();

        let expected_data: [u8; 19] = [
            0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x20,
            0x66, 0x69, 0x6c, 0x65, 0x0a,
        ];
        let mut uncompressed_data: Vec<u8> = vec![0; 19];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(uncompressed_data, expected_data);

        Ok(())
    }
}
