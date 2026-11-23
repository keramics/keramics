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

//! LZMA decompression.
//!
//! Provides decompression support for LZMA (LZMA1) compressed data.

use std::cmp::min;

use keramics_core::{DebugTrace, DebugTraceScope, ErrorTrace};
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_be, bytes_to_u32_le, bytes_to_u64_le};

/// Maximum value of the LZMA properties byte ((4 * 5 + 4) * 9 + 8).
const LZMA_MAXIMUM_PROPERTIES: u8 = 224;

/// Minimum LZMA dictionary size.
const LZMA_MINIMUM_DICTIONARY_SIZE: usize = 1 << 12;

/// Value of the uncompressed data size used to indicate that the uncompressed data size is unknown.
const LZMA_UNCOMPRESSED_SIZE_UNKNOWN: u64 = u64::MAX;

/// LZMA end of payload marker (EOPM) match distance value.
const LZMA_END_OF_PAYLOAD_MARKER: u32 = 0xffffffff;

/// Number of LZMA decoder states.
const LZMA_NUMBER_OF_STATES: usize = 12;

/// Number of LZMA literal states (states 0 to 6 indicate that the previous state was a literal).
const LZMA_NUMBER_OF_LITERAL_STATES: usize = 7;

/// Number of LZMA position states (1 << 4).
const LZMA_NUMBER_OF_POSITION_STATES: usize = 16;

/// Number of LZMA distance slot states.
const LZMA_NUMBER_OF_DISTANCE_SLOT_STATES: usize = 4;

/// Number of LZMA distance slots (1 << 6).
const LZMA_NUMBER_OF_DISTANCE_SLOTS: usize = 64;

/// First LZMA distance slot that uses the distance low bits table.
const LZMA_DISTANCE_SLOT_LOW_BITS: usize = 4;

/// First LZMA distance slot that uses plain and alignment bits.
const LZMA_DISTANCE_SLOT_PLAIN_BITS: usize = 14;

/// Number of LZMA distance alignment bits.
const LZMA_NUMBER_OF_DISTANCE_ALIGNMENT_BITS: usize = 4;

/// Minimum LZMA match length.
const LZMA_MINIMUM_MATCH_LENGTH: usize = 2;

/// Number of LZMA match length low symbols (1 << 3).
const LZMA_MATCH_LENGTH_LOW_SYMBOLS: usize = 8;

/// Number of LZMA match length middle symbols (1 << 3).
const LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS: usize = 8;

/// Number of LZMA match length high symbols (1 << 8).
const LZMA_MATCH_LENGTH_HIGH_SYMBOLS: usize = 256;

/// Number of probability cells used by a match length decoder.
///
/// Consists of: choice (1), choice2 (1), low (16 * 8), middle (16 * 8) and high (256) cells.
const LZMA_MATCH_LENGTH_NUMBER_OF_CELLS: usize = 2
    + (LZMA_NUMBER_OF_POSITION_STATES * LZMA_MATCH_LENGTH_LOW_SYMBOLS)
    + (LZMA_NUMBER_OF_POSITION_STATES * LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS)
    + LZMA_MATCH_LENGTH_HIGH_SYMBOLS;

/// Number of LZMA probability bit model total bits.
const LZMA_PROBABILITY_TOTAL_BITS: u32 = 11;

/// Number of LZMA probability bit model bits.
const LZMA_PROBABILITY_TOTAL: u16 = 1 << LZMA_PROBABILITY_TOTAL_BITS;

/// Initial value of an LZMA probability bit model.
const LZMA_PROBABILITY_INITIAL_VALUE: u16 = LZMA_PROBABILITY_TOTAL >> 1;

/// Number of bits used when normalizing an LZMA probability bit model.
const LZMA_PROBABILITY_MOVE_BITS: u32 = 5;

/// Bitstream of an LZMA range decoder.
struct LzmaRangeDecoder<'a> {
    /// Byte stream.
    data: &'a [u8],

    /// Current offset in the byte stream.
    pub data_offset: usize,

    /// Size of the byte stream in bytes.
    pub data_size: usize,

    /// Code value.
    code: u32,

    /// Range value.
    range: u32,
}

impl<'a> LzmaRangeDecoder<'a> {
    /// Creates a new range decoder.
    pub fn new(data: &'a [u8], data_offset: usize) -> Self {
        let data_size: usize = data.len();
        Self {
            data,
            data_offset,
            data_size,
            code: 0,
            range: 0xffffffff,
        }
    }

    /// Determines whether the end of the chunk was reached.
    pub fn check_for_end_of_chunk(
        &mut self,
        allow_end_of_payload_marker: bool,
    ) -> Result<bool, ErrorTrace> {
        let finished: bool = match self.is_finished() {
            Ok(finished) => finished,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine if range decoder is finished"
                );
                return Err(error);
            }
        };
        if !finished {
            if !allow_end_of_payload_marker {
                return Err(keramics_core::error_trace_new!(
                    "Invalid end of chunk, incomplete range decoder stream"
                ));
            }
            return Ok(false);
        }
        // When the end of payload marker (EOPM) is disallowed, i.e. for an
        // LZMA2 chunk, the chunk must have consumed exactly its compressed data.
        if !allow_end_of_payload_marker && self.data_offset != self.data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid compressed data value too large"
            ));
        }
        Ok(true)
    }

    /// Decodes a bit using a probability bit model.
    pub fn decode_bit(&mut self, probability: &mut u16) -> Result<u32, ErrorTrace> {
        match self.normalize() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to normalize");
                return Err(error);
            }
        }
        let bound: u32 = (self.range >> LZMA_PROBABILITY_TOTAL_BITS) * (*probability as u32);

        if self.code < bound {
            self.range = bound;
            *probability += (LZMA_PROBABILITY_TOTAL - *probability) >> LZMA_PROBABILITY_MOVE_BITS;

            Ok(0)
        } else {
            self.range -= bound;
            self.code -= bound;
            *probability -= *probability >> LZMA_PROBABILITY_MOVE_BITS;

            Ok(1)
        }
    }

    /// Decodes a bit without using a probability bit model.
    pub fn decode_bit_direct(&mut self) -> Result<u32, ErrorTrace> {
        match self.normalize() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to normalize");
                return Err(error);
            }
        }
        self.range >>= 1;

        if self.code >= self.range {
            self.code -= self.range;

            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Decodes a match length (2 to 273) using a match length decoder.
    pub fn decode_length(
        &mut self,
        position_state: usize,
        table: &mut [u16],
    ) -> Result<usize, ErrorTrace> {
        let base_index: usize;
        let limit: usize;
        let offset: usize;

        let bit_value: u32 = match self.decode_bit(&mut table[0]) {
            Ok(value) => value,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode bit value");
                return Err(error);
            }
        };
        if bit_value == 0 {
            // 2 + low value (0 to 7).
            base_index = 2 + (position_state * LZMA_MATCH_LENGTH_LOW_SYMBOLS);
            limit = LZMA_MATCH_LENGTH_LOW_SYMBOLS;
            offset = 0;
        } else {
            let bit_value: u32 = match self.decode_bit(&mut table[1]) {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit value");
                    return Err(error);
                }
            };
            if bit_value == 0 {
                // 2 + 8 + middle value (0 to 7).
                base_index = 2
                    + (LZMA_NUMBER_OF_POSITION_STATES * LZMA_MATCH_LENGTH_LOW_SYMBOLS)
                    + (position_state * LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS);
                limit = LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS;
                offset = LZMA_MATCH_LENGTH_LOW_SYMBOLS;
            } else {
                // 2 + 8 + 8 + high value (0 to 255).
                base_index = 2
                    + (LZMA_NUMBER_OF_POSITION_STATES * LZMA_MATCH_LENGTH_LOW_SYMBOLS)
                    + (LZMA_NUMBER_OF_POSITION_STATES * LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS);
                limit = LZMA_MATCH_LENGTH_HIGH_SYMBOLS;
                offset = LZMA_MATCH_LENGTH_LOW_SYMBOLS + LZMA_MATCH_LENGTH_MIDDLE_SYMBOLS;
            }
        }
        let mut symbol: usize = 1;

        while symbol < limit {
            let table_index: usize = base_index + symbol;

            let bit_value: u32 = match self.decode_bit(&mut table[table_index]) {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit value");
                    return Err(error);
                }
            };
            symbol = (symbol << 1) + (bit_value as usize);
        }
        Ok(LZMA_MINIMUM_MATCH_LENGTH + (symbol - limit) + offset)
    }

    /// Normalizes the range decoder and determines whether it is at the end of the bitstream.
    pub fn is_finished(&mut self) -> Result<bool, ErrorTrace> {
        match self.normalize() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to normalize");
                return Err(error);
            }
        }
        Ok(self.code == 0)
    }

    /// Normalizes the range and code values.
    pub fn normalize(&mut self) -> Result<(), ErrorTrace> {
        // 1 << 24 = 16777216.
        if self.range < 16777216 {
            if self.data_offset >= self.data_size {
                return Err(keramics_core::error_trace_new!(
                    "Invalid compressed data value too small"
                ));
            }
            self.range <<= 8;
            self.code = (self.code << 8) | (self.data[self.data_offset] as u32);
            self.data_offset += 1;
        }
        Ok(())
    }

    /// Reads the initialization bytes.
    pub fn read_initialization_bytes(&mut self) -> Result<(), ErrorTrace> {
        if self.data_offset + 5 > self.data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid compressed data value too small"
            ));
        }
        let initialization_byte: u8 = self.data[self.data_offset];

        if initialization_byte != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported range decoder initialization byte: 0x{:02x}",
                initialization_byte
            )));
        }
        self.code = bytes_to_u32_be!(self.data, self.data_offset + 1);
        self.data_offset += 5;

        Ok(())
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "properties", data_type = "u8", format = "hex"),
        field(name = "dictionary_size", data_type = "u32"),
        field(name = "uncompressed_data_size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Stream header used by LZMA compressed data.
struct LzmaStreamHeader {
    /// Number of literal context bits (lc).
    literal_context_bits: u8,

    /// Number of literal position bits (lp).
    literal_position_bits: u8,

    /// Number of position bits (pb).
    position_bits: u8,

    /// Dictionary size.
    dictionary_size: usize,

    /// Uncompressed data size.
    uncompressed_data_size: u64,
}

impl LzmaStreamHeader {
    /// Creates a new stream header.
    pub fn new() -> Self {
        Self {
            literal_context_bits: 0,
            literal_position_bits: 0,
            position_bits: 0,
            dictionary_size: 0,
            uncompressed_data_size: 0,
        }
    }

    /// Reads the stream header.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 13 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let properties: u8 = data[0];

        let (literal_context_bits, literal_position_bits, position_bits): (u8, u8, u8) =
            match LzmaDecoder::decode_properties(properties) {
                Ok(properties) => properties,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode properties");
                    return Err(error);
                }
            };
        self.literal_context_bits = literal_context_bits;
        self.literal_position_bits = literal_position_bits;
        self.position_bits = position_bits;

        let mut dictionary_size: usize = bytes_to_u32_le!(data, 1) as usize;

        if dictionary_size < LZMA_MINIMUM_DICTIONARY_SIZE {
            dictionary_size = LZMA_MINIMUM_DICTIONARY_SIZE;
        }
        self.dictionary_size = dictionary_size;
        self.uncompressed_data_size = bytes_to_u64_le!(data, 5);

        Ok(())
    }
}

/// Decoder for decompressing LZMA compressed data.
pub(super) struct LzmaDecoder {
    /// Number of literal context bits (lc).
    literal_context_bits: u8,

    /// Number of literal position bits (lp).
    literal_position_bits: u8,

    /// Number of position bits (pb).
    position_bits: u8,

    /// Decoder state (0 to 11).
    state: usize,

    /// Repeated match distances history (zero-based, rep0 to rep3).
    repeated_distances: [usize; 4],

    /// Last decoded byte value, or zero if no bytes have been decoded yet.
    previous_byte: u8,

    /// Probabilities whether a literal or match is decoded next (state * 16 + position_state).
    match_probabilities: Vec<u16>,

    /// Probabilities of a repeated match (state).
    repeated_match_probabilities: Vec<u16>,

    /// Probabilities that the repeated match distance is rep0 (state).
    repeated_match_distance0_probabilities: Vec<u16>,

    /// Probabilities that the repeated match distance is rep1 (state).
    repeated_match_distance1_probabilities: Vec<u16>,

    /// Probabilities that the repeated match distance is rep2 (state).
    repeated_match_distance2_probabilities: Vec<u16>,

    /// Probabilities of a long repeated match at rep0 (state * 16 + position_state).
    repeated_match_distance0_long_probabilities: Vec<u16>,

    /// Probabilities of the literal symbols (coder * 0x300 + subcoder).
    literal_probabilities: Vec<u16>,

    /// Probabilities of the match length decoder.
    ///
    /// Consists of: choice (cell 0), choice2 (cell 1),
    /// low (cells 2 + position_state * 8), middle (cells 2 + 128 + position_state * 8)
    /// and high (cells 258).
    match_length_probabilities: Vec<u16>,

    /// Probabilities of the repeated match length decoder (same layout as match_length_probabilities).
    repeated_match_length_probabilities: Vec<u16>,

    /// Probabilities of the distance slot (length_state * 64 + slot).
    distance_slot_probabilities: Vec<u16>,

    /// Probabilities of the distance low bits (position special decoders).
    distance_low_bit_probabilities: Vec<u16>,

    /// Probabilities of the distance alignment bits.
    distance_alignment_probabilities: Vec<u16>,
}

impl LzmaDecoder {
    /// Creates a new decoder with the specified properties.
    pub fn new(
        literal_context_bits: u8,
        literal_position_bits: u8,
        position_bits: u8,
    ) -> Result<Self, ErrorTrace> {
        if (literal_context_bits as u32) + (literal_position_bits as u32) > 4
            || (position_bits as u32) > 4
        {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported LZMA properties value: lc: {}, lp: {}, pb: {}",
                literal_context_bits, literal_position_bits, position_bits
            )));
        }
        let number_of_literal_states: usize = 1 << (literal_context_bits + literal_position_bits);

        Ok(Self {
            literal_context_bits,
            literal_position_bits,
            position_bits,
            state: 0,
            repeated_distances: [0; 4],
            previous_byte: 0,
            match_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES * LZMA_NUMBER_OF_POSITION_STATES
            ],
            repeated_match_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES
            ],
            repeated_match_distance0_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES
            ],
            repeated_match_distance1_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES
            ],
            repeated_match_distance2_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES
            ],
            repeated_match_distance0_long_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_STATES
                    * LZMA_NUMBER_OF_POSITION_STATES
            ],
            literal_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                number_of_literal_states * 0x300
            ],
            match_length_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_MATCH_LENGTH_NUMBER_OF_CELLS
            ],
            repeated_match_length_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_MATCH_LENGTH_NUMBER_OF_CELLS
            ],
            distance_slot_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                LZMA_NUMBER_OF_DISTANCE_SLOT_STATES
                    * LZMA_NUMBER_OF_DISTANCE_SLOTS
            ],
            // Note that 114 probability cells (indexes 0 to 113) are used by the
            // distance low bits (position special) decoders for slots 4 to 13.
            distance_low_bit_probabilities: vec![LZMA_PROBABILITY_INITIAL_VALUE; 114],
            distance_alignment_probabilities: vec![
                LZMA_PROBABILITY_INITIAL_VALUE;
                1 << LZMA_NUMBER_OF_DISTANCE_ALIGNMENT_BITS
            ],
        })
    }

    /// Decodes a match distance using plain bits and the distance alignment decoder.
    fn decode_distance_high_bits(
        &mut self,
        range_decoder: &mut LzmaRangeDecoder,
        slot: usize,
    ) -> Result<usize, ErrorTrace> {
        let number_of_bits: usize = (slot / 2) - 1;
        let number_of_plain_bits: usize = number_of_bits - LZMA_NUMBER_OF_DISTANCE_ALIGNMENT_BITS;
        let mut value: usize = 2 | (slot & 1);

        for _ in 0..number_of_plain_bits {
            let bit_value: u32 = match range_decoder.decode_bit_direct() {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                    return Err(error);
                }
            };
            value = (value << 1) | bit_value as usize;
        }
        value <<= LZMA_NUMBER_OF_DISTANCE_ALIGNMENT_BITS;
        let mut table_symbol: usize = 1;
        let mut bit_offset: usize = 0;

        // Decode the distance low 4 bits using the alignment decoder.
        while bit_offset < LZMA_NUMBER_OF_DISTANCE_ALIGNMENT_BITS {
            let bit_value: u32 = match range_decoder
                .decode_bit(&mut self.distance_alignment_probabilities[table_symbol])
            {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                    return Err(error);
                }
            };
            if bit_value == 1 {
                value |= 1 << bit_offset;
            }
            table_symbol = (table_symbol << 1) + (bit_value as usize);
            bit_offset += 1;
        }
        Ok(value)
    }

    /// Decodes a match distance using the position special (distance low bits) decoders.
    fn decode_distance_low_bits(
        &mut self,
        range_decoder: &mut LzmaRangeDecoder,
        slot: usize,
    ) -> Result<usize, ErrorTrace> {
        let number_of_bits: usize = (slot / 2) - 1;
        let mut value: usize = (2 | (slot & 1)) << number_of_bits;
        // Note that the first distance low bits cell index is (value - slot).
        let table_base: usize = value - slot;
        let mut table_symbol: usize = 1;
        let mut bit_offset: usize = 0;

        while bit_offset < number_of_bits {
            let table_index: usize = table_base + table_symbol - 1;

            let bit_value: u32 = match range_decoder
                .decode_bit(&mut self.distance_low_bit_probabilities[table_index])
            {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                    return Err(error);
                }
            };
            if bit_value == 1 {
                value += 1 << bit_offset;
            }
            table_symbol = (table_symbol << 1) + (bit_value as usize);
            bit_offset += 1;
        }
        Ok(value)
    }

    /// Decodes a literal and copies the byte value to the uncompressed data.
    fn decode_literal(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        range_decoder: &mut LzmaRangeDecoder,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_end_offset: usize,
        literal_position_mask: usize,
        dictionary_start_offset: usize,
    ) -> Result<(), ErrorTrace> {
        let subcoder: usize = (((*uncompressed_data_offset & literal_position_mask)
            << self.literal_context_bits)
            + (self.previous_byte as usize >> (8 - self.literal_context_bits)))
            * 0x300;
        let mut match_byte: u8 = if self.state >= LZMA_NUMBER_OF_LITERAL_STATES
            && *uncompressed_data_offset - dictionary_start_offset > self.repeated_distances[0]
        {
            uncompressed_data[*uncompressed_data_offset - 1 - self.repeated_distances[0]]
        } else {
            0
        };
        let mut matched: bool = self.state >= LZMA_NUMBER_OF_LITERAL_STATES;
        let mut symbol: usize = 1;

        while symbol < 0x100 {
            let literal_bit: u32 = if matched {
                // Use the subcoder for the matched byte bit.
                // Note that the subcoder index of the matched literal cells is
                // 0x100 + (match_bit * 0x100) + symbol.
                let match_bit: u8 = (match_byte >> 7) & 1;
                let literal_index: usize = subcoder + ((1 + match_bit as usize) * 0x100) + symbol;
                let literal_bit: u32 = match range_decoder
                    .decode_bit(&mut self.literal_probabilities[literal_index])
                {
                    Ok(value) => value,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                        return Err(error);
                    }
                };
                if literal_bit != match_bit as u32 {
                    matched = false;
                }
                // Note that only the 8 bits MSB are used.
                match_byte = match_byte.wrapping_shl(1);
                literal_bit
            } else {
                let literal_index: usize = subcoder + symbol;

                match range_decoder.decode_bit(&mut self.literal_probabilities[literal_index]) {
                    Ok(value) => value,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                        return Err(error);
                    }
                }
            };
            symbol = (symbol << 1) + (literal_bit as usize);
        }
        let literal_value: u8 = (symbol & 0xff) as u8;

        if *uncompressed_data_offset >= uncompressed_data_end_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid uncompressed data value too small"
            ));
        }
        if debug_trace.debug_output {
            debug_trace.print_field("symbol", format!("literal: 0x{:02x}", literal_value));
        }
        uncompressed_data[*uncompressed_data_offset] = literal_value;
        *uncompressed_data_offset += 1;

        self.previous_byte = literal_value;
        self.update_state_literal();

        Ok(())
    }

    /// Decodes a long repeated match (i.e. a repeated match with a decoded length).
    fn decode_long_repeated_match(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        range_decoder: &mut LzmaRangeDecoder,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_end_offset: usize,
        position_state: usize,
        dictionary_start_offset: usize,
        match_distance: usize,
    ) -> Result<(), ErrorTrace> {
        self.update_state_repeated();

        let match_length: usize = match range_decoder.decode_length(
            position_state,
            &mut self.repeated_match_length_probabilities,
        ) {
            Ok(match_length) => match_length,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode match length");
                return Err(error);
            }
        };
        for _ in 0..match_length {
            if *uncompressed_data_offset >= uncompressed_data_end_offset {
                return Err(keramics_core::error_trace_new!(
                    "Invalid uncompressed data value too small"
                ));
            }
            // Note that the repeated distance must have been a valid distance when it was stored.
            if match_distance >= *uncompressed_data_offset - dictionary_start_offset {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid match distance: {} value out of bounds",
                    match_distance
                )));
            }
            uncompressed_data[*uncompressed_data_offset] =
                uncompressed_data[*uncompressed_data_offset - 1 - match_distance];
            *uncompressed_data_offset += 1;

            self.previous_byte = uncompressed_data[*uncompressed_data_offset - 1];
        }
        if debug_trace.debug_output {
            debug_trace.print_field(
                "symbol",
                format!(
                    "long repeated match: length: {}, distance: {}",
                    match_length, match_distance
                ),
            );
        }
        Ok(())
    }

    /// Decodes a match with a decoded distance.
    fn decode_match(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        range_decoder: &mut LzmaRangeDecoder,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_end_offset: usize,
        position_state: usize,
        dictionary_start_offset: usize,
        expect_end_of_payload_marker: bool,
    ) -> Result<bool, ErrorTrace> {
        self.update_state_match();

        let match_length: usize = match range_decoder
            .decode_length(position_state, &mut self.match_length_probabilities)
        {
            Ok(match_length) => match_length,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode match length");
                return Err(error);
            }
        };
        // Zero-based match length capped at 3 is used to select the distance slot table.
        let length_state: usize = min(match_length - LZMA_MINIMUM_MATCH_LENGTH, 3);

        let distance: usize = match self.decode_match_distance(range_decoder, length_state) {
            Ok(distance) => distance,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode match distance");
                return Err(error);
            }
        };
        // The distance value 0xffffffff is reserved for the end of payload marker (EOPM).
        if distance as u32 == LZMA_END_OF_PAYLOAD_MARKER {
            // The EOPM is only allowed when the end of the payload has been reached.
            if !expect_end_of_payload_marker {
                return Err(keramics_core::error_trace_new!(
                    "Unexpected end of payload marker (EOPM)"
                ));
            }
            let finished: bool = match range_decoder.is_finished() {
                Ok(finished) => finished,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to determine if range decoder is finished"
                    );
                    return Err(error);
                }
            };
            if !finished {
                return Err(keramics_core::error_trace_new!(
                    "Invalid end of payload marker (EOPM)"
                ));
            }
            if debug_trace.debug_output {
                debug_trace.print_field("symbol", "EOPM");
            }
            return Ok(true);
        }
        self.repeated_distances[3] = self.repeated_distances[2];
        self.repeated_distances[2] = self.repeated_distances[1];
        self.repeated_distances[1] = self.repeated_distances[0];
        self.repeated_distances[0] = distance;

        if distance >= *uncompressed_data_offset - dictionary_start_offset {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid match distance: {} value out of bounds",
                distance
            )));
        }
        for _ in 0..match_length {
            if *uncompressed_data_offset >= uncompressed_data_end_offset {
                return Err(keramics_core::error_trace_new!(
                    "Invalid uncompressed data value too small"
                ));
            }
            uncompressed_data[*uncompressed_data_offset] =
                uncompressed_data[*uncompressed_data_offset - 1 - distance];
            *uncompressed_data_offset += 1;
            self.previous_byte = uncompressed_data[*uncompressed_data_offset - 1];
        }
        if debug_trace.debug_output {
            debug_trace.print_field(
                "symbol",
                format!("match: length: {}, distance: {}", match_length, distance),
            );
        }
        Ok(false)
    }

    /// Decodes a match distance, including the end of payload marker (EOPM) value.
    fn decode_match_distance(
        &mut self,
        range_decoder: &mut LzmaRangeDecoder,
        length_state: usize,
    ) -> Result<usize, ErrorTrace> {
        let distance_slot_base: usize = length_state * LZMA_NUMBER_OF_DISTANCE_SLOTS;
        let mut symbol: usize = 1;

        while symbol < LZMA_NUMBER_OF_DISTANCE_SLOTS {
            let distance_slot_index: usize = distance_slot_base + symbol;
            let bit_value: u32 = match range_decoder
                .decode_bit(&mut self.distance_slot_probabilities[distance_slot_index])
            {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                    return Err(error);
                }
            };
            symbol = (symbol << 1) + (bit_value as usize);
        }
        let slot: usize = symbol - LZMA_NUMBER_OF_DISTANCE_SLOTS;

        if slot < LZMA_DISTANCE_SLOT_LOW_BITS {
            return Ok(slot);
        }
        if slot < LZMA_DISTANCE_SLOT_PLAIN_BITS {
            // Decode the distance low bits using the position special decoders.
            return self.decode_distance_low_bits(range_decoder, slot);
        }
        // Decode the distance middle bits as plain bits and the low bits using the
        // distance alignment decoder.
        self.decode_distance_high_bits(range_decoder, slot)
    }

    /// Decodes an LZMA properties byte into literal context bits, literal position bits and
    /// position bits.
    pub(super) fn decode_properties(properties_value: u8) -> Result<(u8, u8, u8), ErrorTrace> {
        if properties_value > LZMA_MAXIMUM_PROPERTIES {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported LZMA properties value: 0x{:02x}",
                properties_value
            )));
        }
        let mut value: usize = properties_value as usize;

        let literal_context_bits: u8 = (value % 9) as u8;
        value /= 9;
        let literal_position_bits: u8 = (value % 5) as u8;
        let position_bits: u8 = (value / 5) as u8;

        if (literal_context_bits + literal_position_bits) > 4 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported LZMA properties value: 0x{:02x} (lc + lp > 4)",
                properties_value
            )));
        }
        Ok((literal_context_bits, literal_position_bits, position_bits))
    }

    /// Decodes a repeated match (i.e. a match which uses one of the 4 latest distances).
    fn decode_repeated_match(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        range_decoder: &mut LzmaRangeDecoder,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_end_offset: usize,
        position_state: usize,
        dictionary_start_offset: usize,
    ) -> Result<(), ErrorTrace> {
        if *uncompressed_data_offset - dictionary_start_offset == 0 {
            return Err(keramics_core::error_trace_new!("Invalid match at offset 0"));
        }
        let bit_value: u32 = match range_decoder
            .decode_bit(&mut self.repeated_match_distance0_probabilities[self.state])
        {
            Ok(value) => value,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                return Err(error);
            }
        };
        if bit_value == 0 {
            let table_index: usize = (self.state * LZMA_NUMBER_OF_POSITION_STATES) + position_state;
            let bit_value: u32 = match range_decoder
                .decode_bit(&mut self.repeated_match_distance0_long_probabilities[table_index])
            {
                Ok(value) => value,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                    return Err(error);
                }
            };
            if bit_value == 0 {
                // Short repeated match: repeat the last decoded byte at rep0.
                return self.decode_short_repeated_match(
                    debug_trace,
                    uncompressed_data,
                    uncompressed_data_offset,
                    uncompressed_data_end_offset,
                    dictionary_start_offset,
                );
            }
            // Long repeated match at rep0.
            let match_distance: usize = self.repeated_distances[0];

            return self.decode_long_repeated_match(
                debug_trace,
                range_decoder,
                uncompressed_data,
                uncompressed_data_offset,
                uncompressed_data_end_offset,
                position_state,
                dictionary_start_offset,
                match_distance,
            );
        }
        let match_distance: usize = match self.select_repeated_match_distance(range_decoder) {
            Ok(match_distance) => match_distance,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to select repeated match distance"
                );
                return Err(error);
            }
        };
        self.decode_long_repeated_match(
            debug_trace,
            range_decoder,
            uncompressed_data,
            uncompressed_data_offset,
            uncompressed_data_end_offset,
            position_state,
            dictionary_start_offset,
            match_distance,
        )
    }

    /// Decodes a short repeated match (i.e. a single byte at the rep0 distance).
    fn decode_short_repeated_match(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        uncompressed_data: &mut [u8],
        uncompressed_data_offset: &mut usize,
        uncompressed_data_end_offset: usize,
        dictionary_start_offset: usize,
    ) -> Result<(), ErrorTrace> {
        self.update_state_short_repeated();

        let match_distance: usize = self.repeated_distances[0];

        if *uncompressed_data_offset >= uncompressed_data_end_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid uncompressed data value too small"
            ));
        }
        // Note that a short repeated match must be a valid distance.
        if match_distance >= *uncompressed_data_offset - dictionary_start_offset {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid match distance: {} value out of bounds",
                match_distance
            )));
        }
        uncompressed_data[*uncompressed_data_offset] =
            uncompressed_data[*uncompressed_data_offset - 1 - match_distance];
        *uncompressed_data_offset += 1;

        self.previous_byte = uncompressed_data[*uncompressed_data_offset - 1];

        if debug_trace.debug_output {
            debug_trace.print_field(
                "symbol",
                format!("short repeated match: distance: {}", match_distance),
            );
        }
        Ok(())
    }

    /// Decompresses a single LZMA stream chunk.
    pub fn decompress_chunk(
        &mut self,
        debug_trace: &mut DebugTraceScope,
        compressed_data: &[u8],
        uncompressed_data: &mut [u8],
        uncompressed_data_size: usize,
        uncompressed_data_offset: usize,
        dictionary_start_offset: usize,
        expected_uncompressed_size: usize,
        allow_end_of_payload_marker: bool,
    ) -> Result<usize, ErrorTrace> {
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(compressed_data, 0);

        match range_decoder.read_initialization_bytes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read initialization bytes");
                return Err(error);
            }
        }
        let size_defined: bool = expected_uncompressed_size != usize::MAX;
        let uncompressed_data_end_offset: usize = if size_defined {
            let end_offset: usize =
                match uncompressed_data_offset.checked_add(expected_uncompressed_size) {
                    Some(value) => value,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid uncompressed data offset value out of bounds"
                        ));
                    }
                };
            if end_offset > uncompressed_data_size {
                return Err(keramics_core::error_trace_new!(
                    "Invalid uncompressed data value too small"
                ));
            }
            end_offset
        } else {
            uncompressed_data_size
        };
        let position_mask: usize = (1 << self.position_bits) - 1;
        let literal_position_mask: usize = (1 << self.literal_position_bits) - 1;

        // The end of payload marker (EOPM) is always allowed in a stream with an unknown
        // uncompressed data size. When the uncompressed data size is known, EOPM is only allowed
        // after exactly that many bytes have been decompressed.
        let mut expect_end_of_payload_marker: bool = !size_defined;
        let mut current_uncompressed_data_offset: usize = uncompressed_data_offset;

        debug_trace.print_start("LzmaChunk");

        loop {
            if size_defined && current_uncompressed_data_offset >= uncompressed_data_end_offset {
                let end_of_chunk: bool =
                    match range_decoder.check_for_end_of_chunk(allow_end_of_payload_marker) {
                        Ok(end_of_chunk) => end_of_chunk,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to check for end of chunk"
                            );
                            return Err(error);
                        }
                    };
                if end_of_chunk {
                    debug_trace.print_end();

                    return Ok(current_uncompressed_data_offset - uncompressed_data_offset);
                }
                expect_end_of_payload_marker = true;
            }
            let position_state: usize = current_uncompressed_data_offset & position_mask;
            let match_index: usize = (self.state * LZMA_NUMBER_OF_POSITION_STATES) + position_state;

            let bit_value: u32 =
                match range_decoder.decode_bit(&mut self.match_probabilities[match_index]) {
                    Ok(value) => value,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                        return Err(error);
                    }
                };
            if bit_value == 0 {
                match self.decode_literal(
                    debug_trace,
                    &mut range_decoder,
                    uncompressed_data,
                    &mut current_uncompressed_data_offset,
                    uncompressed_data_end_offset,
                    literal_position_mask,
                    dictionary_start_offset,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode literal");
                        return Err(error);
                    }
                }
            } else {
                // Decode a match.
                let bit_value: u32 = match range_decoder
                    .decode_bit(&mut self.repeated_match_probabilities[self.state])
                {
                    Ok(value) => value,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                        return Err(error);
                    }
                };
                if bit_value == 0 {
                    // Simple match.
                    let end_of_chunk: bool = match self.decode_match(
                        debug_trace,
                        &mut range_decoder,
                        uncompressed_data,
                        &mut current_uncompressed_data_offset,
                        uncompressed_data_end_offset,
                        position_state,
                        dictionary_start_offset,
                        expect_end_of_payload_marker,
                    ) {
                        Ok(end_of_chunk) => end_of_chunk,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to decode match");
                            return Err(error);
                        }
                    };
                    if end_of_chunk {
                        debug_trace.print_end();

                        return Ok(current_uncompressed_data_offset - uncompressed_data_offset);
                    }
                } else {
                    // Repeated match.
                    match self.decode_repeated_match(
                        debug_trace,
                        &mut range_decoder,
                        uncompressed_data,
                        &mut current_uncompressed_data_offset,
                        uncompressed_data_end_offset,
                        position_state,
                        dictionary_start_offset,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to decode repeated match"
                            );
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    /// Resets the decoder state while keeping the current properties.
    pub fn reset_state(&mut self) {
        self.state = 0;
        self.repeated_distances = [0; 4];
        self.previous_byte = 0;

        self.match_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_distance0_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_distance1_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_distance2_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_distance0_long_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.literal_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.match_length_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.repeated_match_length_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.distance_slot_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.distance_low_bit_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
        self.distance_alignment_probabilities
            .fill(LZMA_PROBABILITY_INITIAL_VALUE);
    }

    /// Resets the decoder state and changes the properties.
    pub fn reset_state_and_properties(
        &mut self,
        literal_context_bits: u8,
        literal_position_bits: u8,
        position_bits: u8,
    ) -> Result<(), ErrorTrace> {
        let decoder: LzmaDecoder =
            match Self::new(literal_context_bits, literal_position_bits, position_bits) {
                Ok(decoder) => decoder,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to create decoder");
                    return Err(error);
                }
            };
        *self = decoder;

        Ok(())
    }

    /// Selects the distance of a repeated match (rep1 to rep3) and updates the distance history.
    fn select_repeated_match_distance(
        &mut self,
        range_decoder: &mut LzmaRangeDecoder,
    ) -> Result<usize, ErrorTrace> {
        let bit_value: u32 = match range_decoder
            .decode_bit(&mut self.repeated_match_distance1_probabilities[self.state])
        {
            Ok(value) => value,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                return Err(error);
            }
        };
        if bit_value == 0 {
            // rep1 becomes rep0.
            let distance: usize = self.repeated_distances[1];

            self.repeated_distances[1] = self.repeated_distances[0];
            self.repeated_distances[0] = distance;
            return Ok(self.repeated_distances[0]);
        }
        let bit_value: u32 = match range_decoder
            .decode_bit(&mut self.repeated_match_distance2_probabilities[self.state])
        {
            Ok(value) => value,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode bit");
                return Err(error);
            }
        };
        if bit_value == 0 {
            // rep2 becomes rep0.
            let distance: usize = self.repeated_distances[2];

            self.repeated_distances[2] = self.repeated_distances[1];
            self.repeated_distances[1] = self.repeated_distances[0];
            self.repeated_distances[0] = distance;
            return Ok(self.repeated_distances[0]);
        }
        // rep3 becomes rep0.
        let distance: usize = self.repeated_distances[3];

        self.repeated_distances[3] = self.repeated_distances[2];
        self.repeated_distances[2] = self.repeated_distances[1];
        self.repeated_distances[1] = self.repeated_distances[0];
        self.repeated_distances[0] = distance;

        Ok(self.repeated_distances[0])
    }

    /// Updates the decoder state after a literal has been decoded.
    fn update_state_literal(&mut self) {
        let state: usize = self.state;

        self.state = if state <= 3 {
            0
        } else if state <= 9 {
            state - 3
        } else {
            state - 6
        };
    }

    /// Updates the decoder state after a simple match has been decoded.
    fn update_state_match(&mut self) {
        let state: usize = self.state;

        self.state = if state < LZMA_NUMBER_OF_LITERAL_STATES {
            7
        } else {
            10
        };
    }

    /// Updates the decoder state after a (long) repeated match has been decoded.
    fn update_state_repeated(&mut self) {
        let state: usize = self.state;

        self.state = if state < LZMA_NUMBER_OF_LITERAL_STATES {
            8
        } else {
            11
        };
    }

    /// Updates the decoder state after a short repeated match has been decoded.
    fn update_state_short_repeated(&mut self) {
        let state: usize = self.state;

        self.state = if state < LZMA_NUMBER_OF_LITERAL_STATES {
            9
        } else {
            11
        };
    }
}

/// Context for decompressing LZMA compressed data.
pub struct LzmaContext {
    /// Uncompressed data size.
    pub uncompressed_data_size: usize,
}

impl LzmaContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            uncompressed_data_size: 0,
        }
    }

    /// Decompress data.
    pub fn decompress(
        &mut self,
        compressed_data: &[u8],
        uncompressed_data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let compressed_data_size: usize = compressed_data.len();
        let uncompressed_data_size: usize = uncompressed_data.len();

        if compressed_data_size < 13 {
            return Err(keramics_core::error_trace_new!(
                "Invalid compressed data value too small"
            ));
        }
        keramics_core::debug_trace_data_and_structure!(
            "LzmaStreamHeader",
            0,
            &compressed_data[0..13],
            13,
            LzmaStreamHeader::debug_read_data(compressed_data)
        );
        let mut stream_header: LzmaStreamHeader = LzmaStreamHeader::new();

        match stream_header.read_data(compressed_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read stream header");
                return Err(error);
            }
        }
        if stream_header.uncompressed_data_size != LZMA_UNCOMPRESSED_SIZE_UNKNOWN
            && stream_header.uncompressed_data_size as usize > uncompressed_data_size
        {
            return Err(keramics_core::error_trace_new!(
                "Invalid uncompressed data value too small"
            ));
        }
        let mut decoder: LzmaDecoder = match LzmaDecoder::new(
            stream_header.literal_context_bits,
            stream_header.literal_position_bits,
            stream_header.position_bits,
        ) {
            Ok(decoder) => decoder,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create decoder");
                return Err(error);
            }
        };
        let expected_uncompressed_size: usize =
            if stream_header.uncompressed_data_size == LZMA_UNCOMPRESSED_SIZE_UNKNOWN {
                usize::MAX
            } else {
                stream_header.uncompressed_data_size as usize
            };
        let compressed_stream: &[u8] = &compressed_data[13..];

        DebugTrace::scope(|debug_trace| {
            match decoder.decompress_chunk(
                debug_trace,
                compressed_stream,
                uncompressed_data,
                uncompressed_data_size,
                0,
                0,
                expected_uncompressed_size,
                true,
            ) {
                Ok(number_of_bytes) => {
                    self.uncompressed_data_size = number_of_bytes;
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decompress data");
                    return Err(error);
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::mediator::Mediator;

    /// LZMA stream of "hello world" repeated 200 times (2200 bytes).
    /// Uses an unknown uncompressed data size and an end of payload marker (EOPM).
    const TEST_DATA_P1_ALONE: [u8; 49] = [
        0x5d, 0x00, 0x00, 0x10, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x34,
        0x19, 0x49, 0xdb, 0x85, 0x5c, 0x63, 0xad, 0x3e, 0xf9, 0x63, 0x76, 0x52, 0x91, 0xc1, 0x28,
        0xe5, 0xe3, 0x72, 0x09, 0x59, 0xe6, 0x44, 0x88, 0x1d, 0x01, 0x63, 0x02, 0x47, 0xff, 0xff,
        0xf4, 0xd4, 0x80, 0x00,
    ];

    /// LZMA stream of "hello world" repeated 200 times (2200 bytes).
    /// Uses a known uncompressed data size and an end of payload marker (EOPM).
    const TEST_DATA_P1_KNOWN_SIZE_EOPM: [u8; 49] = [
        0x5d, 0x00, 0x00, 0x10, 0x00, 0x98, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34,
        0x19, 0x49, 0xdb, 0x85, 0x5c, 0x63, 0xad, 0x3e, 0xf9, 0x63, 0x76, 0x52, 0x91, 0xc1, 0x28,
        0xe5, 0xe3, 0x72, 0x09, 0x59, 0xe6, 0x44, 0x88, 0x1d, 0x01, 0x63, 0x02, 0x47, 0xff, 0xff,
        0xf4, 0xd4, 0x80, 0x00,
    ];

    /// LZMA stream of "hello world" repeated 200 times (2200 bytes).
    /// Uses an unknown uncompressed data size, 8 MiB dictionary size, and an end of payload marker (EOPM).
    const TEST_DATA_UNKNOWN_SIZE_LARGE_DICTIONARY: [u8; 49] = [
        0x5d, 0x00, 0x00, 0x80, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x34,
        0x19, 0x49, 0xee, 0x8d, 0xe9, 0x17, 0x89, 0x3a, 0x33, 0x60, 0x0a, 0x36, 0x3d, 0xd0, 0xbf,
        0x6e, 0xfc, 0xeb, 0xa2, 0x43, 0x50, 0x04, 0xde, 0x91, 0x7b, 0x7b, 0x3d, 0x95, 0xff, 0xff,
        0xc1, 0xf4, 0x00, 0x00,
    ];

    /// LZMA stream of bytes 0x00 to 0xff repeated 16 times (4096 bytes).
    const TEST_DATA_PATTERN_4096: [u8; 277] = [
        0x5d, 0x00, 0x00, 0x80, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
        0x00, 0x52, 0x50, 0x0a, 0x84, 0xf9, 0x9b, 0xb2, 0x80, 0x21, 0xa9, 0x69, 0xd6, 0x27, 0xe0,
        0x3e, 0x06, 0x5a, 0x5f, 0x04, 0x8d, 0x53, 0xd4, 0x04, 0xba, 0x39, 0x57, 0x05, 0x09, 0xc1,
        0x55, 0x24, 0xde, 0x9d, 0xb8, 0x71, 0x59, 0x31, 0x60, 0xa1, 0x9f, 0xf9, 0x6f, 0x49, 0x73,
        0xf2, 0xc8, 0xea, 0x8c, 0xba, 0x1a, 0x8b, 0x29, 0x69, 0x21, 0x80, 0xfe, 0x33, 0x83, 0x66,
        0xaf, 0x46, 0x6d, 0xec, 0x9e, 0x89, 0x8a, 0x0b, 0x83, 0xf0, 0x3c, 0x0e, 0x89, 0x8e, 0x3f,
        0xed, 0x5f, 0xe7, 0x9e, 0x90, 0xd9, 0x1c, 0xff, 0x32, 0xf4, 0xb2, 0xe0, 0x39, 0x51, 0xb2,
        0xd2, 0x14, 0x15, 0xb4, 0xc5, 0x71, 0xba, 0xdb, 0x06, 0xe3, 0x79, 0x9a, 0x9f, 0xbb, 0x38,
        0xc1, 0xb0, 0x00, 0xac, 0x93, 0x0b, 0xaa, 0x06, 0x19, 0x03, 0x12, 0x08, 0x15, 0x5b, 0x9b,
        0xc8, 0x48, 0xf0, 0x32, 0x2e, 0xfe, 0x2d, 0xa0, 0x87, 0xc8, 0xf0, 0xa4, 0xe0, 0xd2, 0x51,
        0xeb, 0x8d, 0x67, 0x56, 0x92, 0xb2, 0x4d, 0x84, 0xc5, 0xf1, 0x86, 0x31, 0xdf, 0x6a, 0x62,
        0x5b, 0xc2, 0x79, 0x2d, 0xd9, 0xf7, 0x3c, 0x73, 0xba, 0x74, 0x74, 0x07, 0xd8, 0x3c, 0xa9,
        0x56, 0x22, 0x24, 0xa1, 0x66, 0xf8, 0x5a, 0x84, 0x5f, 0x30, 0x67, 0xd2, 0xf6, 0x4b, 0x49,
        0x2e, 0x7f, 0x20, 0xeb, 0xdb, 0xf8, 0x10, 0x0e, 0x94, 0x78, 0x77, 0xc7, 0x3f, 0x6b, 0xef,
        0xb4, 0xcd, 0x95, 0xe2, 0x6f, 0xf6, 0x44, 0x6e, 0x06, 0xcf, 0x0b, 0x82, 0x1a, 0xcb, 0xdb,
        0x7a, 0xf0, 0x57, 0x8d, 0x98, 0xff, 0x90, 0xc0, 0x3e, 0xe6, 0xc1, 0x12, 0x41, 0x75, 0xee,
        0x03, 0x28, 0x96, 0xeb, 0x13, 0xfb, 0xa7, 0x28, 0xcc, 0xaf, 0x32, 0xbb, 0xa4, 0x0e, 0x25,
        0xf2, 0x58, 0xb0, 0xde, 0xd8, 0x56, 0x1c, 0x66, 0xf0, 0xe2, 0x1b, 0x35, 0x5e, 0xdb, 0x2c,
        0x46, 0xff, 0xff, 0x76, 0x21, 0x40, 0x00,
    ];

    /// LZMA stream of data that uses simple matches at each distance slot 4 to 13.
    const TEST_DATA_DISTANCE_SLOTS: [u8; 662] = [
        0x5d, 0x00, 0x00, 0x80, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x6f,
        0xbe, 0xd6, 0xaf, 0xbf, 0xdd, 0x0d, 0xc9, 0xb9, 0x18, 0x3d, 0x79, 0x74, 0xdf, 0xa6, 0xf5,
        0x45, 0x65, 0x19, 0xbb, 0x24, 0x47, 0x09, 0x7e, 0xfc, 0xa8, 0xe8, 0x62, 0x70, 0x39, 0x65,
        0x56, 0x75, 0xbd, 0x21, 0x71, 0xe2, 0x5b, 0x84, 0x1f, 0xc9, 0xff, 0x8c, 0x9a, 0x2a, 0x26,
        0xb7, 0x77, 0x3e, 0x48, 0xa0, 0x8a, 0xaf, 0x4f, 0x12, 0x6f, 0x6c, 0x74, 0x19, 0x91, 0x72,
        0xce, 0x4d, 0x3b, 0x54, 0x42, 0x2a, 0x52, 0xdd, 0xc8, 0x95, 0xbf, 0xd8, 0x74, 0x2f, 0x81,
        0xc3, 0x7b, 0xad, 0x38, 0x11, 0xcc, 0x42, 0x01, 0xfb, 0x89, 0x84, 0xbb, 0xb0, 0x78, 0xfd,
        0x8c, 0x04, 0x9e, 0xc3, 0x90, 0x2d, 0x8c, 0x11, 0x16, 0x10, 0x30, 0x33, 0xdb, 0x61, 0xd6,
        0x2b, 0x3b, 0xf6, 0x47, 0x19, 0x22, 0xcd, 0x41, 0xc1, 0x62, 0xdb, 0xdb, 0x54, 0x52, 0xe6,
        0x13, 0x39, 0x20, 0x0f, 0x88, 0x26, 0x6b, 0x07, 0x09, 0x1e, 0x72, 0x67, 0x4b, 0x43, 0xe1,
        0xea, 0xd0, 0x74, 0xdc, 0xbe, 0x0f, 0x1e, 0x9c, 0x4f, 0x75, 0x8e, 0x8f, 0x23, 0x0d, 0x43,
        0xe3, 0x35, 0x3f, 0x97, 0x5e, 0x79, 0x54, 0x92, 0x79, 0x64, 0x04, 0xd3, 0xa8, 0xdf, 0xf9,
        0xb7, 0x8a, 0x6b, 0x62, 0x9d, 0x55, 0x48, 0x1b, 0xa2, 0x33, 0x84, 0x68, 0xd6, 0x0a, 0x5c,
        0x39, 0x07, 0xa9, 0x18, 0x76, 0x22, 0x25, 0x09, 0x44, 0x45, 0x9c, 0x48, 0x70, 0x1f, 0x56,
        0xee, 0xd5, 0x87, 0xb4, 0x89, 0xcf, 0xec, 0x7b, 0xc3, 0x6b, 0xc2, 0xeb, 0xee, 0xbe, 0x61,
        0x8b, 0x5e, 0xe1, 0x28, 0xb6, 0x55, 0x47, 0xb8, 0x2e, 0x73, 0x22, 0x6a, 0xde, 0xc9, 0x21,
        0x24, 0x2c, 0x7c, 0x85, 0x21, 0x14, 0x88, 0x75, 0xa7, 0xd2, 0x67, 0xa8, 0x0f, 0x52, 0x30,
        0x8f, 0x9f, 0x9a, 0x3d, 0x0d, 0xbf, 0xe9, 0xfc, 0x34, 0x50, 0x75, 0x9b, 0xb0, 0x0f, 0xea,
        0xa9, 0x46, 0x14, 0x05, 0x53, 0xdb, 0xf3, 0x0d, 0x44, 0x8a, 0x6e, 0x54, 0x42, 0x04, 0xb8,
        0x35, 0x20, 0x96, 0x58, 0xef, 0xbd, 0x97, 0xe5, 0x91, 0xb5, 0xa3, 0xfe, 0x54, 0xd3, 0xba,
        0x78, 0x7d, 0xf3, 0x42, 0x72, 0xe9, 0x1a, 0x3e, 0x27, 0x3e, 0x69, 0xe0, 0x0c, 0x6b, 0x6c,
        0xc5, 0x23, 0x91, 0xd6, 0xfe, 0x2c, 0x8d, 0x11, 0xa6, 0xf2, 0xb4, 0x68, 0x6e, 0x5d, 0x58,
        0x99, 0x43, 0x52, 0xb8, 0x63, 0xf2, 0x26, 0xc1, 0xff, 0x7b, 0xcf, 0x5a, 0xff, 0xde, 0x4a,
        0x40, 0x26, 0xc2, 0xa4, 0x5e, 0x2a, 0x85, 0xaa, 0x47, 0xb7, 0xc9, 0x1e, 0x4c, 0xf6, 0x78,
        0xda, 0x33, 0x38, 0x8f, 0xa6, 0x07, 0x61, 0xd4, 0xae, 0x52, 0x47, 0xb3, 0x8f, 0xd2, 0x75,
        0x1f, 0xd6, 0x61, 0x9f, 0xb9, 0x07, 0xc5, 0xaa, 0xdc, 0x94, 0xbf, 0xdb, 0x4e, 0xa4, 0x20,
        0x41, 0xf0, 0x37, 0xa8, 0x66, 0xa3, 0x27, 0x6d, 0xac, 0x22, 0xea, 0x41, 0xeb, 0x35, 0x66,
        0x88, 0xe0, 0x3a, 0xb5, 0x9f, 0xd7, 0xc5, 0x41, 0xd3, 0x77, 0x6c, 0x2c, 0x75, 0x77, 0x53,
        0x8c, 0x41, 0x45, 0x18, 0x03, 0x46, 0x8d, 0x9b, 0x74, 0x6f, 0x61, 0x3b, 0x35, 0xf5, 0xfe,
        0x40, 0x16, 0x25, 0xa9, 0x0f, 0x6d, 0x60, 0x50, 0xc7, 0x07, 0xba, 0x96, 0x9d, 0xae, 0xc7,
        0x8b, 0x39, 0xb0, 0x46, 0x7b, 0xb9, 0xf9, 0xe8, 0xf5, 0x33, 0x04, 0x31, 0x9e, 0xb6, 0x9f,
        0x47, 0x59, 0x85, 0x7f, 0x8c, 0x06, 0x1e, 0x71, 0xf3, 0xe2, 0xea, 0x54, 0x0c, 0x2d, 0x9c,
        0x42, 0xd2, 0x22, 0xb0, 0x9c, 0xff, 0x95, 0xac, 0x18, 0x80, 0xfb, 0xcb, 0xae, 0xf8, 0xf3,
        0xf6, 0x7b, 0x6b, 0xc0, 0x5d, 0xc0, 0xa3, 0x65, 0x4c, 0x30, 0x49, 0x78, 0x17, 0xd5, 0xe1,
        0xff, 0x2b, 0x25, 0xec, 0x62, 0xe3, 0x67, 0x8b, 0x10, 0x24, 0x42, 0x5c, 0x92, 0xde, 0x55,
        0xb4, 0x04, 0xb9, 0xbe, 0xd1, 0xa2, 0xb5, 0x95, 0x5d, 0x8b, 0x90, 0xb7, 0x72, 0x9b, 0x41,
        0xe9, 0x16, 0x44, 0x11, 0x1c, 0x43, 0x7f, 0x00, 0x13, 0xe8, 0xdb, 0x83, 0xde, 0x36, 0xfc,
        0x81, 0xc5, 0x90, 0x9c, 0x73, 0x4f, 0x3f, 0x8a, 0xc8, 0x79, 0x81, 0x44, 0x11, 0xc5, 0x35,
        0x73, 0x33, 0x68, 0x77, 0x11, 0x5a, 0x66, 0x63, 0x80, 0x26, 0xd2, 0x6c, 0x00, 0x13, 0xbb,
        0xcd, 0x5f, 0x86, 0xc9, 0xfe, 0x73, 0x6e, 0x25, 0xe1, 0x2a, 0xb5, 0xc9, 0x4e, 0x05, 0x39,
        0x57, 0x95, 0xab, 0x4d, 0xb0, 0xe8, 0x90, 0x2b, 0x96, 0xb6, 0xeb, 0x8a, 0x17, 0xe9, 0x2d,
        0xa0, 0x5e, 0x03, 0x02, 0x69, 0xff, 0x49, 0x40, 0x14, 0x5b, 0x72, 0x84, 0x27, 0x76, 0xe9,
        0x48, 0x4e, 0x47, 0x88, 0x2f, 0xa7, 0x76, 0xf3, 0x5a, 0x7d, 0x3d, 0xe6, 0x98, 0xff, 0xa9,
        0xd4, 0xb5, 0x62, 0x85, 0xee, 0xad, 0x6d, 0xf1, 0x23, 0xdc, 0x52, 0xff, 0xff, 0xb3, 0x92,
        0x00, 0x00,
    ];

    /// LZMA stream of data that uses small match distances 1 to 300.
    const TEST_DATA_SMALL_DISTANCES: [u8; 355] = [
        0x5d, 0x00, 0x00, 0x80, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
        0xe0, 0x07, 0xbf, 0xda, 0x3e, 0x02, 0x6b, 0x9c, 0x30, 0xd9, 0x73, 0x37, 0xdd, 0x1c, 0x3d,
        0x12, 0x7f, 0x70, 0x50, 0x74, 0x08, 0xf9, 0x81, 0x74, 0xee, 0x26, 0x17, 0xa4, 0x7a, 0xff,
        0xf0, 0x84, 0xa6, 0x59, 0xa4, 0x4a, 0x7d, 0x40, 0x95, 0xa5, 0x79, 0xd2, 0xb9, 0x4a, 0x0f,
        0xf5, 0xf2, 0xf3, 0x5e, 0x59, 0xf8, 0xda, 0x9d, 0x45, 0x12, 0x75, 0x57, 0x28, 0x6f, 0x4b,
        0x47, 0x71, 0x4b, 0x19, 0x27, 0xbf, 0x91, 0x4b, 0x68, 0x9b, 0x4f, 0xd8, 0x94, 0xba, 0x42,
        0x6a, 0x12, 0xfd, 0x53, 0x38, 0x16, 0x2c, 0x3e, 0xa5, 0xe7, 0x2c, 0x10, 0xe3, 0x29, 0xda,
        0xde, 0xeb, 0x59, 0x78, 0x51, 0x5b, 0xa7, 0xce, 0xd7, 0x57, 0x76, 0xd3, 0xad, 0x2a, 0x36,
        0xa6, 0x0a, 0x4c, 0x48, 0x8b, 0xfa, 0x01, 0x83, 0x27, 0xa1, 0x49, 0x1a, 0xda, 0x92, 0x5b,
        0x57, 0x14, 0x80, 0x25, 0x9e, 0x7b, 0x18, 0x3a, 0xf3, 0x27, 0x95, 0x09, 0x2f, 0xc4, 0x4f,
        0x35, 0xba, 0x50, 0x68, 0x77, 0xa1, 0x03, 0xb7, 0xef, 0x1e, 0xc7, 0xa4, 0x71, 0x68, 0x47,
        0x78, 0x9d, 0x40, 0x06, 0x2f, 0xb0, 0xa6, 0xe7, 0xd8, 0xab, 0x8f, 0x7f, 0x7f, 0x14, 0x19,
        0x3f, 0xf0, 0x77, 0x39, 0x0c, 0xed, 0xcd, 0x30, 0xab, 0x4b, 0x55, 0x66, 0x6b, 0x60, 0x42,
        0xd1, 0x98, 0xd5, 0x16, 0xe6, 0x8b, 0x28, 0xb6, 0xe4, 0xcc, 0x09, 0xc7, 0x43, 0x2c, 0xf4,
        0x37, 0xc9, 0x0d, 0x50, 0x62, 0x27, 0x7d, 0x86, 0xfe, 0xdf, 0xa3, 0x15, 0x53, 0xe5, 0x2e,
        0xb6, 0xc4, 0x24, 0xab, 0x82, 0x99, 0xbf, 0x4e, 0x38, 0xa9, 0xc5, 0x07, 0xc7, 0xb9, 0xfb,
        0xd5, 0x7b, 0x86, 0xee, 0x87, 0x97, 0x38, 0xa8, 0x3c, 0xbb, 0xf2, 0x6f, 0x19, 0xf2, 0x32,
        0xbd, 0xb1, 0x5c, 0xe9, 0x28, 0xa7, 0x90, 0xdc, 0xb9, 0x66, 0xcc, 0x61, 0x87, 0xe8, 0x0c,
        0x69, 0xcf, 0x8e, 0x40, 0x31, 0x86, 0x7f, 0x99, 0x3a, 0x2f, 0xc9, 0xeb, 0x54, 0x4e, 0xf4,
        0xca, 0x86, 0xc1, 0xf8, 0x30, 0xf8, 0x50, 0xcf, 0x20, 0x35, 0xa3, 0x7b, 0x13, 0xe5, 0xbd,
        0x20, 0x35, 0xdf, 0xb4, 0x19, 0x0b, 0xa3, 0x8a, 0x1e, 0x59, 0x39, 0xc6, 0x1a, 0xe2, 0x5b,
        0x36, 0x56, 0x58, 0x75, 0x91, 0xc9, 0xab, 0x35, 0xe5, 0x9c, 0x20, 0xd3, 0x7b, 0xe9, 0x88,
        0xc6, 0x31, 0x2b, 0x5d, 0x0f, 0xf7, 0x82, 0x5e, 0x4d, 0x6a, 0x75, 0x3d, 0xff, 0x5f, 0xe1,
        0x99, 0xce, 0x49, 0xb5, 0x4c, 0xff, 0xfe, 0x13, 0xc8, 0x00,
    ];

    /// LZMA stream of 5000 zero bytes that uses repeated matches.
    const TEST_DATA_ZEROS: [u8; 49] = [
        0x5d, 0x00, 0x00, 0x80, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
        0x6f, 0xfd, 0xff, 0xff, 0xa3, 0xb7, 0xff, 0x47, 0x3e, 0x48, 0x15, 0x72, 0x39, 0x61, 0x51,
        0xb8, 0x92, 0x28, 0xe6, 0xa3, 0x86, 0x07, 0xf9, 0xee, 0xcd, 0xd4, 0xd4, 0x26, 0x1f, 0xff,
        0xfd, 0x84, 0x40, 0x00,
    ];

    /// Generates a pseudo-random data sequence.
    fn pseudo_random(number_of_bytes: usize, mut seed: u64) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        while data.len() < number_of_bytes {
            seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7fffffff;
            data.push(((seed >> 16) % 251 + 3) as u8);
        }
        data
    }

    fn get_test_data_distance_slots() -> Vec<u8> {
        let distances: [usize; 13] = [5, 6, 7, 9, 13, 18, 30, 36, 52, 65, 90, 102, 127];
        let mut data: Vec<u8> = Vec::new();

        for distance in distances {
            let block_size: usize = distance - 1;
            let block: Vec<u8> = pseudo_random(block_size, (7 * distance + 13) as u64);

            data.extend_from_slice(&block);
            data.push(220 + (distance % 30) as u8);
            data.extend_from_slice(&block);
            data.push(2);
            data.push(200 + (distance % 50) as u8);
        }
        data
    }

    #[test]
    fn test_decode_properties() -> Result<(), ErrorTrace> {
        let (literal_context_bits, literal_position_bits, position_bits): (u8, u8, u8) =
            LzmaDecoder::decode_properties(0x5d)?;
        assert_eq!(literal_context_bits, 3);
        assert_eq!(literal_position_bits, 0);
        assert_eq!(position_bits, 2);

        let (literal_context_bits, literal_position_bits, position_bits): (u8, u8, u8) =
            LzmaDecoder::decode_properties(0x00)?;
        assert_eq!(literal_context_bits, 0);
        assert_eq!(literal_position_bits, 0);
        assert_eq!(position_bits, 0);

        let (literal_context_bits, literal_position_bits, position_bits): (u8, u8, u8) =
            LzmaDecoder::decode_properties(0xd8)?;
        assert_eq!(literal_context_bits, 0);
        assert_eq!(literal_position_bits, 4);
        assert_eq!(position_bits, 4);

        Ok(())
    }

    #[test]
    fn test_decode_properties_with_invalid_value() {
        let result: Result<(u8, u8, u8), ErrorTrace> = LzmaDecoder::decode_properties(0xe1);
        assert!(result.is_err());

        let result: Result<(u8, u8, u8), ErrorTrace> = LzmaDecoder::decode_properties(0x96);
        assert!(result.is_err());

        let result: Result<(u8, u8, u8), ErrorTrace> = LzmaDecoder::decode_properties(0xd9);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_stream_header() -> Result<(), ErrorTrace> {
        let mut stream_header: LzmaStreamHeader = LzmaStreamHeader::new();

        stream_header.read_data(&TEST_DATA_P1_ALONE)?;
        assert_eq!(stream_header.literal_context_bits, 3);
        assert_eq!(stream_header.literal_position_bits, 0);
        assert_eq!(stream_header.position_bits, 2);
        assert_eq!(stream_header.dictionary_size, 1048576);
        assert_eq!(stream_header.uncompressed_data_size, u64::MAX);

        Ok(())
    }

    #[test]
    fn test_read_stream_header_with_invalid_size() {
        let mut stream_header: LzmaStreamHeader = LzmaStreamHeader::new();

        let test_data: [u8; 12] = [
            0x5d, 0x00, 0x00, 0x10, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let result: Result<(), ErrorTrace> = stream_header.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_stream_header_with_invalid_properties() {
        let mut stream_header: LzmaStreamHeader = LzmaStreamHeader::new();

        let mut test_data: [u8; 13] = [
            0x5d, 0x00, 0x00, 0x10, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        test_data[0] = 0xe0;
        let result: Result<(), ErrorTrace> = stream_header.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let expected_data: Vec<u8> = b"hello world".to_vec().repeat(200);
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_P1_ALONE, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_known_size_and_end_of_payload_marker() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let expected_data: Vec<u8> = b"hello world".to_vec().repeat(200);
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_P1_KNOWN_SIZE_EOPM, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_unknown_size_and_large_dictionary() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let expected_data: Vec<u8> = b"hello world".to_vec().repeat(200);
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(
            &TEST_DATA_UNKNOWN_SIZE_LARGE_DICTIONARY,
            &mut uncompressed_data,
        )?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_pattern_4096() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let mut expected_data: Vec<u8> = Vec::new();
        for _ in 0..16 {
            expected_data.extend_from_slice(&(0..=255).collect::<Vec<u8>>());
        }
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_PATTERN_4096, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_distance_slots() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let expected_data: Vec<u8> = get_test_data_distance_slots();
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_DISTANCE_SLOTS, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_small_distances() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let mut expected_data: Vec<u8> = Vec::new();
        for value in 1..300 {
            let byte: u8 = (value % 251) as u8;

            for _ in 0..(value % 3 + 1) {
                expected_data.push(byte);
            }
        }
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_SMALL_DISTANCES, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_repeated_matches() -> Result<(), ErrorTrace> {
        let mut test_context: LzmaContext = LzmaContext::new();

        let expected_data: Vec<u8> = vec![0; 5000];
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        test_context.decompress(&TEST_DATA_ZEROS, &mut uncompressed_data)?;
        assert_eq!(test_context.uncompressed_data_size, expected_data.len());
        assert_eq!(&uncompressed_data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decompress_with_insufficient_compressed_data() {
        let mut test_context: LzmaContext = LzmaContext::new();

        let test_data: [u8; 12] = [
            0x5d, 0x00, 0x00, 0x10, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let mut uncompressed_data: Vec<u8> = vec![0; 2200];
        let result: Result<(), ErrorTrace> =
            test_context.decompress(&test_data, &mut uncompressed_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_with_invalid_range_decoder_initialization_byte() {
        let mut test_context: LzmaContext = LzmaContext::new();

        let mut test_data: Vec<u8> = TEST_DATA_P1_ALONE.to_vec();

        test_data[13] = 0x01;
        let mut uncompressed_data: Vec<u8> = vec![0; 2200];
        let result: Result<(), ErrorTrace> =
            test_context.decompress(&test_data, &mut uncompressed_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_with_unsupported_size() {
        let mut test_context: LzmaContext = LzmaContext::new();

        let mut uncompressed_data: Vec<u8> = vec![0; 100];
        let result: Result<(), ErrorTrace> =
            test_context.decompress(&TEST_DATA_P1_KNOWN_SIZE_EOPM, &mut uncompressed_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_with_truncated_stream() {
        let mut test_context: LzmaContext = LzmaContext::new();

        let mut uncompressed_data: Vec<u8> = vec![0; 2200];
        let result: Result<(), ErrorTrace> = test_context.decompress(
            &TEST_DATA_P1_ALONE[0..TEST_DATA_P1_ALONE.len() - 2],
            &mut uncompressed_data,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_range_decoder_read_initialization_bytes_with_insufficient_data() {
        let test_data: [u8; 4] = [0x00, 0x01, 0x02, 0x03];
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&test_data, 0);

        let result: Result<(), ErrorTrace> = range_decoder.read_initialization_bytes();
        assert!(result.is_err());
    }

    #[test]
    fn test_range_decoder_decode_bit_and_direct() -> Result<(), ErrorTrace> {
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;

        let mut probability: u16 = LZMA_PROBABILITY_INITIAL_VALUE;
        let result: Result<u32, ErrorTrace> = range_decoder.decode_bit(&mut probability);
        assert!(result.is_err());

        let result: Result<u32, ErrorTrace> = range_decoder.decode_bit_direct();
        assert!(result.is_err());

        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x10000000;
        range_decoder.code = 0x00000000;

        let bit_value: u32 = range_decoder.decode_bit_direct()?;
        assert_eq!(bit_value, 0);

        range_decoder.code = range_decoder.range - 1;
        let bit_value: u32 = range_decoder.decode_bit_direct()?;
        assert_eq!(bit_value, 1);

        Ok(())
    }

    #[test]
    fn test_range_decoder_decode_length_with_errors() {
        let mut table: Vec<u16> =
            vec![LZMA_PROBABILITY_INITIAL_VALUE; LZMA_MATCH_LENGTH_NUMBER_OF_CELLS];

        // Error on first choice bit.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;
        let result: Result<usize, ErrorTrace> = range_decoder.decode_length(0, &mut table);
        assert!(result.is_err());

        // Error on second choice bit (first bit decodes 1, halving range below threshold).
        let mut table: Vec<u16> =
            vec![LZMA_PROBABILITY_INITIAL_VALUE; LZMA_MATCH_LENGTH_NUMBER_OF_CELLS];
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x01000000;
        range_decoder.code = 0x00ffffff;
        let result: Result<usize, ErrorTrace> = range_decoder.decode_length(0, &mut table);
        assert!(result.is_err());

        // Error inside symbol bit loop (first bit decodes 0, halving range below threshold).
        let mut table: Vec<u16> =
            vec![LZMA_PROBABILITY_INITIAL_VALUE; LZMA_MATCH_LENGTH_NUMBER_OF_CELLS];
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x01000000;
        range_decoder.code = 0x00000000;
        let result: Result<usize, ErrorTrace> = range_decoder.decode_length(0, &mut table);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_decoder_is_finished_and_check_for_end_of_chunk() -> Result<(), ErrorTrace> {
        // Normalization failure in is_finished and check_for_end_of_chunk.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;

        let result: Result<bool, ErrorTrace> = range_decoder.is_finished();
        assert!(result.is_err());

        let result: Result<bool, ErrorTrace> = range_decoder.check_for_end_of_chunk(false);
        assert!(result.is_err());

        // Incomplete stream (code != 0) when EOPM is disallowed.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x10000000;
        range_decoder.code = 0x00000001;

        let result: Result<bool, ErrorTrace> = range_decoder.check_for_end_of_chunk(false);
        assert!(result.is_err());

        // Trailing compressed data when EOPM is disallowed.
        let test_data: [u8; 2] = [0x00, 0x00];
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&test_data, 0);
        range_decoder.range = 0x10000000;
        range_decoder.code = 0x00000000;

        let result: Result<bool, ErrorTrace> = range_decoder.check_for_end_of_chunk(false);
        assert!(result.is_err());

        // Exact end of chunk when EOPM is disallowed.
        range_decoder.data_offset = 2;
        let finished: bool = range_decoder.check_for_end_of_chunk(false)?;
        assert!(finished);

        Ok(())
    }

    #[test]
    fn test_read_stream_header_with_small_dictionary_size() -> Result<(), ErrorTrace> {
        let mut stream_header: LzmaStreamHeader = LzmaStreamHeader::new();

        let test_data: [u8; 13] = [
            0x5d, 0x00, 0x04, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        stream_header.read_data(&test_data)?;
        assert_eq!(stream_header.dictionary_size, LZMA_MINIMUM_DICTIONARY_SIZE);

        Ok(())
    }

    #[test]
    fn test_decoder_new_with_invalid_properties() {
        let result: Result<LzmaDecoder, ErrorTrace> = LzmaDecoder::new(4, 1, 0);
        assert!(result.is_err());

        let result: Result<LzmaDecoder, ErrorTrace> = LzmaDecoder::new(3, 0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_reset_state_and_properties() -> Result<(), ErrorTrace> {
        let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
        decoder.state = 9;
        decoder.repeated_distances = [1, 2, 3, 4];
        decoder.previous_byte = 0xaa;
        decoder.match_probabilities[0] = 100;

        decoder.reset_state();
        assert_eq!(decoder.state, 0);
        assert_eq!(decoder.repeated_distances, [0; 4]);
        assert_eq!(decoder.previous_byte, 0);
        assert_eq!(
            decoder.match_probabilities[0],
            LZMA_PROBABILITY_INITIAL_VALUE
        );

        decoder.reset_state_and_properties(2, 1, 1)?;
        assert_eq!(decoder.literal_context_bits, 2);
        assert_eq!(decoder.literal_position_bits, 1);
        assert_eq!(decoder.position_bits, 1);

        let result: Result<(), ErrorTrace> = decoder.reset_state_and_properties(4, 1, 0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_decoder_select_repeated_match_distance() -> Result<(), ErrorTrace> {
        let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
        decoder.repeated_distances = [10, 20, 30, 40];

        // Select rep1 (bit 0).
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0xffffffff;
        range_decoder.code = 0x00000000;
        let distance: usize = decoder.select_repeated_match_distance(&mut range_decoder)?;
        assert_eq!(distance, 20);
        assert_eq!(decoder.repeated_distances, [20, 10, 30, 40]);

        // Select rep2 (bits 1, 0).
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0xffffffff;
        range_decoder.code = 0x90000000;
        let distance: usize = decoder.select_repeated_match_distance(&mut range_decoder)?;
        assert_eq!(distance, 30);
        assert_eq!(decoder.repeated_distances, [30, 20, 10, 40]);

        // Select rep3 (bits 1, 1).
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0xffffffff;
        range_decoder.code = 0xfffffffe;
        let distance: usize = decoder.select_repeated_match_distance(&mut range_decoder)?;
        assert_eq!(distance, 40);
        assert_eq!(decoder.repeated_distances, [40, 30, 20, 10]);

        // Error on first bit.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;
        let result: Result<usize, ErrorTrace> =
            decoder.select_repeated_match_distance(&mut range_decoder);
        assert!(result.is_err());

        // Error on second bit.
        let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x01000000;
        range_decoder.code = 0x00ffffff;
        let result: Result<usize, ErrorTrace> =
            decoder.select_repeated_match_distance(&mut range_decoder);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_decoder_decode_distance_methods() -> Result<(), ErrorTrace> {
        // Distance slot < 4.
        let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0xffffffff;
        range_decoder.code = 0x00000000;
        let distance: usize = decoder.decode_match_distance(&mut range_decoder, 0)?;
        assert_eq!(distance, 0);

        // Error in decode_match_distance slot loop.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;
        let result: Result<usize, ErrorTrace> =
            decoder.decode_match_distance(&mut range_decoder, 0);
        assert!(result.is_err());

        // Error in decode_distance_low_bits.
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x00000001;
        let result: Result<usize, ErrorTrace> =
            decoder.decode_distance_low_bits(&mut range_decoder, 4);
        assert!(result.is_err());

        // Error in decode_distance_high_bits alignment loop (after 2 plain 0-bits).
        let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
        range_decoder.range = 0x04000000;
        range_decoder.code = 0x00000000;
        let result: Result<usize, ErrorTrace> =
            decoder.decode_distance_high_bits(&mut range_decoder, 14);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_decoder_decode_literal_with_errors() -> Result<(), ErrorTrace> {
        DebugTrace::scope(|debug_trace: &mut DebugTraceScope| {
            // Normal literal bit decode error.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x00000001;
            let mut uncompressed_data: [u8; 4] = [0; 4];
            let mut uncompressed_data_offset: usize = 0;

            let result: Result<(), ErrorTrace> = decoder.decode_literal(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
            );
            assert!(result.is_err());

            // Matched literal bit decode error (state >= 7).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            decoder.state = 7;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x00000001;
            uncompressed_data[0] = 0x55;
            uncompressed_data_offset = 1;

            let result: Result<(), ErrorTrace> = decoder.decode_literal(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
            );
            assert!(result.is_err());

            // Output buffer too small.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[0x00; 4], 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x00000000;
            uncompressed_data_offset = 4;

            let result: Result<(), ErrorTrace> = decoder.decode_literal(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
            );
            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn test_decoder_decode_match_with_errors() -> Result<(), ErrorTrace> {
        DebugTrace::scope(|debug_trace: &mut DebugTraceScope| {
            // Length decode error.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x00000001;
            let mut uncompressed_data: [u8; 4] = [0x41; 4];
            let mut uncompressed_data_offset: usize = 1;

            let result: Result<bool, ErrorTrace> = decoder.decode_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
                false,
            );
            assert!(result.is_err());

            // Unexpected EOPM (all-1s range decoder decodes distance 0xffffffff).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let all_ones_data: [u8; 16] = [0xff; 16];
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&all_ones_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0xfffffffe;
            uncompressed_data_offset = 1;

            let result: Result<bool, ErrorTrace> = decoder.decode_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
                false,
            );
            assert!(result.is_err());

            // Invalid EOPM (expected, but range decoder code != 0).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&all_ones_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0xfffffffe;
            uncompressed_data_offset = 1;

            let result: Result<bool, ErrorTrace> = decoder.decode_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
                true,
            );
            assert!(result.is_err());

            // EOPM where is_finished() fails normalization.
            for shift in 0..8 {
                let initial_range: u32 = 0xffffffff >> shift;
                let mut probe_decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
                let mut probe_range_decoder: LzmaRangeDecoder =
                    LzmaRangeDecoder::new(&all_ones_data, 0);
                probe_range_decoder.range = initial_range;
                probe_range_decoder.code = initial_range - 1;
                _ = probe_range_decoder
                    .decode_length(0, &mut probe_decoder.match_length_probabilities);
                _ = probe_decoder.decode_match_distance(&mut probe_range_decoder, 3);
                if probe_range_decoder.range < 16777216 {
                    let bytes_consumed: usize = probe_range_decoder.data_offset;
                    let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
                    let mut range_decoder: LzmaRangeDecoder =
                        LzmaRangeDecoder::new(&all_ones_data[0..bytes_consumed], 0);
                    range_decoder.range = initial_range;
                    range_decoder.code = initial_range - 1;
                    uncompressed_data_offset = 1;

                    let result: Result<bool, ErrorTrace> = decoder.decode_match(
                        debug_trace,
                        &mut range_decoder,
                        &mut uncompressed_data,
                        &mut uncompressed_data_offset,
                        4,
                        0,
                        0,
                        true,
                    );
                    assert!(result.is_err());
                    break;
                }
            }

            // Distance out of bounds (offset 0 with distance 0).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let all_zeros_data: [u8; 16] = [0x00; 16];
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&all_zeros_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x00000000;
            uncompressed_data_offset = 0;

            let result: Result<bool, ErrorTrace> = decoder.decode_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                4,
                0,
                0,
                false,
            );
            assert!(result.is_err());

            // Output buffer too small during match copy.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&all_zeros_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x00000000;
            uncompressed_data_offset = 1;

            let result: Result<bool, ErrorTrace> = decoder.decode_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                2,
                0,
                0,
                false,
            );
            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn test_decoder_decode_repeated_match_with_errors() -> Result<(), ErrorTrace> {
        DebugTrace::scope(|debug_trace: &mut DebugTraceScope| {
            let mut uncompressed_data: [u8; 8] = [0x41, 0x42, 0x43, 0x44, 0, 0, 0, 0];

            // Repeated match at offset 0.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            let mut uncompressed_data_offset: usize = 0;
            let result: Result<(), ErrorTrace> = decoder.decode_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
            );
            assert!(result.is_err());

            // Error decoding repeated_match_distance0 bit.
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x00000001;
            uncompressed_data_offset = 4;
            let result: Result<(), ErrorTrace> = decoder.decode_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
            );
            assert!(result.is_err());

            // Error decoding repeated_match_distance0_long bit.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x01000000;
            range_decoder.code = 0x00000000;
            let result: Result<(), ErrorTrace> = decoder.decode_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
            );
            assert!(result.is_err());

            // Error in select_repeated_match_distance from decode_repeated_match.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x01000000;
            range_decoder.code = 0x00ffffff;
            let result: Result<(), ErrorTrace> = decoder.decode_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
            );
            assert!(result.is_err());

            // Valid repeated match selecting rep1 (bit 1 then bit 0, then length 2 via 0-bits).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            decoder.repeated_distances = [0, 1, 2, 3];
            let zeros_data: [u8; 8] = [0x00; 8];
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&zeros_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x80000000;
            uncompressed_data_offset = 4;

            decoder.decode_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
            )?;
            assert_eq!(uncompressed_data_offset, 6);
            assert_eq!(uncompressed_data[4], 0x43);
            assert_eq!(uncompressed_data[5], 0x44);

            // Test decode_short_repeated_match with output buffer too small and out-of-bounds distance.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            uncompressed_data_offset = 8;
            let result: Result<(), ErrorTrace> = decoder.decode_short_repeated_match(
                debug_trace,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
            );
            assert!(result.is_err());

            decoder.repeated_distances[0] = 10;
            uncompressed_data_offset = 2;
            let result: Result<(), ErrorTrace> = decoder.decode_short_repeated_match(
                debug_trace,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
            );
            assert!(result.is_err());

            // Test decode_long_repeated_match with length error, output too small, and out-of-bounds distance.
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&[], 0);
            range_decoder.range = 0x00000001;
            uncompressed_data_offset = 2;
            let result: Result<(), ErrorTrace> = decoder.decode_long_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
                0,
            );
            assert!(result.is_err());

            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&zeros_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x00000000;
            uncompressed_data_offset = 1;
            let result: Result<(), ErrorTrace> = decoder.decode_long_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                2,
                0,
                0,
                0,
            );
            assert!(result.is_err());

            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut range_decoder: LzmaRangeDecoder = LzmaRangeDecoder::new(&zeros_data, 0);
            range_decoder.range = 0xffffffff;
            range_decoder.code = 0x00000000;
            uncompressed_data_offset = 2;
            let result: Result<(), ErrorTrace> = decoder.decode_long_repeated_match(
                debug_trace,
                &mut range_decoder,
                &mut uncompressed_data,
                &mut uncompressed_data_offset,
                8,
                0,
                0,
                10,
            );
            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn test_decoder_decompress_chunk_with_errors() -> Result<(), ErrorTrace> {
        DebugTrace::scope(|debug_trace: &mut DebugTraceScope| {
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let mut uncompressed_data: [u8; 8] = [0; 8];
            let initialization_bytes: [u8; 5] = [0x00, 0x00, 0x00, 0x00, 0x01];

            // Overflow on uncompressed_data_offset + expected_uncompressed_size.
            let result: Result<usize, ErrorTrace> = decoder.decompress_chunk(
                debug_trace,
                &initialization_bytes,
                &mut uncompressed_data,
                8,
                usize::MAX - 1,
                0,
                2,
                false,
            );
            assert!(result.is_err());

            // Expected uncompressed size exceeds output buffer size.
            let result: Result<usize, ErrorTrace> = decoder.decompress_chunk(
                debug_trace,
                &initialization_bytes,
                &mut uncompressed_data,
                8,
                4,
                0,
                6,
                false,
            );
            assert!(result.is_err());

            // End of chunk check fails (code != 0 and EOPM disallowed).
            let result: Result<usize, ErrorTrace> = decoder.decompress_chunk(
                debug_trace,
                &initialization_bytes,
                &mut uncompressed_data,
                8,
                0,
                0,
                0,
                false,
            );
            assert!(result.is_err());

            // Repeated match error at offset 0 via decompress_chunk (all-1s -> match + repeated).
            let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
            let repeated_at_zero: [u8; 5] = [0x00, 0xff, 0xff, 0xff, 0xff];
            let result: Result<usize, ErrorTrace> = decoder.decompress_chunk(
                debug_trace,
                &repeated_at_zero,
                &mut uncompressed_data,
                8,
                0,
                0,
                8,
                false,
            );
            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn test_decompress_with_invalid_stream_header() {
        let mut test_context: LzmaContext = LzmaContext::new();
        let mut test_data: Vec<u8> = TEST_DATA_P1_ALONE.to_vec();
        test_data[0] = 0xe1;

        let mut uncompressed_data: Vec<u8> = vec![0; 2200];
        let result: Result<(), ErrorTrace> =
            test_context.decompress(&test_data, &mut uncompressed_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_with_truncated_prefixes() {
        let mut uncompressed_data: Vec<u8> = vec![0; 5000];

        for length in 13..TEST_DATA_P1_ALONE.len() {
            let mut test_context: LzmaContext = LzmaContext::new();
            let result: Result<(), ErrorTrace> = test_context.decompress(
                &TEST_DATA_P1_ALONE[0..length],
                &mut uncompressed_data[0..2200],
            );
            assert!(result.is_err());
        }
        for length in 13..TEST_DATA_ZEROS.len() {
            let mut test_context: LzmaContext = LzmaContext::new();
            let result: Result<(), ErrorTrace> =
                test_context.decompress(&TEST_DATA_ZEROS[0..length], &mut uncompressed_data);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_decompress_with_debug_output() -> Result<(), ErrorTrace> {
        let original_debug_output: bool = Mediator::current().debug_output;
        Mediator::new(true).make_current();

        let mut test_context: LzmaContext = LzmaContext::new();
        let expected_data: Vec<u8> = b"hello world".to_vec().repeat(200);
        let mut uncompressed_data: Vec<u8> = vec![0; expected_data.len()];
        let result: Result<(), ErrorTrace> =
            test_context.decompress(&TEST_DATA_P1_ALONE, &mut uncompressed_data);

        let mut zeros_context: LzmaContext = LzmaContext::new();
        let mut zeros_data: Vec<u8> = vec![0; 5000];
        let zeros_result: Result<(), ErrorTrace> =
            zeros_context.decompress(&TEST_DATA_ZEROS, &mut zeros_data);

        let short_repeated_result: Result<(), ErrorTrace> =
            DebugTrace::scope(|debug_trace: &mut DebugTraceScope| {
                let mut decoder: LzmaDecoder = LzmaDecoder::new(3, 0, 2)?;
                let mut buffer: [u8; 4] = [0x41, 0, 0, 0];
                let mut offset: usize = 1;
                decoder.decode_short_repeated_match(debug_trace, &mut buffer, &mut offset, 4, 0)
            });

        Mediator::new(original_debug_output).make_current();

        result?;
        assert_eq!(&uncompressed_data, &expected_data);
        zeros_result?;
        assert_eq!(zeros_data, vec![0; 5000]);
        short_repeated_result?;

        Ok(())
    }
}
