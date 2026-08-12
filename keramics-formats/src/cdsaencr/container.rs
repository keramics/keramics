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

use std::cmp::min;
use std::io::SeekFrom;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::lru_cache::LruCache;

use super::constants::*;
use super::container_footer::CdsaEncrContainerFooter;
use super::container_header::CdsaEncrContainerHeader;
use super::credential::CdsaEncrCredential;
use super::encryption::{CdsaEncrEncryption, CdsaEncrEncryptionContext, CdsaEncrHmacContext};
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
    encryption_context: CdsaEncrEncryptionContext,

    /// HMAC context.
    hmac_context: CdsaEncrHmacContext,

    /// Decrypted block cache.
    block_cache: LruCache<u32, Vec<u8>>,

    /// Value to indicate the container is locked.
    is_locked: bool,

    /// The current offset.
    current_offset: u64,

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
            encryption_context: CdsaEncrEncryptionContext::None,
            hmac_context: CdsaEncrHmacContext::None,
            block_cache: LruCache::new(64),
            is_locked: true,
            current_offset: 0,
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

    /// Decrypts a block.
    pub(crate) fn decrypt_block(
        &mut self,
        block_number: u32,
        encrypted_data: &[u8],
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let block_number_data: [u8; 4] = (block_number as u32).to_be_bytes();

        let mut initialization_vector: Vec<u8> =
            match self.hmac_context.calculate_hmac(&block_number_data) {
                Ok(data) => data,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to HMAC initialization vector for decrypting block: {}",
                            block_number
                        )
                    );
                    return Err(error);
                }
            };
        match self
            .encryption_context
            .decrypt_cbc(&mut initialization_vector, encrypted_data, data)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt block: {}", block_number)
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads and decrypts a block.
    fn read_block(
        &mut self,
        block_number: u32,
        block_data_offset: u64,
    ) -> Result<Vec<u8>, ErrorTrace> {
        let mut encrypted_data: Vec<u8> = vec![0; self.block_size as usize];

        match self.data_stream.as_ref() {
            Some(data_stream) => {
                keramics_core::data_stream_read_exact_at_position!(
                    data_stream,
                    &mut encrypted_data,
                    SeekFrom::Start(block_data_offset)
                );
            }
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        }
        let mut block_data: Vec<u8> = vec![0; self.block_size as usize];

        match self.decrypt_block(block_number as u32, &encrypted_data, &mut block_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to decrypt block: {}", block_number)
                );
                return Err(error);
            }
        }
        Ok(block_data)
    }

    /// Reads container data based on the encrypted blocks.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Container is locked"));
        }
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut container_offset: u64 = self.current_offset;

        let mut block_number: u64 = container_offset / (self.block_size as u64);
        let mut block_offset: u64 = block_number * (self.block_size as u64);

        while data_offset < read_size {
            if container_offset >= self.size {
                break;
            }
            let range_relative_offset: u64 = container_offset - block_offset;
            let range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            if block_number > u32::MAX as u64 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid block number value out of bounds"
                ));
            }
            if !self.block_cache.contains(&(block_number as u32)) {
                let block_data_offset: u64 = self.data_fork_offset + block_offset;

                let block_data: Vec<u8> =
                    match self.read_block(block_number as u32, block_data_offset) {
                        Ok(data) => data,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read block: {}", block_number)
                            );
                            return Err(error);
                        }
                    };
                self.block_cache.insert(block_number as u32, block_data);
            }
            let range_data: &[u8] = match self.block_cache.get(&(block_number as u32)) {
                Some(data) => data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block: {} from cache",
                        block_number
                    )));
                }
            };
            let data_end_offset: usize = data_offset + range_read_size;
            let range_data_offset: usize = range_relative_offset as usize;
            let range_data_end_offset: usize = range_data_offset + range_read_size;

            data[data_offset..data_end_offset]
                .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);

            data_offset = data_end_offset;

            container_offset += range_read_size as u64;
            block_offset += self.block_size as u64;
            block_number += 1;
        }
        Ok(data_offset)
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

            self.encryption_context =
                match CdsaEncrEncryption::get_encryption_context(&self.encryption_type, &block_key)
                {
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
                                "Unable to retrieve encryption type: {}",
                                self.encryption_type
                            )
                        );
                        return Err(error);
                    }
                };
            self.hmac_context =
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
            self.is_locked = false;
        }
        Ok(!self.is_locked)
    }
}

impl DataStream for CdsaEncrContainer {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = match self.read_data_from_blocks(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                return Err(error);
            }
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset value out of bounds"
                    ));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
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
