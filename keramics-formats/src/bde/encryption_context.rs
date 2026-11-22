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
use keramics_encryption::{AesContext, CryptEcb};

use super::cipher_context::BdeCipherContext;
use super::diffuser::BdeDiffuser;

/// BitLocker Drive Encryption (BDE) encryption context.
pub struct BdeEncryptionContext {
    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Cipher context.
    cipher_context: BdeCipherContext,

    /// Diffuser context.
    diffuser_context: Option<AesContext>,
}

impl BdeEncryptionContext {
    /// Creates a new encryption context.
    pub fn new(
        bytes_per_sector: u16,
        cipher_context: BdeCipherContext,
        diffuser_context: Option<AesContext>,
    ) -> Self {
        Self {
            bytes_per_sector,
            cipher_context,
            diffuser_context,
        }
    }

    /// Decrypts a sector in place.
    pub fn decrypt_sector(
        &self,
        sector_data_offset: u64,
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut initialization_vector: [u8; 16] = [0; 16];

        match &self.cipher_context {
            BdeCipherContext::AesCbc(aes_context) => {
                let mut block_key_data: [u8; 16] = [0; 16];
                block_key_data[0..8].copy_from_slice(&sector_data_offset.to_le_bytes());

                match aes_context.encrypt_ecb(&block_key_data, &mut initialization_vector) {
                    Ok(_) => {}
                    Err(mut error) => {
                        let sector_number: u64 =
                            sector_data_offset / (self.bytes_per_sector as u64);
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to encrypt initialization vector for sector: {}",
                                sector_number
                            )
                        );
                        return Err(error);
                    }
                }
                match aes_context.decrypt_cbc_in_place(&initialization_vector, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        let sector_number: u64 =
                            sector_data_offset / (self.bytes_per_sector as u64);
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to decrypt sector: {}", sector_number)
                        );
                        return Err(error);
                    }
                }
                if let Some(diffuser_context) = &self.diffuser_context {
                    let mut sector_key_data: [u8; 32] = [0; 32];

                    match diffuser_context.encrypt_ecb(&block_key_data, &mut sector_key_data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            let sector_number: u64 =
                                sector_data_offset / (self.bytes_per_sector as u64);
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to encrypt initialization vector for sector: {}",
                                    sector_number
                                )
                            );
                            return Err(error);
                        }
                    }
                    block_key_data[15] = 0x80;

                    match diffuser_context.encrypt_ecb(&block_key_data, &mut sector_key_data[16..])
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            let sector_number: u64 =
                                sector_data_offset / (self.bytes_per_sector as u64);
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to encrypt initialization vector for sector: {}",
                                    sector_number
                                )
                            );
                            return Err(error);
                        }
                    }
                    match BdeDiffuser::decrypt(data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            let sector_number: u64 =
                                sector_data_offset / (self.bytes_per_sector as u64);
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to diffuser decrypt sector: {}", sector_number)
                            );
                            return Err(error);
                        }
                    }
                    let mut sector_key_index: usize = 0;

                    for byte_value in data.iter_mut() {
                        *byte_value ^= sector_key_data[sector_key_index & 0x1f];
                        sector_key_index += 1;
                    }
                }
            }
            BdeCipherContext::AesXts(xts_context) => {
                let sector_number: u64 = sector_data_offset / (self.bytes_per_sector as u64);
                initialization_vector[0..8].copy_from_slice(&sector_number.to_le_bytes());

                match xts_context.decrypt_xts_in_place(&initialization_vector, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to decrypt sector: {}", sector_number)
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Decrypts one or more contiguous sectors in place.
    pub fn decrypt_sectors_in_place(
        &self,
        first_sector_data_offset: u64,
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let sector_size: usize = self.bytes_per_sector as usize;
        let mut sector_data_offset: u64 = first_sector_data_offset;

        for sector_data in data.chunks_exact_mut(sector_size) {
            match self.decrypt_sector(sector_data_offset, sector_data) {
                Ok(_) => {}
                Err(mut error) => {
                    let sector_number: u64 = sector_data_offset / (sector_size as u64);
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to decrypt sector: {}", sector_number)
                    );
                    return Err(error);
                }
            }
            sector_data_offset += sector_size as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_encryption::{AesXtsContext, CryptContext};

    #[test]
    fn test_decrypt_sector_and_sectors_in_place() -> Result<(), ErrorTrace> {
        // Test AES-CBC without diffuser.
        let mut aes_context: AesContext = AesContext::new();
        aes_context.set_key(&[0x11; 16])?;
        let cbc_context: BdeEncryptionContext =
            BdeEncryptionContext::new(512, BdeCipherContext::AesCbc(aes_context), None);

        let mut sectors_data: Vec<u8> = vec![0xaa; 1024];
        cbc_context.decrypt_sectors_in_place(512, &mut sectors_data)?;
        assert_ne!(sectors_data, vec![0xaa; 1024]);

        // Test AES-CBC with Elephant diffuser.
        let mut aes_context: AesContext = AesContext::new();
        aes_context.set_key(&[0x11; 16])?;
        let mut diffuser_context: AesContext = AesContext::new();
        diffuser_context.set_key(&[0x22; 16])?;
        let diffuser_encryption_context: BdeEncryptionContext = BdeEncryptionContext::new(
            512,
            BdeCipherContext::AesCbc(aes_context),
            Some(diffuser_context),
        );
        let mut sector_data: Vec<u8> = vec![0xbb; 512];
        diffuser_encryption_context.decrypt_sector(512, &mut sector_data)?;
        assert_ne!(sector_data, vec![0xbb; 512]);

        // Test AES-XTS decryption.
        let mut xts_context: AesXtsContext = AesXtsContext::new();
        xts_context.set_keys(&[0x33; 16], &[0x44; 16])?;
        let xts_encryption_context: BdeEncryptionContext =
            BdeEncryptionContext::new(512, BdeCipherContext::AesXts(xts_context), None);
        let mut xts_data: Vec<u8> = vec![0xcc; 1024];
        xts_encryption_context.decrypt_sectors_in_place(0, &mut xts_data)?;
        assert_ne!(xts_data, vec![0xcc; 1024]);

        // Test error paths with invalid non-block-aligned sector sizes.
        let mut invalid_data: [u8; 7] = [0; 7];
        let result: Result<(), ErrorTrace> = cbc_context.decrypt_sector(0, &mut invalid_data);
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> =
            xts_encryption_context.decrypt_sector(0, &mut invalid_data);
        assert!(result.is_err());

        // Test diffuser error when sector size is smaller than 32 bytes (valid for AES-CBC).
        let mut wrong_diffuser_size: [u8; 16] = [0; 16];
        let result: Result<(), ErrorTrace> =
            diffuser_encryption_context.decrypt_sector(0, &mut wrong_diffuser_size);
        assert!(result.is_err());

        Ok(())
    }
}
