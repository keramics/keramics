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
use keramics_types::{Uuid, bytes_to_u16_be};

use super::block_reader::LuksBlockReader;
use super::block_stream::LuksBlockStream;
use super::constants::*;
use super::credential::LuksCredential;
use super::encryption::{
    LuksDiffuserContext, LuksEncryption, LuksEncryptionContext, LuksKeyDerivationContext,
};
use super::encryption_type::LuksEncryptionType;
use super::key_slot::LuksKeySlot;
use super::metadata::LuksMetadata;
use super::volume_header_v1::LuksVolumeHeaderV1;
use super::volume_header_v2::LuksVolumeHeaderV2;

/// Linux Unified Key Setup (LUKS) Disk Encryption encrypted volume.
pub struct LuksEncryptedVolume {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    format_version: u16,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Volume identifier.
    volume_identifier: Uuid,

    /// Encryption type.
    encryption_type: LuksEncryptionType,

    /// Hashing method.
    hashing_method: String,

    /// Salt.
    salt: Vec<u8>,

    /// Number of iterations.
    number_of_iterations: u32,

    /// Key size.
    key_size: usize,

    /// Validation hash.
    validation_hash: Vec<u8>,

    /// Key slots.
    key_slots: Vec<LuksKeySlot>,

    /// Encrypted data offset.
    encrypted_data_offset: u64,

    /// Encryption context.
    encryption_context: Option<LuksEncryptionContext>,

    /// The size.
    size: u64,

    /// Value to indicate the container is locked.
    is_locked: bool,
}

impl LuksEncryptedVolume {
    /// Creates a new encrypted volume.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            bytes_per_sector: 0,
            volume_identifier: Uuid::new(),
            encryption_type: LuksEncryptionType::new(),
            hashing_method: String::new(),
            salt: Vec::new(),
            number_of_iterations: 0,
            key_size: 0,
            validation_hash: Vec::new(),
            key_slots: Vec::new(),
            encrypted_data_offset: 0,
            encryption_context: None,
            size: 0,
            is_locked: true,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.data_stream {
            Some(data_stream) => match &self.encryption_context {
                Some(encryption_context) => Some(Arc::new(RwLock::new(LuksBlockStream::new(
                    LuksBlockReader::new(
                        data_stream,
                        self.bytes_per_sector,
                        self.encrypted_data_offset,
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
    pub fn get_encryption_type(&self) -> &LuksEncryptionType {
        &self.encryption_type
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u16 {
        self.format_version
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.volume_identifier
    }

    /// Retrieves the volume size.
    pub fn get_volume_size(&self) -> u64 {
        self.size
    }

    /// Determines if the container is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Reads the encrypted volume from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        let mut data: [u8; 4096] = [0; 4096];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0),
        );
        keramics_core::debug_trace_data!("LuksVolumeHeader", offset, &data, 4096);

        if &data[0..6] != LUKS_VOLUME_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.bytes_per_sector = 512;
        self.format_version = bytes_to_u16_be!(data, 6);

        let volume_identifier_string: String;

        match &self.format_version {
            1 => {
                keramics_core::debug_trace_structure!(LuksVolumeHeaderV1::debug_read_data(&data));

                let mut volume_header: LuksVolumeHeaderV1 = LuksVolumeHeaderV1::new();

                match volume_header.read_data(&data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read version 1 volume header at offset: {} (0x{:08x})",
                                offset, offset
                            ),
                        );
                        return Err(error);
                    }
                }
                volume_identifier_string = volume_header.volume_identifier.to_string();

                let encrypted_data_offset: u64 = (volume_header.encrypted_data_start_sector as u64)
                    * (self.bytes_per_sector as u64);

                if encrypted_data_offset >= data_stream_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid encrypted data offset: {} (0x{:08x}) value out of bounds",
                        encrypted_data_offset, encrypted_data_offset
                    )));
                }
                self.encryption_type
                    .set_encryption_method(&volume_header.encryption_method);
                self.encryption_type
                    .set_encryption_mode(&volume_header.encryption_mode);
                self.encryption_type.key_size = volume_header.key_size as usize;

                self.hashing_method = volume_header.hashing_method.to_string().to_lowercase();
                self.salt = volume_header.salt;
                self.number_of_iterations = volume_header.number_of_iterations;

                self.key_size = volume_header.key_size as usize;
                self.validation_hash = volume_header.validation_hash;
                self.key_slots = volume_header.key_slots;
                self.encrypted_data_offset = encrypted_data_offset;
            }
            2 => {
                keramics_core::debug_trace_structure!(LuksVolumeHeaderV2::debug_read_data(&data));

                let mut volume_header: LuksVolumeHeaderV2 = LuksVolumeHeaderV2::new();

                match volume_header.read_data(&data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read version 2 volume header at offset: {} (0x{:08x})",
                                offset, offset
                            ),
                        );
                        return Err(error);
                    }
                }
                if volume_header.metadata_area_size < 4096
                    || volume_header.metadata_area_size > data_stream_size
                {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported metadata area size - value out of bounds"
                    ));
                }
                // TODO: calculate and compare checksum

                volume_identifier_string = volume_header.volume_identifier.to_string();

                let mut metadata: LuksMetadata = LuksMetadata::new();

                match metadata.read_at_position(
                    data_stream,
                    volume_header.metadata_area_size - 4096,
                    SeekFrom::Start(4096),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read JSON metadata"
                        );
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format version: {}",
                    self.format_version
                )));
            }
        }
        self.volume_identifier = match Uuid::from_string(volume_identifier_string.as_str()) {
            Ok(uuid) => uuid,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to convert volume identifier string to UUID"
                );
                return Err(error);
            }
        };
        self.data_stream = Some(data_stream.clone());
        self.size = data_stream_size - self.encrypted_data_offset;
        self.is_locked = true;

        Ok(())
    }

    /// Derives the master key from a key slot and user key.
    fn derive_master_key(
        &self,
        key_slot: &LuksKeySlot,
        user_key: &[u8],
        key: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if key_slot.number_of_stripes > 4000 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported number of stripes - value out of bounds"
            ));
        }
        let key_material_offset: u64 =
            (key_slot.key_material_start_sector as u64) * (self.bytes_per_sector as u64);
        let key_material_size: usize = self.key_size * (key_slot.number_of_stripes as usize);

        if key_material_size % (self.bytes_per_sector as usize) != 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported key material size - not a multitude of sector size"
            ));
        }
        let mut key_material: Vec<u8> = vec![0; key_material_size];

        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut key_material,
            SeekFrom::Start(key_material_offset),
        );
        let encryption_context: LuksEncryptionContext =
            match LuksEncryption::get_encryption_context(&self.encryption_type, &user_key) {
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
                            "Unable to retrieve encryption context for type: {}",
                            self.encryption_type
                        )
                    );
                    return Err(error);
                }
            };
        let mut split_master_key_data: Vec<u8> = vec![0; key_material_size];

        for (sector_number, data_offset) in (0..key_material_size)
            .step_by(self.bytes_per_sector as usize)
            .enumerate()
        {
            let data_end_offset: usize = data_offset + (self.bytes_per_sector as usize);

            match encryption_context.decrypt_sector(
                sector_number as u64,
                &key_material[data_offset..data_end_offset],
                &mut split_master_key_data[data_offset..data_end_offset],
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to decrypt key material sector: {}", sector_number)
                    );
                    return Err(error);
                }
            }
        }
        let mut diffuser_context: LuksDiffuserContext =
            match LuksEncryption::get_diffuser_context(&self.hashing_method) {
                Some(context) => context,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported hashing method: {}",
                        self.hashing_method
                    )));
                }
            };
        diffuser_context.merge(key_slot.number_of_stripes, &split_master_key_data, key);

        Ok(())
    }

    /// Derives the user key from a key slot and passphrase.
    fn derive_user_key(
        &self,
        key_slot: &LuksKeySlot,
        passphrase: &[u8],
        key: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut key_derivation_context: LuksKeyDerivationContext =
            match LuksEncryption::get_key_derivation_context(
                self.hashing_method.as_str(),
                &key_slot.salt,
                key_slot.number_of_iterations as usize,
            ) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported key derivation method: {}",
                        self.hashing_method
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve key derivation context for method: {}",
                            self.hashing_method
                        )
                    );
                    return Err(error);
                }
            };
        match key_derivation_context.derive_key(passphrase, key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to derive key from passphrase"
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Unlocks a locked volume.
    pub fn unlock(&mut self, credentials: &[LuksCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let mut master_key: Vec<u8> = vec![0; self.key_size];
        let mut master_key_unlocked: bool = false;

        for credential in credentials.iter() {
            match credential {
                LuksCredential::Passphrase(passphrase) => {
                    for key_slot in self.key_slots.iter() {
                        if key_slot.number_of_stripes == 0 {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported key slot - number of stripes not set"
                            ));
                        }
                        let mut user_key: Vec<u8> = vec![0; self.key_size];

                        match self.derive_user_key(key_slot, passphrase, &mut user_key) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to derive user key"
                                );
                                return Err(error);
                            }
                        }
                        match self.derive_master_key(key_slot, &user_key, &mut master_key) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to derive master key"
                                );
                                return Err(error);
                            }
                        }
                        let mut key_derivation_context: LuksKeyDerivationContext =
                            match LuksEncryption::get_key_derivation_context(
                                self.hashing_method.as_str(),
                                &self.salt,
                                self.number_of_iterations as usize,
                            ) {
                                Ok(Some(context)) => context,
                                Ok(None) => {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Unsupported key derivation method: {}",
                                        self.hashing_method
                                    )));
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to retrieve key derivation context for method: {}",
                                            self.hashing_method
                                        )
                                    );
                                    return Err(error);
                                }
                            };
                        let mut master_key_validation_hash: [u8; 20] = [0; 20];

                        match key_derivation_context
                            .derive_key(&master_key, &mut master_key_validation_hash)
                        {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to derive validation hash from master key"
                                );
                                return Err(error);
                            }
                        }
                        if &master_key_validation_hash == self.validation_hash.as_slice() {
                            master_key_unlocked = true;
                        }
                    }
                }
                _ => {}
            }
        }
        if master_key_unlocked {
            keramics_core::debug_trace_data!("LuksMasterKey", 0, &master_key, master_key.len());

            self.encryption_context =
                match LuksEncryption::get_encryption_context(&self.encryption_type, &master_key) {
                    Ok(Some(context)) => Some(context),
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
                                "Unable to retrieve encryption context for type: {}",
                                self.encryption_type
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_encrypted_volume() -> Result<LuksEncryptedVolume, ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        Ok(encrypted_volume)
    }

    // TODO: add tests for get_bytes_per_sector
    // TODO: add tests for get_data_stream
    // TODO: add tests for get_encryption_type

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let encrypted_volume: LuksEncryptedVolume = get_encrypted_volume()?;

        let format_version: u16 = encrypted_volume.get_format_version();
        assert_eq!(format_version, 1);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let encrypted_volume: LuksEncryptedVolume = get_encrypted_volume()?;

        let identifier: &Uuid = encrypted_volume.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "20bc2795-63f3-4dc4-80d8-07911913a031"
        );
        Ok(())
    }

    // TODO: add tests for get_volume_size

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let encrypted_volume: LuksEncryptedVolume = get_encrypted_volume()?;

        let is_locked: bool = encrypted_volume.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        assert_eq!(encrypted_volume.format_version, 1);
        assert_eq!(encrypted_volume.is_locked, true);
        assert_eq!(
            encrypted_volume.volume_identifier.to_string(),
            "20bc2795-63f3-4dc4-80d8-07911913a031"
        );
        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        assert_eq!(encrypted_volume.is_locked, true);

        let credentials: Vec<LuksCredential> =
            vec![LuksCredential::Passphrase(b"KeRaMiCs".to_vec())];
        encrypted_volume.unlock(&credentials)?;

        assert_eq!(encrypted_volume.is_locked, false);

        Ok(())
    }
}
