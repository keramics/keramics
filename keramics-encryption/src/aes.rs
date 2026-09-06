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

//! Advanced Encryption Standard (AES) encryption.
//!
//! Provides AES encryption and decryption support.

use keramics_core::ErrorTrace;
use keramics_types::bytes_to_u32_le;

use super::cbc::CbcContext;
use super::ccm::CcmContext;
use super::traits::{CryptCbc, CryptCcm, CryptContext, CryptEcb};
use super::xts::XtsContext;

/// Calculate the next GF(2^8) value using the generator polynomial 0x1b.
const fn calculate_next_gf28_value(value: usize) -> usize {
    if value & 0x80 == 0 {
        (value << 1) & 0xff
    } else {
        ((value << 1) & 0xff) ^ 0x1b
    }
}

/// Generates a derived forward or reverse table by rotating the elements of table 0.
const fn generate_derived_table(table0: &[u32; 256], rotation_bits: u32) -> [u32; 256] {
    let mut table: [u32; 256] = [0; 256];
    let mut table_index: usize = 0;

    while table_index < 256 {
        table[table_index] = table0[table_index].rotate_left(rotation_bits);
        table_index += 1;
    }
    table
}

/// Generates the forward S-Box.
const fn generate_forward_sbox(powers_table: &[u8; 256], logs_table: &[u8; 256]) -> [u8; 256] {
    let mut forward_sbox: [u8; 256] = [0; 256];
    let mut sbox_index: usize = 1;

    forward_sbox[0] = 99;

    while sbox_index < 256 {
        let table_index: u8 = 255 - logs_table[sbox_index];
        let mut byte_value: u8 = powers_table[table_index as usize];

        let mut substitution_value: u8 = ((byte_value << 1) & 0xff) | (byte_value >> 7);
        byte_value ^= substitution_value;

        substitution_value = ((substitution_value << 1) & 0xff) | (substitution_value >> 7);
        byte_value ^= substitution_value;

        substitution_value = ((substitution_value << 1) & 0xff) | (substitution_value >> 7);
        byte_value ^= substitution_value;

        substitution_value = ((substitution_value << 1) & 0xff) | (substitution_value >> 7);
        byte_value ^= substitution_value ^ 0x63;

        forward_sbox[sbox_index] = byte_value;
        sbox_index += 1;
    }
    forward_sbox
}

/// Generates forward table 0.
const fn generate_forward_table0(forward_sbox: &[u8; 256]) -> [u32; 256] {
    let mut table: [u32; 256] = [0; 256];
    let mut table_index: usize = 0;

    while table_index < 256 {
        let byte_value: u32 = forward_sbox[table_index] as u32;

        let substitution_value = if byte_value & 0x80 == 0 {
            (byte_value << 1) & 0xff
        } else {
            ((byte_value << 1) & 0xff) ^ 0x1b
        };
        let forward_value: u32 =
            ((((((byte_value ^ substitution_value) << 8) | byte_value) << 8) | byte_value) << 8)
                | substitution_value;

        table[table_index] = forward_value;
        table_index += 1;
    }
    table
}

/// Generates the logs table by inverting the powers table.
const fn generate_logs_table(powers_table: &[u8; 256]) -> [u8; 256] {
    let mut logs_table: [u8; 256] = [0; 256];
    let mut byte_value: usize = 0;

    while byte_value < 256 {
        let value: usize = powers_table[byte_value] as usize;
        logs_table[value] = byte_value as u8;
        byte_value += 1;
    }
    logs_table
}

/// Generates the powers table.
const fn generate_powers_table() -> [u8; 256] {
    let mut powers_table: [u8; 256] = [0; 256];
    let mut byte_value: usize = 0;
    let mut value: usize = 1;

    while byte_value < 256 {
        powers_table[byte_value] = value as u8;
        value ^= calculate_next_gf28_value(value);
        byte_value += 1;
    }
    powers_table
}

/// Generates the reverse S-Box by inverting the forward S-Box.
const fn generate_reverse_sbox(forward_sbox: &[u8; 256]) -> [u8; 256] {
    let mut reverse_sbox: [u8; 256] = [0; 256];
    let mut sbox_index: usize = 0;

    while sbox_index < 256 {
        let forward_value: u8 = forward_sbox[sbox_index];
        reverse_sbox[forward_value as usize] = sbox_index as u8;
        sbox_index += 1;
    }
    reverse_sbox
}

/// Generates reverse table 0
const fn generate_reverse_table0(
    powers_table: &[u8; 256],
    logs_table: &[u8; 256],
    reverse_sbox: &[u8; 256],
) -> [u32; 256] {
    let mut table: [u32; 256] = [0; 256];
    let mut table_index: usize = 0;

    while table_index < 256 {
        let substitution_value: u32 = reverse_sbox[table_index] as u32;

        let mut reverse_value: u32 = 0;

        if substitution_value != 0 {
            let log_value: usize = logs_table[substitution_value as usize] as usize;

            // GF(2^8) multiplication by 11
            let value1 = ((logs_table[11] as usize) + log_value) % 255;
            let power_value1 = powers_table[value1] as u32;

            // GF(2^8) multiplication by 13
            let value2 = ((logs_table[13] as usize) + log_value) % 255;
            let power_value2 = powers_table[value2] as u32;

            // GF(2^8) multiplication by 9
            let value3 = ((logs_table[9] as usize) + log_value) % 255;
            let power_value3 = powers_table[value3] as u32;

            // GF(2^8) multiplication by 14
            let value4 = ((logs_table[14] as usize) + log_value) % 255;
            let power_value4 = powers_table[value4] as u32;

            reverse_value ^=
                (((((power_value1 << 8) | power_value2) << 8) | power_value3) << 8) | power_value4;
        };
        table[table_index] = reverse_value;
        table_index += 1;
    }
    table
}

/// Generates the round constants.
const fn generate_round_constants() -> [u32; 10] {
    let mut round_constants: [u32; 10] = [0; 10];
    let mut value: usize = 1;
    let mut index: usize = 0;

    while index < 10 {
        round_constants[index] = value as u32;
        value = calculate_next_gf28_value(value);
        index += 1;
    }
    round_constants
}

/// AES block size.
const AES_BLOCK_SIZE: usize = 16;

/// AES supported key sizes.
const AES_SUPPORTED_KEY_SIZES: [usize; 3] = [16, 24, 32];

/// AES powers table.
const AES_POWERS_TABLE: [u8; 256] = generate_powers_table();

/// AES logs table.
const AES_LOGS_TABLE: [u8; 256] = generate_logs_table(&AES_POWERS_TABLE);

/// AES round constants.
const AES_ROUND_CONSTANTS: [u32; 10] = generate_round_constants();

/// AES forward (encryption) S-Box.
const AES_FORWARD_SBOX: [u8; 256] = generate_forward_sbox(&AES_POWERS_TABLE, &AES_LOGS_TABLE);

/// AES reverse (decryption) S-Box.
const AES_REVERSE_SBOX: [u8; 256] = generate_reverse_sbox(&AES_FORWARD_SBOX);

/// AES forward (encryption) tables.
const AES_FORWARD_TABLE0: [u32; 256] = generate_forward_table0(&AES_FORWARD_SBOX);
const AES_FORWARD_TABLE1: [u32; 256] = generate_derived_table(&AES_FORWARD_TABLE0, 8);
const AES_FORWARD_TABLE2: [u32; 256] = generate_derived_table(&AES_FORWARD_TABLE0, 16);
const AES_FORWARD_TABLE3: [u32; 256] = generate_derived_table(&AES_FORWARD_TABLE0, 24);

/// AES reverse (decryption) tables.
const AES_REVERSE_TABLE0: [u32; 256] =
    generate_reverse_table0(&AES_POWERS_TABLE, &AES_LOGS_TABLE, &AES_REVERSE_SBOX);
const AES_REVERSE_TABLE1: [u32; 256] = generate_derived_table(&AES_REVERSE_TABLE0, 8);
const AES_REVERSE_TABLE2: [u32; 256] = generate_derived_table(&AES_REVERSE_TABLE0, 16);
const AES_REVERSE_TABLE3: [u32; 256] = generate_derived_table(&AES_REVERSE_TABLE0, 24);

/// Context for AES encryption.
#[derive(Clone)]
pub struct AesContext {
    /// Decryption round keys.
    decryption_round_keys: Vec<u32>,

    /// Number of decryption round keys.
    number_of_decryption_round_keys: usize,

    /// Encryption round keys.
    encryption_round_keys: Vec<u32>,

    /// Number of encryption round keys.
    number_of_encryption_round_keys: usize,
}

impl AesContext {
    /// Calculates a forward substitution round.
    #[inline(always)]
    fn calculate_forward_substitution_round(
        &self,
        encryption_round_keys: &[u32],
        block_values: &mut [u32],
        cipher_values: &[u32],
    ) {
        let cipher_value0: usize = cipher_values[0] as usize;
        let cipher_value1: usize = cipher_values[1] as usize;
        let cipher_value2: usize = cipher_values[2] as usize;
        let cipher_value3: usize = cipher_values[3] as usize;

        let substitution_value: u32 = self.calculate_forward_substitution_value(
            cipher_value0,
            cipher_value1,
            cipher_value2,
            cipher_value3,
        );
        block_values[0] = encryption_round_keys[0] ^ substitution_value;

        let substitution_value: u32 = self.calculate_forward_substitution_value(
            cipher_value1,
            cipher_value2,
            cipher_value3,
            cipher_value0,
        );
        block_values[1] = encryption_round_keys[1] ^ substitution_value;

        let substitution_value: u32 = self.calculate_forward_substitution_value(
            cipher_value2,
            cipher_value3,
            cipher_value0,
            cipher_value1,
        );
        block_values[2] = encryption_round_keys[2] ^ substitution_value;

        let substitution_value: u32 = self.calculate_forward_substitution_value(
            cipher_value3,
            cipher_value0,
            cipher_value1,
            cipher_value2,
        );
        block_values[3] = encryption_round_keys[3] ^ substitution_value;
    }

    /// Calculates a forward substitution value.
    #[inline(always)]
    fn calculate_forward_substitution_value(
        &self,
        value0: usize,
        value1: usize,
        value2: usize,
        value3: usize,
    ) -> u32 {
        let index0: usize = value0 & 0xff;
        let index1: usize = (value1 >> 8) & 0xff;
        let index2: usize = (value2 >> 16) & 0xff;
        let index3: usize = (value3 >> 24) & 0xff;

        let sbox_value0: u32 = AES_FORWARD_SBOX[index0] as u32;
        let sbox_value1: u32 = AES_FORWARD_SBOX[index1] as u32;
        let sbox_value2: u32 = AES_FORWARD_SBOX[index2] as u32;
        let sbox_value3: u32 = AES_FORWARD_SBOX[index3] as u32;

        (((((sbox_value3 << 8) | sbox_value2) << 8) | sbox_value1) << 8) | sbox_value0
    }

    /// Calculates a forward table round.
    #[inline(always)]
    fn calculate_forward_table_round(
        &self,
        encryption_round_keys: &[u32],
        block_values: &mut [u32],
        cipher_values: &[u32],
    ) {
        let cipher_value0: usize = cipher_values[0] as usize;
        let cipher_value1: usize = cipher_values[1] as usize;
        let cipher_value2: usize = cipher_values[2] as usize;
        let cipher_value3: usize = cipher_values[3] as usize;

        let table_value: u32 = self.calculate_forward_table_value(
            cipher_value0,
            cipher_value1,
            cipher_value2,
            cipher_value3,
        );
        block_values[0] = encryption_round_keys[0] ^ table_value;

        let table_value: u32 = self.calculate_forward_table_value(
            cipher_value1,
            cipher_value2,
            cipher_value3,
            cipher_value0,
        );
        block_values[1] = encryption_round_keys[1] ^ table_value;

        let table_value: u32 = self.calculate_forward_table_value(
            cipher_value2,
            cipher_value3,
            cipher_value0,
            cipher_value1,
        );
        block_values[2] = encryption_round_keys[2] ^ table_value;

        let table_value: u32 = self.calculate_forward_table_value(
            cipher_value3,
            cipher_value0,
            cipher_value1,
            cipher_value2,
        );
        block_values[3] = encryption_round_keys[3] ^ table_value;
    }

    /// Calculates a forward table value.
    #[inline(always)]
    fn calculate_forward_table_value(
        &self,
        value0: usize,
        value1: usize,
        value2: usize,
        value3: usize,
    ) -> u32 {
        let index0: usize = value0 & 0xff;
        let index1: usize = (value1 >> 8) & 0xff;
        let index2: usize = (value2 >> 16) & 0xff;
        let index3: usize = (value3 >> 24) & 0xff;

        AES_FORWARD_TABLE0[index0]
            ^ AES_FORWARD_TABLE1[index1]
            ^ AES_FORWARD_TABLE2[index2]
            ^ AES_FORWARD_TABLE3[index3]
    }

    /// Calculates a reverse substitution round.
    #[inline(always)]
    fn calculate_reverse_substitution_round(
        &self,
        encryption_round_keys: &[u32],
        block_values: &mut [u32],
        cipher_values: &[u32],
    ) {
        let cipher_value0: usize = cipher_values[0] as usize;
        let cipher_value1: usize = cipher_values[1] as usize;
        let cipher_value2: usize = cipher_values[2] as usize;
        let cipher_value3: usize = cipher_values[3] as usize;

        let substitution_value: u32 = self.calculate_reverse_substitution_value(
            cipher_value0,
            cipher_value3,
            cipher_value2,
            cipher_value1,
        );
        block_values[0] = encryption_round_keys[0] ^ substitution_value;

        let substitution_value: u32 = self.calculate_reverse_substitution_value(
            cipher_value1,
            cipher_value0,
            cipher_value3,
            cipher_value2,
        );
        block_values[1] = encryption_round_keys[1] ^ substitution_value;

        let substitution_value: u32 = self.calculate_reverse_substitution_value(
            cipher_value2,
            cipher_value1,
            cipher_value0,
            cipher_value3,
        );
        block_values[2] = encryption_round_keys[2] ^ substitution_value;

        let substitution_value: u32 = self.calculate_reverse_substitution_value(
            cipher_value3,
            cipher_value2,
            cipher_value1,
            cipher_value0,
        );
        block_values[3] = encryption_round_keys[3] ^ substitution_value;
    }

    /// Calculates a reverse substitution value.
    #[inline(always)]
    fn calculate_reverse_substitution_value(
        &self,
        value0: usize,
        value1: usize,
        value2: usize,
        value3: usize,
    ) -> u32 {
        let index0: usize = value0 & 0xff;
        let index1: usize = (value1 >> 8) & 0xff;
        let index2: usize = (value2 >> 16) & 0xff;
        let index3: usize = (value3 >> 24) & 0xff;

        let sbox_value0: u32 = AES_REVERSE_SBOX[index0] as u32;
        let sbox_value1: u32 = AES_REVERSE_SBOX[index1] as u32;
        let sbox_value2: u32 = AES_REVERSE_SBOX[index2] as u32;
        let sbox_value3: u32 = AES_REVERSE_SBOX[index3] as u32;

        (((((sbox_value3 << 8) | sbox_value2) << 8) | sbox_value1) << 8) | sbox_value0
    }

    /// Calculates a reverse table round.
    #[inline(always)]
    fn calculate_reverse_table_round(
        &self,
        encryption_round_keys: &[u32],
        block_values: &mut [u32],
        cipher_values: &[u32],
    ) {
        let cipher_value0: usize = cipher_values[0] as usize;
        let cipher_value1: usize = cipher_values[1] as usize;
        let cipher_value2: usize = cipher_values[2] as usize;
        let cipher_value3: usize = cipher_values[3] as usize;

        let table_value: u32 = self.calculate_reverse_table_value(
            cipher_value0,
            cipher_value3,
            cipher_value2,
            cipher_value1,
        );
        block_values[0] = encryption_round_keys[0] ^ table_value;

        let table_value: u32 = self.calculate_reverse_table_value(
            cipher_value1,
            cipher_value0,
            cipher_value3,
            cipher_value2,
        );
        block_values[1] = encryption_round_keys[1] ^ table_value;

        let table_value: u32 = self.calculate_reverse_table_value(
            cipher_value2,
            cipher_value1,
            cipher_value0,
            cipher_value3,
        );
        block_values[2] = encryption_round_keys[2] ^ table_value;

        let table_value: u32 = self.calculate_reverse_table_value(
            cipher_value3,
            cipher_value2,
            cipher_value1,
            cipher_value0,
        );
        block_values[3] = encryption_round_keys[3] ^ table_value;
    }

    /// Calculates a reverse table value.
    #[inline(always)]
    fn calculate_reverse_table_value(
        &self,
        value0: usize,
        value1: usize,
        value2: usize,
        value3: usize,
    ) -> u32 {
        let index0: usize = value0 & 0xff;
        let index1: usize = (value1 >> 8) & 0xff;
        let index2: usize = (value2 >> 16) & 0xff;
        let index3: usize = (value3 >> 24) & 0xff;

        AES_REVERSE_TABLE0[index0]
            ^ AES_REVERSE_TABLE1[index1]
            ^ AES_REVERSE_TABLE2[index2]
            ^ AES_REVERSE_TABLE3[index3]
    }

    /// Decrypts a 16 byte block (4 32-bit values).
    #[inline(always)]
    fn decrypt_block(&self, block_values: &mut [u32], cipher_values: &mut [u32]) {
        let mut round_key_index: usize = 0;

        for _ in 0..4 {
            block_values[round_key_index] ^= self.decryption_round_keys[round_key_index];
            round_key_index += 1;
        }
        // Note that below the cipher_values and block_values deliberately alternate between calls.
        let number_of_iterations: usize = self.number_of_decryption_round_keys / 2;

        for _ in 1..number_of_iterations {
            self.calculate_reverse_table_round(
                &self.decryption_round_keys[round_key_index..],
                cipher_values,
                block_values,
            );
            round_key_index += 4;

            self.calculate_reverse_table_round(
                &self.decryption_round_keys[round_key_index..],
                block_values,
                cipher_values,
            );
            round_key_index += 4;
        }
        self.calculate_reverse_table_round(
            &self.decryption_round_keys[round_key_index..],
            cipher_values,
            block_values,
        );
        round_key_index += 4;

        self.calculate_reverse_substitution_round(
            &self.decryption_round_keys[round_key_index..],
            block_values,
            cipher_values,
        );
    }

    /// Encrypts a 16 byte block (4 32-bit values).
    #[inline(always)]
    fn encrypt_block(&self, block_values: &mut [u32], cipher_values: &mut [u32]) {
        let mut round_key_index: usize = 0;

        for _ in 0..4 {
            block_values[round_key_index] ^= self.encryption_round_keys[round_key_index];
            round_key_index += 1;
        }
        // Note that below the cipher_values and block_values deliberately alternate between calls.
        let number_of_iterations: usize = self.number_of_encryption_round_keys / 2;

        for _ in 1..number_of_iterations {
            self.calculate_forward_table_round(
                &self.encryption_round_keys[round_key_index..],
                cipher_values,
                block_values,
            );
            round_key_index += 4;

            self.calculate_forward_table_round(
                &self.encryption_round_keys[round_key_index..],
                block_values,
                cipher_values,
            );
            round_key_index += 4;
        }
        self.calculate_forward_table_round(
            &self.encryption_round_keys[round_key_index..],
            cipher_values,
            block_values,
        );
        round_key_index += 4;

        self.calculate_forward_substitution_round(
            &self.encryption_round_keys[round_key_index..],
            block_values,
            cipher_values,
        );
    }

    /// Initializes the context for encryption with a 128 bits key.
    #[inline(always)]
    fn initialize_for_encryption_with_128bits_key(&mut self) {
        let mut round_key_index: usize = 0;

        for round_constant in AES_ROUND_CONSTANTS[0..10].iter() {
            let round_key0: u32 = self.encryption_round_keys[round_key_index];
            let round_key1: u32 = self.encryption_round_keys[round_key_index + 1];
            let round_key2: u32 = self.encryption_round_keys[round_key_index + 2];
            let round_key3: u32 = self.encryption_round_keys[round_key_index + 3];

            let sbox_index: usize = ((round_key3 as usize) >> 8) & 0xff;
            let substitution_value1: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key3 as usize) >> 16) & 0xff;
            let substitution_value2: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key3 as usize) >> 24) & 0xff;
            let substitution_value3: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = (round_key3 as usize) & 0xff;
            let substitution_value4: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let round_key4: u32 = *round_constant
                ^ round_key0
                ^ substitution_value1
                ^ (substitution_value2 << 8)
                ^ (substitution_value3 << 16)
                ^ (substitution_value4 << 24);

            let round_key5: u32 = round_key1 ^ round_key4;
            let round_key6: u32 = round_key2 ^ round_key5;

            self.encryption_round_keys[round_key_index + 4] = round_key4;
            self.encryption_round_keys[round_key_index + 5] = round_key5;
            self.encryption_round_keys[round_key_index + 6] = round_key6;
            self.encryption_round_keys[round_key_index + 7] = round_key3 ^ round_key6;

            round_key_index += 4;
        }
        self.number_of_encryption_round_keys = 10;
    }

    /// Initializes the context for encryption with a 192 bits key.
    #[inline(always)]
    fn initialize_for_encryption_with_192bits_key(&mut self) {
        let mut round_key_index: usize = 0;

        for round_constant in AES_ROUND_CONSTANTS[0..8].iter() {
            let round_key0: u32 = self.encryption_round_keys[round_key_index];
            let round_key1: u32 = self.encryption_round_keys[round_key_index + 1];
            let round_key2: u32 = self.encryption_round_keys[round_key_index + 2];
            let round_key3: u32 = self.encryption_round_keys[round_key_index + 3];
            let round_key4: u32 = self.encryption_round_keys[round_key_index + 4];
            let round_key5: u32 = self.encryption_round_keys[round_key_index + 5];

            let sbox_index: usize = ((round_key5 as usize) >> 8) & 0xff;
            let substitution_value1: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key5 as usize) >> 16) & 0xff;
            let substitution_value2: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key5 as usize) >> 24) & 0xff;
            let substitution_value3: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = (round_key5 as usize) & 0xff;
            let substitution_value4: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let round_key6: u32 = *round_constant
                ^ round_key0
                ^ substitution_value1
                ^ (substitution_value2 << 8)
                ^ (substitution_value3 << 16)
                ^ (substitution_value4 << 24);

            let round_key7: u32 = round_key1 ^ round_key6;
            let round_key8: u32 = round_key2 ^ round_key7;
            let round_key9: u32 = round_key3 ^ round_key8;
            let round_key10: u32 = round_key4 ^ round_key9;

            self.encryption_round_keys[round_key_index + 6] = round_key6;
            self.encryption_round_keys[round_key_index + 7] = round_key7;
            self.encryption_round_keys[round_key_index + 8] = round_key8;
            self.encryption_round_keys[round_key_index + 9] = round_key9;
            self.encryption_round_keys[round_key_index + 10] = round_key10;
            self.encryption_round_keys[round_key_index + 11] = round_key5 ^ round_key10;

            round_key_index += 6;
        }
        self.number_of_encryption_round_keys = 12;
    }

    /// Initializes the context for encryption with a 256 bits key.
    #[inline(always)]
    fn initialize_for_encryption_with_256bits_key(&mut self) {
        let mut round_key_index: usize = 0;

        for round_constant in AES_ROUND_CONSTANTS[0..7].iter() {
            let round_key0: u32 = self.encryption_round_keys[round_key_index];
            let round_key1: u32 = self.encryption_round_keys[round_key_index + 1];
            let round_key2: u32 = self.encryption_round_keys[round_key_index + 2];
            let round_key3: u32 = self.encryption_round_keys[round_key_index + 3];
            let round_key4: u32 = self.encryption_round_keys[round_key_index + 4];
            let round_key5: u32 = self.encryption_round_keys[round_key_index + 5];
            let round_key6: u32 = self.encryption_round_keys[round_key_index + 6];
            let round_key7: u32 = self.encryption_round_keys[round_key_index + 7];

            let sbox_index: usize = ((round_key7 as usize) >> 8) & 0xff;
            let substitution_value1: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key7 as usize) >> 16) & 0xff;
            let substitution_value2: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key7 as usize) >> 24) & 0xff;
            let substitution_value3: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = (round_key7 as usize) & 0xff;
            let substitution_value4: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let round_key8: u32 = *round_constant
                ^ round_key0
                ^ substitution_value1
                ^ (substitution_value2 << 8)
                ^ (substitution_value3 << 16)
                ^ (substitution_value4 << 24);

            let round_key9: u32 = round_key1 ^ round_key8;
            let round_key10: u32 = round_key2 ^ round_key9;
            let round_key11: u32 = round_key3 ^ round_key10;

            let sbox_index: usize = (round_key11 as usize) & 0xff;
            let substitution_value1: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key11 as usize) >> 8) & 0xff;
            let substitution_value2: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key11 as usize) >> 16) & 0xff;
            let substitution_value3: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let sbox_index: usize = ((round_key11 as usize) >> 24) & 0xff;
            let substitution_value4: u32 = AES_FORWARD_SBOX[sbox_index] as u32;

            let round_key12: u32 = round_key4
                ^ substitution_value1
                ^ (substitution_value2 << 8)
                ^ (substitution_value3 << 16)
                ^ (substitution_value4 << 24);

            let round_key13: u32 = round_key5 ^ round_key12;
            let round_key14: u32 = round_key6 ^ round_key13;

            self.encryption_round_keys[round_key_index + 8] = round_key8;
            self.encryption_round_keys[round_key_index + 9] = round_key9;
            self.encryption_round_keys[round_key_index + 10] = round_key10;
            self.encryption_round_keys[round_key_index + 11] = round_key11;
            self.encryption_round_keys[round_key_index + 12] = round_key12;
            self.encryption_round_keys[round_key_index + 13] = round_key13;
            self.encryption_round_keys[round_key_index + 14] = round_key14;
            self.encryption_round_keys[round_key_index + 15] = round_key7 ^ round_key14;

            round_key_index += 8;
        }
        self.number_of_encryption_round_keys = 14;
    }

    /// Calculates the initial CBC-MAC for CCM mode.
    fn crypt_ccm_calculate_initial_cbc_mac(
        &self,
        nonce: &[u8],
        associated_data: &[u8],
        tag_size: usize,
        data_size: usize,
        tag_values: &mut [u32],
    ) {
        let nonce_size: usize = nonce.len();
        let associated_data_size: usize = associated_data.len();

        let l_value: usize = 15 - nonce_size;

        let l_prime: u8 = (l_value - 1) as u8;
        let m_prime: u8 = ((tag_size - 2) / 2) as u8;
        let flags: u8 = ((m_prime & 0x07) << 3)
            | (l_prime & 0x07)
            | if associated_data_size == 0 {
                0x00
            } else {
                0x40
            };

        let mut flags_block_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        flags_block_data[0] = flags;
        flags_block_data[1..1 + nonce_size].copy_from_slice(nonce);

        let mut remaining_data_size: usize = data_size;
        for index in 0..l_value {
            flags_block_data[15 - index] = (remaining_data_size & 0xff) as u8;
            remaining_data_size >>= 8;
        }
        tag_values[0] = bytes_to_u32_le!(flags_block_data, 0);
        tag_values[1] = bytes_to_u32_le!(flags_block_data, 4);
        tag_values[2] = bytes_to_u32_le!(flags_block_data, 8);
        tag_values[3] = bytes_to_u32_le!(flags_block_data, 12);

        let mut cipher_values: [u32; 4] = [0; 4];
        self.encrypt_block(tag_values, &mut cipher_values);

        if associated_data_size > 0 {
            let mut associated_block_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            associated_block_data[0] = (associated_data_size >> 8) as u8;
            associated_block_data[1] = (associated_data_size & 0xff) as u8;

            let mut block_offset: usize = 2;

            for byte_value in associated_data.iter() {
                associated_block_data[block_offset] = *byte_value;
                block_offset += 1;

                if block_offset == AES_BLOCK_SIZE {
                    tag_values[0] ^= bytes_to_u32_le!(associated_block_data, 0);
                    tag_values[1] ^= bytes_to_u32_le!(associated_block_data, 4);
                    tag_values[2] ^= bytes_to_u32_le!(associated_block_data, 8);
                    tag_values[3] ^= bytes_to_u32_le!(associated_block_data, 12);

                    cipher_values.fill(0);
                    self.encrypt_block(tag_values, &mut cipher_values);

                    associated_block_data.fill(0);
                    block_offset = 0;
                }
            }
            if block_offset > 0 {
                tag_values[0] ^= bytes_to_u32_le!(associated_block_data, 0);
                tag_values[1] ^= bytes_to_u32_le!(associated_block_data, 4);
                tag_values[2] ^= bytes_to_u32_le!(associated_block_data, 8);
                tag_values[3] ^= bytes_to_u32_le!(associated_block_data, 12);

                cipher_values.fill(0);
                self.encrypt_block(tag_values, &mut cipher_values);
            }
        }
    }
}

impl CryptContext for AesContext {
    /// Creates a new context.
    fn new() -> Self {
        Self {
            decryption_round_keys: Vec::new(),
            number_of_decryption_round_keys: 0,
            encryption_round_keys: Vec::new(),
            number_of_encryption_round_keys: 0,
        }
    }

    /// Sets the key.
    fn set_key(&mut self, key: &[u8]) -> Result<(), ErrorTrace> {
        let key_size: usize = key.len();

        if !AES_SUPPORTED_KEY_SIZES.contains(&key_size) {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        self.encryption_round_keys = vec![0; 68];

        let mut encryption_round_key_index: usize = 0;

        for offset in (0..key_size).step_by(4) {
            self.encryption_round_keys[encryption_round_key_index] = bytes_to_u32_le!(key, offset);
            encryption_round_key_index += 1;
        }
        if key_size == 16 {
            self.initialize_for_encryption_with_128bits_key();
            self.number_of_decryption_round_keys = 10;
        } else if key_size == 24 {
            self.initialize_for_encryption_with_192bits_key();
            self.number_of_decryption_round_keys = 12;
        } else if key_size == 32 {
            self.initialize_for_encryption_with_256bits_key();
            self.number_of_decryption_round_keys = 14;
        }
        self.decryption_round_keys = vec![0; 68];

        encryption_round_key_index = self.number_of_encryption_round_keys * 4;

        let mut decryption_round_key_index: usize = 0;
        for _ in 0..4 {
            self.decryption_round_keys[decryption_round_key_index] =
                self.encryption_round_keys[encryption_round_key_index];
            decryption_round_key_index += 1;
            encryption_round_key_index += 1;
        }
        encryption_round_key_index -= 8;

        for _ in 1..self.number_of_decryption_round_keys {
            for _ in 0..4 {
                let encryption_round_key: u32 =
                    self.encryption_round_keys[encryption_round_key_index];
                encryption_round_key_index += 1;

                let sbox_index0: usize = (encryption_round_key & 0xff) as usize;
                let sbox_index1: usize = ((encryption_round_key >> 8) & 0xff) as usize;
                let sbox_index2: usize = ((encryption_round_key >> 16) & 0xff) as usize;
                let sbox_index3: usize = ((encryption_round_key >> 24) & 0xff) as usize;

                let byte_value0: usize = AES_FORWARD_SBOX[sbox_index0] as usize;
                let byte_value1: usize = AES_FORWARD_SBOX[sbox_index1] as usize;
                let byte_value2: usize = AES_FORWARD_SBOX[sbox_index2] as usize;
                let byte_value3: usize = AES_FORWARD_SBOX[sbox_index3] as usize;

                let decryption_round_key: u32 = AES_REVERSE_TABLE0[byte_value0]
                    ^ AES_REVERSE_TABLE1[byte_value1]
                    ^ AES_REVERSE_TABLE2[byte_value2]
                    ^ AES_REVERSE_TABLE3[byte_value3];

                self.decryption_round_keys[decryption_round_key_index] = decryption_round_key;
                decryption_round_key_index += 1;
            }
            encryption_round_key_index -= 8;
        }
        for _ in 0..4 {
            self.decryption_round_keys[decryption_round_key_index] =
                self.encryption_round_keys[encryption_round_key_index];
            decryption_round_key_index += 1;
            encryption_round_key_index += 1;
        }
        Ok(())
    }
}

impl CryptCbc for AesContext {
    /// Decrypts data using CBC (Cipher Block Chaining) mode.
    fn decrypt_cbc(
        &self,
        initialization_vector: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.decryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        if initialization_vector.len() < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid initialization vector value too small"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data size value too small"
            ));
        }
        if encrypted_data_size % AES_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid encrypted data size value not a multitude of block size: {}",
                AES_BLOCK_SIZE
            )));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let mut block_values: [u32; 4] = [0; 4];
        let mut cipher_values: [u32; 4] = [0; 4];
        let mut initialization_vector_values: [u32; 4] = [0; 4];
        let mut data_offset: usize = 0;

        initialization_vector_values[0] = bytes_to_u32_le!(initialization_vector, 0);
        initialization_vector_values[1] = bytes_to_u32_le!(initialization_vector, 4);
        initialization_vector_values[2] = bytes_to_u32_le!(initialization_vector, 8);
        initialization_vector_values[3] = bytes_to_u32_le!(initialization_vector, 12);

        for block_data in encrypted_data.chunks_exact(AES_BLOCK_SIZE) {
            let input_value0: u32 = bytes_to_u32_le!(block_data, 0);
            let input_value1: u32 = bytes_to_u32_le!(block_data, 4);
            let input_value2: u32 = bytes_to_u32_le!(block_data, 8);
            let input_value3: u32 = bytes_to_u32_le!(block_data, 12);

            block_values[0] = input_value0;
            block_values[1] = input_value1;
            block_values[2] = input_value2;
            block_values[3] = input_value3;

            self.decrypt_block(&mut block_values, &mut cipher_values);

            for (index, block_value) in block_values.iter().enumerate() {
                let output_value: u32 = *block_value ^ initialization_vector_values[index];

                let data_end_offset: usize = data_offset + 4;
                data[data_offset..data_end_offset].copy_from_slice(&output_value.to_le_bytes());

                data_offset = data_end_offset;
            }
            initialization_vector_values[0] = input_value0;
            initialization_vector_values[1] = input_value1;
            initialization_vector_values[2] = input_value2;
            initialization_vector_values[3] = input_value3;
        }
        Ok(())
    }

    /// Encrypts data using CBC (Cipher Block Chaining) mode.
    fn encrypt_cbc(
        &self,
        initialization_vector: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.encryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        if initialization_vector.len() < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid initialization vector value too small"
            ));
        }
        let data_size: usize = data.len();

        if data_size < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid data size value too small"
            ));
        }
        if data_size % AES_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid data size value not a multitude of block size: {}",
                AES_BLOCK_SIZE
            )));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let mut block_values: [u32; 4] = [0; 4];
        let mut cipher_values: [u32; 4] = [0; 4];
        let mut initialization_vector_values: [u32; 4] = [0; 4];
        let mut data_offset: usize = 0;

        initialization_vector_values[0] = bytes_to_u32_le!(initialization_vector, 0);
        initialization_vector_values[1] = bytes_to_u32_le!(initialization_vector, 4);
        initialization_vector_values[2] = bytes_to_u32_le!(initialization_vector, 8);
        initialization_vector_values[3] = bytes_to_u32_le!(initialization_vector, 12);

        for block_data in data.chunks_exact(AES_BLOCK_SIZE) {
            block_values[0] = bytes_to_u32_le!(block_data, 0) ^ initialization_vector_values[0];
            block_values[1] = bytes_to_u32_le!(block_data, 4) ^ initialization_vector_values[1];
            block_values[2] = bytes_to_u32_le!(block_data, 8) ^ initialization_vector_values[2];
            block_values[3] = bytes_to_u32_le!(block_data, 12) ^ initialization_vector_values[3];

            self.encrypt_block(&mut block_values, &mut cipher_values);

            for block_value in block_values.iter() {
                let data_end_offset: usize = data_offset + 4;
                encrypted_data[data_offset..data_end_offset]
                    .copy_from_slice(&block_value.to_le_bytes());

                data_offset = data_end_offset;
            }
            initialization_vector_values[0] = block_values[0];
            initialization_vector_values[1] = block_values[1];
            initialization_vector_values[2] = block_values[2];
            initialization_vector_values[3] = block_values[3];
        }
        Ok(())
    }
}

impl CryptCcm for AesContext {
    /// Decrypts data using CCM (Counter with CBC-MAC) mode.
    fn decrypt_ccm(
        &self,
        nonce: &[u8],
        associated_data: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.decryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let nonce_size: usize = nonce.len();

        if nonce_size < 7 || nonce_size > 13 {
            return Err(keramics_core::error_trace_new!("Unsupported nonce size"));
        }
        let associated_data_size: usize = associated_data.len();

        if associated_data_size >= 65280 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported associated data size"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid data value too small",
            )));
        }
        let tag_size: usize = tag.len();

        if !(4..=16).contains(&tag_size) || (tag_size % 2) != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported tag size"));
        }
        let l_value: usize = 15 - nonce_size;
        let l_prime: u8 = (l_value - 1) as u8;

        let mut cipher_values: [u32; 4] = [0; 4];

        let mut counter_block_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        counter_block_data[0] = l_prime;
        counter_block_data[1..1 + nonce_size].copy_from_slice(nonce);

        let mut counter: usize = 0;
        let mut data_offset: usize = 0;

        for block_data in encrypted_data.chunks(AES_BLOCK_SIZE) {
            counter += 1;

            let mut remaining_counter: usize = counter;
            for index in 0..l_value {
                counter_block_data[15 - index] = (remaining_counter & 0xff) as u8;
                remaining_counter >>= 8;
            }
            let mut counter_values: [u32; 4] = [
                bytes_to_u32_le!(counter_block_data, 0),
                bytes_to_u32_le!(counter_block_data, 4),
                bytes_to_u32_le!(counter_block_data, 8),
                bytes_to_u32_le!(counter_block_data, 12),
            ];
            cipher_values.fill(0);
            self.encrypt_block(&mut counter_values, &mut cipher_values);

            let mut ctr_keystream: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            ctr_keystream[0..4].copy_from_slice(&counter_values[0].to_le_bytes());
            ctr_keystream[4..8].copy_from_slice(&counter_values[1].to_le_bytes());
            ctr_keystream[8..12].copy_from_slice(&counter_values[2].to_le_bytes());
            ctr_keystream[12..16].copy_from_slice(&counter_values[3].to_le_bytes());

            let block_data_size: usize = block_data.len();

            for index in 0..block_data_size {
                data[data_offset + index] = block_data[index] ^ ctr_keystream[index];
            }
            data_offset += block_data_size;
        }
        let mut tag_values: [u32; 4] = [0; 4];

        self.crypt_ccm_calculate_initial_cbc_mac(
            nonce,
            associated_data,
            tag_size,
            encrypted_data_size,
            &mut tag_values,
        );
        for block_data in data[0..encrypted_data_size].chunks(AES_BLOCK_SIZE) {
            let block_data_size: usize = block_data.len();
            let mut padded_plaintext: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            padded_plaintext[0..block_data_size].copy_from_slice(block_data);

            tag_values[0] ^= bytes_to_u32_le!(padded_plaintext, 0);
            tag_values[1] ^= bytes_to_u32_le!(padded_plaintext, 4);
            tag_values[2] ^= bytes_to_u32_le!(padded_plaintext, 8);
            tag_values[3] ^= bytes_to_u32_le!(padded_plaintext, 12);

            cipher_values.fill(0);
            self.encrypt_block(&mut tag_values, &mut cipher_values);
        }
        let mut counter_block_data_zero: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        counter_block_data_zero[0] = l_prime;
        counter_block_data_zero[1..1 + nonce_size].copy_from_slice(nonce);

        let mut masked_tag_values: [u32; 4] = [
            bytes_to_u32_le!(counter_block_data_zero, 0),
            bytes_to_u32_le!(counter_block_data_zero, 4),
            bytes_to_u32_le!(counter_block_data_zero, 8),
            bytes_to_u32_le!(counter_block_data_zero, 12),
        ];
        cipher_values.fill(0);
        self.encrypt_block(&mut masked_tag_values, &mut cipher_values);

        tag_values[0] ^= masked_tag_values[0];
        tag_values[1] ^= masked_tag_values[1];
        tag_values[2] ^= masked_tag_values[2];
        tag_values[3] ^= masked_tag_values[3];

        let mut tag_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        tag_data[0..4].copy_from_slice(&tag_values[0].to_le_bytes());
        tag_data[4..8].copy_from_slice(&tag_values[1].to_le_bytes());
        tag_data[8..12].copy_from_slice(&tag_values[2].to_le_bytes());
        tag_data[12..16].copy_from_slice(&tag_values[3].to_le_bytes());

        tag.copy_from_slice(&tag_data[0..tag_size]);

        Ok(())
    }

    /// Encrypts data using CCM (Counter with CBC-MAC) mode.
    fn encrypt_ccm(
        &self,
        nonce: &[u8],
        associated_data: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
        tag: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if self.encryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let nonce_size: usize = nonce.len();
        let data_size: usize = data.len();

        if nonce_size < 7 || nonce_size > 13 {
            return Err(keramics_core::error_trace_new!("Unsupported nonce size"));
        }
        let associated_data_size: usize = associated_data.len();

        if associated_data_size >= 65280 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported associated data size"
            ));
        }
        let tag_size: usize = tag.len();

        if !(4..=16).contains(&tag_size) || (tag_size % 2) != 0 {
            return Err(keramics_core::error_trace_new!("Unsupported tag size"));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let l_value: usize = 15 - nonce_size;
        let l_prime: u8 = (l_value - 1) as u8;

        let maximum_data_size: u64 = if l_value >= 8 {
            u64::MAX
        } else {
            (1 << (l_value * 8)) - 1
        };
        if (data_size as u64) > maximum_data_size {
            return Err(keramics_core::error_trace_new!(
                "Nonce size not supported for data size"
            ));
        }
        let mut tag_values: [u32; 4] = [0; 4];

        self.crypt_ccm_calculate_initial_cbc_mac(
            nonce,
            associated_data,
            tag_size,
            data_size,
            &mut tag_values,
        );
        let mut counter_block_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        counter_block_data[0] = l_prime;
        counter_block_data[1..1 + nonce_size].copy_from_slice(nonce);

        let mut masked_tag_values: [u32; 4] = [
            bytes_to_u32_le!(counter_block_data, 0),
            bytes_to_u32_le!(counter_block_data, 4),
            bytes_to_u32_le!(counter_block_data, 8),
            bytes_to_u32_le!(counter_block_data, 12),
        ];
        let mut cipher_values: [u32; 4] = [0; 4];
        self.encrypt_block(&mut masked_tag_values, &mut cipher_values);

        let mut counter: usize = 0;
        let mut data_offset: usize = 0;

        for block_data in data.chunks(AES_BLOCK_SIZE) {
            counter += 1;

            let mut remaining_counter: usize = counter;
            for index in 0..l_value {
                counter_block_data[15 - index] = (remaining_counter & 0xff) as u8;
                remaining_counter >>= 8;
            }
            let mut counter_values: [u32; 4] = [
                bytes_to_u32_le!(counter_block_data, 0),
                bytes_to_u32_le!(counter_block_data, 4),
                bytes_to_u32_le!(counter_block_data, 8),
                bytes_to_u32_le!(counter_block_data, 12),
            ];
            cipher_values.fill(0);
            self.encrypt_block(&mut counter_values, &mut cipher_values);

            let block_data_size: usize = block_data.len();

            let mut padded_plaintext: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            padded_plaintext[0..block_data_size].copy_from_slice(block_data);

            tag_values[0] ^= bytes_to_u32_le!(padded_plaintext, 0);
            tag_values[1] ^= bytes_to_u32_le!(padded_plaintext, 4);
            tag_values[2] ^= bytes_to_u32_le!(padded_plaintext, 8);
            tag_values[3] ^= bytes_to_u32_le!(padded_plaintext, 12);

            cipher_values.fill(0);
            self.encrypt_block(&mut tag_values, &mut cipher_values);

            let mut ctr_keystream: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            ctr_keystream[0..4].copy_from_slice(&counter_values[0].to_le_bytes());
            ctr_keystream[4..8].copy_from_slice(&counter_values[1].to_le_bytes());
            ctr_keystream[8..12].copy_from_slice(&counter_values[2].to_le_bytes());
            ctr_keystream[12..16].copy_from_slice(&counter_values[3].to_le_bytes());

            for index in 0..block_data_size {
                encrypted_data[data_offset + index] = block_data[index] ^ ctr_keystream[index];
            }
            data_offset += block_data_size;
        }
        tag_values[0] ^= masked_tag_values[0];
        tag_values[1] ^= masked_tag_values[1];
        tag_values[2] ^= masked_tag_values[2];
        tag_values[3] ^= masked_tag_values[3];

        let mut tag_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
        tag_data[0..4].copy_from_slice(&tag_values[0].to_le_bytes());
        tag_data[4..8].copy_from_slice(&tag_values[1].to_le_bytes());
        tag_data[8..12].copy_from_slice(&tag_values[2].to_le_bytes());
        tag_data[12..16].copy_from_slice(&tag_values[3].to_le_bytes());

        tag.copy_from_slice(&tag_data[0..tag_size]);

        Ok(())
    }
}

impl CryptEcb for AesContext {
    /// Decrypts data using ECB (Electronic CodeBook) mode.
    fn decrypt_ecb(&self, encrypted_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.decryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data size value too small"
            ));
        }
        if encrypted_data_size % AES_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid encrypted data size value not a multitude of block size: {}",
                AES_BLOCK_SIZE
            )));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let mut block_values: [u32; 4] = [0; 4];
        let mut cipher_values: [u32; 4] = [0; 4];
        let mut data_offset: usize = 0;

        for block_data in encrypted_data.chunks_exact(AES_BLOCK_SIZE) {
            block_values[0] = bytes_to_u32_le!(block_data, 0);
            block_values[1] = bytes_to_u32_le!(block_data, 4);
            block_values[2] = bytes_to_u32_le!(block_data, 8);
            block_values[3] = bytes_to_u32_le!(block_data, 12);

            self.decrypt_block(&mut block_values, &mut cipher_values);

            for block_value in block_values.iter() {
                let data_end_offset: usize = data_offset + 4;
                data[data_offset..data_end_offset].copy_from_slice(&block_value.to_le_bytes());

                data_offset = data_end_offset;
            }
        }
        Ok(())
    }

    /// Encrypts data using ECB (Electronic CodeBook) mode.
    fn encrypt_ecb(&self, data: &[u8], encrypted_data: &mut [u8]) -> Result<(), ErrorTrace> {
        if self.encryption_round_keys.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Invalid context - key was not set"
            ));
        }
        let data_size: usize = data.len();

        if data_size < AES_BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid data size value too small"
            ));
        }
        if data_size % AES_BLOCK_SIZE != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid data size value not a multitude of block size: {}",
                AES_BLOCK_SIZE
            )));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let mut block_values: [u32; 4] = [0; 4];
        let mut cipher_values: [u32; 4] = [0; 4];
        let mut data_offset: usize = 0;

        for block_data in data.chunks_exact(AES_BLOCK_SIZE) {
            block_values[0] = bytes_to_u32_le!(block_data, 0);
            block_values[1] = bytes_to_u32_le!(block_data, 4);
            block_values[2] = bytes_to_u32_le!(block_data, 8);
            block_values[3] = bytes_to_u32_le!(block_data, 12);

            self.encrypt_block(&mut block_values, &mut cipher_values);

            for block_value in block_values.iter() {
                let data_end_offset: usize = data_offset + 4;
                encrypted_data[data_offset..data_end_offset]
                    .copy_from_slice(&block_value.to_le_bytes());

                data_offset = data_end_offset;
            }
        }
        Ok(())
    }
}

/// Context for AES-CBC
pub type AesCbcContext = CbcContext<AesContext, 16>;

/// Context for AES-CCM
pub type AesCcmContext = CcmContext<AesContext>;

/// Context for AES-XTS
pub type AesXtsContext = XtsContext<AesContext, 16>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt_cbc() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        aes_context.set_key(&key)?;

        let initialization_vector: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let encrypted_data: [u8; 16] = [
            0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9,
            0x19, 0x7d,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        aes_context.decrypt_cbc(&initialization_vector, &encrypted_data, &mut data)?;

        let expected_data: [u8; 16] = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
            0x17, 0x2a,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decrypt_cbc_without_key() {
        let aes_context: AesContext = AesContext::new();

        let initialization_vector: [u8; 16] = [0; 16];
        let encrypted_data: [u8; 16] = [0; 16];
        let mut data: Vec<u8> = vec![0; 16];

        let result = aes_context.decrypt_cbc(&initialization_vector, &encrypted_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_cbc_with_unsupported_initialization_vector() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let initialization_vector: [u8; 15] = [0; 15];
        let encrypted_data: [u8; 16] = [0; 16];
        let mut data: Vec<u8> = vec![0; 16];

        let result = aes_context.decrypt_cbc(&initialization_vector, &encrypted_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_cbc_with_unsupported_encrypted_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let initialization_vector: [u8; 16] = [0; 16];
        let encrypted_data: [u8; 15] = [0; 15];
        let mut data: Vec<u8> = vec![0; 16];

        let result = aes_context.decrypt_cbc(&initialization_vector, &encrypted_data, &mut data);
        assert!(result.is_err());

        let encrypted_data: [u8; 32] = [0; 32];
        let result =
            aes_context.decrypt_cbc(&initialization_vector, &encrypted_data[0..17], &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_cbc_with_unsupported_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let initialization_vector: [u8; 16] = [0; 16];
        let encrypted_data: [u8; 16] = [0; 16];
        let mut data: Vec<u8> = vec![0; 8];

        let result = aes_context.decrypt_cbc(&initialization_vector, &encrypted_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_cbc() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        aes_context.set_key(&key)?;

        let initialization_vector: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        let data: [u8; 16] = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
            0x17, 0x2a,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 16];
        aes_context.encrypt_cbc(&initialization_vector, &data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 16] = [
            0x76, 0x49, 0xab, 0xac, 0x81, 0x19, 0xb2, 0x46, 0xce, 0xe9, 0x8e, 0x9b, 0x12, 0xe9,
            0x19, 0x7d,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_cbc_without_key() {
        let aes_context: AesContext = AesContext::new();

        let initialization_vector: [u8; 16] = [0; 16];
        let data: [u8; 16] = [0; 16];
        let mut encrypted_data: Vec<u8> = vec![0; 16];

        let result = aes_context.encrypt_cbc(&initialization_vector, &data, &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_cbc_with_unsupported_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let initialization_vector: [u8; 16] = [0; 16];
        let data: [u8; 15] = [0; 15];
        let mut encrypted_data: Vec<u8> = vec![0; 16];

        let result = aes_context.encrypt_cbc(&initialization_vector, &data, &mut encrypted_data);
        assert!(result.is_err());

        let data: [u8; 32] = [0; 32];
        let result =
            aes_context.encrypt_cbc(&initialization_vector, &data[0..17], &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_cbc_with_unsupported_encrypted_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let initialization_vector: [u8; 16] = [0; 16];
        let data: [u8; 16] = [0; 16];
        let mut encrypted_data: Vec<u8> = vec![0; 8];

        let result = aes_context.encrypt_cbc(&initialization_vector, &data, &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_ecb_with_128bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        // This test uses the FIPS-197 test vector.
        let key: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        aes_context.set_key(&key)?;

        let encrypted_data: [u8; 16] = [
            0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
            0xc5, 0x5a,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        aes_context.decrypt_ecb(&encrypted_data, &mut data)?;

        let expected_data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decrypt_ecb_with_192bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        // This test uses the FIPS-197 test vector.
        let key: [u8; 24] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        aes_context.set_key(&key)?;

        let encrypted_data: [u8; 16] = [
            0xdd, 0xa9, 0x7c, 0xa4, 0x86, 0x4c, 0xdf, 0xe0, 0x6e, 0xaf, 0x70, 0xa0, 0xec, 0x0d,
            0x71, 0x91,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        aes_context.decrypt_ecb(&encrypted_data, &mut data)?;

        let expected_data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decrypt_ecb_with_256bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        // This test uses the FIPS-197 test vector.
        let key: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        aes_context.set_key(&key)?;

        let encrypted_data: [u8; 16] = [
            0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf, 0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49,
            0x60, 0x89,
        ];
        let mut data: Vec<u8> = vec![0; 16];
        aes_context.decrypt_ecb(&encrypted_data, &mut data)?;

        let expected_data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_decrypt_ecb_without_key() {
        let aes_context: AesContext = AesContext::new();

        let encrypted_data: [u8; 16] = [0; 16];
        let mut data: Vec<u8> = vec![0; 16];

        let result = aes_context.decrypt_ecb(&encrypted_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_ecb_with_unsupported_encrypted_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [0; 16];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let encrypted_data: [u8; 15] = [0; 15];
        let mut data: Vec<u8> = vec![0; 16];

        let result = aes_context.decrypt_ecb(&encrypted_data, &mut data);
        assert!(result.is_err());

        let encrypted_data: [u8; 32] = [0; 32];
        let result = aes_context.decrypt_ecb(&encrypted_data[0..17], &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_ecb_with_unsupported_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [0; 16];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let encrypted_data: [u8; 16] = [0; 16];
        let mut data: Vec<u8> = vec![0; 8];

        let result = aes_context.decrypt_ecb(&encrypted_data, &mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_ecb_with_128bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];
        aes_context.set_key(&key)?;

        let data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 16];
        aes_context.encrypt_ecb(&data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 16] = [
            0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
            0xc5, 0x5a,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_ecb_with_192bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 24] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        aes_context.set_key(&key)?;

        let data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 16];
        aes_context.encrypt_ecb(&data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 16] = [
            0xdd, 0xa9, 0x7c, 0xa4, 0x86, 0x4c, 0xdf, 0xe0, 0x6e, 0xaf, 0x70, 0xa0, 0xec, 0x0d,
            0x71, 0x91,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_ecb_with_256bits_key() -> Result<(), ErrorTrace> {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 32] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        aes_context.set_key(&key)?;

        let data: [u8; 16] = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 16];
        aes_context.encrypt_ecb(&data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 16] = [
            0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf, 0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49,
            0x60, 0x89,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_ecb_without_key() {
        let aes_context: AesContext = AesContext::new();

        let data: [u8; 16] = [0; 16];
        let mut encrypted_data: Vec<u8> = vec![0; 16];

        let result = aes_context.encrypt_ecb(&data, &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_ecb_with_unsupported_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [0; 16];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let data: [u8; 15] = [0; 15];
        let mut encrypted_data: Vec<u8> = vec![0; 16];

        let result = aes_context.encrypt_ecb(&data, &mut encrypted_data);
        assert!(result.is_err());

        let data: [u8; 32] = [0; 32];
        let result = aes_context.encrypt_ecb(&data[0..17], &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_ecb_with_unsupported_encrypted_data_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 16] = [0; 16];

        let result = aes_context.set_key(&key);
        assert!(result.is_ok());

        let data: [u8; 16] = [0; 16];
        let mut encrypted_data: Vec<u8> = vec![0; 8];

        let result = aes_context.encrypt_ecb(&data, &mut encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_key_with_unsupported_key_size() {
        let mut aes_context: AesContext = AesContext::new();

        let key: [u8; 8] = [0; 8];
        let result = aes_context.set_key(&key);
        assert!(result.is_err());

        let key: [u8; 33] = [0; 33];
        let result = aes_context.set_key(&key);
        assert!(result.is_err());
    }
}
