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
use keramics_encryption::AesCcmContext;
use keramics_types::{Ucs2String, Uuid, bytes_to_u32_le};

use super::aes_ccm_encrypted_key::BdeAesCcmEncryptedKey;
use super::block_range::{BdeBlockRange, BdeBlockRangeType};
use super::block_reader::BdeBlockReader;
use super::block_stream::BdeBlockStream;
use super::boot_record::BdeBootRecord;
use super::boot_record_togo::BdeBootRecordToGo;
use super::boot_record_vista::BdeBootRecordVista;
use super::constants::*;
use super::credential::BdeCredential;
use super::encryption::{BdeCipherContext, BdeEncryption};
use super::encryption_context::BdeEncryptionContext;
use super::encryption_type::BdeEncryptionType;
use super::enums::BdeKeyProtectorType;
use super::key_protector::BdeKeyProtector;
use super::metadata_block::BdeMetadataBlock;
use super::password::BdePassword;
use super::volume_master_key::BdeVolumeMasterKey;

/// BitLocker disk encryption (BDE) encrypted volume.
pub struct BdeEncryptedVolume {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Volume identifier.
    volume_identifier: Uuid,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Encryption type.
    encryption_type: BdeEncryptionType,

    /// Description.
    description: Option<Ucs2String>,

    /// Metadata ranges (boot record and metadata blocks).
    metadata_ranges: Vec<BdeBlockRange>,

    /// Full volume encryption key (FVEK).
    full_volume_encryption_key: Option<BdeAesCcmEncryptedKey>,

    /// Key protectors.
    key_protectors: Vec<BdeKeyProtector>,

    /// Block ranges.
    block_ranges: Vec<BdeBlockRange>,

    /// Encryption context.
    encryption_context: Option<BdeEncryptionContext>,

    /// The volume size.
    volume_size: u64,

    /// Value to indicate the container is locked.
    is_locked: bool,
}

impl BdeEncryptedVolume {
    /// Creates a new encrypted volume.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            volume_identifier: Uuid::new(),
            bytes_per_sector: 0,
            encryption_type: BdeEncryptionType::new(0),
            description: None,
            metadata_ranges: Vec::new(),
            full_volume_encryption_key: None,
            key_protectors: Vec::new(),
            block_ranges: Vec::new(),
            encryption_context: None,
            volume_size: 0,
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
                Some(encryption_context) => Some(Arc::new(RwLock::new(BdeBlockStream::new(
                    BdeBlockReader::new(
                        data_stream,
                        self.bytes_per_sector,
                        &self.block_ranges,
                        encryption_context,
                        self.volume_size,
                    ),
                )))),
                None => None,
            },
            None => None,
        }
    }

    /// Retrieves the description.
    pub fn get_description(&self) -> Option<&Ucs2String> {
        self.description.as_ref()
    }

    /// Retrieves the encryption type.
    pub fn get_encryption_type(&self) -> &BdeEncryptionType {
        &self.encryption_type
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.volume_identifier
    }

    /// Retrieves a specific of key protector.
    pub fn get_key_protector_by_index(
        &self,
        key_protector_index: usize,
    ) -> Option<&BdeKeyProtector> {
        self.key_protectors.get(key_protector_index)
    }

    /// Retrieves the number of key protectors.
    pub fn get_number_of_key_protectors(&self) -> usize {
        self.key_protectors.len()
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

        let mut data: [u8; 512] = [0; 512];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0),
        );
        keramics_core::debug_trace_data!("BdeBootSector", offset, &data, 512);

        let mut volume_size: u64 = 0;
        let mut boot_record_offset: u64 = 0;
        let metadata_block_offset1: u64;
        let metadata_block_offset2: u64;
        let metadata_block_offset3: u64;
        let metadata_block_size: usize;

        if &data[160..176] == BDE_IDENTIFIER
            || &data[160..176] == BDE_USED_DISK_SPACE_ONLY_IDENTIFIER
        {
            keramics_core::debug_trace_structure!(BdeBootRecord::debug_read_data(&data));

            let mut boot_record: BdeBootRecord = BdeBootRecord::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read boot record at offset: {} (0x{:08x})",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            }
            metadata_block_offset1 = boot_record.metadata_block_offset1;
            metadata_block_offset2 = boot_record.metadata_block_offset2;
            metadata_block_offset3 = boot_record.metadata_block_offset3;
            metadata_block_size = 65536;

            self.bytes_per_sector = boot_record.bytes_per_sector;
        } else if &data[424..440] == BDE_IDENTIFIER {
            keramics_core::debug_trace_structure!(BdeBootRecordToGo::debug_read_data(&data));

            let mut boot_record: BdeBootRecordToGo = BdeBootRecordToGo::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read ToGo boot record at offset: {} (0x{:08x})",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            }
            metadata_block_offset1 = boot_record.metadata_block_offset1;
            metadata_block_offset2 = boot_record.metadata_block_offset2;
            metadata_block_offset3 = boot_record.metadata_block_offset3;
            metadata_block_size = 65536;

            self.bytes_per_sector = boot_record.bytes_per_sector;
        } else if &data[0..3] == BDE_BOOT_ENTRY_POINT_VISTA {
            keramics_core::debug_trace_structure!(BdeBootRecordVista::debug_read_data(&data));

            let mut boot_record: BdeBootRecordVista = BdeBootRecordVista::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Vista boot record at offset: {} (0x{:08x})",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            }
            // NTFS omits the last sector from the number of sectors.
            volume_size =
                (boot_record.number_of_sectors + 1) * (boot_record.bytes_per_sector as u64);
            metadata_block_offset1 =
                boot_record.metadata_cluster_block_number * (boot_record.cluster_block_size as u64);
            metadata_block_offset2 = 0;
            metadata_block_offset3 = 0;
            metadata_block_size = 16384;

            self.bytes_per_sector = boot_record.bytes_per_sector;
        } else {
            return Err(keramics_core::error_trace_new!("Unsupported format"));
        }
        let mut metadata_block: BdeMetadataBlock = BdeMetadataBlock::new();

        match metadata_block.read_at_position(
            data_stream,
            metadata_block_size,
            SeekFrom::Start(metadata_block_offset1),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read metadata block at offset: {} (0x{:08x})",
                        metadata_block_offset1, metadata_block_offset1
                    ),
                );
                return Err(error);
            }
        }
        if metadata_block_offset1 != metadata_block.metadata_block_offset1 {
            return Err(keramics_core::error_trace_new!(
                "Invalid metadata block - metadata block offset 1 value does not value in boot record"
            ));
        }
        if metadata_block_offset2 != 0
            && metadata_block_offset2 != metadata_block.metadata_block_offset2
        {
            return Err(keramics_core::error_trace_new!(
                "Invalid metadata block - metadata block offset 2 value does not value in boot record"
            ));
        }
        if metadata_block_offset3 != 0
            && metadata_block_offset3 != metadata_block.metadata_block_offset3
        {
            return Err(keramics_core::error_trace_new!(
                "Invalid metadata block - metadata block offset 3 value does not value in boot record"
            ));
        }
        self.encryption_type = BdeEncryptionType::new(metadata_block.encryption_method);
        self.volume_identifier = metadata_block.volume_identifier;

        if !metadata_block.description.is_empty() {
            self.description = Some(metadata_block.description);
        }
        self.full_volume_encryption_key = metadata_block.full_volume_encryption_key;
        self.key_protectors = metadata_block.key_protectors;

        if boot_record_offset == 0 && metadata_block.boot_record_offset != 0 {
            boot_record_offset = metadata_block.boot_record_offset;
        }
        if boot_record_offset == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unable to determine boot record offset",
            ));
        }
        if metadata_block.boot_record_size == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unable to determine boot record size",
            ));
        }
        self.metadata_ranges.push(BdeBlockRange::new(
            0,
            boot_record_offset,
            metadata_block.boot_record_size,
            BdeBlockRangeType::Encrypted,
        ));
        self.metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset1,
            metadata_block.metadata_block_offset1,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        self.metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset2,
            metadata_block.metadata_block_offset2,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        self.metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset3,
            metadata_block.metadata_block_offset3,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        if volume_size == 0 {
            volume_size = metadata_block.volume_size;
        }
        if volume_size == 0 {
            volume_size = data_stream_size;
        }
        self.volume_size = volume_size;
        self.data_stream = Some(data_stream.clone());

        // TODO: check for clear key and unlock volume

        Ok(())
    }

    /// Unlocks a locked volume.
    pub fn unlock(&mut self, credentials: &[BdeCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut vmk_key: Vec<u8> = Vec::new();
        let mut vmk_key_unlocked: bool = false;

        for credential in credentials.iter() {
            match credential {
                BdeCredential::Passphrase(passphrase) => {
                    for (key_protector_index, key_protector) in
                        self.key_protectors.iter().enumerate()
                    {
                        match key_protector.protector_type {
                            BdeKeyProtectorType::Passphrase => {
                                let password_hash: Vec<u8> =
                                    match BdePassword::calculate_hash(passphrase) {
                                        Ok(password_hash) => password_hash,
                                        Err(mut error) => {
                                            keramics_core::error_trace_add_frame!(
                                                error,
                                                "Unable to calculate password hash"
                                            );
                                            return Err(error);
                                        }
                                    };
                                let mut volume_master_key: BdeVolumeMasterKey =
                                    BdeVolumeMasterKey::new();

                                match volume_master_key.read_at_position(
                                    data_stream,
                                    key_protector.size,
                                    SeekFrom::Start(key_protector.offset),
                                ) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable to read volume master key: {}",
                                                key_protector_index
                                            ),
                                        );
                                        return Err(error);
                                    }
                                }
                                match volume_master_key.unlock_with_password(&password_hash) {
                                    Ok(true) => {
                                        vmk_key = volume_master_key.key;
                                        vmk_key_unlocked = true;
                                    }
                                    Ok(false) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            format!(
                                                "Unable to unlock volume master key: {}",
                                                key_protector_index
                                            ),
                                        );
                                        return Err(error);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if vmk_key_unlocked {
                        break;
                    }
                }
                _ => {}
            }
            if vmk_key_unlocked {
                break;
            }
        }
        if vmk_key_unlocked {
            match self.full_volume_encryption_key.as_ref() {
                Some(aes_ccm_encrypted_key) => {
                    let vmk_key_size: usize = vmk_key.len();

                    keramics_core::debug_trace_data!(
                        "BdeVolumeMasterKey",
                        0,
                        &vmk_key,
                        vmk_key_size,
                    );
                    if vmk_key_size < 12 {
                        return Err(keramics_core::error_trace_new!("Unsupported VMK data size"));
                    }
                    let key_data_size: u32 = bytes_to_u32_le!(&vmk_key, 0);

                    if (key_data_size as usize) != 44 {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid VMK - unsupported data size"
                        ));
                    }
                    let mut ccm_context: AesCcmContext =
                        AesCcmContext::new(&aes_ccm_encrypted_key.nonce, &[]);

                    match ccm_context.set_key(&vmk_key[12..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to set key of AES-CCM context"
                            );
                            return Err(error);
                        }
                    };
                    let key_size: usize = aes_ccm_encrypted_key.encrypted_data.len();
                    let mut fvek_key: Vec<u8> = vec![0; key_size];
                    let mut tag: Vec<u8> = vec![0; 16];

                    match ccm_context.decrypt(
                        &aes_ccm_encrypted_key.encrypted_data,
                        &mut fvek_key,
                        &mut tag,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to decrypt AES-CCM encrypted key"
                            );
                            return Err(error);
                        }
                    };
                    if &aes_ccm_encrypted_key.tag == &tag {
                        let fvek_key_size: usize = fvek_key.len();

                        keramics_core::debug_trace_data!(
                            "BdeFullVolumeEncryptionKey",
                            0,
                            &fvek_key,
                            fvek_key_size
                        );
                        if fvek_key_size < 12 {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported FVEK data size"
                            ));
                        }
                        let key_data_size: u32 = bytes_to_u32_le!(&fvek_key, 0);

                        if (key_data_size as usize) != 12 + self.encryption_type.get_key_data_size()
                        {
                            return Err(keramics_core::error_trace_new!(
                                "Invalid FVEK - unsupported data size"
                            ));
                        }
                        let mut metadata_ranges: Vec<BdeBlockRange> = self.metadata_ranges.clone();
                        metadata_ranges.sort_by_key(|block_range| block_range.logical_offset);

                        // TODO: handle unencrypted ranges.
                        let mut volume_offset: u64 = 0;

                        for metadata_block_range in metadata_ranges.drain(..) {
                            if metadata_block_range.logical_offset > self.volume_size {
                                return Err(keramics_core::error_trace_new!(
                                    "Invalid metadata block offset value out of bounds"
                                ));
                            }
                            if volume_offset < metadata_block_range.logical_offset {
                                let range_size: u64 =
                                    metadata_block_range.logical_offset - volume_offset;

                                self.block_ranges.push(BdeBlockRange::new(
                                    volume_offset,
                                    volume_offset,
                                    range_size,
                                    BdeBlockRangeType::Encrypted,
                                ));
                                volume_offset += range_size;
                            }
                            volume_offset += metadata_block_range.size;

                            self.block_ranges.push(metadata_block_range);
                        }
                        let range_size: u64 = self.volume_size - volume_offset;

                        if range_size > 0 {
                            self.block_ranges.push(BdeBlockRange::new(
                                volume_offset,
                                volume_offset,
                                range_size,
                                BdeBlockRangeType::Encrypted,
                            ));
                        }
                        let cipher_context: BdeCipherContext =
                            match BdeEncryption::get_cipher_context(
                                &self.encryption_type,
                                &fvek_key[12..],
                            ) {
                                Ok(Some(cipher_context)) => cipher_context,
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
                        self.encryption_context = Some(BdeEncryptionContext::new(
                            self.bytes_per_sector,
                            cipher_context,
                        ));

                        // TODO: determine or check unencrypted volume size

                        self.is_locked = false;
                    }
                }
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing full volume encryption key (FVEK)"
                    ));
                }
            }
        }
        Ok(!self.is_locked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::open_os_data_stream;

    use crate::RangeStream;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;

    fn get_encrypted_volume() -> Result<BdeEncryptedVolume, ErrorTrace> {
        let mut encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

        let path_string: String = get_test_data_path("bde/bde_aes128.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        encrypted_volume.read_data_stream(&data_stream)?;

        Ok(encrypted_volume)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let bytes_per_sector: u16 = encrypted_volume.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_description() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let description: Option<&Ucs2String> = encrypted_volume.get_description();
        assert_eq!(
            description,
            Some(Ucs2String::from("TEST TestVolume 2026-09-04")).as_ref()
        );

        Ok(())
    }

    // TODO: add tests for get_encryption_type

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let identifier: &Uuid = encrypted_volume.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "fbdde069-e6b1-4cf9-8064-6b68d5955171",
        );
        Ok(())
    }

    // TODO: add tests for get_key_protector_by_index
    // TODO: add tests for get_number_of_key_protectors
    // TODO: add tests for get_volume_size

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let is_locked: bool = encrypted_volume.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

        let path_string: String = get_test_data_path("bde/bde_aes128.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        encrypted_volume.read_data_stream(&data_stream)?;

        assert_eq!(encrypted_volume.is_locked, true);

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

        let path_string: String = get_test_data_path("bde/bde_aes128.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        encrypted_volume.read_data_stream(&data_stream)?;

        assert_eq!(encrypted_volume.is_locked, true);

        let credentials: Vec<BdeCredential> = vec![BdeCredential::Passphrase(b"KeRaMiCs".to_vec())];
        encrypted_volume.unlock(&credentials)?;

        assert_eq!(encrypted_volume.is_locked, false);

        Ok(())
    }
}
