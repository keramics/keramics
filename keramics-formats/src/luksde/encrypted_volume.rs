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
use keramics_hashes::DigestHashContext;
use keramics_types::{ByteString, Uuid, bytes_to_u16_be};

use super::block_reader::LuksBlockReader;
use super::block_stream::LuksBlockStream;
use super::constants::*;
use super::credential::LuksCredential;
use super::encryption::{LuksDiffuserContext, LuksEncryption, LuksKeyDerivationContext};
use super::encryption_context::LuksEncryptionContext;
use super::encryption_type::LuksEncryptionType;
use super::key_slot::LuksKeySlot;
use super::metadata::LuksMetadata;
use super::metadata_digest::LuksMetadataDigest;
use super::metadata_key_slot::LuksMetadataKeySlot;
use super::metadata_segment::LuksMetadataSegment;
use super::volume_header_v1::LuksVolumeHeaderV1;
use super::volume_header_v2::LuksVolumeHeaderV2;

/// Standard LUKS2 backup metadata area offsets.
const LUKS2_BACKUP_HEADER_OFFSETS: [u64; 9] = [
    16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304,
];

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

    /// Volume label.
    volume_label: ByteString,

    /// Encryption type.
    encryption_type: LuksEncryptionType,

    /// Hashing method (LUKS1).
    hashing_method: String,

    /// Salt (LUKS1).
    salt: Vec<u8>,

    /// Number of iterations (LUKS1).
    number_of_iterations: u32,

    /// Key size.
    key_size: usize,

    /// Validation hash (LUKS1).
    validation_hash: Vec<u8>,

    /// Key slots (LUKS1).
    key_slots: Vec<LuksKeySlot>,

    /// Metadata key slots (LUKS2).
    metadata_key_slots: Vec<LuksMetadataKeySlot>,

    /// Metadata digests (LUKS2).
    metadata_digests: Vec<LuksMetadataDigest>,

    /// Active segment identifier (LUKS2).
    segment_identifier: String,

    /// Initialization vector tweak (LUKS2).
    initialization_vector_tweak: u64,

    /// Encrypted data offset.
    encrypted_data_offset: u64,

    /// Encryption context.
    encryption_context: Option<Arc<LuksEncryptionContext>>,

    /// The volume size.
    volume_size: u64,

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
            volume_label: ByteString::new(),
            encryption_type: LuksEncryptionType::new(),
            hashing_method: String::new(),
            salt: Vec::new(),
            number_of_iterations: 0,
            key_size: 0,
            validation_hash: Vec::new(),
            key_slots: Vec::new(),
            metadata_key_slots: Vec::new(),
            metadata_digests: Vec::new(),
            segment_identifier: String::new(),
            initialization_vector_tweak: 0,
            encrypted_data_offset: 0,
            encryption_context: None,
            volume_size: 0,
            is_locked: true,
        }
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

        if !key_material_size.is_multiple_of(self.bytes_per_sector as usize) {
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
            match LuksEncryption::get_encryption_context(&self.encryption_type, user_key) {
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

    /// Derives the master key from a LUKS2 key slot and user key.
    fn derive_master_key_v2(
        &self,
        key_slot: &LuksMetadataKeySlot,
        user_key: &[u8],
        key: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        if key_slot.af_type != "luks1" {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported anti-forensic diffuser type: {}",
                key_slot.af_type
            )));
        }
        if key_slot.af_number_of_stripes == 0 || key_slot.af_number_of_stripes > 4000 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported number of stripes - value out of bounds"
            ));
        }
        if key_slot.area_type != "raw" {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported key slot area type: {}",
                key_slot.area_type
            )));
        }
        // LUKS2 keyslot binary areas are always encrypted in 512-byte sectors.
        let key_slot_sector_size: usize = 512;
        let key_material_size: usize = key_slot.key_size * (key_slot.af_number_of_stripes as usize);

        if (key_material_size as u64) > key_slot.area_size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported key material size - exceeds key slot area size"
            ));
        }
        if !key_material_size.is_multiple_of(key_slot_sector_size) {
            return Err(keramics_core::error_trace_new!(
                "Unsupported key material size - not a multiple of 512 bytes"
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
            SeekFrom::Start(key_slot.area_offset),
        );
        let mut area_encryption_type: LuksEncryptionType = LuksEncryptionType::new();
        area_encryption_type.set_encryption_string(key_slot.area_encryption.as_str());
        area_encryption_type.key_size = key_slot.area_key_size;

        let encryption_context: LuksEncryptionContext =
            match LuksEncryption::get_encryption_context(&area_encryption_type, user_key) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported key slot encryption type: {}",
                        area_encryption_type
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve encryption context for type: {}",
                            area_encryption_type
                        )
                    );
                    return Err(error);
                }
            };
        let mut split_master_key_data: Vec<u8> = vec![0; key_material_size];

        for (sector_number, data_offset) in (0..key_material_size)
            .step_by(key_slot_sector_size)
            .enumerate()
        {
            let data_end_offset: usize = data_offset + key_slot_sector_size;

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
            match LuksEncryption::get_diffuser_context(key_slot.af_hashing_method.as_str()) {
                Some(context) => context,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported hashing method: {}",
                        key_slot.af_hashing_method
                    )));
                }
            };
        diffuser_context.merge(key_slot.af_number_of_stripes, &split_master_key_data, key);

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

    /// Derives the user key from a LUKS2 key slot and passphrase.
    fn derive_user_key_v2(
        &self,
        key_slot: &LuksMetadataKeySlot,
        passphrase: &[u8],
        key: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let mut key_derivation_context: LuksKeyDerivationContext = match key_slot.kdf_type.as_str()
        {
            "pbkdf2" => match LuksEncryption::get_key_derivation_context(
                key_slot.kdf_hashing_method.as_str(),
                &key_slot.kdf_salt,
                key_slot.kdf_number_of_iterations as usize,
            ) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported key derivation hashing method: {}",
                        key_slot.kdf_hashing_method
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve key derivation context for method: {}",
                            key_slot.kdf_hashing_method
                        )
                    );
                    return Err(error);
                }
            },
            "argon2i" | "argon2id" => match LuksEncryption::get_argon2_key_derivation_context(
                key_slot.kdf_type.as_str(),
                &key_slot.kdf_salt,
                key_slot.kdf_time,
                key_slot.kdf_memory,
                key_slot.kdf_cpus,
            ) {
                Some(context) => context,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported Argon2 key derivation type: {}",
                        key_slot.kdf_type
                    )));
                }
            },
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported key derivation type: {}",
                    key_slot.kdf_type
                )));
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
                        self.initialization_vector_tweak,
                        self.volume_size,
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

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&ByteString> {
        if self.volume_label.is_empty() {
            None
        } else {
            Some(&self.volume_label)
        }
    }

    /// Retrieves the volume size.
    pub fn get_volume_size(&self) -> u64 {
        self.volume_size
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
        let volume_size: u64;

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
                self.initialization_vector_tweak = 0;
                self.encrypted_data_offset = encrypted_data_offset;
                volume_size = data_stream_size - encrypted_data_offset;
            }
            2 => {
                let primary_result: Result<(LuksVolumeHeaderV2, Vec<u8>), ErrorTrace> =
                    Self::read_v2_metadata_area(data_stream, data_stream_size, 0, false);

                let (volume_header, json_data): (LuksVolumeHeaderV2, Vec<u8>) = match primary_result
                {
                    Ok((primary_header, primary_json)) => {
                        // Check backup header at metadata_area_size; prefer higher sequence_identifier.
                        let backup_offset: u64 = primary_header.metadata_area_size;
                        match Self::read_v2_metadata_area(
                            data_stream,
                            data_stream_size,
                            backup_offset,
                            true,
                        ) {
                            Ok((backup_header, backup_json))
                                if backup_header.sequence_identifier
                                    > primary_header.sequence_identifier =>
                            {
                                (backup_header, backup_json)
                            }
                            _ => (primary_header, primary_json),
                        }
                    }
                    Err(primary_error) => {
                        let mut recovered: Option<(LuksVolumeHeaderV2, Vec<u8>)> = None;

                        for backup_offset in LUKS2_BACKUP_HEADER_OFFSETS {
                            if backup_offset + 4096 > data_stream_size {
                                break;
                            }
                            if let Ok((backup_header, backup_json)) = Self::read_v2_metadata_area(
                                data_stream,
                                data_stream_size,
                                backup_offset,
                                true,
                            ) {
                                let should_replace: bool = match &recovered {
                                    Some((current_header, _)) => {
                                        backup_header.sequence_identifier
                                            > current_header.sequence_identifier
                                    }
                                    None => true,
                                };
                                if should_replace {
                                    recovered = Some((backup_header, backup_json));
                                }
                            }
                        }
                        match recovered {
                            Some(result) => result,
                            None => return Err(primary_error),
                        }
                    }
                };
                volume_identifier_string = volume_header.volume_identifier.to_string();
                self.volume_label = volume_header.volume_label;

                let mut metadata: LuksMetadata = LuksMetadata::new();

                match metadata.read_data(&json_data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read JSON metadata"
                        );
                        return Err(error);
                    }
                }
                let segment: &LuksMetadataSegment = match metadata
                    .segments
                    .iter()
                    .find(|segment| segment.identifier == "0")
                    .or_else(|| {
                        metadata
                            .segments
                            .iter()
                            .find(|segment| segment.segment_type == "crypt")
                    }) {
                    Some(segment) => segment,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing data segment in LUKS2 metadata"
                        ));
                    }
                };
                if segment.segment_type != "crypt" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported LUKS2 segment type: {}",
                        segment.segment_type
                    )));
                }
                if segment.sector_size != 512
                    && segment.sector_size != 1024
                    && segment.sector_size != 2048
                    && segment.sector_size != 4096
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported bytes per sector: {}",
                        segment.sector_size
                    )));
                }
                if segment.offset >= data_stream_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid encrypted data offset: {} (0x{:08x}) value out of bounds",
                        segment.offset, segment.offset
                    )));
                }
                volume_size = match segment.size {
                    Some(segment_size) => {
                        if segment.offset.saturating_add(segment_size) > data_stream_size {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Invalid segment size: {} value out of bounds",
                                segment_size
                            )));
                        }
                        segment_size
                    }
                    None => data_stream_size - segment.offset,
                };
                // Determine volume key size from a keyslot associated with this segment.
                let segment_identifier: String = segment.identifier.clone();
                let mut key_size: usize = 0;

                for digest in metadata.digests.iter() {
                    if digest.segments.contains(&segment_identifier) {
                        for keyslot_identifier in digest.keyslots.iter() {
                            if let Some(key_slot) = metadata.key_slots.iter().find(|slot| {
                                slot.identifier == *keyslot_identifier
                                    && slot.keyslot_type == "luks2"
                            }) {
                                key_size = key_slot.key_size;
                                break;
                            }
                        }
                    }
                    if key_size != 0 {
                        break;
                    }
                }
                if key_size == 0 {
                    if let Some(key_slot) = metadata
                        .key_slots
                        .iter()
                        .find(|slot| slot.keyslot_type == "luks2")
                    {
                        key_size = key_slot.key_size;
                    }
                }
                self.bytes_per_sector = segment.sector_size;
                self.key_size = key_size;
                self.encryption_type
                    .set_encryption_string(segment.encryption.as_str());
                self.encryption_type.key_size = key_size;
                self.encrypted_data_offset = segment.offset;
                self.initialization_vector_tweak = segment.initialization_vector_tweak;
                self.segment_identifier = segment_identifier;
                self.metadata_key_slots = metadata.key_slots;
                self.metadata_digests = metadata.digests;
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
        self.volume_size = volume_size;
        self.is_locked = true;

        Ok(())
    }

    /// Reads and validates a LUKS2 metadata area (binary header and JSON area) at the given offset.
    fn read_v2_metadata_area(
        data_stream: &DataStreamReference,
        data_stream_size: u64,
        header_offset: u64,
        is_backup: bool,
    ) -> Result<(LuksVolumeHeaderV2, Vec<u8>), ErrorTrace> {
        if header_offset.saturating_add(4096) > data_stream_size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported header offset - value out of bounds"
            ));
        }
        let mut header_data: [u8; 4096] = [0; 4096];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_data,
            SeekFrom::Start(header_offset),
        );
        keramics_core::debug_trace_data!("LuksVolumeHeader", offset, &header_data, 4096);
        keramics_core::debug_trace_structure!(LuksVolumeHeaderV2::debug_read_data(&header_data));

        let mut volume_header: LuksVolumeHeaderV2 = LuksVolumeHeaderV2::new();
        let read_result: Result<(), ErrorTrace> = if is_backup {
            volume_header.read_backup_data(&header_data)
        } else {
            volume_header.read_data(&header_data)
        };
        match read_result {
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
        if volume_header.header_offset != header_offset {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid header offset: {} does not match expected offset: {}",
                volume_header.header_offset, header_offset
            )));
        }
        if volume_header.metadata_area_size < 16384
            || volume_header.metadata_area_size > 4194304
            || !volume_header.metadata_area_size.is_power_of_two()
            || header_offset.saturating_add(volume_header.metadata_area_size) > data_stream_size
        {
            return Err(keramics_core::error_trace_new!(
                "Unsupported metadata area size - value out of bounds"
            ));
        }
        let json_area_size: usize = (volume_header.metadata_area_size - 4096) as usize;
        let json_area_offset: u64 = header_offset + 4096;
        let mut json_data: Vec<u8> = vec![0; json_area_size];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut json_data,
            SeekFrom::Start(json_area_offset),
        );
        keramics_core::debug_trace_data!(
            "LuksMetadata",
            json_area_offset,
            &json_data,
            json_area_size as u64
        );

        // Verify metadata area checksum over header (with checksum field zeroed) and JSON area.
        let checksum_method: String = volume_header
            .metadata_checksum_method
            .to_string()
            .to_lowercase();
        let mut digest_context: Box<dyn DigestHashContext> =
            match LuksEncryption::get_digest_hash_context(checksum_method.as_str()) {
                Some(context) => context,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported metadata checksum method: {}",
                        checksum_method
                    )));
                }
            };
        let mut header_for_checksum: [u8; 4096] = header_data;
        header_for_checksum[448..512].fill(0);
        digest_context.update(&header_for_checksum);
        digest_context.update(&json_data);

        let calculated_checksum: Vec<u8> = digest_context.finalize();
        let checksum_size: usize = calculated_checksum.len();

        if checksum_size > volume_header.metadata_checksum.len()
            || calculated_checksum.as_slice() != &volume_header.metadata_checksum[0..checksum_size]
        {
            return Err(keramics_core::error_trace_new!(
                "Metadata area checksum mismatch"
            ));
        }
        Ok((volume_header, json_data))
    }

    /// Unlocks a locked volume.
    pub fn unlock(&mut self, credentials: &[LuksCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let mut master_key: Vec<u8> = vec![0; self.key_size];
        let mut master_key_unlocked: bool = false;

        // Check for key data credentials first to skip key derivation if not needed.
        for credential in credentials.iter() {
            if let LuksCredential::KeyData { identifier, data } = credential {
                if identifier.len() != 16 {
                    continue;
                }
                let volume_identifier: Uuid = Uuid::from_be_bytes(identifier);

                if volume_identifier != self.volume_identifier {
                    continue;
                }
                if data.len() != self.key_size {
                    continue;
                }
                master_key = data.to_vec();
                master_key_unlocked = true;
                break;
            }
        }
        if !master_key_unlocked {
            for credential in credentials.iter() {
                if let LuksCredential::Passphrase(passphrase) = credential {
                    match self.format_version {
                        1 => {
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
                                if master_key_validation_hash == self.validation_hash.as_slice() {
                                    master_key_unlocked = true;
                                    break;
                                }
                            }
                        }
                        2 => {
                            let mut ordered_slots: Vec<&LuksMetadataKeySlot> = self
                                .metadata_key_slots
                                .iter()
                                .filter(|slot| slot.keyslot_type == "luks2")
                                .collect();
                            ordered_slots.sort_by(|left, right| {
                                let left_priority: i64 = left.priority.unwrap_or(1);
                                let right_priority: i64 = right.priority.unwrap_or(1);
                                right_priority.cmp(&left_priority)
                            });

                            for key_slot in ordered_slots {
                                let digest: &LuksMetadataDigest =
                                    match self.metadata_digests.iter().find(|digest| {
                                        digest.keyslots.contains(&key_slot.identifier)
                                            && digest.segments.contains(&self.segment_identifier)
                                    }) {
                                        Some(digest) => digest,
                                        None => continue,
                                    };
                                let mut user_key: Vec<u8> = vec![0; key_slot.area_key_size];

                                match self.derive_user_key_v2(key_slot, passphrase, &mut user_key) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable to derive user key for key slot: {}",
                                                key_slot.identifier
                                            )
                                        );
                                        return Err(error);
                                    }
                                }
                                let mut candidate_master_key: Vec<u8> = vec![0; key_slot.key_size];

                                match self.derive_master_key_v2(
                                    key_slot,
                                    &user_key,
                                    &mut candidate_master_key,
                                ) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable to derive master key for key slot: {}",
                                                key_slot.identifier
                                            )
                                        );
                                        return Err(error);
                                    }
                                }
                                let is_valid: bool = match Self::validate_master_key_v2(
                                    digest,
                                    &candidate_master_key,
                                ) {
                                    Ok(is_valid) => is_valid,
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable to validate master key for digest: {}",
                                                digest.identifier
                                            )
                                        );
                                        return Err(error);
                                    }
                                };
                                if is_valid {
                                    master_key = candidate_master_key;
                                    master_key_unlocked = true;
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                    if master_key_unlocked {
                        break;
                    }
                }
            }
        }
        if master_key_unlocked {
            keramics_core::debug_trace_data!("LuksMasterKey", 0, &master_key, master_key.len());

            let encryption_context: LuksEncryptionContext =
                match LuksEncryption::get_encryption_context(&self.encryption_type, &master_key) {
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
            self.encryption_context = Some(Arc::new(encryption_context));
            self.is_locked = false;
        }
        Ok(!self.is_locked)
    }

    /// Validates a derived master key against a LUKS2 digest object.
    fn validate_master_key_v2(
        digest: &LuksMetadataDigest,
        master_key: &[u8],
    ) -> Result<bool, ErrorTrace> {
        if digest.digest_type != "pbkdf2" {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported digest type: {}",
                digest.digest_type
            )));
        }
        if digest.digest.is_empty() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported digest - empty digest value"
            ));
        }
        let mut key_derivation_context: LuksKeyDerivationContext =
            match LuksEncryption::get_key_derivation_context(
                digest.hashing_method.as_str(),
                &digest.salt,
                digest.number_of_iterations as usize,
            ) {
                Ok(Some(context)) => context,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported digest hashing method: {}",
                        digest.hashing_method
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve key derivation context for digest method: {}",
                            digest.hashing_method
                        )
                    );
                    return Err(error);
                }
            };
        let mut validation_hash: Vec<u8> = vec![0; digest.digest.len()];

        match key_derivation_context.derive_key(master_key, &mut validation_hash) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to derive validation hash from master key"
                );
                return Err(error);
            }
        }
        Ok(validation_hash == digest.digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;

    use keramics_core::{open_fake_data_stream, open_os_data_stream};
    use keramics_hashes::Sha256Context;

    use crate::tests::get_test_data_path;

    fn get_encrypted_volume() -> Result<LuksEncryptedVolume, ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        Ok(encrypted_volume)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let encrypted_volume: LuksEncryptedVolume = get_encrypted_volume()?;

        let bytes_per_sector: u16 = encrypted_volume.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

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
    fn test_unlock_with_key_data() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks1.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        assert_eq!(encrypted_volume.is_locked, true);

        let credentials: Vec<LuksCredential> = vec![LuksCredential::KeyData {
            identifier: vec![
                0x20, 0xbc, 0x27, 0x95, 0x63, 0xf3, 0x4d, 0xc4, 0x80, 0xd8, 0x07, 0x91, 0x19, 0x13,
                0xa0, 0x31,
            ],
            data: vec![
                0xd2, 0xc4, 0x0d, 0xbb, 0xe4, 0xba, 0x5c, 0xe7, 0xf7, 0x85, 0xa4, 0x8a, 0xfc, 0x34,
                0xfa, 0x40, 0xfa, 0x90, 0x86, 0x76, 0xf7, 0xa1, 0x16, 0x40, 0x2c, 0xbb, 0xb3, 0x62,
                0x43, 0x29, 0x54, 0xed,
            ],
        }];
        encrypted_volume.unlock(&credentials)?;

        assert_eq!(encrypted_volume.is_locked, false);

        Ok(())
    }

    #[test]
    fn test_unlock_with_passphrase() -> Result<(), ErrorTrace> {
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

    #[test]
    fn test_read_data_stream_v2() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        let format_version: u16 = encrypted_volume.get_format_version();
        assert_eq!(format_version, 2);
        let bytes_per_sector: u16 = encrypted_volume.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 4096);
        let volume_size: u64 = encrypted_volume.get_volume_size();
        assert_eq!(volume_size, 3899392);
        let is_locked: bool = encrypted_volume.is_locked();
        assert!(is_locked);
        let identifier_string: String = encrypted_volume.get_identifier().to_string();
        assert_eq!(identifier_string, "a2b7a8e6-ce85-42ea-8479-7497a22f955c");
        let volume_label: Option<&ByteString> = encrypted_volume.get_volume_label();
        assert_eq!(volume_label, None);
        let encryption_type_string: String = encrypted_volume.get_encryption_type().to_string();
        assert_eq!(encryption_type_string, "AES-512-XTS");

        Ok(())
    }

    #[test]
    fn test_read_data_stream_v2_with_backup_header_fallback() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("luksde/luks2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let mut raw_data: Vec<u8> = fs::read(&path_buf).unwrap();

        // Corrupt the primary metadata checksum so reading falls back to the backup metadata area.
        raw_data[448] ^= 0xff;

        let data_stream: DataStreamReference = open_fake_data_stream(&raw_data);
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        encrypted_volume.read_data_stream(&data_stream)?;

        let format_version: u16 = encrypted_volume.get_format_version();
        assert_eq!(format_version, 2);
        let identifier_string: String = encrypted_volume.get_identifier().to_string();
        assert_eq!(identifier_string, "a2b7a8e6-ce85-42ea-8479-7497a22f955c");

        // Corrupt both primary and backup metadata checksums to verify checksum failure.
        let backup_checksum_offset: usize = 16384 + 448;
        raw_data[backup_checksum_offset] ^= 0xff;
        let corrupt_stream: DataStreamReference = open_fake_data_stream(&raw_data);
        let mut corrupt_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = corrupt_volume.read_data_stream(&corrupt_stream);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_unlock_with_key_data_v2() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        let is_locked: bool = encrypted_volume.is_locked();
        assert!(is_locked);

        // Wrong key size should not unlock.
        let wrong_credential: LuksCredential = LuksCredential::KeyData {
            identifier: vec![
                0xa2, 0xb7, 0xa8, 0xe6, 0xce, 0x85, 0x42, 0xea, 0x84, 0x79, 0x74, 0x97, 0xa2, 0x2f,
                0x95, 0x5c,
            ],
            data: vec![0; 32],
        };
        let wrong_credentials: Vec<LuksCredential> = vec![wrong_credential];
        let unlocked: bool = encrypted_volume.unlock(&wrong_credentials)?;
        assert!(!unlocked);
        assert!(encrypted_volume.is_locked);

        // Matching UUID and 64-byte key unlocks the volume.
        let credential: LuksCredential = LuksCredential::KeyData {
            identifier: vec![
                0xa2, 0xb7, 0xa8, 0xe6, 0xce, 0x85, 0x42, 0xea, 0x84, 0x79, 0x74, 0x97, 0xa2, 0x2f,
                0x95, 0x5c,
            ],
            data: vec![0x11; 64],
        };
        let credentials: Vec<LuksCredential> = vec![credential];
        let unlocked: bool = encrypted_volume.unlock(&credentials)?;
        assert!(unlocked);
        assert!(!encrypted_volume.is_locked);

        Ok(())
    }

    #[test]
    fn test_unlock_with_passphrase_v2() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        let path_string: String = get_test_data_path("luksde/luks2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        encrypted_volume.read_data_stream(&data_stream)?;

        let is_locked: bool = encrypted_volume.is_locked();
        assert!(is_locked);

        let credentials: Vec<LuksCredential> =
            vec![LuksCredential::Passphrase(b"KeRaMiCs".to_vec())];
        let unlocked: bool = encrypted_volume.unlock(&credentials)?;
        assert!(unlocked);
        assert!(!encrypted_volume.is_locked);

        // Verify decrypted volume contents contain the ext2 superblock signature, label,
        // and valid group descriptor table in sector 1 (offset 4096).
        let volume_stream: DataStreamReference = encrypted_volume.get_data_stream().unwrap();
        let mut superblock: [u8; 256] = [0; 256];
        keramics_core::data_stream_read_exact_at_position!(
            &volume_stream,
            &mut superblock,
            SeekFrom::Start(1024),
        );
        assert_eq!(&superblock[56..58], &[0x53, 0xef]);
        assert_eq!(&superblock[120..129], b"ext2_test");

        let mut ext_file_system: crate::ext::ExtFileSystem = crate::ext::ExtFileSystem::new();
        ext_file_system.read_data_stream(&volume_stream)?;
        let root_directory: Option<crate::ext::ExtFileEntry> =
            ext_file_system.get_root_directory()?;
        assert!(root_directory.is_some());
        let file_entry: Option<crate::ext::ExtFileEntry> =
            ext_file_system.get_file_entry_by_path(&crate::Path::from("/testdir1/testfile1"))?;
        assert!(file_entry.is_some());

        Ok(())
    }

    #[test]
    fn test_unlock_with_pbkdf2_v2() -> Result<(), ErrorTrace> {
        // Build a minimal synthetic LUKS2 volume with a PBKDF2 keyslot and 1-stripe AF diffuser
        // (16-byte key, 32 stripes = 512 bytes keyslot area, AES-128-ECB, SHA-256).
        let mut image: Vec<u8> = vec![0; 40960];
        let mut header: [u8; 4096] = [0; 4096];
        header[0..6].copy_from_slice(LUKS_VOLUME_HEADER_SIGNATURE);
        header[6..8].copy_from_slice(&2_u16.to_be_bytes());
        header[8..16].copy_from_slice(&16384_u64.to_be_bytes());
        header[16..24].copy_from_slice(&1_u64.to_be_bytes());
        header[24..34].copy_from_slice(b"test_label");
        header[72..78].copy_from_slice(b"sha256");
        header[168..204].copy_from_slice(b"11111111-2222-3333-4444-555555555555");

        // Zero salt ("AAAAAAAAAAAAAAAAAAAAAA==") and 1 iteration for fast deterministic test.
        let mut key_derivation_context: LuksKeyDerivationContext =
            LuksEncryption::get_key_derivation_context("sha256", &[0; 16], 1)?.unwrap();
        let mut user_key: [u8; 16] = [0; 16];
        key_derivation_context.derive_key(b"secret", &mut user_key)?;

        // Derive the candidate master key that decrypting all-zero ciphertext with user_key produces.
        let mut area_type: LuksEncryptionType = LuksEncryptionType::new();
        area_type.set_encryption_string("aes-ecb");
        area_type.key_size = 16;
        let area_context: LuksEncryptionContext =
            LuksEncryption::get_encryption_context(&area_type, &user_key)?.unwrap();
        let mut decrypted_split: [u8; 512] = [0; 512];
        area_context.decrypt_sector(0, &[0; 512], &mut decrypted_split)?;

        let mut diffuser: LuksDiffuserContext =
            LuksEncryption::get_diffuser_context("sha256").unwrap();
        let mut master_key: [u8; 16] = [0; 16];
        diffuser.merge(32, &decrypted_split, &mut master_key);

        let mut digest_key_derivation_context: LuksKeyDerivationContext =
            LuksEncryption::get_key_derivation_context("sha256", &[0; 16], 1)?.unwrap();
        let mut digest_bytes: [u8; 32] = [0; 32];
        digest_key_derivation_context.derive_key(&master_key, &mut digest_bytes)?;

        // Encode 32-byte digest as Base64 for JSON metadata.
        let base64_alphabet: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut digest_base64: String = String::new();
        let mut index: usize = 0;
        while index < digest_bytes.len() {
            let next_index1: usize = index + 1;
            let next_index2: usize = index + 2;
            let byte0: u32 = digest_bytes[index] as u32;
            let byte1: u32 = if next_index1 < digest_bytes.len() {
                digest_bytes[next_index1] as u32
            } else {
                0
            };
            let byte2: u32 = if next_index2 < digest_bytes.len() {
                digest_bytes[next_index2] as u32
            } else {
                0
            };
            let triple: u32 = (byte0 << 16) | (byte1 << 8) | byte2;
            let char0: usize = ((triple >> 18) & 0x3f) as usize;
            let char1: usize = ((triple >> 12) & 0x3f) as usize;
            let char2: usize = ((triple >> 6) & 0x3f) as usize;
            let char3: usize = (triple & 0x3f) as usize;
            digest_base64.push(base64_alphabet[char0] as char);
            digest_base64.push(base64_alphabet[char1] as char);
            if next_index1 < digest_bytes.len() {
                digest_base64.push(base64_alphabet[char2] as char);
            } else {
                digest_base64.push('=');
            }
            if next_index2 < digest_bytes.len() {
                digest_base64.push(base64_alphabet[char3] as char);
            } else {
                digest_base64.push('=');
            }
            index += 3;
        }

        let json_string: String = format!(
            concat!(
                "{{\"keyslots\":{{\"0\":{{\"type\":\"luks2\",\"key_size\":16,",
                "\"af\":{{\"type\":\"luks1\",\"stripes\":32,\"hash\":\"sha256\"}},",
                "\"area\":{{\"type\":\"raw\",\"offset\":\"32768\",\"size\":\"4096\",",
                "\"encryption\":\"aes-ecb\",\"key_size\":16}},",
                "\"kdf\":{{\"type\":\"pbkdf2\",\"hash\":\"sha256\",\"iterations\":1,",
                "\"salt\":\"AAAAAAAAAAAAAAAAAAAAAA==\"}}}}}},",
                "\"tokens\":{{}},",
                "\"segments\":{{\"0\":{{\"type\":\"crypt\",\"offset\":\"36864\",",
                "\"size\":\"4096\",\"iv_tweak\":\"0\",\"encryption\":\"aes-ecb\",",
                "\"sector_size\":512}}}},",
                "\"digests\":{{\"0\":{{\"type\":\"pbkdf2\",\"keyslots\":[\"0\"],",
                "\"segments\":[\"0\"],\"hash\":\"sha256\",\"iterations\":1,",
                "\"salt\":\"AAAAAAAAAAAAAAAAAAAAAA==\",\"digest\":\"{}\"}}}},",
                "\"config\":{{\"json_size\":\"12288\",\"keyslots_size\":\"4096\"}}}}"
            ),
            digest_base64
        );
        let json_bytes: &[u8] = json_string.as_bytes();
        let json_end_offset: usize = 4096 + json_bytes.len();
        image[4096..json_end_offset].copy_from_slice(json_bytes);

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&header);
        sha256_context.update(&image[4096..16384]);
        let checksum: Vec<u8> = sha256_context.finalize();
        header[448..480].copy_from_slice(&checksum);
        image[0..4096].copy_from_slice(&header);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut encrypted_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        encrypted_volume.read_data_stream(&data_stream)?;

        let expected_label: ByteString = ByteString::from("test_label");
        let volume_label: Option<&ByteString> = encrypted_volume.get_volume_label();
        assert_eq!(volume_label, Some(&expected_label));
        let volume_size: u64 = encrypted_volume.get_volume_size();
        assert_eq!(volume_size, 4096);

        let wrong_credentials: Vec<LuksCredential> =
            vec![LuksCredential::Passphrase(b"wrong".to_vec())];
        let unlocked: bool = encrypted_volume.unlock(&wrong_credentials)?;
        assert!(!unlocked);
        assert!(encrypted_volume.is_locked);

        let credentials: Vec<LuksCredential> = vec![LuksCredential::Passphrase(b"secret".to_vec())];
        let unlocked: bool = encrypted_volume.unlock(&credentials)?;
        assert!(unlocked);
        assert!(!encrypted_volume.is_locked);

        // Test with unlock called on an already unlocked volume.
        let already_unlocked: bool = encrypted_volume.unlock(&[])?;
        assert!(already_unlocked);

        Ok(())
    }

    /// Writes a checksummed LUKS2 metadata area (header + JSON) into `image` at `header_offset`.
    fn write_synthetic_v2_metadata_area(
        image: &mut [u8],
        header_offset: usize,
        is_backup: bool,
        metadata_area_size: u64,
        sequence_identifier: u64,
        label: &[u8],
        checksum_method: &[u8],
        uuid: &[u8],
        json_string: &str,
    ) {
        let mut header: [u8; 4096] = [0; 4096];
        let signature: &[u8] = if is_backup {
            LUKS_BACKUP_VOLUME_HEADER_SIGNATURE
        } else {
            LUKS_VOLUME_HEADER_SIGNATURE
        };
        header[0..6].copy_from_slice(signature);
        header[6..8].copy_from_slice(&2_u16.to_be_bytes());
        header[8..16].copy_from_slice(&metadata_area_size.to_be_bytes());
        header[16..24].copy_from_slice(&sequence_identifier.to_be_bytes());
        header[24..24 + label.len()].copy_from_slice(label);
        header[72..72 + checksum_method.len()].copy_from_slice(checksum_method);
        header[168..168 + uuid.len()].copy_from_slice(uuid);
        header[256..264].copy_from_slice(&(header_offset as u64).to_be_bytes());

        let json_area_offset: usize = header_offset + 4096;
        let json_area_end_offset: usize = header_offset + (metadata_area_size as usize);
        image[json_area_offset..json_area_end_offset].fill(0);
        let json_bytes: &[u8] = json_string.as_bytes();
        let json_end_offset: usize = json_area_offset + json_bytes.len();
        image[json_area_offset..json_end_offset].copy_from_slice(json_bytes);

        let mut sha256_context: Sha256Context = Sha256Context::new();
        sha256_context.update(&header);
        sha256_context.update(&image[json_area_offset..json_area_end_offset]);
        let checksum: Vec<u8> = sha256_context.finalize();
        header[448..480].copy_from_slice(&checksum);

        let header_end_offset: usize = header_offset + 4096;
        image[header_offset..header_end_offset].copy_from_slice(&header);
    }

    #[test]
    fn test_read_data_stream_v2_with_backup_sequence_preference() -> Result<(), ErrorTrace> {
        let valid_json: &str = concat!(
            "{\"keyslots\":{\"0\":{\"type\":\"luks2\",\"key_size\":16}},",
            "\"tokens\":{},",
            "\"segments\":{\"1\":{\"type\":\"crypt\",\"offset\":\"32768\",",
            "\"size\":\"4096\",\"iv_tweak\":\"0\",\"encryption\":\"aes-ecb\",",
            "\"sector_size\":512}},",
            "\"digests\":{},\"config\":{\"json_size\":\"12288\",\"keyslots_size\":\"4096\"}}"
        );
        let mut image: Vec<u8> = vec![0; 65536];
        write_synthetic_v2_metadata_area(
            &mut image,
            0,
            false,
            16384,
            1,
            b"old_label",
            b"sha256",
            b"11111111-2222-3333-4444-555555555555",
            valid_json,
        );
        write_synthetic_v2_metadata_area(
            &mut image,
            16384,
            true,
            16384,
            5,
            b"newer_backup",
            b"sha256",
            b"11111111-2222-3333-4444-555555555555",
            valid_json,
        );
        // Test with backup header having higher sequence identifier than valid primary.
        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        volume.read_data_stream(&data_stream)?;

        let expected_label: ByteString = ByteString::from("newer_backup");
        assert_eq!(volume.get_volume_label(), Some(&expected_label));
        // Also verifies non-"0" crypt segment fallback and key_size fallback without digest.
        assert_eq!(volume.key_size, 16);
        assert_eq!(volume.segment_identifier, "1");

        // Test with get_data_stream returning None while locked or unopened.
        let locked_stream: Option<DataStreamReference> = volume.get_data_stream();
        assert!(locked_stream.is_none());
        let unopened_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let unopened_stream: Option<DataStreamReference> = unopened_volume.get_data_stream();
        assert!(unopened_stream.is_none());

        // Test with corrupt primary and two valid backups at 16384 and 32768 selecting higher epoch.
        image[448] ^= 0xff;
        write_synthetic_v2_metadata_area(
            &mut image,
            32768,
            true,
            16384,
            9,
            b"newest_backup",
            b"sha256",
            b"11111111-2222-3333-4444-555555555555",
            valid_json,
        );
        let multi_backup_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut recovered_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        recovered_volume.read_data_stream(&multi_backup_stream)?;

        let expected_newest_label: ByteString = ByteString::from("newest_backup");
        assert_eq!(
            recovered_volume.get_volume_label(),
            Some(&expected_newest_label)
        );
        Ok(())
    }

    #[test]
    fn test_read_data_stream_errors() {
        // Test with unsupported signature.
        let bad_sig_stream: DataStreamReference = open_fake_data_stream(&vec![0; 4096]);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&bad_sig_stream);
        assert!(result.is_err());

        // Test with unsupported format version.
        let mut bad_version: Vec<u8> = vec![0; 4096];
        bad_version[0..6].copy_from_slice(LUKS_VOLUME_HEADER_SIGNATURE);
        bad_version[6..8].copy_from_slice(&3_u16.to_be_bytes());
        let bad_version_stream: DataStreamReference = open_fake_data_stream(&bad_version);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&bad_version_stream);
        assert!(result.is_err());

        // Test with LUKS1 encrypted data offset out of bounds.
        let mut bad_v1: Vec<u8> = vec![0; 4096];
        bad_v1[0..6].copy_from_slice(LUKS_VOLUME_HEADER_SIGNATURE);
        bad_v1[6..8].copy_from_slice(&1_u16.to_be_bytes());
        bad_v1[104..108].copy_from_slice(&9999_u32.to_be_bytes());
        bad_v1[168..204].copy_from_slice(b"11111111-2222-3333-4444-555555555555");
        let bad_v1_stream: DataStreamReference = open_fake_data_stream(&bad_v1);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&bad_v1_stream);
        assert!(result.is_err());

        // Test with LUKS2 validation errors via synthetic metadata areas.
        let valid_json: &str = concat!(
            "{\"keyslots\":{},\"tokens\":{},",
            "\"segments\":{\"0\":{\"type\":\"crypt\",\"offset\":\"16384\",",
            "\"size\":\"4096\",\"iv_tweak\":\"0\",\"encryption\":\"aes-ecb\",",
            "\"sector_size\":512}},",
            "\"digests\":{},\"config\":{}}"
        );
        let bad_jsons: [&str; 6] = [
            // Invalid JSON syntax.
            "{invalid",
            // Missing segment.
            "{\"keyslots\":{},\"tokens\":{},\"segments\":{},\"digests\":{},\"config\":{}}",
            // Unsupported segment type.
            "{\"segments\":{\"0\":{\"type\":\"linear\",\"offset\":\"16384\",\"size\":\"4096\",\"sector_size\":512}}}",
            // Unsupported sector size.
            "{\"segments\":{\"0\":{\"type\":\"crypt\",\"offset\":\"16384\",\"size\":\"4096\",\"sector_size\":256}}}",
            // Offset out of bounds.
            "{\"segments\":{\"0\":{\"type\":\"crypt\",\"offset\":\"999999\",\"size\":\"4096\",\"sector_size\":512}}}",
            // Segment size out of bounds.
            "{\"segments\":{\"0\":{\"type\":\"crypt\",\"offset\":\"16384\",\"size\":\"999999\",\"sector_size\":512}}}",
        ];
        for bad_json in bad_jsons {
            let mut image: Vec<u8> = vec![0; 24576];
            write_synthetic_v2_metadata_area(
                &mut image,
                0,
                false,
                16384,
                1,
                b"",
                b"sha256",
                b"11111111-2222-3333-4444-555555555555",
                bad_json,
            );
            let stream: DataStreamReference = open_fake_data_stream(&image);
            let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
            let result: Result<(), ErrorTrace> = volume.read_data_stream(&stream);
            assert!(result.is_err());
        }
        // Test with invalid UUID string.
        let mut bad_uuid_image: Vec<u8> = vec![0; 24576];
        write_synthetic_v2_metadata_area(
            &mut bad_uuid_image,
            0,
            false,
            16384,
            1,
            b"",
            b"sha256",
            b"not-a-valid-uuid",
            valid_json,
        );
        let stream: DataStreamReference = open_fake_data_stream(&bad_uuid_image);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&stream);
        assert!(result.is_err());

        // Test with unsupported metadata checksum method.
        let mut bad_hash_image: Vec<u8> = vec![0; 24576];
        write_synthetic_v2_metadata_area(
            &mut bad_hash_image,
            0,
            false,
            16384,
            1,
            b"",
            b"md5",
            b"11111111-2222-3333-4444-555555555555",
            valid_json,
        );
        let stream: DataStreamReference = open_fake_data_stream(&bad_hash_image);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&stream);
        assert!(result.is_err());

        // Test with mismatched header_offset and unsupported metadata_area_size.
        let mut bad_offset_image: Vec<u8> = vec![0; 24576];
        write_synthetic_v2_metadata_area(
            &mut bad_offset_image,
            0,
            false,
            16384,
            1,
            b"",
            b"sha256",
            b"11111111-2222-3333-4444-555555555555",
            valid_json,
        );
        bad_offset_image[256..264].copy_from_slice(&4096_u64.to_be_bytes());
        let stream: DataStreamReference = open_fake_data_stream(&bad_offset_image);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&stream);
        assert!(result.is_err());

        let mut bad_area_size_image: Vec<u8> = vec![0; 24576];
        bad_area_size_image[0..6].copy_from_slice(LUKS_VOLUME_HEADER_SIGNATURE);
        bad_area_size_image[6..8].copy_from_slice(&2_u16.to_be_bytes());
        bad_area_size_image[8..16].copy_from_slice(&12288_u64.to_be_bytes());
        let stream: DataStreamReference = open_fake_data_stream(&bad_area_size_image);
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        let result: Result<(), ErrorTrace> = volume.read_data_stream(&stream);
        assert!(result.is_err());

        // Test with header_offset out of bounds directly.
        let small_stream: DataStreamReference = open_fake_data_stream(&[0; 128]);
        let result: Result<(LuksVolumeHeaderV2, Vec<u8>), ErrorTrace> =
            LuksEncryptedVolume::read_v2_metadata_area(&small_stream, 128, 0, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_unlock_and_key_derivation_error_paths() -> Result<(), ErrorTrace> {
        let mut volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        volume.format_version = 2;
        volume.key_size = 16;
        volume.volume_identifier = Uuid::from_string("11111111-2222-3333-4444-555555555555")?;
        volume.segment_identifier = String::from("0");
        volume
            .encryption_type
            .set_encryption_string("twofish-cbc-plain");
        volume.encryption_type.key_size = 16;

        // Test KeyData credential with invalid identifier length and mismatched UUID.
        let wrong_id_len: LuksCredential = LuksCredential::KeyData {
            identifier: vec![0; 4],
            data: vec![0; 16],
        };
        let wrong_uuid: LuksCredential = LuksCredential::KeyData {
            identifier: vec![0xff; 16],
            data: vec![0; 16],
        };
        let unlocked: bool = volume.unlock(&[wrong_id_len, wrong_uuid])?;
        assert!(!unlocked);

        // Test KeyData matching UUID but unsupported volume encryption type.
        let matching_key: LuksCredential = LuksCredential::KeyData {
            identifier: vec![
                0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0x44, 0x44, 0x55, 0x55, 0x55, 0x55,
                0x55, 0x55,
            ],
            data: vec![0; 16],
        };
        let result: Result<bool, ErrorTrace> = volume.unlock(&[matching_key]);
        assert!(result.is_err());

        // Test validate_master_key_v2 error branches.
        let mut bad_digest: LuksMetadataDigest = LuksMetadataDigest::new(String::from("0"));
        bad_digest.digest_type = String::from("unknown");
        let result: Result<bool, ErrorTrace> =
            LuksEncryptedVolume::validate_master_key_v2(&bad_digest, &[0; 16]);
        assert!(result.is_err());

        bad_digest.digest_type = String::from("pbkdf2");
        let result: Result<bool, ErrorTrace> =
            LuksEncryptedVolume::validate_master_key_v2(&bad_digest, &[0; 16]);
        assert!(result.is_err());

        bad_digest.digest = vec![1; 32];
        bad_digest.hashing_method = String::from("unknown");
        let result: Result<bool, ErrorTrace> =
            LuksEncryptedVolume::validate_master_key_v2(&bad_digest, &[0; 16]);
        assert!(result.is_err());

        // Test derive_user_key_v2 error branches.
        let mut key_slot: LuksMetadataKeySlot = LuksMetadataKeySlot::new(String::from("0"));
        let mut user_key: [u8; 16] = [0; 16];
        key_slot.kdf_type = String::from("unknown");
        let result: Result<(), ErrorTrace> =
            volume.derive_user_key_v2(&key_slot, b"pass", &mut user_key);
        assert!(result.is_err());

        key_slot.kdf_type = String::from("pbkdf2");
        key_slot.kdf_hashing_method = String::from("unknown");
        let result: Result<(), ErrorTrace> =
            volume.derive_user_key_v2(&key_slot, b"pass", &mut user_key);
        assert!(result.is_err());

        key_slot.kdf_type = String::from("argon2id");
        key_slot.kdf_time = 0;
        let result: Result<(), ErrorTrace> =
            volume.derive_user_key_v2(&key_slot, b"pass", &mut user_key);
        assert!(result.is_err());

        // Test derive_master_key_v2 error branches.
        let mut master_key: [u8; 16] = [0; 16];
        key_slot.af_type = String::from("unknown");
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.af_type = String::from("luks1");
        key_slot.af_number_of_stripes = 0;
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.af_number_of_stripes = 32;
        key_slot.area_type = String::from("journal");
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.area_type = String::from("raw");
        key_slot.key_size = 16;
        key_slot.area_size = 100;
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.area_size = 4096;
        key_slot.af_number_of_stripes = 3;
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.af_number_of_stripes = 32;
        // Missing data stream.
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        // Attach fake data stream and test unsupported area encryption & AF hashing method.
        let raw_data: Vec<u8> = vec![0; 4096];
        volume.data_stream = Some(open_fake_data_stream(&raw_data));
        key_slot.area_encryption = String::from("twofish-cbc-plain");
        key_slot.area_key_size = 16;
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        key_slot.area_encryption = String::from("aes-ecb");
        key_slot.af_hashing_method = String::from("unknown");
        let result: Result<(), ErrorTrace> =
            volume.derive_master_key_v2(&key_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        // Test priority sorting with multiple keyslots and skipping keyslot without digest.
        let mut low_priority_slot: LuksMetadataKeySlot =
            LuksMetadataKeySlot::new(String::from("0"));
        low_priority_slot.keyslot_type = String::from("luks2");
        low_priority_slot.priority = Some(0);
        let mut high_priority_slot: LuksMetadataKeySlot =
            LuksMetadataKeySlot::new(String::from("1"));
        high_priority_slot.keyslot_type = String::from("luks2");
        high_priority_slot.priority = Some(2);
        volume.metadata_key_slots = vec![low_priority_slot, high_priority_slot];
        volume.metadata_digests = Vec::new();
        let pass_creds: Vec<LuksCredential> = vec![LuksCredential::Passphrase(b"pass".to_vec())];
        let unlocked: bool = volume.unlock(&pass_creds)?;
        assert!(!unlocked);

        // Test LUKS1 derive_master_key / derive_user_key / unlock error branches.
        let mut v1_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();
        v1_volume.format_version = 1;
        v1_volume.bytes_per_sector = 512;
        v1_volume.key_size = 16;
        v1_volume.hashing_method = String::from("unknown");
        let mut v1_slot: LuksKeySlot = LuksKeySlot::new();
        v1_slot.number_of_stripes = 0;
        let zero_stripe_slot: LuksKeySlot = LuksKeySlot::new();
        v1_volume.key_slots = vec![zero_stripe_slot];
        let result: Result<bool, ErrorTrace> = v1_volume.unlock(&pass_creds);
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> =
            v1_volume.derive_user_key(&v1_slot, b"pass", &mut user_key);
        assert!(result.is_err());

        v1_slot.number_of_stripes = 4001;
        let result: Result<(), ErrorTrace> =
            v1_volume.derive_master_key(&v1_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        v1_slot.number_of_stripes = 3;
        let result: Result<(), ErrorTrace> =
            v1_volume.derive_master_key(&v1_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        v1_slot.number_of_stripes = 32;
        // Missing data stream.
        let result: Result<(), ErrorTrace> =
            v1_volume.derive_master_key(&v1_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        v1_volume.data_stream = Some(open_fake_data_stream(&raw_data));
        v1_volume
            .encryption_type
            .set_encryption_string("twofish-cbc-plain");
        let result: Result<(), ErrorTrace> =
            v1_volume.derive_master_key(&v1_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        v1_volume.encryption_type.set_encryption_string("aes-ecb");
        let result: Result<(), ErrorTrace> =
            v1_volume.derive_master_key(&v1_slot, &user_key, &mut master_key);
        assert!(result.is_err());

        Ok(())
    }
}
