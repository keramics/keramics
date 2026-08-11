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
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u32_be;

use super::constants::*;
use super::credential::{UdifCredential, UdifCredentialType};
use super::encryption::{UdifEncryption, UdifEncryptionContext, UdifKeyDerivationContext};
use super::encryption_type::UdifEncryptionType;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "identifier", data_type = "Uuid"),
        field(name = "block_size", data_type = "u32"),
        field(name = "kek_encryption_method", data_type = "u32", format = "hex"),
        field(name = "kek_padding_type", data_type = "u32"),
        field(name = "kek_encryption_mode", data_type = "u32"),
        field(name = "kek_key_size", data_type = "u32"),
        field(name = "kek_initialization_vector_size", data_type = "u32"),
        field(name = "key_derivation_method", data_type = "u32"),
        field(name = "unknown1", data_type = "u32"),
        field(name = "key_derivation_number_of_iterations", data_type = "u32"),
        field(name = "key_derivation_salt_size", data_type = "u32"),
        field(name = "key_derivation_salt", data_type = "[u8; 32]", format = "hex"),
        field(name = "block_initialization_vector_size", data_type = "u32"),
        field(name = "block_encryption_mode", data_type = "u32"),
        field(name = "block_encryption_method", data_type = "u32", format = "hex"),
        field(name = "block_key_size", data_type = "u32"),
        field(
            name = "wrapped_block_key_initialization_vector",
            data_type = "[u8; 32]",
            format = "hex"
        ),
        field(name = "wrapped_block_key_size", data_type = "u32"),
        field(name = "wrapped_block_key", data_type = "[u8; 256]", format = "hex"),
        field(name = "hmac_method", data_type = "u32"),
        field(name = "hmac_key_size", data_type = "u32"),
        field(
            name = "wrapped_hmac_key_initialization_vector",
            data_type = "[u8; 32]",
            format = "hex"
        ),
        field(name = "wrapped_hmac_key_size", data_type = "u32"),
        field(name = "wrapped_hmac_key", data_type = "[u8; 256]", format = "hex"),
        field(name = "integrity_encryption_method", data_type = "u32"),
        field(name = "integrity_key_size", data_type = "u32"),
        field(
            name = "wrapped_integrity_key_initialization_vector",
            data_type = "[u8; 32]",
            format = "hex"
        ),
        field(name = "wrapped_integrity_key_size", data_type = "u32"),
        field(
            name = "wrapped_integrity_key",
            data_type = "[u8; 256]",
            format = "hex"
        ),
        field(name = "unknown4", data_type = "u32"),
        field(name = "unknown5", data_type = "[u8; 256]", format = "hex"),
        field(name = "data_fork_offset", data_type = "u32"),
        field(name = "data_fork_size", data_type = "u32"),
        field(name = "format_version", data_type = "u32"),
        field(name = "signature", data_type = "ByteString<8>"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Universal Disk Image Format (UDIF) encrypted file footer.
pub struct UdifEncryptedFileFooter {
    /// Block size.
    pub block_size: u32,

    /// Key encryption key (KEK) encryption type.
    pub kek_encryption_type: UdifEncryptionType,

    /// Key encryption key (KEK) padding type.
    pub kek_padding_type: u32,

    /// Key derivation method.
    pub key_derivation_method: u32,

    /// Key encryption key (KEK) initialization vector size.
    pub kek_initialization_vector_size: usize,

    /// Number of iterations.
    pub number_of_iterations: u32,

    /// Salt.
    pub salt: Vec<u8>,

    /// Initialization vector size.
    pub initialization_vector_size: u32,

    /// Encryption type.
    pub encryption_type: UdifEncryptionType,

    /// Wrapped block key data.
    pub wrapped_block_key_data: Vec<u8>,

    /// Block key data.
    pub block_key_data: Vec<u8>,

    /// HMAC method.
    pub hmac_method: u32,

    /// Initialization vector encryption method.
    pub hmac_key_size: u32,

    /// Wrapped HMAC key data.
    pub wrapped_hmac_key_data: Vec<u8>,

    /// HMAC key data.
    pub hmac_key_data: Vec<u8>,

    /// Data fork offset.
    pub data_fork_offset: u32,

    /// Data fork size.
    pub data_fork_size: u32,

    /// Format version.
    pub format_version: u32,
}

impl UdifEncryptedFileFooter {
    /// Creates a new encrypted file footer.
    pub fn new() -> Self {
        Self {
            block_size: 0,
            kek_encryption_type: UdifEncryptionType::new(),
            kek_padding_type: 0,
            kek_initialization_vector_size: 0,
            key_derivation_method: 0,
            number_of_iterations: 0,
            salt: Vec::new(),
            initialization_vector_size: 0,
            encryption_type: UdifEncryptionType::new(),
            wrapped_block_key_data: Vec::new(),
            block_key_data: Vec::new(),
            hmac_method: 0,
            hmac_key_size: 0,
            wrapped_hmac_key_data: Vec::new(),
            hmac_key_data: Vec::new(),
            data_fork_offset: 0,
            data_fork_size: 0,
            format_version: 0,
        }
    }

    /// Reads the file encrypted footer from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 1276 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[1268..1276] != UDIF_ENCRYPTED_FILE_FOOTER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.format_version = bytes_to_u32_be!(data, 1264);

        if self.format_version != 1 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format version: {}",
                self.format_version
            )));
        }
        self.block_size = bytes_to_u32_be!(data, 16);
        self.kek_encryption_type.method = bytes_to_u32_be!(data, 20);
        self.kek_padding_type = bytes_to_u32_be!(data, 24);
        self.kek_encryption_type.mode = bytes_to_u32_be!(data, 28);
        self.kek_encryption_type.key_size = (bytes_to_u32_be!(data, 32) / 8) as usize;
        self.kek_initialization_vector_size = bytes_to_u32_be!(data, 36) as usize;

        if self.kek_initialization_vector_size > 32 {
            return Err(keramics_core::error_trace_new!(
                "Invalid KEK initialization vector size value out of bounds"
            ));
        }
        self.key_derivation_method = bytes_to_u32_be!(data, 40);
        self.number_of_iterations = bytes_to_u32_be!(data, 48);

        let salt_size: usize = bytes_to_u32_be!(data, 52) as usize;

        if salt_size > 52 {
            return Err(keramics_core::error_trace_new!(
                "Invalid salt size value out of bounds"
            ));
        }
        let data_end_offset: usize = 56 + salt_size;

        self.salt = data[56..data_end_offset].to_vec();

        self.initialization_vector_size = bytes_to_u32_be!(data, 88);
        self.encryption_type.mode = bytes_to_u32_be!(data, 92);
        self.encryption_type.method = bytes_to_u32_be!(data, 96);
        self.encryption_type.key_size = (bytes_to_u32_be!(data, 100) / 8) as usize;

        let wrapped_block_key_data_size: usize = bytes_to_u32_be!(data, 136) as usize;

        if wrapped_block_key_data_size > 256 {
            return Err(keramics_core::error_trace_new!(
                "Invalid wrapped block key data size value out of bounds"
            ));
        }
        let data_end_offset: usize = 140 + wrapped_block_key_data_size;
        self.wrapped_block_key_data = data[140..data_end_offset].to_vec();

        self.hmac_method = bytes_to_u32_be!(data, 396);
        self.hmac_key_size = bytes_to_u32_be!(data, 400);

        let wrapped_hmac_key_data_size: usize = bytes_to_u32_be!(data, 436) as usize;

        if wrapped_hmac_key_data_size > 256 {
            return Err(keramics_core::error_trace_new!(
                "Invalid wrapped HMAC key data size value out of bounds"
            ));
        }
        let data_end_offset: usize = 440 + wrapped_hmac_key_data_size;
        self.wrapped_hmac_key_data = data[440..data_end_offset].to_vec();

        self.data_fork_offset = bytes_to_u32_be!(data, 1256);
        self.data_fork_size = bytes_to_u32_be!(data, 1260);

        Ok(())
    }

    /// Unlocks the key.
    pub fn unlock(&mut self, credential: &UdifCredential) -> Result<bool, ErrorTrace> {
        if credential.credential_type != UdifCredentialType::Passphrase {
            return Ok(false);
        }
        let mut key_derivation_context: UdifKeyDerivationContext =
            match UdifEncryption::get_key_derivation_context(
                self.key_derivation_method,
                &self.salt,
                self.number_of_iterations as usize,
            ) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported key deriviation method: {}",
                        self.key_derivation_method
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve key derivation context for method: {}",
                            self.key_derivation_method
                        )
                    );
                    return Err(error);
                }
            };
        let mut key: Vec<u8> = vec![0; self.kek_encryption_type.key_size];

        match key_derivation_context.derive_key(&credential.data, &mut key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to derive key from passphrase"
                );
                return Err(error);
            }
        }
        match key_derivation_context.derive_key(&credential.data, &mut key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to derive key from passphrase"
                );
                return Err(error);
            }
        }
        let encryption_context: UdifEncryptionContext =
            match UdifEncryption::get_encryption_context(&self.kek_encryption_type, &key) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported encryption type: {}",
                        self.kek_encryption_type
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve encryption context for type: {}",
                            self.kek_encryption_type
                        )
                    );
                    return Err(error);
                }
            };
        match self.unwrap_key(&encryption_context, &self.wrapped_block_key_data) {
            Ok(Some(key_data)) => self.block_key_data = key_data,
            Ok(None) => return Ok(false),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to unwrap block key data");
                return Err(error);
            }
        }
        match self.unwrap_key(&encryption_context, &self.wrapped_hmac_key_data) {
            Ok(Some(key_data)) => self.hmac_key_data = key_data,
            Ok(None) => return Ok(false),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to unwrap HMAC key data");
                return Err(error);
            }
        }
        Ok(true)
    }

    /// Unwraps a key.
    fn unwrap_key(
        &self,
        encryption_context: &UdifEncryptionContext,
        wrapped_key_data: &[u8],
    ) -> Result<Option<Vec<u8>>, ErrorTrace> {
        let mut initialization_vector: Vec<u8> = vec![
            0x4a, 0xdd, 0xa2, 0x2c, 0x79, 0xe8, 0x21, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let intermediate_key_data_size: usize = wrapped_key_data.len();

        if intermediate_key_data_size < 8 {
            return Err(keramics_core::error_trace_new!(
                "Invalid intermediate key data size value out of bounds"
            ));
        }
        let mut intermediate_key_data: Vec<u8> = vec![0; intermediate_key_data_size];

        match encryption_context.decrypt_cbc(
            &mut initialization_vector,
            &wrapped_key_data,
            &mut intermediate_key_data,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decrypt wrapped key data");
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!(
            "UdifPaddedIntermediateKeyData",
            0,
            &intermediate_key_data,
            intermediate_key_data_size,
        );
        let result_key_data: &[u8] = match UdifEncryption::remove_padding(
            self.kek_padding_type,
            self.kek_initialization_vector_size,
            &intermediate_key_data,
        ) {
            Ok(data) => data,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to remove padding from intermediate key data"
                );
                return Err(error);
            }
        };
        let mut reversed_key_data: Vec<u8> = result_key_data.to_vec();
        reversed_key_data.reverse();

        let mut initialization_vector: Vec<u8> = reversed_key_data[0..8].to_vec();

        let final_key_data_size: usize = reversed_key_data[8..].len();

        if final_key_data_size < 12 {
            return Err(keramics_core::error_trace_new!(
                "Invalid final key data size value out of bounds"
            ));
        }
        let mut final_key_data: Vec<u8> = vec![0; final_key_data_size];

        match encryption_context.decrypt_cbc(
            &mut initialization_vector,
            &reversed_key_data[8..],
            &mut final_key_data,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to decrypt reversed intermediate key data"
                );
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!(
            "UdifFinalKeyData",
            0,
            &final_key_data,
            final_key_data_size,
        );
        let result_key_data: &[u8] = match UdifEncryption::remove_padding(
            self.kek_padding_type,
            self.kek_initialization_vector_size,
            &final_key_data,
        ) {
            Ok(data) => data,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to remove padding from final key data"
                );
                return Err(error);
            }
        };
        if &result_key_data[0..4] == &[0; 4] {
            Ok(Some(result_key_data[4..].to_vec()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xac, 0x93, 0xae, 0xe0, 0xd0, 0x45, 0x48, 0x3a, 0x94, 0x0d, 0xf0, 0x33, 0xaa, 0x11,
            0xb8, 0x5d, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x07,
            0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
            0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, 0x00, 0x00, 0x00, 0x14,
            0x9c, 0x82, 0xb4, 0x19, 0xbd, 0xac, 0x1b, 0x3e, 0x6b, 0x71, 0xf8, 0xa6, 0xb9, 0x9a,
            0x75, 0x01, 0xf3, 0x4b, 0x69, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x05, 0x80, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x80, 0xf2, 0xb2, 0x1b, 0x35, 0xa9, 0x2e, 0xff, 0x4f,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28,
            0x5d, 0xa4, 0x79, 0xe2, 0x92, 0xe0, 0xac, 0xf6, 0x7a, 0x9f, 0xa3, 0xe2, 0x4d, 0x0a,
            0x76, 0x7c, 0xae, 0x2f, 0x64, 0x5f, 0xf6, 0x38, 0x36, 0x66, 0x50, 0x68, 0x63, 0x71,
            0x88, 0xf4, 0xb8, 0x02, 0x95, 0xde, 0x79, 0xaa, 0xbd, 0xbc, 0x25, 0x36, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5b, 0x00, 0x00, 0x00, 0xa0, 0x01, 0xe2,
            0x33, 0xe4, 0x48, 0x59, 0xaa, 0x83, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x9b, 0x13, 0x61, 0x65, 0xee, 0x73, 0x41, 0x86,
            0x31, 0xcc, 0xf2, 0x8d, 0x5e, 0x77, 0x07, 0x37, 0x88, 0xae, 0x92, 0x1d, 0xf5, 0x96,
            0x64, 0x9a, 0x7a, 0x77, 0x89, 0x58, 0x5d, 0xb0, 0xf1, 0x3f, 0x44, 0x6d, 0x59, 0x27,
            0x96, 0x7e, 0x2e, 0xde, 0x20, 0xce, 0x8a, 0x4f, 0x53, 0x89, 0x18, 0x5d, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5b,
            0x00, 0x00, 0x00, 0xa0, 0x98, 0x64, 0xaf, 0xb6, 0x15, 0x07, 0xca, 0x97, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x56, 0x4b,
            0x63, 0x71, 0x1e, 0xec, 0x4e, 0xf3, 0x98, 0x0a, 0xea, 0x1d, 0xad, 0x33, 0xec, 0x19,
            0x73, 0x82, 0x46, 0x5e, 0x1c, 0xa7, 0xb8, 0xa9, 0x27, 0xf9, 0xb4, 0x2a, 0xd7, 0xfa,
            0xa1, 0x90, 0xb9, 0x0f, 0x32, 0x71, 0xca, 0x2e, 0xfa, 0x57, 0xcf, 0x47, 0x5e, 0x49,
            0xa4, 0xe9, 0x79, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x7e, 0xbd, 0x9b, 0xf7, 0xfe, 0xab, 0x33, 0xb7,
            0x52, 0xec, 0x2b, 0x76, 0xda, 0xed, 0x55, 0x61, 0x01, 0x5c, 0x4b, 0x00, 0x00, 0x00,
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
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x63, 0x64, 0x73, 0x61, 0x65, 0x6e,
            0x63, 0x72,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifEncryptedFileFooter::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.kek_encryption_type.method, 0x00000011);
        assert_eq!(test_struct.kek_padding_type, 7);
        assert_eq!(test_struct.kek_encryption_type.mode, 6);
        assert_eq!(test_struct.kek_encryption_type.key_size, 24);
        assert_eq!(test_struct.kek_initialization_vector_size, 8);
        assert_eq!(test_struct.key_derivation_method, 103);
        assert_eq!(test_struct.number_of_iterations, 1000);
        assert_eq!(test_struct.salt, &test_data[56..76]);
        assert_eq!(test_struct.initialization_vector_size, 16);
        assert_eq!(test_struct.encryption_type.mode, 5);
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.encryption_type.key_size, 16);
        assert_eq!(test_struct.wrapped_block_key_data, &test_data[140..180]);
        assert_eq!(test_struct.hmac_method, 91);
        assert_eq!(test_struct.hmac_key_size, 160);
        assert_eq!(test_struct.wrapped_hmac_key_data, &test_data[440..488]);
        assert_eq!(test_struct.data_fork_offset, 0);
        assert_eq!(test_struct.data_fork_size, 65536);
        assert_eq!(test_struct.format_version, 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifEncryptedFileFooter::new();
        let result = test_struct.read_data(&test_data[0..1275]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[1268] = 0xff;

        let mut test_struct = UdifEncryptedFileFooter::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[1264] = 0xff;

        let mut test_struct = UdifEncryptedFileFooter::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = UdifEncryptedFileFooter::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.kek_encryption_type.method, 0x00000011);
        assert_eq!(test_struct.kek_padding_type, 7);
        assert_eq!(test_struct.kek_encryption_type.mode, 6);
        assert_eq!(test_struct.kek_encryption_type.key_size, 24);
        assert_eq!(test_struct.kek_initialization_vector_size, 8);
        assert_eq!(test_struct.key_derivation_method, 103);
        assert_eq!(test_struct.number_of_iterations, 1000);
        assert_eq!(test_struct.salt, &test_data[56..76]);
        assert_eq!(test_struct.initialization_vector_size, 16);
        assert_eq!(test_struct.encryption_type.mode, 5);
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.encryption_type.key_size, 16);
        assert_eq!(test_struct.wrapped_block_key_data, &test_data[140..180]);
        assert_eq!(test_struct.hmac_method, 91);
        assert_eq!(test_struct.wrapped_hmac_key_data, &test_data[440..488]);
        assert_eq!(test_struct.data_fork_offset, 0);
        assert_eq!(test_struct.data_fork_size, 65536);
        assert_eq!(test_struct.format_version, 1);

        Ok(())
    }
}
