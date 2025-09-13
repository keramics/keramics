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

//! Bzip2 decompression.
//!
//! Provides decompression support for bzip2 compressed data.

use std::cmp;
use std::io;

use keramics_checksums::Crc32Context;
use keramics_core::formatters::debug_format_array;
use keramics_core::mediator::{Mediator, MediatorReference};
use layout_map::LayoutMap;

use super::huffman::HuffmanTree;
use super::traits::Bitstream;

/// Bzip2 data header signature.
pub(super) const BZIP2_DATA_HEADER_SIGNATURE: [u8; 2] = [0x42, 0x5a]; // BZ

/// Bzip2 block size;
pub(super) const BZIP2_BLOCK_SIZE: usize = 100000;

/// Bitstream for bzip2 compressed data.
pub(super) struct Bzip2Bitstream<'a> {
    /// Byte steam.
    data: &'a [u8],

    /// Current offset in the byte stream.
    pub data_offset: usize,

    /// Size of the byte stream in bytes.
    pub data_size: usize,

    /// Bits buffer.
    bits: u32,

    /// Number of bits in the bits buffer.
    pub number_of_bits: usize,
}

impl<'a> Bzip2Bitstream<'a> {
    /// Creates a new bitstream.
    pub fn new(data: &'a [u8], data_offset: usize) -> Self {
        let data_size: usize = data.len();
        Self {
            data: data,
            data_offset: data_offset,
            data_size: data_size,
            bits: 0,
            number_of_bits: 0,
        }
    }

    /// Reads input data forwards into the bits buffer in big-endian byte order.
    #[inline(always)]
    fn read_data(&mut self, number_of_bits: usize) {
        while number_of_bits > self.number_of_bits {
            self.bits <<= 8;

            // If the bit stream overflows fill the bit buffer with 0 byte values.
            if self.data_offset < self.data_size {
                self.bits |= self.data[self.data_offset] as u32;
                self.data_offset += 1;
            }
            self.number_of_bits += 8;
        }
    }
}

impl<'a> Bitstream for Bzip2Bitstream<'a> {
    /// Retrieves a bit value.
    fn get_value(&mut self, number_of_bits: usize) -> u32 {
        // Note that this does not check if number_of_bits <= 32
        let mut bit_value: u32 = 0;

        let mut bit_offset: usize = 0;
        while bit_offset < number_of_bits {
            let mut read_size: usize = number_of_bits - bit_offset;
            if read_size > 24 {
                read_size = 24;
            }
            if self.number_of_bits < read_size {
                self.read_data(read_size);
            }
            let mut value_32bit: u32 = self.bits;

            self.number_of_bits -= read_size;

            if self.number_of_bits == 0 {
                self.bits = 0;
            } else {
                self.bits &= 0xffffffff >> (32 - self.number_of_bits);

                value_32bit >>= self.number_of_bits;
            }
            if bit_offset > 0 {
                bit_value <<= read_size;
            }
            bit_value |= value_32bit;
            bit_offset += read_size;
        }
        bit_value
    }

    /// Skips a number of bits.
    fn skip_bits(&mut self, number_of_bits: usize) {
        // Note that this does not check if number_of_bits <= 32
        let mut bit_offset: usize = 0;
        while bit_offset < number_of_bits {
            let mut read_size: usize = number_of_bits - bit_offset;
            if read_size > 24 {
                read_size = 24;
            }
            if self.number_of_bits < read_size {
                self.read_data(read_size);
            }
            self.number_of_bits -= read_size;

            if self.number_of_bits == 0 {
                self.bits = 0;
            } else {
                self.bits &= 0xffffffff >> (32 - self.number_of_bits);
            }
            bit_offset += read_size;
        }
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 2]", format = "char"),
        field(name = "format_version", data_type = "u8"),
        field(name = "compression_level", data_type = "u8", format = "char"),
    ),
    method(name = "debug_read_data")
)]
/// Stream header used by bzip2 compressed data.
struct Bzip2StreamHeader {}

impl Bzip2StreamHeader {
    /// Creates a new stream header.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the stream header.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..2] != BZIP2_DATA_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        let compression_level: u8 = data[3];

        if compression_level < 0x31 || compression_level > 0x39 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported compression level: {}",
                    compression_level as char
                ),
            ));
        }
        Ok(())
    }
}

/// Block header used by bzip2 compressed data.
struct Bzip2BlockHeader {
    /// Signature.
    pub signature: u64,

    /// Checksum.
    pub checksum: u32,

    pub randomized_flag: u32,

    /// Origin pointer.
    pub origin_pointer: u32,
}

impl Bzip2BlockHeader {
    /// Creates a new block header.
    pub fn new() -> Self {
        Self {
            signature: 0,
            checksum: 0,
            randomized_flag: 0,
            origin_pointer: 0,
        }
    }

    /// Reads the block header from a bitstream.
    pub fn read_from_bitstream(&mut self, bitstream: &mut Bzip2Bitstream) -> io::Result<()> {
        self.signature = ((bitstream.get_value(24) as u64) << 24) | bitstream.get_value(24) as u64;

        if self.signature == 0x177245385090 {
            self.checksum = bitstream.get_value(32);
            self.randomized_flag = 0;
            self.origin_pointer = 0;
        } else if self.signature == 0x314159265359 {
            self.checksum = bitstream.get_value(32);
            self.randomized_flag = bitstream.get_value(1);
            self.origin_pointer = bitstream.get_value(24);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature: 0x{:12x}", self.signature),
            ));
        }
        let mediator = Mediator::current();
        if mediator.debug_output {
            let mut string_parts: Vec<String> = Vec::new();
            string_parts.push(format!("Bzip2BlockHeader {{\n"));
            string_parts.push(format!("    signature: 0x{:012x},\n", self.signature));
            string_parts.push(format!("    checksum: 0x{:08x},\n", self.checksum));

            if self.signature == 0x314159265359 {
                string_parts.push(format!("    randomized_flag: {},\n", self.randomized_flag));
                string_parts.push(format!(
                    "    origin_pointer: 0x{:06x},\n",
                    self.origin_pointer
                ));
            }
            string_parts.push(format!("}}\n\n"));

            mediator.debug_print(string_parts.join(""));
        }
        Ok(())
    }
}

/// Context for decompressing bzip2 compressed data.
pub struct Bzip2Context {
    /// Mediator.
    mediator: MediatorReference,

    /// Uncompressed data size.
    pub uncompressed_data_size: usize,
}

impl Bzip2Context {
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
        let compressed_data_size: usize = compressed_data.len();

        if compressed_data_size < 14 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid compressed data value too small",
            ));
        }
        let mut data_header: Bzip2StreamHeader = Bzip2StreamHeader::new();

        if self.mediator.debug_output {
            let header_size: usize = if compressed_data[0] & 0x20 == 0 { 2 } else { 6 };

            self.mediator.debug_print(format!(
                "Bzip2StreamHeader data of size: {} at offset: 0 (0x00000000)\n",
                header_size,
            ));
            self.mediator.debug_print_data(compressed_data, true);
            self.mediator
                .debug_print(Bzip2StreamHeader::debug_read_data(compressed_data));
        }
        data_header.read_data(compressed_data)?;

        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&compressed_data, 4);
        self.decompress_bitstream(&mut bitstream, uncompressed_data)?;

        Ok(())
    }

    /// Decompress a bitstream.
    pub(super) fn decompress_bitstream(
        &mut self,
        bitstream: &mut Bzip2Bitstream,
        uncompressed_data: &mut [u8],
    ) -> io::Result<()> {
        let mut block_data: [u8; BZIP2_BLOCK_SIZE] = [0; BZIP2_BLOCK_SIZE];
        let mut selectors: [u8; 32769] = [0; 32769]; // ( 1 << 15 ) + 1 = 32769
        let mut symbol_stack: [u8; 256] = [0; 256];
        let mut uncompressed_data_offset: usize = 0;
        let uncompressed_data_size: usize = uncompressed_data.len();

        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();
        while bitstream.data_offset < bitstream.data_size {
            block_header.read_from_bitstream(bitstream)?;

            if block_header.signature == 0x177245385090 {
                break;
            }
            if (block_header.origin_pointer as usize) >= BZIP2_BLOCK_SIZE {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid origin pointer: 0x{:06x} value out of bounds",
                        block_header.origin_pointer
                    ),
                ));
            }
            let number_of_symbols: usize = self.read_symbol_stack(bitstream, &mut symbol_stack)?;
            let number_of_trees: u32 = bitstream.get_value(3);
            let number_of_selectors: u32 = bitstream.get_value(15);

            if self.mediator.debug_output {
                self.mediator.debug_print(format!("Bzip2Bitstream {{\n",));
                self.mediator
                    .debug_print(format!("    number_of_symbols: {}\n", number_of_symbols));
                self.mediator
                    .debug_print(format!("    number_of_trees: {}\n", number_of_trees));
                self.mediator.debug_print(format!(
                    "    number_of_selectors: {}\n",
                    number_of_selectors
                ));
                self.mediator.debug_print(format!("}}\n\n",));
            }
            self.read_selectors(
                bitstream,
                &mut selectors,
                number_of_selectors as usize,
                number_of_trees as usize,
            )?;
            let mut huffman_trees: Vec<HuffmanTree> = Vec::new();

            for _ in 0..number_of_trees {
                let mut huffman_tree: HuffmanTree = HuffmanTree::new(number_of_symbols, 20);

                self.read_huffman_tree(bitstream, &mut huffman_tree, number_of_symbols)?;

                huffman_trees.push(huffman_tree);
            }
            let block_data_size: usize = self.read_block_data(
                bitstream,
                &huffman_trees,
                number_of_trees as usize,
                &selectors,
                number_of_selectors as usize,
                &mut symbol_stack,
                number_of_symbols,
                &mut block_data,
            )?;
            self.reverse_burrows_wheeler_transform(
                &block_data,
                block_data_size,
                block_header.origin_pointer,
                uncompressed_data,
                &mut uncompressed_data_offset,
                uncompressed_data_size,
            )?;
        }
        let mut crc32_context: Crc32Context = Crc32Context::new(0x04c11db7, 0);
        crc32_context.update(&uncompressed_data[0..uncompressed_data_offset]);
        let calculated_checksum: u32 = crc32_context.finalize();

        if block_header.checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    block_header.checksum, calculated_checksum
                ),
            ));
        }
        self.uncompressed_data_size = uncompressed_data_offset;

        Ok(())
    }

    /// Reads block data from a bitstream.
    fn read_block_data(
        &self,
        bitstream: &mut Bzip2Bitstream,
        huffman_trees: &[HuffmanTree],
        number_of_trees: usize,
        selectors: &[u8],
        number_of_selectors: usize,
        symbol_stack: &mut [u8],
        number_of_symbols: usize,
        block_data: &mut [u8],
    ) -> io::Result<usize> {
        let end_of_block_symbol: u16 = (number_of_symbols - 1) as u16;
        let mut block_data_offset: usize = 0;
        let mut number_of_run_length_symbols: u64 = 0;
        let mut run_length_value: u64 = 0;
        let mut symbol_index: usize = 0;
        let mut tree_index: usize = selectors[0] as usize;

        if self.mediator.debug_output {
            self.mediator.debug_print(format!("Bzip2BlockData {{\n",));
        }
        loop {
            if tree_index >= number_of_trees {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid tree index: {} value out of bounds", tree_index),
                ));
            }
            let huffman_tree: &HuffmanTree = &huffman_trees[tree_index];
            let symbol: u16 = huffman_tree.decode_symbol(bitstream)?;

            if number_of_run_length_symbols != 0 && symbol > 1 {
                let mut run_length: u64 =
                    ((1 << number_of_run_length_symbols) | run_length_value) - 1;

                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    0-byte run-length: {}\n", run_length,));
                }
                if (run_length as usize) > BZIP2_BLOCK_SIZE - block_data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid run length: {} value out of bounds", run_length),
                    ));
                }
                number_of_run_length_symbols = 0;
                run_length_value = 0;

                while run_length > 0 {
                    // Inverse move-to-front transform.
                    // Note that 0 is already at the front of the stack hence the stack does not need to be reordered.
                    block_data[block_data_offset] = symbol_stack[0];

                    block_data_offset += 1;
                    run_length -= 1;
                }
            }
            if symbol == end_of_block_symbol {
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    symbol: {}\n", symbol));
                }
                break;
            }
            if symbol == 0 || symbol == 1 {
                run_length_value |= (symbol as u64) << (number_of_run_length_symbols as u64);
                number_of_run_length_symbols += 1;

                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    symbol: {} (run-length)\n", symbol));
                }
            } else if symbol < end_of_block_symbol {
                // Inverse move-to-front transform.
                let stack_value_index: usize = (symbol as usize) - 1;

                let stack_value: u8 = symbol_stack[stack_value_index];

                for stack_index in (0..stack_value_index).rev() {
                    symbol_stack[stack_index + 1] = symbol_stack[stack_index];
                }
                symbol_stack[0] = stack_value;

                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    symbol: {} (MTF: {})\n", symbol, stack_value));
                }
                if block_data_offset >= BZIP2_BLOCK_SIZE {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid block data offset: {} value out of bounds",
                            block_data_offset
                        ),
                    ));
                }
                block_data[block_data_offset] = stack_value;

                block_data_offset += 1;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid symbol: {} value out of bounds", symbol),
                ));
            }
            symbol_index += 1;

            if symbol_index % 50 == 0 {
                let selector_index: usize = symbol_index / 50;

                if selector_index > number_of_selectors {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid selector index: {} value out of bounds",
                            selector_index
                        ),
                    ));
                }
                tree_index = selectors[selector_index] as usize;
            }
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("}}\n\n",));
        }
        Ok(block_data_offset)
    }

    /// Reads a Huffman tree from a bitstream.
    fn read_huffman_tree(
        &self,
        bitstream: &mut Bzip2Bitstream,
        huffman_tree: &mut HuffmanTree,
        number_of_symbols: usize,
    ) -> io::Result<()> {
        let mut code_size: u32 = bitstream.get_value(5);
        let mut code_size_array: [u8; 258] = [0; 258];
        let mut largest_code_size: u32 = code_size;

        for symbol_index in 0..number_of_symbols {
            while code_size < 20 {
                let value_32bit: u32 = bitstream.get_value(1);
                if value_32bit == 0 {
                    break;
                }
                let value_32bit: u32 = bitstream.get_value(1);
                if value_32bit == 0 {
                    code_size += 1;
                } else {
                    code_size -= 1;
                }
            }
            if code_size >= 20 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid code size: {} value out of bounds", code_size),
                ));
            }
            code_size_array[symbol_index] = code_size as u8;

            largest_code_size = cmp::max(code_size, largest_code_size);
        }
        if largest_code_size > 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid largest code size: {} value out of bounds",
                    largest_code_size
                ),
            ));
        }
        let mut check_value: u32 = 1 << largest_code_size;

        for symbol_index in 0..number_of_symbols {
            code_size = code_size_array[symbol_index] as u32;
            check_value -= 1 << (largest_code_size - code_size);
        }
        if check_value != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid check value: {} value out of bounds", check_value),
            ));
        }
        huffman_tree.build(&code_size_array[0..number_of_symbols])
    }

    /// Reads the selectors from a bitstream.
    fn read_selectors(
        &self,
        bitstream: &mut Bzip2Bitstream,
        selectors: &mut [u8],
        number_of_selectors: usize,
        number_of_trees: usize,
    ) -> io::Result<()> {
        let mut selector_index: usize = 0;
        let mut stack: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];

        while selector_index < number_of_selectors {
            let mut tree_index: usize = 0;

            while tree_index < number_of_trees {
                let value_32bit: u32 = bitstream.get_value(1);
                if value_32bit == 0 {
                    break;
                }
                tree_index += 1;
            }
            if tree_index >= number_of_trees {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid tree index: {} value out of bounds", tree_index),
                ));
            }
            // Inverse move-to-front transform.
            let selector_value: u8 = stack[tree_index];

            selectors[selector_index] = selector_value;

            for stack_index in (0..tree_index).rev() {
                stack[stack_index + 1] = stack[stack_index];
            }
            stack[0] = selector_value;

            selector_index += 1;
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("Bzip2Selectors {{\n",));

            let array_parts: Vec<String> = selectors
                .iter()
                .map(|&element| element.to_string())
                .collect();
            self.mediator.debug_print(format!(
                "    selectors: {},",
                debug_format_array(&array_parts),
            ));
            self.mediator.debug_print(format!("}}\n\n",));
        }
        Ok(())
    }

    /// Reads the symbol stack from a bitstream.
    fn read_symbol_stack(
        &self,
        bitstream: &mut Bzip2Bitstream,
        symbol_stack: &mut [u8],
    ) -> io::Result<usize> {
        let level1_value: u32 = bitstream.get_value(16);
        let mut level1_bitmask: u32 = 0x00008000;
        let mut symbol_index: usize = 0;

        if self.mediator.debug_output {
            self.mediator.debug_print(format!("Bzip2SymbolStack {{\n",));
            self.mediator
                .debug_print(format!("    level1_value: 0x{:04x},\n", level1_value,));
            self.mediator.debug_print(format!("    level2_values: [",));
        }
        for level1_bit_index in 0..16 {
            if level1_value & level1_bitmask != 0 {
                let level2_value: u32 = bitstream.get_value(16);
                let mut level2_bitmask: u32 = 0x00008000;

                if self.mediator.debug_output {
                    if level1_bit_index > 0 {
                        self.mediator.debug_print(format!(", "));
                    }
                    self.mediator.debug_print(format!("0x{:04x}", level2_value));
                }
                for level2_bit_index in 0..16 {
                    if level2_value & level2_bitmask != 0 {
                        if symbol_index > 256 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Invalid symbol index: {} value out of bounds",
                                    symbol_index
                                ),
                            ));
                        }
                        symbol_stack[symbol_index] = (16 * level1_bit_index) + level2_bit_index;

                        symbol_index += 1;
                    }
                    level2_bitmask >>= 1;
                }
            }
            level1_bitmask >>= 1;
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("],\n",));
            // TODO: move to a debug_format_array like function.
            self.mediator
                .debug_print(format!("    symbols: [0x{:02x}", symbol_stack[0]));
            for symbol in &symbol_stack[1..symbol_index] {
                self.mediator.debug_print(format!(", 0x{:02x}", symbol));
            }
            self.mediator.debug_print(format!("],\n",));
            self.mediator.debug_print(format!("}}\n\n",));
        }
        Ok(symbol_index + 2)
    }

    /// Performs a Burrows-Wheeler transform
    fn reverse_burrows_wheeler_transform(
        &self,
        block_data: &[u8],
        block_data_size: usize,
        origin_pointer: u32,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_size: usize,
    ) -> io::Result<()> {
        let mut data_offset: usize = *uncompressed_data_offset;
        let mut distribution_value: usize = 0;
        let mut distributions: [usize; 256] = [0; 256];
        let mut last_byte_value: u8 = 0;
        let mut number_of_last_byte_values: u8 = 0;
        let mut permutations: [usize; BZIP2_BLOCK_SIZE] = [0; BZIP2_BLOCK_SIZE];

        for block_data_offset in 0..block_data_size {
            let byte_value: u8 = block_data[block_data_offset];
            distributions[byte_value as usize] += 1;
        }
        for byte_value in 0..256 {
            let number_of_values: usize = distributions[byte_value as usize];
            distributions[byte_value] = distribution_value;
            distribution_value += number_of_values;
        }
        for block_data_offset in 0..block_data_size {
            let byte_value: u8 = block_data[block_data_offset];
            distribution_value = distributions[byte_value as usize];
            permutations[distribution_value] = block_data_offset;
            distributions[byte_value as usize] += 1;
        }
        let mut permutation_value: usize = permutations[origin_pointer as usize];

        for _ in 0..block_data_size {
            let mut byte_value: u8 = block_data[permutation_value];

            if number_of_last_byte_values == 4 {
                if byte_value as usize > uncompressed_data_size - data_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid uncompressed data value too small",
                    ));
                }
                while byte_value > 0 {
                    uncompressed_data[data_offset] = last_byte_value;

                    data_offset += 1;
                    byte_value -= 1;
                }
                last_byte_value = 0;
                number_of_last_byte_values = 0;
            } else {
                if byte_value != last_byte_value {
                    number_of_last_byte_values = 0;
                }
                last_byte_value = byte_value;
                number_of_last_byte_values += 1;

                if data_offset >= uncompressed_data_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid uncompressed data value too small",
                    ));
                }
                uncompressed_data[data_offset] = byte_value;

                data_offset += 1;
            }
            permutation_value = permutations[permutation_value];
        }
        *uncompressed_data_offset = data_offset;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x42, 0x5a, 0x68, 0x31, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x5a, 0x55, 0xc4, 0x1e,
            0x00, 0x00, 0x0c, 0x5f, 0x80, 0x20, 0x00, 0x40, 0x84, 0x00, 0x00, 0x80, 0x20, 0x40,
            0x00, 0x2f, 0x6c, 0xdc, 0x80, 0x20, 0x00, 0x48, 0x4a, 0x9a, 0x4c, 0xd5, 0x53, 0xfc,
            0x69, 0xa5, 0x53, 0xff, 0x55, 0x3f, 0x69, 0x50, 0x15, 0x48, 0x95, 0x4f, 0xff, 0x55,
            0x51, 0xff, 0xaa, 0xa0, 0xff, 0xf5, 0x55, 0x31, 0xff, 0xaa, 0xa7, 0xfb, 0x4b, 0x34,
            0xc9, 0xb8, 0x38, 0xff, 0x16, 0x14, 0x56, 0x5a, 0xe2, 0x8b, 0x9d, 0x50, 0xb9, 0x00,
            0x81, 0x1a, 0x91, 0xfa, 0x25, 0x4f, 0x08, 0x5f, 0x4b, 0x5f, 0x53, 0x92, 0x4b, 0x11,
            0xc5, 0x22, 0x92, 0xd9, 0x50, 0x56, 0x6b, 0x6f, 0x9e, 0x17, 0x72, 0x45, 0x38, 0x50,
            0x90, 0x5a, 0x55, 0xc4, 0x1e,
        ];
    }

    #[test]
    fn test_bitstream_get_value() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);

        let test_value: u32 = test_bitstream.get_value(0);
        assert_eq!(test_value, 0);

        let test_value: u32 = test_bitstream.get_value(4);
        assert_eq!(test_value, 0x00000003);

        let test_value: u32 = test_bitstream.get_value(12);
        assert_eq!(test_value, 0x00000141);

        let test_value: u32 = test_bitstream.get_value(32);
        assert_eq!(test_value, 0x59265359);

        let mut test_bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);

        let test_value: u32 = test_bitstream.get_value(12);
        assert_eq!(test_value, 0x00000314);

        let test_value: u32 = test_bitstream.get_value(32);
        assert_eq!(test_value, 0x15926535);
    }

    #[test]
    fn test_bitstream_skip_bits() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);

        test_bitstream.skip_bits(4);
        let test_value: u32 = test_bitstream.get_value(12);
        assert_eq!(test_value, 0x00000141);
    }

    #[test]
    fn test_read_stream_header() -> io::Result<()> {
        let mut stream_header: Bzip2StreamHeader = Bzip2StreamHeader::new();

        let test_data: Vec<u8> = get_test_data();
        stream_header.read_data(&test_data)?;

        Ok(())
    }

    #[test]
    fn test_read_block_header() -> io::Result<()> {
        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        block_header.read_from_bitstream(&mut bitstream)?;

        assert_eq!(block_header.signature, 0x314159265359);
        assert_eq!(block_header.checksum, 0x5a55c41e);
        assert_eq!(block_header.randomized_flag, 0);
        assert_eq!(block_header.origin_pointer, 0x000018);

        Ok(())
    }

    #[test]
    fn test_read_symbol_stack() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let test_context: Bzip2Context = Bzip2Context::new();

        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();
        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        block_header.read_from_bitstream(&mut bitstream)?;

        let mut symbol_stack: [u8; 256] = [0; 256];
        let number_of_symbols: usize =
            test_context.read_symbol_stack(&mut bitstream, &mut symbol_stack)?;

        let expected_symbol_stack: [u8; 24] = [
            1, 32, 39, 44, 63, 73, 80, 97, 99, 100, 101, 102, 104, 105, 107, 108, 111, 112, 114,
            115, 116, 119, 0, 0,
        ];
        assert_eq!(number_of_symbols, 24);
        assert_eq!(&symbol_stack[0..24], &expected_symbol_stack);

        Ok(())
    }

    #[test]
    fn test_read_selectors() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let test_context: Bzip2Context = Bzip2Context::new();

        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();
        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        block_header.read_from_bitstream(&mut bitstream)?;

        let mut symbol_stack: [u8; 256] = [0; 256];
        test_context.read_symbol_stack(&mut bitstream, &mut symbol_stack)?;

        let number_of_trees: u32 = bitstream.get_value(3);
        let number_of_selectors: u32 = bitstream.get_value(15);

        let mut selectors: [u8; 32769] = [0; 32769];
        test_context.read_selectors(
            &mut bitstream,
            &mut selectors,
            number_of_selectors as usize,
            number_of_trees as usize,
        )?;
        let expected_selectors: [u8; 2] = [0, 1];
        assert_eq!(&selectors[0..2], &expected_selectors);

        Ok(())
    }

    #[test]
    fn test_read_huffman_tree() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let test_context: Bzip2Context = Bzip2Context::new();

        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();
        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        block_header.read_from_bitstream(&mut bitstream)?;

        let mut symbol_stack: [u8; 256] = [0; 256];
        let number_of_symbols: usize =
            test_context.read_symbol_stack(&mut bitstream, &mut symbol_stack)?;
        let number_of_trees: u32 = bitstream.get_value(3);
        let number_of_selectors: u32 = bitstream.get_value(15);

        let mut selectors: [u8; 32769] = [0; 32769];
        test_context.read_selectors(
            &mut bitstream,
            &mut selectors,
            number_of_selectors as usize,
            number_of_trees as usize,
        )?;

        let mut huffman_tree: HuffmanTree = HuffmanTree::new(number_of_symbols, 20);
        test_context.read_huffman_tree(&mut bitstream, &mut huffman_tree, number_of_symbols)?;

        Ok(())
    }

    #[test]
    fn test_read_block_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let test_context: Bzip2Context = Bzip2Context::new();

        let mut block_header: Bzip2BlockHeader = Bzip2BlockHeader::new();
        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        block_header.read_from_bitstream(&mut bitstream)?;

        let mut symbol_stack: [u8; 256] = [0; 256];
        let number_of_symbols: usize =
            test_context.read_symbol_stack(&mut bitstream, &mut symbol_stack)?;
        let number_of_trees: u32 = bitstream.get_value(3);
        let number_of_selectors: u32 = bitstream.get_value(15);

        let mut selectors: [u8; 32769] = [0; 32769];
        test_context.read_selectors(
            &mut bitstream,
            &mut selectors,
            number_of_selectors as usize,
            number_of_trees as usize,
        )?;
        let mut huffman_trees: Vec<HuffmanTree> = Vec::new();

        for _ in 0..number_of_trees {
            let mut huffman_tree: HuffmanTree = HuffmanTree::new(number_of_symbols, 20);

            test_context.read_huffman_tree(&mut bitstream, &mut huffman_tree, number_of_symbols)?;

            huffman_trees.push(huffman_tree);
        }
        let mut block_data: [u8; BZIP2_BLOCK_SIZE] = [0; BZIP2_BLOCK_SIZE];
        let block_data_size: usize = test_context.read_block_data(
            &mut bitstream,
            &huffman_trees,
            number_of_trees as usize,
            &selectors,
            number_of_selectors as usize,
            &mut symbol_stack,
            number_of_symbols,
            &mut block_data,
        )?;
        let expected_block_data: [u8; 108] = [
            0x3f, 0x66, 0x73, 0x72, 0x72, 0x64, 0x6b, 0x6b, 0x65, 0x61, 0x64, 0x64, 0x72, 0x72,
            0x66, 0x66, 0x73, 0x2c, 0x65, 0x73, 0x3f, 0x3f, 0x3f, 0x64, 0x01, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x65, 0x65, 0x69, 0x69, 0x69, 0x69, 0x65, 0x65, 0x65, 0x65, 0x68, 0x72,
            0x70, 0x70, 0x6b, 0x6c, 0x6c, 0x6b, 0x70, 0x70, 0x74, 0x74, 0x70, 0x70, 0x68, 0x70,
            0x70, 0x50, 0x50, 0x49, 0x6f, 0x6f, 0x74, 0x77, 0x70, 0x70, 0x70, 0x70, 0x50, 0x50,
            0x63, 0x63, 0x63, 0x63, 0x63, 0x63, 0x6b, 0x6b, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x69, 0x69, 0x70, 0x70, 0x20, 0x20, 0x20, 0x20, 0x65, 0x65, 0x65, 0x65, 0x65, 0x65,
            0x65, 0x65, 0x65, 0x72, 0x27, 0x72, 0x65, 0x65, 0x20, 0x20,
        ];
        assert_eq!(block_data_size, 108);
        assert_eq!(&block_data[0..108], &expected_block_data);

        Ok(())
    }

    #[test]
    fn test_reverse_burrows_wheeler_transform() -> io::Result<()> {
        let test_context: Bzip2Context = Bzip2Context::new();

        let block_data: [u8; 35] = [
            0x73, 0x73, 0x65, 0x65, 0x79, 0x65, 0x65, 0x20, 0x68, 0x68, 0x73, 0x73, 0x68, 0x73,
            0x72, 0x74, 0x73, 0x73, 0x73, 0x65, 0x65, 0x6c, 0x6c, 0x68, 0x6f, 0x6c, 0x6c, 0x20,
            0x20, 0x20, 0x65, 0x61, 0x61, 0x20, 0x62,
        ];
        let mut uncompressed_data: [u8; 35] = [0; 35];
        let mut uncompressed_data_offset: usize = 0;
        test_context.reverse_burrows_wheeler_transform(
            &block_data,
            35,
            30,
            &mut uncompressed_data,
            &mut uncompressed_data_offset,
            35,
        )?;
        let expected_uncompressed_data: [u8; 35] = [
            0x73, 0x68, 0x65, 0x20, 0x73, 0x65, 0x6c, 0x6c, 0x73, 0x20, 0x73, 0x65, 0x61, 0x73,
            0x68, 0x65, 0x6c, 0x6c, 0x73, 0x20, 0x62, 0x79, 0x20, 0x74, 0x68, 0x65, 0x20, 0x73,
            0x65, 0x61, 0x73, 0x68, 0x6f, 0x72, 0x65,
        ];
        assert_eq!(uncompressed_data_offset, 35);
        assert_eq!(&uncompressed_data, &expected_uncompressed_data);

        Ok(())
    }

    #[test]
    fn test_decompress_bitstream() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_context: Bzip2Context = Bzip2Context::new();

        let mut bitstream: Bzip2Bitstream = Bzip2Bitstream::new(&test_data, 4);
        let mut uncompressed_data: Vec<u8> = vec![0; 512];
        test_context.decompress_bitstream(&mut bitstream, &mut uncompressed_data)?;

        let expected_uncompressed_data: [u8; 108] = [
            0x49, 0x66, 0x20, 0x50, 0x65, 0x74, 0x65, 0x72, 0x20, 0x50, 0x69, 0x70, 0x65, 0x72,
            0x20, 0x70, 0x69, 0x63, 0x6b, 0x65, 0x64, 0x20, 0x61, 0x20, 0x70, 0x65, 0x63, 0x6b,
            0x20, 0x6f, 0x66, 0x20, 0x70, 0x69, 0x63, 0x6b, 0x6c, 0x65, 0x64, 0x20, 0x70, 0x65,
            0x70, 0x70, 0x65, 0x72, 0x73, 0x2c, 0x20, 0x77, 0x68, 0x65, 0x72, 0x65, 0x27, 0x73,
            0x20, 0x74, 0x68, 0x65, 0x20, 0x70, 0x65, 0x63, 0x6b, 0x20, 0x6f, 0x66, 0x20, 0x70,
            0x69, 0x63, 0x6b, 0x6c, 0x65, 0x64, 0x20, 0x70, 0x65, 0x70, 0x70, 0x65, 0x72, 0x73,
            0x20, 0x50, 0x65, 0x74, 0x65, 0x72, 0x20, 0x50, 0x69, 0x70, 0x65, 0x72, 0x20, 0x70,
            0x69, 0x63, 0x6b, 0x65, 0x64, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f,
        ];
        assert_eq!(
            &uncompressed_data[0..test_context.uncompressed_data_size],
            expected_uncompressed_data
        );

        Ok(())
    }

    #[test]
    fn test1_decompress() -> io::Result<()> {
        let mut test_context: Bzip2Context = Bzip2Context::new();

        let test_data: Vec<u8> = get_test_data();
        let mut uncompressed_data: Vec<u8> = vec![0; 512];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, 108);

        let expected_uncompressed_data: [u8; 108] = [
            0x49, 0x66, 0x20, 0x50, 0x65, 0x74, 0x65, 0x72, 0x20, 0x50, 0x69, 0x70, 0x65, 0x72,
            0x20, 0x70, 0x69, 0x63, 0x6b, 0x65, 0x64, 0x20, 0x61, 0x20, 0x70, 0x65, 0x63, 0x6b,
            0x20, 0x6f, 0x66, 0x20, 0x70, 0x69, 0x63, 0x6b, 0x6c, 0x65, 0x64, 0x20, 0x70, 0x65,
            0x70, 0x70, 0x65, 0x72, 0x73, 0x2c, 0x20, 0x77, 0x68, 0x65, 0x72, 0x65, 0x27, 0x73,
            0x20, 0x74, 0x68, 0x65, 0x20, 0x70, 0x65, 0x63, 0x6b, 0x20, 0x6f, 0x66, 0x20, 0x70,
            0x69, 0x63, 0x6b, 0x6c, 0x65, 0x64, 0x20, 0x70, 0x65, 0x70, 0x70, 0x65, 0x72, 0x73,
            0x20, 0x50, 0x65, 0x74, 0x65, 0x72, 0x20, 0x50, 0x69, 0x70, 0x65, 0x72, 0x20, 0x70,
            0x69, 0x63, 0x6b, 0x65, 0x64, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f,
        ];
        assert_eq!(
            &uncompressed_data[0..test_context.uncompressed_data_size],
            expected_uncompressed_data
        );
        Ok(())
    }

    #[test]
    fn test2_decompress() -> io::Result<()> {
        let mut test_context: Bzip2Context = Bzip2Context::new();

        let test_data: [u8; 122] = [
            0x42, 0x5a, 0x68, 0x31, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0xef, 0x2d, 0xfa, 0x16,
            0x00, 0x00, 0x21, 0xfe, 0x57, 0xf8, 0x00, 0x00, 0xc2, 0xda, 0x00, 0x00, 0x30, 0x23,
            0x30, 0x54, 0x04, 0x49, 0x89, 0x68, 0x40, 0x05, 0x00, 0x01, 0x01, 0x00, 0x40, 0x00,
            0x09, 0xa0, 0x00, 0x54, 0x61, 0xa1, 0xa3, 0x26, 0x20, 0xc2, 0x1a, 0x06, 0x20, 0xf2,
            0x83, 0x45, 0x06, 0x80, 0x1a, 0x00, 0xd1, 0xa1, 0x90, 0xc8, 0x20, 0xe4, 0x11, 0x4d,
            0x1b, 0xf8, 0x40, 0x2d, 0x15, 0x01, 0x98, 0x51, 0x82, 0x01, 0x06, 0x0b, 0x63, 0x21,
            0xd1, 0xad, 0xa9, 0xf9, 0xeb, 0x4b, 0xb3, 0xc9, 0xac, 0xf1, 0xcc, 0x68, 0xf3, 0x2f,
            0x19, 0x0a, 0x3e, 0x96, 0x3e, 0x82, 0x0a, 0x03, 0xa8, 0x0a, 0x0b, 0x35, 0x44, 0xfc,
            0x5d, 0xc9, 0x14, 0xe1, 0x42, 0x43, 0xbc, 0xb7, 0xe8, 0x58,
        ];
        let mut uncompressed_data: Vec<u8> = vec![0; 512];
        test_context.decompress(&test_data, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, 512);

        let expected_uncompressed_data: [u8; 512] = [
            0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54, 0x00, 0x00, 0x01, 0x00, 0x5c, 0x00,
            0x00, 0x00, 0x17, 0xc4, 0x17, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x22, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xde, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x87, 0x3a, 0xd6, 0x8e, 0x83, 0x17, 0xfe, 0x4a, 0x8b, 0xa3, 0x18, 0x39, 0xc6, 0x23,
            0x25, 0x86, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x80, 0x00, 0x00, 0x00, 0xe8, 0xa8, 0x45, 0xa0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            &uncompressed_data[0..test_context.uncompressed_data_size],
            expected_uncompressed_data
        );

        Ok(())
    }
}
