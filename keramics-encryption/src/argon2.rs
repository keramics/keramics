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

//! Argon2 password-based key derivation function.
//!
//! Provides support for Argon2d, Argon2i, and Argon2id key derivation (RFC 9106).

use keramics_core::ErrorTrace;
use keramics_hashes::{Blake2bContext, DigestHashContext};
use keramics_types::bytes_to_u64_le;

/// Argon2 variant type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Argon2Type {
    /// Data-dependent memory access (Argon2d).
    Argon2d = 0,

    /// Data-independent memory access (Argon2i).
    Argon2i = 1,

    /// Hybrid data-independent and data-dependent memory access (Argon2id).
    Argon2id = 2,
}

/// Argon2 version 1.3 identifier.
const ARGON2_VERSION: u32 = 0x13;

/// Number of synchronization points (slices) per pass.
const ARGON2_SYNC_POINTS: usize = 4;

/// Context for Argon2 key derivation.
pub struct Argon2Context {
    /// Argon2 variant type.
    argon2_type: Argon2Type,

    /// Salt.
    salt: Vec<u8>,

    /// Number of iterations (time cost).
    number_of_iterations: u32,

    /// Memory size in KiB.
    memory_size_kib: u32,

    /// Parallelism (number of lanes).
    parallelism: u32,
}

impl Argon2Context {
    /// Creates a new context.
    pub fn new(
        argon2_type: Argon2Type,
        salt: &[u8],
        number_of_iterations: u32,
        memory_size_kib: u32,
        parallelism: u32,
    ) -> Self {
        Self {
            argon2_type,
            salt: salt.to_vec(),
            number_of_iterations,
            memory_size_kib,
            parallelism,
        }
    }

    /// Computes the variable-length BLAKE2b hash function `H'` defined in RFC 9106.
    fn blake2b_long(input: &[u8], output: &mut [u8]) -> Result<(), ErrorTrace> {
        let output_length: usize = output.len();
        let output_length_bytes: [u8; 4] = (output_length as u32).to_le_bytes();

        if output_length <= 64 {
            let mut context: Blake2bContext =
                match Blake2bContext::new_with_digest_size(output_length) {
                    Ok(context) => context,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to create BLAKE2b context"
                        );
                        return Err(error);
                    }
                };
            context.update(&output_length_bytes);
            context.update(input);
            context.finalize_into(output);
            return Ok(());
        }
        let mut context: Blake2bContext = Blake2bContext::new();
        context.update(&output_length_bytes);
        context.update(input);

        let mut block_hash: [u8; 64] = [0; 64];
        context.finalize_into(&mut block_hash);
        output[0..32].copy_from_slice(&block_hash[0..32]);

        let mut output_offset: usize = 32;
        let mut remaining_size: usize = output_length - 32;

        while remaining_size > 64 {
            context.update(&block_hash);
            context.finalize_into(&mut block_hash);

            let output_end_offset: usize = output_offset + 32;
            output[output_offset..output_end_offset].copy_from_slice(&block_hash[0..32]);

            output_offset = output_end_offset;
            remaining_size -= 32;
        }
        let mut final_context: Blake2bContext =
            match Blake2bContext::new_with_digest_size(remaining_size) {
                Ok(context) => context,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to create final BLAKE2b context"
                    );
                    return Err(error);
                }
            };
        final_context.update(&block_hash);
        final_context.finalize_into(&mut output[output_offset..output_length]);

        Ok(())
    }

    /// Computes the Argon2 compression function `G(X, Y)`.
    fn compress_block(block_x: &[u64; 128], block_y: &[u64; 128], output_block: &mut [u64; 128]) {
        let mut block_r: [u64; 128] = [0; 128];
        for index in 0..128 {
            block_r[index] = block_x[index] ^ block_y[index];
        }
        let mut block_q: [u64; 128] = block_r;

        // Apply permutation P row-wise to 8 rows of 16 64-bit words.
        for row_index in 0..8 {
            let base: usize = row_index * 16;
            Self::permute_16(
                &mut block_q,
                [
                    base,
                    base + 1,
                    base + 2,
                    base + 3,
                    base + 4,
                    base + 5,
                    base + 6,
                    base + 7,
                    base + 8,
                    base + 9,
                    base + 10,
                    base + 11,
                    base + 12,
                    base + 13,
                    base + 14,
                    base + 15,
                ],
            );
        }
        // Apply permutation P column-wise to 8 columns of 16 64-bit words.
        for column_index in 0..8 {
            let base: usize = column_index * 2;
            Self::permute_16(
                &mut block_q,
                [
                    base,
                    base + 1,
                    base + 16,
                    base + 17,
                    base + 32,
                    base + 33,
                    base + 48,
                    base + 49,
                    base + 64,
                    base + 65,
                    base + 80,
                    base + 81,
                    base + 96,
                    base + 97,
                    base + 112,
                    base + 113,
                ],
            );
        }
        for index in 0..128 {
            output_block[index] = block_q[index] ^ block_r[index];
        }
    }

    /// Computes a data-independent address block for Argon2i / Argon2id.
    fn compute_address_block(
        pass: u64,
        lane: u64,
        slice: u64,
        memory_blocks: u64,
        number_of_iterations: u64,
        argon2_type: u64,
        counter: u64,
        address_block: &mut [u64; 128],
    ) {
        let zero_block: [u64; 128] = [0; 128];
        let mut input_block: [u64; 128] = [0; 128];
        input_block[0] = pass;
        input_block[1] = lane;
        input_block[2] = slice;
        input_block[3] = memory_blocks;
        input_block[4] = number_of_iterations;
        input_block[5] = argon2_type;
        input_block[6] = counter;

        let mut intermediate_block: [u64; 128] = [0; 128];
        Self::compress_block(&zero_block, &input_block, &mut intermediate_block);
        Self::compress_block(&zero_block, &intermediate_block, address_block);
    }

    /// Derives a key from the password.
    pub fn derive_key(&self, password: &[u8], key: &mut [u8]) -> Result<(), ErrorTrace> {
        let key_size: usize = key.len();
        if key_size < 4 {
            return Err(keramics_core::error_trace_new!(
                "Invalid key size - value too small"
            ));
        }
        if self.salt.len() < 8 {
            return Err(keramics_core::error_trace_new!(
                "Invalid salt size - value too small"
            ));
        }
        if self.number_of_iterations == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - number of iterations value out of bounds"
            ));
        }
        if self.parallelism == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - parallelism value out of bounds"
            ));
        }
        let parallelism: usize = self.parallelism as usize;
        let minimum_memory: u32 = self.parallelism.saturating_mul(8);

        if self.memory_size_kib < minimum_memory {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - memory size value out of bounds"
            ));
        }
        let lane_length: usize = ((self.memory_size_kib as usize)
            / (ARGON2_SYNC_POINTS * parallelism))
            * ARGON2_SYNC_POINTS;
        let segment_length: usize = lane_length / ARGON2_SYNC_POINTS;
        let memory_blocks: usize = lane_length * parallelism;

        // Calculate H_0 using BLAKE2b-512.
        let parallelism_bytes: [u8; 4] = self.parallelism.to_le_bytes();
        let key_size_bytes: [u8; 4] = (key_size as u32).to_le_bytes();
        let memory_size_bytes: [u8; 4] = self.memory_size_kib.to_le_bytes();
        let number_of_iterations_bytes: [u8; 4] = self.number_of_iterations.to_le_bytes();
        let version_bytes: [u8; 4] = ARGON2_VERSION.to_le_bytes();
        let argon2_type_bytes: [u8; 4] = (self.argon2_type as u32).to_le_bytes();
        let password_size_bytes: [u8; 4] = (password.len() as u32).to_le_bytes();
        let salt_size_bytes: [u8; 4] = (self.salt.len() as u32).to_le_bytes();
        let zero_bytes: [u8; 4] = 0_u32.to_le_bytes();

        let mut hash_context: Blake2bContext = Blake2bContext::new();
        hash_context.update(&parallelism_bytes);
        hash_context.update(&key_size_bytes);
        hash_context.update(&memory_size_bytes);
        hash_context.update(&number_of_iterations_bytes);
        hash_context.update(&version_bytes);
        hash_context.update(&argon2_type_bytes);
        hash_context.update(&password_size_bytes);
        hash_context.update(password);
        hash_context.update(&salt_size_bytes);
        hash_context.update(&self.salt);
        // Empty secret key and associated data.
        hash_context.update(&zero_bytes);
        hash_context.update(&zero_bytes);

        let mut initial_hash: [u8; 64] = [0; 64];
        hash_context.finalize_into(&mut initial_hash);

        let mut memory: Vec<[u64; 128]> = vec![[0; 128]; memory_blocks];

        // Initialize the first two blocks of each lane.
        let mut block_seed: [u8; 72] = [0; 72];
        block_seed[0..64].copy_from_slice(&initial_hash);
        let mut block_bytes: [u8; 1024] = [0; 1024];

        for lane in 0..parallelism {
            let lane_bytes: [u8; 4] = (lane as u32).to_le_bytes();
            block_seed[68..72].copy_from_slice(&lane_bytes);

            for block_index in 0..2 {
                let block_index_bytes: [u8; 4] = (block_index as u32).to_le_bytes();
                block_seed[64..68].copy_from_slice(&block_index_bytes);

                match Self::blake2b_long(&block_seed, &mut block_bytes) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize Argon2 memory block"
                        );
                        return Err(error);
                    }
                }
                let memory_index: usize = (lane * lane_length) + block_index;
                for word_index in 0..128 {
                    let byte_offset: usize = word_index * 8;
                    memory[memory_index][word_index] = bytes_to_u64_le!(block_bytes, byte_offset);
                }
            }
        }
        let mut address_block: [u64; 128] = [0; 128];
        let mut new_block: [u64; 128] = [0; 128];

        for pass in 0..(self.number_of_iterations as usize) {
            for slice in 0..ARGON2_SYNC_POINTS {
                let data_independent: bool = match self.argon2_type {
                    Argon2Type::Argon2d => false,
                    Argon2Type::Argon2i => true,
                    Argon2Type::Argon2id => pass == 0 && slice < 2,
                };
                for lane in 0..parallelism {
                    let mut address_counter: u64 = 0;

                    if data_independent && pass == 0 && slice == 0 {
                        address_counter = 1;
                        Self::compute_address_block(
                            pass as u64,
                            lane as u64,
                            slice as u64,
                            memory_blocks as u64,
                            self.number_of_iterations as u64,
                            self.argon2_type as u64,
                            address_counter,
                            &mut address_block,
                        );
                    }
                    let starting_index: usize = if pass == 0 && slice == 0 { 2 } else { 0 };
                    let lane_base: usize = lane * lane_length;

                    for index in starting_index..segment_length {
                        if data_independent && (index % 128 == 0) {
                            address_counter += 1;
                            Self::compute_address_block(
                                pass as u64,
                                lane as u64,
                                slice as u64,
                                memory_blocks as u64,
                                self.number_of_iterations as u64,
                                self.argon2_type as u64,
                                address_counter,
                                &mut address_block,
                            );
                        }
                        let current_offset: usize = (slice * segment_length) + index;
                        let previous_offset: usize = if current_offset == 0 {
                            lane_length - 1
                        } else {
                            current_offset - 1
                        };
                        let previous_memory_index: usize = lane_base + previous_offset;

                        let pseudo_random: u64 = if data_independent {
                            let address_index: usize = index % 128;
                            address_block[address_index]
                        } else {
                            memory[previous_memory_index][0]
                        };
                        let pseudo_random_low: u64 = (pseudo_random as u32) as u64;
                        let pseudo_random_high: usize = (pseudo_random >> 32) as usize;

                        let reference_lane: usize = if pass == 0 && slice == 0 {
                            lane
                        } else {
                            pseudo_random_high % parallelism
                        };
                        let same_lane: bool = reference_lane == lane;
                        let reference_area_size: usize = if pass == 0 {
                            if slice == 0 {
                                index - 1
                            } else if same_lane {
                                (slice * segment_length) + index - 1
                            } else if index == 0 {
                                (slice * segment_length) - 1
                            } else {
                                slice * segment_length
                            }
                        } else if same_lane {
                            lane_length - segment_length + index - 1
                        } else if index == 0 {
                            lane_length - segment_length - 1
                        } else {
                            lane_length - segment_length
                        };
                        let squared: u64 = (pseudo_random_low * pseudo_random_low) >> 32;
                        let scaled: u64 = ((reference_area_size as u64) * squared) >> 32;
                        let relative_position: usize =
                            (reference_area_size - 1) - (scaled as usize);

                        let start_position: usize =
                            if pass != 0 && slice != (ARGON2_SYNC_POINTS - 1) {
                                (slice + 1) * segment_length
                            } else {
                                0
                            };
                        let reference_offset: usize =
                            (start_position + relative_position) % lane_length;
                        let reference_memory_index: usize =
                            (reference_lane * lane_length) + reference_offset;
                        let current_memory_index: usize = lane_base + current_offset;

                        Self::compress_block(
                            &memory[previous_memory_index],
                            &memory[reference_memory_index],
                            &mut new_block,
                        );
                        if pass == 0 {
                            memory[current_memory_index] = new_block;
                        } else {
                            for word_index in 0..128 {
                                memory[current_memory_index][word_index] ^= new_block[word_index];
                            }
                        }
                    }
                }
            }
        }
        // XOR the final block of each lane.
        let last_block_index: usize = lane_length - 1;
        let mut final_block: [u64; 128] = memory[last_block_index];
        for lane in 1..parallelism {
            let last_index: usize = (lane * lane_length) + last_block_index;
            for word_index in 0..128 {
                final_block[word_index] ^= memory[last_index][word_index];
            }
        }
        let mut final_block_bytes: [u8; 1024] = [0; 1024];
        for word_index in 0..128 {
            let byte_offset: usize = word_index * 8;
            let byte_end_offset: usize = byte_offset + 8;
            final_block_bytes[byte_offset..byte_end_offset]
                .copy_from_slice(&final_block[word_index].to_le_bytes());
        }
        match Self::blake2b_long(&final_block_bytes, key) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to finalize Argon2 key");
                Err(error)
            }
        }
    }

    /// Applies the BlaMka quarter-round function `GB`.
    #[inline(always)]
    fn gb(
        mut value_a: u64,
        mut value_b: u64,
        mut value_c: u64,
        mut value_d: u64,
    ) -> (u64, u64, u64, u64) {
        let product1: u64 = ((value_a as u32) as u64).wrapping_mul((value_b as u32) as u64);
        value_a = value_a
            .wrapping_add(value_b)
            .wrapping_add(product1.wrapping_add(product1));
        value_d = (value_d ^ value_a).rotate_right(32);

        let product2: u64 = ((value_c as u32) as u64).wrapping_mul((value_d as u32) as u64);
        value_c = value_c
            .wrapping_add(value_d)
            .wrapping_add(product2.wrapping_add(product2));
        value_b = (value_b ^ value_c).rotate_right(24);

        let product3: u64 = ((value_a as u32) as u64).wrapping_mul((value_b as u32) as u64);
        value_a = value_a
            .wrapping_add(value_b)
            .wrapping_add(product3.wrapping_add(product3));
        value_d = (value_d ^ value_a).rotate_right(16);

        let product4: u64 = ((value_c as u32) as u64).wrapping_mul((value_d as u32) as u64);
        value_c = value_c
            .wrapping_add(value_d)
            .wrapping_add(product4.wrapping_add(product4));
        value_b = (value_b ^ value_c).rotate_right(63);

        (value_a, value_b, value_c, value_d)
    }

    /// Applies the Argon2 permutation `P` to 16 indexed words of a 128-word block.
    #[inline(always)]
    fn permute_16(block: &mut [u64; 128], indices: [usize; 16]) {
        let mut value0: u64 = block[indices[0]];
        let mut value1: u64 = block[indices[1]];
        let mut value2: u64 = block[indices[2]];
        let mut value3: u64 = block[indices[3]];
        let mut value4: u64 = block[indices[4]];
        let mut value5: u64 = block[indices[5]];
        let mut value6: u64 = block[indices[6]];
        let mut value7: u64 = block[indices[7]];
        let mut value8: u64 = block[indices[8]];
        let mut value9: u64 = block[indices[9]];
        let mut value10: u64 = block[indices[10]];
        let mut value11: u64 = block[indices[11]];
        let mut value12: u64 = block[indices[12]];
        let mut value13: u64 = block[indices[13]];
        let mut value14: u64 = block[indices[14]];
        let mut value15: u64 = block[indices[15]];

        (value0, value4, value8, value12) = Self::gb(value0, value4, value8, value12);
        (value1, value5, value9, value13) = Self::gb(value1, value5, value9, value13);
        (value2, value6, value10, value14) = Self::gb(value2, value6, value10, value14);
        (value3, value7, value11, value15) = Self::gb(value3, value7, value11, value15);

        (value0, value5, value10, value15) = Self::gb(value0, value5, value10, value15);
        (value1, value6, value11, value12) = Self::gb(value1, value6, value11, value12);
        (value2, value7, value8, value13) = Self::gb(value2, value7, value8, value13);
        (value3, value4, value9, value14) = Self::gb(value3, value4, value9, value14);

        block[indices[0]] = value0;
        block[indices[1]] = value1;
        block[indices[2]] = value2;
        block[indices[3]] = value3;
        block[indices[4]] = value4;
        block[indices[5]] = value5;
        block[indices[6]] = value6;
        block[indices[7]] = value7;
        block[indices[8]] = value8;
        block[indices[9]] = value9;
        block[indices[10]] = value10;
        block[indices[11]] = value11;
        block[indices[12]] = value12;
        block[indices[13]] = value13;
        block[indices[14]] = value14;
        block[indices[15]] = value15;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::formatters::format_as_string;

    #[test]
    fn test_derive_key_with_argon2d() -> Result<(), ErrorTrace> {
        let password: [u8; 32] = [0x01; 32];
        let salt: [u8; 16] = [0x02; 16];

        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2d, &salt, 3, 32, 4);
        let mut key: [u8; 32] = [0; 32];
        context.derive_key(&password, &mut key)?;

        let key_string: String = format_as_string(&key);
        assert_eq!(
            key_string,
            "9e34c31a47866ce0c30a90c69dd21022d5329a3b75f9c513722dd2541fe93a1a"
        );
        Ok(())
    }

    #[test]
    fn test_derive_key_with_argon2i() -> Result<(), ErrorTrace> {
        let password: [u8; 32] = [0x01; 32];
        let salt: [u8; 16] = [0x02; 16];

        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2i, &salt, 3, 32, 4);
        let mut key: [u8; 32] = [0; 32];
        context.derive_key(&password, &mut key)?;

        let key_string: String = format_as_string(&key);
        assert_eq!(
            key_string,
            "a9a7510e6db4d588ba3414cd0e094d480d683f97b9ccb612a544fe8ef65ba8e0"
        );
        Ok(())
    }

    #[test]
    fn test_derive_key_with_argon2id() -> Result<(), ErrorTrace> {
        let password: [u8; 32] = [0x01; 32];
        let salt: [u8; 16] = [0x02; 16];

        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 3, 32, 4);
        let mut key: [u8; 32] = [0; 32];
        context.derive_key(&password, &mut key)?;

        let key_string: String = format_as_string(&key);
        assert_eq!(
            key_string,
            "03aab965c12001c9d7d0d2de33192c0494b684bb148196d73c1df1acaf6d0c2e"
        );
        Ok(())
    }

    #[test]
    fn test_derive_key_with_large_segment_and_long_output() -> Result<(), ErrorTrace> {
        let salt: [u8; 16] = [0x03; 16];

        // Segment length > 2 exercises the pass == 0, slice == 0 indexing loop;
        // output length > 64 exercises the multi-block blake2b_long path.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 2, 64, 1);
        let mut long_key: [u8; 96] = [0; 96];
        context.derive_key(b"password", &mut long_key)?;
        assert_ne!(long_key, [0; 96]);

        // Test with zero-length output buffer on blake2b_long directly.
        let mut empty_output: [u8; 0] = [];
        let result: Result<(), ErrorTrace> =
            Argon2Context::blake2b_long(b"input", &mut empty_output);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_derive_key_with_invalid_parameters() {
        let salt: [u8; 16] = [0x02; 16];
        let mut key: [u8; 32] = [0; 32];

        // Test with key size too small.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 1, 32, 1);
        let mut short_key: [u8; 3] = [0; 3];
        let result: Result<(), ErrorTrace> = context.derive_key(b"password", &mut short_key);
        assert!(result.is_err());

        // Test with salt size too small.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &[0; 4], 1, 32, 1);
        let result: Result<(), ErrorTrace> = context.derive_key(b"password", &mut key);
        assert!(result.is_err());

        // Test with zero iterations.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 0, 32, 1);
        let result: Result<(), ErrorTrace> = context.derive_key(b"password", &mut key);
        assert!(result.is_err());

        // Test with zero parallelism.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 1, 32, 0);
        let result: Result<(), ErrorTrace> = context.derive_key(b"password", &mut key);
        assert!(result.is_err());

        // Test with memory size smaller than 8 * parallelism.
        let context: Argon2Context = Argon2Context::new(Argon2Type::Argon2id, &salt, 1, 16, 4);
        let result: Result<(), ErrorTrace> = context.derive_key(b"password", &mut key);
        assert!(result.is_err());
    }
}
