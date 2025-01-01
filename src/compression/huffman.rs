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

//! Huffman tree.
//!
//! Provides Huffman tree support.

use std::io;

use crate::mediator::{Mediator, MediatorReference};

use super::traits::Bitstream;

/// Huffman tree.
pub struct HuffmanTree {
    /// Mediator.
    mediator: MediatorReference,

    /// Largest code size.
    largest_code_size: usize,

    /// Maximum code size (largest code size + 1).
    maximum_code_size: usize,

    /// Symbols.
    symbols: Vec<u16>,

    /// Code size counts.
    code_size_counts: Vec<isize>,
}

impl HuffmanTree {
    /// Creates a Huffman tree.
    pub fn new(number_of_symbols: usize, largest_code_size: usize) -> Self {
        let maximum_code_size: usize = largest_code_size + 1;
        Self {
            mediator: Mediator::current(),
            largest_code_size: largest_code_size,
            maximum_code_size: maximum_code_size,
            symbols: vec![0; number_of_symbols],
            code_size_counts: vec![0; maximum_code_size],
        }
    }

    /// Builds the Huffman tree from code sizes.
    pub fn build(&mut self, code_sizes: &[u8]) -> io::Result<()> {
        let number_of_code_sizes: usize = code_sizes.len();

        if number_of_code_sizes > u16::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Invalid number of code sizes: {} value out of bounds",
                    number_of_code_sizes
                ),
            ));
        }
        // Determine the code size frequencies.
        self.code_size_counts.fill(0);

        for symbol in 0..number_of_code_sizes {
            let code_size: usize = code_sizes[symbol] as usize;

            if code_size > self.largest_code_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Invalid code size: {} value out of bounds: 0 - {}",
                        code_size, self.largest_code_size
                    ),
                ));
            }
            self.code_size_counts[code_size] += 1;
        }
        if self.code_size_counts[0] == number_of_code_sizes as isize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Huffman tree has no codes",
            ));
        }
        // Check if the set of code sizes is incomplete or over-subscribed
        let mut left_value: isize = 1;

        for bit_index in 1..self.maximum_code_size {
            left_value = (left_value << 1) - self.code_size_counts[bit_index];

            if left_value < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Code sizes are over-subscribed",
                ));
            }
        }
        /* TODO
        if left_value > 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Code sizes are incomplete",
            ));
        }
        */
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("HuffmanTree {{\n",));
        }
        // Calculate the offsets to sort the symbols per code size.
        let mut symbol_offsets: Vec<isize> = Vec::new();

        symbol_offsets.push(0);
        symbol_offsets.push(0);

        for bit_index in 1..self.largest_code_size {
            let symbol_offset: isize = symbol_offsets[bit_index] + self.code_size_counts[bit_index];

            symbol_offsets.push(symbol_offset);
        }
        // Fill the symbols sorted by code size.
        for symbol in 0..number_of_code_sizes {
            let code_size: usize = code_sizes[symbol] as usize;

            if code_size == 0 {
                continue;
            }
            let code_offset: isize = symbol_offsets[code_size];

            if code_offset < 0 || code_offset > number_of_code_sizes as isize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Invalid symbol: {} code offset: {} value out bounds",
                        symbol, code_offset
                    ),
                ));
            }
            symbol_offsets[code_size] += 1;

            self.symbols[code_offset as usize] = symbol as u16;

            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "    symbol: {}, code_size: {},\n",
                    symbol, code_size
                ));
            }
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("}}\n\n",));
        }
        Ok(())
    }

    /// Decodes a symbol from a bitstream.
    pub fn decode_symbol(&self, bitstream: &mut dyn Bitstream) -> io::Result<u16> {
        let mut first_huffman_code: isize = 0;
        let mut huffman_code: isize = 0;
        let mut first_index: isize = 0;

        for bit_index in 1..self.largest_code_size {
            huffman_code = (huffman_code << 1) | (bitstream.get_value(1) as isize);

            let code_size_count: isize = self.code_size_counts[bit_index];

            if (huffman_code - code_size_count) < first_huffman_code {
                let symbol_index: usize =
                    (first_index + (huffman_code - first_huffman_code)) as usize;

                return Ok(self.symbols[symbol_index]);
            }
            first_huffman_code += code_size_count;
            first_huffman_code <<= 1;
            first_index += code_size_count;
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid Huffman code: 0x{:x}", huffman_code),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::compression::deflate::DeflateBitstream;

    fn get_code_sizes() -> Vec<u8> {
        let mut code_sizes: Vec<u8> = vec![0; 318];

        for symbol in 0..318 {
            if symbol < 144 {
                code_sizes[symbol] = 8;
            } else if symbol < 256 {
                code_sizes[symbol] = 9;
            } else if symbol < 280 {
                code_sizes[symbol] = 7;
            } else if symbol < 288 {
                code_sizes[symbol] = 8;
            } else {
                code_sizes[symbol] = 5;
            }
        }
        code_sizes
    }

    #[test]
    fn test_huffman_tree_build() -> io::Result<()> {
        let code_sizes: Vec<u8> = get_code_sizes();

        let mut test_huffman_tree: HuffmanTree = HuffmanTree::new(288, 15);

        test_huffman_tree.build(&code_sizes[0..288])?;

        Ok(())
    }

    #[test]
    fn test_huffman_tree_decode_symbol() -> io::Result<()> {
        let code_sizes: Vec<u8> = get_code_sizes();

        let mut test_huffman_tree: HuffmanTree = HuffmanTree::new(288, 15);

        test_huffman_tree.build(&code_sizes[0..288])?;

        let test_data: [u8; 16] = [
            0x78, 0xda, 0xbd, 0x59, 0x6d, 0x8f, 0xdb, 0xb8, 0x11, 0xfe, 0x7c, 0xfa, 0x15, 0xc4,
            0x7e, 0xb9,
        ];
        let mut test_bitstream: DeflateBitstream = DeflateBitstream::new(&test_data, 2);

        let test_symbol: u16 = test_huffman_tree.decode_symbol(&mut test_bitstream)?;
        assert_eq!(test_symbol, 141);

        Ok(())
    }
}
