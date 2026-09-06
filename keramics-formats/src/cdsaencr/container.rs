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

use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use super::block_reader::CdsaEncrBlockReader;
use super::block_stream::CdsaEncrBlockStream;
use super::constants::*;
use super::container_footer::CdsaEncrContainerFooter;
use super::container_header::CdsaEncrContainerHeader;
use super::credential::CdsaEncrCredential;
use super::encryption::{CdsaEncrCipherContext, CdsaEncrEncryption, CdsaEncrHmacContext};
use super::encryption_context::CdsaEncrEncryptionContext;
use super::encryption_type::CdsaEncrEncryptionType;
use super::enums::CdsaEncrKeyProtectorType;
use super::key_protector::CdsaEncrKeyProtector;
use super::passphrase_wrapped_key::CdsaEncrPassphraseWrappedKey;
use super::public_key_wrapped_key::CdsaEncrPublicKeyWrappedKey;

/// Mac OS Encrypted Encoding (cdsaencr) container.
pub struct CdsaEncrContainer {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    format_version: u32,

    /// Container identifier.
    container_identifier: Uuid,

    /// Data fork offset.
    data_fork_offset: u64,

    /// Data fork size.
    data_fork_size: u64,

    /// Encryption type.
    encryption_type: CdsaEncrEncryptionType,

    /// Block size.
    block_size: u32,

    /// Initialization vector size.
    initialization_vector_size: usize,

    /// HMAC method.
    hmac_method: u32,

    /// HMAC method.
    hmac_key_size: usize,

    /// Key key_protectors.
    key_protectors: Vec<CdsaEncrKeyProtector>,

    /// Encryption context.
    pub(crate) encryption_context: Option<CdsaEncrEncryptionContext>,

    /// Value to indicate the container is locked.
    is_locked: bool,

    /// Size.
    size: u64,
}

impl CdsaEncrContainer {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            container_identifier: Uuid::new(),
            data_fork_offset: 0,
            data_fork_size: 0,
            encryption_type: CdsaEncrEncryptionType::new(),
            block_size: 0,
            initialization_vector_size: 0,
            hmac_method: 0,
            hmac_key_size: 0,
            key_protectors: Vec::new(),
            encryption_context: None,
            is_locked: true,
            size: 0,
        }
    }

    /// Retrieves the block size.
    pub fn get_block_size(&self) -> u32 {
        self.block_size
    }

    /// Retrieves the container identifier.
    pub fn get_container_identifier(&self) -> &Uuid {
        &self.container_identifier
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.data_stream {
            Some(data_stream) => match &self.encryption_context {
                Some(encryption_context) => Some(Arc::new(RwLock::new(CdsaEncrBlockStream::new(
                    CdsaEncrBlockReader::new(
                        data_stream,
                        self.data_fork_offset,
                        self.block_size,
                        encryption_context,
                        self.size,
                    ),
                )))),
                None => None,
            },
            None => None,
        }
    }

    /// Retrieves the encryption type.
    pub fn get_encryption_type(&self) -> &CdsaEncrEncryptionType {
        &self.encryption_type
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u32 {
        self.format_version
    }

    /// Determines if the container is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        let footer_offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        let data_stream_size: u64 = footer_offset + 8;

        if &header_signature != CDSAENCR_CONTAINER_HEADER_SIGNATURE
            && &footer_signature != CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            return Err(keramics_core::error_trace_new!("Missing header and footer"));
        }
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            && &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            return Err(keramics_core::error_trace_new!(
                "Unsupported format with both header and footer"
            ));
        }
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE {
            let mut encrypted_container_header: CdsaEncrContainerHeader =
                CdsaEncrContainerHeader::new();

            match encrypted_container_header.read_at_position(data_stream, SeekFrom::Start(0)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read encrypted container header"
                    );
                    return Err(error);
                }
            }
            self.format_version = encrypted_container_header.format_version;
            self.container_identifier = encrypted_container_header.container_identifier;
            self.data_fork_offset = encrypted_container_header.data_fork_offset;
            self.data_fork_size = encrypted_container_header.data_fork_size;
            self.encryption_type = encrypted_container_header.encryption_type;
            self.block_size = encrypted_container_header.block_size;
            self.initialization_vector_size =
                encrypted_container_header.initialization_vector_size as usize;
            self.hmac_method = encrypted_container_header.hmac_method;
            self.hmac_key_size = (encrypted_container_header.hmac_key_size / 8) as usize;

            for key_protector_descriptor in
                encrypted_container_header.key_protector_descriptors.iter()
            {
                let key_protector_type: CdsaEncrKeyProtectorType = match key_protector_descriptor
                    .unlock_type
                {
                    0x00000001 => CdsaEncrKeyProtectorType::PassphraseWrappedKey,
                    0x00000002 => CdsaEncrKeyProtectorType::PublicKeyWrappedKey,
                    0x00000003 => CdsaEncrKeyProtectorType::KeybagWrappedKey,
                    _ => CdsaEncrKeyProtectorType::Unknown(key_protector_descriptor.unlock_type),
                };
                let key_protector: CdsaEncrKeyProtector = CdsaEncrKeyProtector::new(
                    key_protector_type,
                    key_protector_descriptor.data_offset,
                    key_protector_descriptor.data_size,
                );
                self.key_protectors.push(key_protector);
            }
        } else {
            let mut encrypted_container_footer: CdsaEncrContainerFooter =
                CdsaEncrContainerFooter::new();

            match encrypted_container_footer.read_at_position(data_stream, SeekFrom::End(-1276)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read encrypted container footer"
                    );
                    return Err(error);
                }
            }
            self.format_version = encrypted_container_footer.format_version;
            self.container_identifier = encrypted_container_footer.container_identifier;
            self.data_fork_offset = encrypted_container_footer.data_fork_offset as u64;
            self.data_fork_size = encrypted_container_footer.data_fork_size as u64;
            self.encryption_type = encrypted_container_footer.encryption_type;
            self.block_size = encrypted_container_footer.block_size;
            self.initialization_vector_size =
                encrypted_container_footer.initialization_vector_size as usize;
            self.hmac_method = encrypted_container_footer.hmac_method;
            self.hmac_key_size = (encrypted_container_footer.hmac_key_size / 8) as usize;

            let key_protector: CdsaEncrKeyProtector = CdsaEncrKeyProtector::new(
                CdsaEncrKeyProtectorType::PassphraseWrappedKey,
                data_stream_size - 1276,
                1276,
            );
            self.key_protectors.push(key_protector);
        }
        self.data_stream = Some(data_stream.clone());
        self.size = self.data_fork_size;

        Ok(())
    }

    /// Unlocks a locked container.
    pub fn unlock(&mut self, credentials: &[CdsaEncrCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut block_key: Vec<u8> = Vec::new();
        let mut hmac_key: Vec<u8> = Vec::new();
        let mut keys_unlocked: bool = false;

        for (key_protector_index, key_protector) in self.key_protectors.iter().enumerate() {
            // TODO: refactor read and unlock of key protector data into key protector?

            // Note that 65536 is an arbitrary chosen limit.
            if key_protector.size > 65536 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported key protector: {} size: {} value out of bounds",
                    key_protector_index, key_protector.size
                )));
            }
            let mut data: Vec<u8> = vec![0; key_protector.size as usize];

            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(key_protector.offset)
            );
            match key_protector.protector_type {
                CdsaEncrKeyProtectorType::PassphraseWrappedKey => {
                    if self.format_version == 1 {
                        keramics_core::debug_trace_data_and_structure!(
                            "CdsaEncrContainerFooter",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            CdsaEncrContainerFooter::debug_read_data(&data)
                        );
                        let mut container_footer: CdsaEncrContainerFooter =
                            CdsaEncrContainerFooter::new();

                        match container_footer.read_data(&data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read container footer"
                                );
                                return Err(error);
                            }
                        }
                        for credential in credentials.iter() {
                            match container_footer.unlock(credential) {
                                Ok(result) => {
                                    if result {
                                        let block_key_size: usize = self.encryption_type.key_size;
                                        block_key = container_footer.block_key_data
                                            [0..block_key_size]
                                            .to_vec();

                                        let hmac_key_size: usize = self.hmac_key_size;
                                        hmac_key = container_footer.hmac_key_data[0..hmac_key_size]
                                            .to_vec();

                                        keys_unlocked = true;

                                        break;
                                    }
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to unlock container footer"
                                    );
                                    return Err(error);
                                }
                            }
                        }
                    } else if self.format_version == 2 {
                        keramics_core::debug_trace_data_and_structure!(
                            "CdsaEncrPassphraseWrappedKey",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            CdsaEncrPassphraseWrappedKey::debug_read_data(&data)
                        );
                        let mut wrapped_key: CdsaEncrPassphraseWrappedKey =
                            CdsaEncrPassphraseWrappedKey::new();

                        match wrapped_key.read_data(&data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read passphrase wrapped key"
                                );
                                return Err(error);
                            }
                        }
                        for credential in credentials.iter() {
                            match wrapped_key.unlock(credential) {
                                Ok(result) => {
                                    if result {
                                        let block_key_size: usize = self.encryption_type.key_size;
                                        block_key =
                                            wrapped_key.key_data[0..block_key_size].to_vec();

                                        let hmac_key_size: usize = self.hmac_key_size;
                                        let data_end_offset: usize = block_key_size + hmac_key_size;
                                        hmac_key = wrapped_key.key_data
                                            [block_key_size..data_end_offset]
                                            .to_vec();

                                        keys_unlocked = true;

                                        break;
                                    }
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        "Unable to unlock passphrase wrapped key"
                                    );
                                    return Err(error);
                                }
                            }
                        }
                    }
                    if keys_unlocked {
                        break;
                    }
                }
                CdsaEncrKeyProtectorType::PublicKeyWrappedKey => {
                    if self.format_version == 2 {
                        keramics_core::debug_trace_data_and_structure!(
                            "CdsaEncrPublicKeyWrappedKey",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            CdsaEncrPublicKeyWrappedKey::debug_read_data(&data)
                        );
                        let mut wrapped_key: CdsaEncrPublicKeyWrappedKey =
                            CdsaEncrPublicKeyWrappedKey::new();

                        match wrapped_key.read_data(&data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read public key wrapped key"
                                );
                                return Err(error);
                            }
                        }
                    }
                }
                _ => {
                    if self.format_version == 2 {
                        keramics_core::debug_trace_data!(
                            "CdsaEncrEncryptedKeyProtector",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                        );
                    }
                }
            }
        }
        if keys_unlocked {
            keramics_core::debug_trace_data!("CdsaEncrBlockKey", 0, &block_key, block_key.len());
            keramics_core::debug_trace_data!("CdsaEncrBlockHmacKey", 0, &hmac_key, hmac_key.len());

            let cipher_context: CdsaEncrCipherContext =
                match CdsaEncrEncryption::get_cipher_context(&self.encryption_type, &block_key) {
                    Ok(Some(context)) => context,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported encryption type: {}",
                            self.encryption_type
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve cipher context for type: {}",
                                self.encryption_type
                            )
                        );
                        return Err(error);
                    }
                };
            let hmac_context: CdsaEncrHmacContext =
                match CdsaEncrEncryption::get_hmac_context(self.hmac_method, &hmac_key) {
                    Ok(Some(context)) => context,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported HMAC method: {}",
                            self.hmac_method
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve HMAC context for method: {}",
                                self.hmac_method
                            )
                        );
                        return Err(error);
                    }
                };
            self.encryption_context = Some(CdsaEncrEncryptionContext {
                cipher_context,
                hmac_context,
            });
            self.is_locked = false;
        }
        Ok(!self.is_locked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_container(path_string: &str) -> Result<CdsaEncrContainer, ErrorTrace> {
        let mut container: CdsaEncrContainer = CdsaEncrContainer::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        Ok(container)
    }

    #[test]
    fn test_get_block_size() -> Result<(), ErrorTrace> {
        let container: CdsaEncrContainer = get_container("udif/hfsplus_aes256.dmg")?;

        let block_size: u32 = container.get_block_size();
        assert_eq!(block_size, 512);

        Ok(())
    }

    #[test]
    fn test_get_container_identifier() -> Result<(), ErrorTrace> {
        let container: CdsaEncrContainer = get_container("udif/hfsplus_aes256.dmg")?;

        let container_identifier: &Uuid = container.get_container_identifier();
        assert_eq!(
            container_identifier.to_string(),
            "6dde706c-61d2-45ff-9046-c86b3912bfeb"
        );
        Ok(())
    }

    #[test]
    fn test_get_encryption_type() -> Result<(), ErrorTrace> {
        let container: CdsaEncrContainer = get_container("udif/hfsplus_aes256.dmg")?;

        let encryption_type: &CdsaEncrEncryptionType = container.get_encryption_type();
        assert_eq!(encryption_type.method, 0x80000001);
        assert_eq!(encryption_type.mode, 5);
        assert_eq!(encryption_type.key_size, 32);

        Ok(())
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let container: CdsaEncrContainer = get_container("udif/hfsplus_aes256.dmg")?;

        let format_version: u32 = container.get_format_version();
        assert_eq!(format_version, 1);

        Ok(())
    }

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let container: CdsaEncrContainer = get_container("udif/hfsplus_aes256.dmg")?;

        let is_locked: bool = container.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut container: CdsaEncrContainer = CdsaEncrContainer::new();

        let path_string: String = get_test_data_path("udif/hfsplus_aes256.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        assert_eq!(container.format_version, 1);
        assert_eq!(
            container.container_identifier.to_string(),
            "6dde706c-61d2-45ff-9046-c86b3912bfeb"
        );
        assert_eq!(container.block_size, 512);

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut container: CdsaEncrContainer = CdsaEncrContainer::new();

        let path_string: String = get_test_data_path("udif/hfsplus_aes256.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        container.read_data_stream(&data_stream)?;

        assert_eq!(container.is_locked, true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        container.unlock(&credentials)?;

        assert_eq!(container.is_locked, false);

        Ok(())
    }
}
