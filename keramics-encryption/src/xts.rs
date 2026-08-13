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

use super::traits::CryptEcb;

/// XTS supported key sizes.
const XTS_SUPPORTED_KEY_SIZES: [usize; 2] = [16, 32];

/// Context for XTS (XEX-based tweaked-codebook mode with ciphertext stealing) encryption and decryption.
pub struct XtsContext<T: CryptEcb, const BLOCK_SIZE: usize> {
    /// Encryption and decryption context.
    context: T,

    /// Tweak encryption and decryption context.
    tweak_context: T,
}

impl<T: CryptEcb, const BLOCK_SIZE: usize> XtsContext<T, BLOCK_SIZE> {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            context: T::new(),
            tweak_context: T::new(),
        }
    }

    /// Decrypts data using XTS (XEX-based tweaked-codebook mode with ciphertext stealing) mode.
    pub fn decrypt_xts(
        &self,
        tweak_value: &[u8],
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let tweak_value_size: usize = tweak_value.len();

        if tweak_value_size < BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid tweak value size value too small"
            ));
        }
        let encrypted_data_size: usize = encrypted_data.len();

        if encrypted_data_size < BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data size value too small"
            ));
        }
        if encrypted_data_size > data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid data value too small"
            ));
        }
        let mut block_data: Vec<u8> = vec![0; BLOCK_SIZE];
        let mut encrypted_tweak_value: Vec<u8> = vec![0; BLOCK_SIZE];
        let mut encrypted_tweak_value_copy: Vec<u8> = vec![0; BLOCK_SIZE];

        match self
            .tweak_context
            .encrypt_ecb(&tweak_value[0..BLOCK_SIZE], &mut encrypted_tweak_value)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to encrypt tweak value");
                return Err(error);
            }
        }
        let mut data_offset: usize = 0;
        let mut remaining_data_size: usize = encrypted_data_size;

        while remaining_data_size >= BLOCK_SIZE {
            if remaining_data_size < BLOCK_SIZE + BLOCK_SIZE && remaining_data_size != BLOCK_SIZE {
                encrypted_tweak_value_copy.copy_from_slice(&encrypted_tweak_value);

                let mut carry_bit: u8 = 0;
                for tweak_byte in encrypted_tweak_value.iter_mut() {
                    let byte_value: u8 = (*tweak_byte << 1) | carry_bit;
                    carry_bit = *tweak_byte >> 7;
                    *tweak_byte = byte_value;
                }
                if carry_bit > 0 {
                    encrypted_tweak_value[0] ^= 0x87;
                }
            }
            let data_end_offset: usize = data_offset + BLOCK_SIZE;

            block_data.copy_from_slice(&encrypted_data[data_offset..data_end_offset]);

            for (data_byte, tweak_byte) in block_data.iter_mut().zip(&encrypted_tweak_value) {
                *data_byte ^= tweak_byte;
            }
            match self
                .context
                .decrypt_ecb(&block_data, &mut data[data_offset..])
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decrypt block data");
                    return Err(error);
                }
            }
            for (data_byte, tweak_byte) in data[data_offset..data_end_offset]
                .iter_mut()
                .zip(&encrypted_tweak_value)
            {
                *data_byte ^= tweak_byte;
            }
            remaining_data_size -= BLOCK_SIZE;

            let mut carry_bit: u8 = 0;
            for tweak_byte in encrypted_tweak_value.iter_mut() {
                let byte_value: u8 = (*tweak_byte << 1) | carry_bit;
                carry_bit = *tweak_byte >> 7;
                *tweak_byte = byte_value;
            }
            if carry_bit > 0 {
                encrypted_tweak_value[0] ^= 0x87;
            }
            data_offset = data_end_offset;
        }
        if remaining_data_size > 0 {
            encrypted_tweak_value.copy_from_slice(&encrypted_tweak_value_copy);

            // Swap the data of the previous block with the remaining data
            let previous_block_offset: usize = data_offset - BLOCK_SIZE;
            let previous_block_end_offset: usize = previous_block_offset + remaining_data_size;

            let final_block_offset: usize = data_offset;

            block_data[0..remaining_data_size]
                .copy_from_slice(&encrypted_data[final_block_offset..]);
            block_data[remaining_data_size..]
                .copy_from_slice(&data[previous_block_end_offset..final_block_offset]);

            data.copy_within(
                previous_block_offset..previous_block_end_offset,
                final_block_offset,
            );

            for (data_byte, tweak_byte) in block_data.iter_mut().zip(&encrypted_tweak_value) {
                *data_byte ^= tweak_byte;
            }
            match self
                .context
                .decrypt_ecb(&block_data, &mut data[previous_block_offset..])
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to decrypt block data");
                    return Err(error);
                }
            }
            for (data_byte, tweak_byte) in data[previous_block_offset..final_block_offset]
                .iter_mut()
                .zip(&encrypted_tweak_value)
            {
                *data_byte ^= tweak_byte;
            }
        }
        Ok(())
    }

    /// Encrypts data using XTS (XEX-based tweaked-codebook mode with ciphertext stealing) mode.
    pub fn encrypt_xts(
        &self,
        tweak_value: &[u8],
        data: &[u8],
        encrypted_data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let tweak_value_size: usize = tweak_value.len();

        if tweak_value_size < BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid tweak value size value too small"
            ));
        }
        let data_size: usize = data.len();

        if data_size < BLOCK_SIZE {
            return Err(keramics_core::error_trace_new!(
                "Invalid data size value too small"
            ));
        }
        if data_size > encrypted_data.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid encrypted data value too small"
            ));
        }
        let mut block_data: Vec<u8> = vec![0; BLOCK_SIZE];
        let mut encrypted_tweak_value: Vec<u8> = vec![0; BLOCK_SIZE];

        match self
            .tweak_context
            .encrypt_ecb(&tweak_value[0..BLOCK_SIZE], &mut encrypted_tweak_value)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to encrypt tweak value");
                return Err(error);
            }
        }
        let mut data_offset: usize = 0;
        let mut remaining_data_size: usize = data_size;

        while remaining_data_size >= BLOCK_SIZE {
            let data_end_offset: usize = data_offset + BLOCK_SIZE;

            block_data.copy_from_slice(&data[data_offset..data_end_offset]);

            for (data_byte, tweak_byte) in block_data.iter_mut().zip(&encrypted_tweak_value) {
                *data_byte ^= tweak_byte;
            }
            match self
                .context
                .encrypt_ecb(&block_data, &mut encrypted_data[data_offset..])
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to encrypt block data");
                    return Err(error);
                }
            }
            for (data_byte, tweak_byte) in encrypted_data[data_offset..data_end_offset]
                .iter_mut()
                .zip(&encrypted_tweak_value)
            {
                *data_byte ^= tweak_byte;
            }
            remaining_data_size -= BLOCK_SIZE;

            let mut carry_bit: u8 = 0;
            for tweak_byte in encrypted_tweak_value.iter_mut() {
                let byte_value: u8 = (*tweak_byte << 1) | carry_bit;
                carry_bit = *tweak_byte >> 7;
                *tweak_byte = byte_value;
            }
            if carry_bit > 0 {
                encrypted_tweak_value[0] ^= 0x87;
            }
            data_offset = data_end_offset;
        }
        if remaining_data_size > 0 {
            // Swap the data of the previous block with the remaining data
            let previous_block_offset: usize = data_offset - BLOCK_SIZE;
            let previous_block_end_offset: usize = previous_block_offset + remaining_data_size;

            let final_block_offset: usize = data_offset;

            block_data[0..remaining_data_size].copy_from_slice(&data[final_block_offset..]);
            block_data[remaining_data_size..]
                .copy_from_slice(&encrypted_data[previous_block_end_offset..final_block_offset]);

            encrypted_data.copy_within(
                previous_block_offset..previous_block_end_offset,
                final_block_offset,
            );
            for (data_byte, tweak_byte) in block_data.iter_mut().zip(&encrypted_tweak_value) {
                *data_byte ^= tweak_byte;
            }
            match self
                .context
                .encrypt_ecb(&block_data, &mut encrypted_data[previous_block_offset..])
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to encrypt block data");
                    return Err(error);
                }
            }
            for (data_byte, tweak_byte) in encrypted_data[previous_block_offset..final_block_offset]
                .iter_mut()
                .zip(&encrypted_tweak_value)
            {
                *data_byte ^= tweak_byte;
            }
        }
        Ok(())
    }

    /// Sets the keys.
    pub fn set_keys(&mut self, key: &[u8], tweak_key: &[u8]) -> Result<(), ErrorTrace> {
        let key_size: usize = key.len();
        let tweak_key_size: usize = tweak_key.len();

        if !XTS_SUPPORTED_KEY_SIZES.contains(&key_size) {
            return Err(keramics_core::error_trace_new!("Unsupported key size"));
        }
        if !XTS_SUPPORTED_KEY_SIZES.contains(&tweak_key_size) {
            return Err(keramics_core::error_trace_new!(
                "Unsupported tweak key size"
            ));
        }
        if key_size != tweak_key_size {
            return Err(keramics_core::error_trace_new!(
                "Key and tweak key of different sizes"
            ));
        }
        match self.context.set_key(key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in context");
                return Err(error);
            }
        }
        match self.tweak_context.set_key(tweak_key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to set key in tweak context");
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::aes::AesXtsContext;

    #[test]
    fn test_decrypt_xts() -> Result<(), ErrorTrace> {
        let tweak_value: [u8; 16] = [
            0xb2, 0xf8, 0xc6, 0x37, 0x4e, 0xb2, 0x75, 0xc1, 0x74, 0x4e, 0x85, 0xaa, 0x21, 0xf8,
            0xea, 0x6b,
        ];
        let mut xts_context: AesXtsContext = AesXtsContext::new();

        let key: [u8; 16] = [
            0x8a, 0xfb, 0x90, 0xc2, 0xec, 0x92, 0x4c, 0x4b, 0x0b, 0x0b, 0xd8, 0x40, 0xfb, 0x1e,
            0xfc, 0x84,
        ];
        let tweak_key: [u8; 16] = [
            0x2c, 0x93, 0x85, 0xa1, 0x4d, 0x1c, 0xa9, 0x5b, 0xd4, 0xd1, 0x2c, 0xbf, 0x9a, 0xb5,
            0x88, 0xed,
        ];
        xts_context.set_keys(&key, &tweak_key)?;

        let encrypted_data: [u8; 25] = [
            0xf4, 0xbb, 0xaa, 0x8e, 0xbd, 0x48, 0x0d, 0x2a, 0x2a, 0x37, 0x1b, 0xea, 0xb3, 0xd8,
            0xb3, 0x87, 0xc0, 0x22, 0x82, 0x67, 0x8c, 0x60, 0x00, 0x22, 0x7b,
        ];
        let mut data: Vec<u8> = vec![0; 25];
        xts_context.decrypt_xts(&tweak_value, &encrypted_data, &mut data)?;

        let expected_data: [u8; 25] = [
            0xd9, 0xd8, 0xf0, 0x06, 0x83, 0xbc, 0xd4, 0x89, 0x15, 0x48, 0x82, 0x29, 0x0f, 0x24,
            0x62, 0x47, 0x26, 0xe0, 0x93, 0x39, 0x07, 0x83, 0xd4, 0x95, 0x9a,
        ];
        assert_eq!(&data, &expected_data);

        Ok(())
    }

    #[test]
    fn test_encrypt_xts() -> Result<(), ErrorTrace> {
        let tweak_value: [u8; 16] = [
            0xb2, 0xf8, 0xc6, 0x37, 0x4e, 0xb2, 0x75, 0xc1, 0x74, 0x4e, 0x85, 0xaa, 0x21, 0xf8,
            0xea, 0x6b,
        ];
        let mut xts_context: AesXtsContext = AesXtsContext::new();

        let key: [u8; 16] = [
            0x8a, 0xfb, 0x90, 0xc2, 0xec, 0x92, 0x4c, 0x4b, 0x0b, 0x0b, 0xd8, 0x40, 0xfb, 0x1e,
            0xfc, 0x84,
        ];
        let tweak_key: [u8; 16] = [
            0x2c, 0x93, 0x85, 0xa1, 0x4d, 0x1c, 0xa9, 0x5b, 0xd4, 0xd1, 0x2c, 0xbf, 0x9a, 0xb5,
            0x88, 0xed,
        ];
        xts_context.set_keys(&key, &tweak_key)?;

        let data: [u8; 25] = [
            0xd9, 0xd8, 0xf0, 0x06, 0x83, 0xbc, 0xd4, 0x89, 0x15, 0x48, 0x82, 0x29, 0x0f, 0x24,
            0x62, 0x47, 0x26, 0xe0, 0x93, 0x39, 0x07, 0x83, 0xd4, 0x95, 0x9a,
        ];
        let mut encrypted_data: Vec<u8> = vec![0; 25];
        xts_context.encrypt_xts(&tweak_value, &data, &mut encrypted_data)?;

        let expected_encrypted_data: [u8; 25] = [
            0xf4, 0xbb, 0xaa, 0x8e, 0xbd, 0x48, 0x0d, 0x2a, 0x2a, 0x37, 0x1b, 0xea, 0xb3, 0xd8,
            0xb3, 0x87, 0xc0, 0x22, 0x82, 0x67, 0x8c, 0x60, 0x00, 0x22, 0x7b,
        ];
        assert_eq!(&encrypted_data, &expected_encrypted_data);

        Ok(())
    }
}
