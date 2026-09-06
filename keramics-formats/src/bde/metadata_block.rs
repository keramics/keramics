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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{Ucs2String, Uuid};

use super::aes_ccm_encrypted_key::BdeAesCcmEncryptedKey;
use super::boot_record_descriptor::BdeBootRecordDescriptor;
use super::enums::BdeKeyProtectorType;
use super::key_protector::BdeKeyProtector;
use super::metadata_block_header::BdeMetadataBlockHeader;
use super::metadata_entry_header::BdeMetadataEntryHeader;
use super::metadata_header::BdeMetadataHeader;
use super::volume_master_key::BdeVolumeMasterKey;

/// BitLocker Drive Encryption (BDE) metadata block.
pub struct BdeMetadataBlock {
    /// Volume identifier.
    pub volume_identifier: Uuid,

    /// Description.
    pub description: Ucs2String,

    /// Encryption method.
    pub encryption_method: u16,

    /// Volume size.
    pub volume_size: u64,

    /// Encrypted volume size.
    pub encrypted_volume_size: u64,

    /// Metadata block offset 1.
    pub metadata_block_offset1: u64,

    /// Metadata block offset 2.
    pub metadata_block_offset2: u64,

    /// Metadata block offset 3.
    pub metadata_block_offset3: u64,

    /// Boot record offset.
    pub boot_record_offset: u64,

    /// Boot record size.
    pub boot_record_size: u64,

    /// Full volume encryption key (FVEK).
    pub full_volume_encryption_key: Option<BdeAesCcmEncryptedKey>,

    /// Key protectors.
    pub key_protectors: Vec<BdeKeyProtector>,
}

impl BdeMetadataBlock {
    /// Creates a new metadata block.
    pub fn new() -> Self {
        Self {
            volume_identifier: Uuid::new(),
            description: Ucs2String::new(),
            encryption_method: 0,
            volume_size: 0,
            encrypted_volume_size: 0,
            metadata_block_offset1: 0,
            metadata_block_offset2: 0,
            metadata_block_offset3: 0,
            boot_record_offset: 0,
            boot_record_size: 0,
            full_volume_encryption_key: None,
            key_protectors: Vec::new(),
        }
    }

    /// Reads the metadata block from a buffer.
    fn read_data(&mut self, data: &[u8], metadata_block_offset: u64) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 112 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        keramics_core::debug_trace_structure!(BdeMetadataBlockHeader::debug_read_data(data));

        let mut block_header: BdeMetadataBlockHeader = BdeMetadataBlockHeader::new();

        match block_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read metadata block header"
                );
                return Err(error);
            }
        }
        self.volume_size = block_header.volume_size;
        self.encrypted_volume_size = block_header.encrypted_volume_size;
        self.metadata_block_offset1 = block_header.metadata_block_offset1;
        self.metadata_block_offset2 = block_header.metadata_block_offset2;
        self.metadata_block_offset3 = block_header.metadata_block_offset3;
        self.boot_record_offset = block_header.boot_record_offset;

        keramics_core::debug_trace_structure!(BdeMetadataHeader::debug_read_data(&data[64..]));

        let mut header: BdeMetadataHeader = BdeMetadataHeader::new();

        match header.read_data(&data[64..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata header");
                return Err(error);
            }
        }
        if (header.metadata_size as usize) > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid metadata size value out of bounds"
            ));
        }
        self.volume_identifier = header.volume_identifier;
        self.encryption_method = header.encryption_method;

        let mut data_offset: usize = 112;
        let mut entry_index: usize = 0;

        while data_offset < (header.metadata_size as usize) {
            keramics_core::debug_trace_structure!(BdeMetadataEntryHeader::debug_read_data(
                &data[data_offset..]
            ));
            let mut entry_header: BdeMetadataEntryHeader = BdeMetadataEntryHeader::new();

            match entry_header.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read metadata entry: {} header", entry_index),
                    );
                    return Err(error);
                }
            }
            if entry_header.entry_size < 8
                || (entry_header.entry_size as usize) > data_size - data_offset
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid metadata entry: {} size value out of bounds",
                    entry_index
                )));
            }
            let entry_data_size: usize = (entry_header.entry_size as usize) - 8;

            data_offset += 8;

            let data_end_offset: usize = data_offset + entry_data_size;

            keramics_core::debug_trace_data!(
                format!("BdeMetadataEntryData: {}", entry_index),
                metadata_block_offset + (data_offset as u64),
                &data[data_offset..data_end_offset],
                entry_data_size
            );
            match entry_header.entry_type {
                0x0002 => {
                    if entry_header.value_type != 0x0008 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid metadata entry: {} of type: 0x{:04x} unsupported value type: 0x{:04x}",
                            entry_index, entry_header.entry_type, entry_header.value_type
                        )));
                    }
                    keramics_core::debug_trace_structure!(BdeVolumeMasterKey::debug_read_data(
                        &data[data_offset..data_end_offset]
                    ));
                    let mut volume_master_key: BdeVolumeMasterKey = BdeVolumeMasterKey::new();

                    match volume_master_key.read_data(&data[data_offset..data_end_offset]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read volume master key (VMK) metadata entry: {}",
                                    entry_index
                                )
                            );
                            return Err(error);
                        }
                    }
                    let key_protector_type: BdeKeyProtectorType =
                        match &volume_master_key.protector_type {
                            0x0000 => BdeKeyProtectorType::ClearKey,
                            0x0100 => BdeKeyProtectorType::Tpm,
                            0x0200 => BdeKeyProtectorType::ExternalKey,
                            0x0500 => BdeKeyProtectorType::TpmAndPin,
                            0x0800 => BdeKeyProtectorType::RecoveryPassphrase,
                            0x2000 => BdeKeyProtectorType::Passphrase,
                            _ => BdeKeyProtectorType::Unknown(volume_master_key.protector_type),
                        };
                    let key_protector: BdeKeyProtector = BdeKeyProtector::new(
                        key_protector_type,
                        volume_master_key.identifier,
                        metadata_block_offset + (data_offset as u64),
                        entry_data_size as u16,
                    );
                    self.key_protectors.push(key_protector);
                }
                0x0003 => {
                    if entry_header.value_type != 0x0005 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid metadata entry: {} of type: 0x{:04x} unsupported value type: 0x{:04x}",
                            entry_index, entry_header.entry_type, entry_header.value_type
                        )));
                    }
                    keramics_core::debug_trace_structure!(BdeAesCcmEncryptedKey::debug_read_data(
                        &data[data_offset..data_end_offset]
                    ));
                    let mut full_volume_encryption_key: BdeAesCcmEncryptedKey =
                        BdeAesCcmEncryptedKey::new();

                    match full_volume_encryption_key.read_data(&data[data_offset..data_end_offset])
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read full volume encryption key (FVEK) metadata entry: {}",
                                    entry_index
                                )
                            );
                            return Err(error);
                        }
                    }
                    if self.full_volume_encryption_key.is_some() {
                        return Err(keramics_core::error_trace_new!(
                            "Full volume encryption key (FVEK) already set",
                        ));
                    }
                    self.full_volume_encryption_key = Some(full_volume_encryption_key);
                }
                0x0007 => {
                    self.description
                        .read_data_le(&data[data_offset..data_end_offset]);
                }
                0x000f => {
                    if entry_header.value_type != 0x000f {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid metadata entry: {} of type: 0x{:04x} unsupported value type: 0x{:04x}",
                            entry_index, entry_header.entry_type, entry_header.value_type
                        )));
                    }
                    keramics_core::debug_trace_structure!(
                        BdeBootRecordDescriptor::debug_read_data(
                            &data[data_offset..data_end_offset]
                        )
                    );
                    let mut boot_record_descriptor: BdeBootRecordDescriptor =
                        BdeBootRecordDescriptor::new();

                    match boot_record_descriptor.read_data(&data[data_offset..data_end_offset]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read boot record descriptor metadata entry: {}",
                                    entry_index
                                )
                            );
                            return Err(error);
                        }
                    }
                    if self.boot_record_offset == 0 {
                        self.boot_record_offset = boot_record_descriptor.boot_record_offset;
                    } else if boot_record_descriptor.boot_record_offset != self.boot_record_offset {
                        return Err(keramics_core::error_trace_new!(
                            "Boot record offset in block header does not match value in boot record descriptor"
                        ));
                    }
                    self.boot_record_size = boot_record_descriptor.boot_record_size;
                }
                _ => {}
            }
            data_offset = data_end_offset;
            entry_index += 1;
        }
        Ok(())
    }

    /// Reads the metadata block from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: usize,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        if data_size < 112 || data_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported metadata block data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("BdeMetadataBlock", offset, &data, data_size);

        match self.read_data(&data, offset) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read metadata block at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x2d, 0x46, 0x56, 0x45, 0x2d, 0x46, 0x53, 0x2d, 0x24, 0x00, 0x02, 0x00, 0x04, 0x00,
            0x04, 0x00, 0x00, 0x00, 0xef, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60,
            0x94, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0xf2, 0x01, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0xf2, 0x01, 0x00, 0x00, 0x69, 0xe0, 0xdd, 0xfb,
            0xb1, 0xe6, 0xf9, 0x4c, 0x80, 0x64, 0x6b, 0x68, 0xd5, 0x95, 0x51, 0x71, 0x08, 0x00,
            0x00, 0x00, 0x02, 0x80, 0x02, 0x80, 0x73, 0x1c, 0x01, 0x13, 0x3a, 0x3c, 0xdd, 0x01,
            0x3e, 0x00, 0x07, 0x00, 0x02, 0x00, 0x01, 0x00, 0x54, 0x00, 0x45, 0x00, 0x53, 0x00,
            0x54, 0x00, 0x20, 0x00, 0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x56, 0x00,
            0x6f, 0x00, 0x6c, 0x00, 0x75, 0x00, 0x6d, 0x00, 0x65, 0x00, 0x20, 0x00, 0x32, 0x00,
            0x30, 0x00, 0x32, 0x00, 0x36, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x39, 0x00, 0x2d, 0x00,
            0x30, 0x00, 0x34, 0x00, 0x00, 0x00, 0xe0, 0x00, 0x02, 0x00, 0x08, 0x00, 0x01, 0x00,
            0xd8, 0xd4, 0x0a, 0x55, 0x07, 0xef, 0xe3, 0x4a, 0xb8, 0x1a, 0x02, 0xee, 0x00, 0x0e,
            0x85, 0x7b, 0xf0, 0x01, 0x7c, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x00, 0x00, 0x00, 0x20,
            0x6c, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x01, 0x10, 0x00, 0x00, 0xfe, 0xdc,
            0xfa, 0xa5, 0xe2, 0x6e, 0xe3, 0x88, 0x0d, 0x2b, 0xdb, 0x2e, 0xe4, 0xe4, 0x42, 0x8f,
            0x50, 0x00, 0x00, 0x00, 0x05, 0x00, 0x01, 0x00, 0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c,
            0xdd, 0x01, 0x02, 0x00, 0x00, 0x00, 0x1f, 0x36, 0xdb, 0xbe, 0x1c, 0x3e, 0x0a, 0xd2,
            0x95, 0x02, 0x90, 0x07, 0xf8, 0xd8, 0x1e, 0xa0, 0xb4, 0x83, 0x96, 0xae, 0x1a, 0xa4,
            0xb1, 0xa1, 0xab, 0x59, 0x0f, 0xda, 0xeb, 0xc7, 0x4c, 0x70, 0x23, 0x49, 0xb8, 0x86,
            0xbd, 0x9c, 0x79, 0x53, 0xc7, 0x51, 0x48, 0x09, 0x9b, 0x52, 0xeb, 0xe7, 0xc3, 0xd9,
            0x06, 0xf2, 0x47, 0x08, 0x0f, 0xeb, 0x46, 0x6a, 0x94, 0x19, 0x50, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x00, 0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x03, 0x00,
            0x00, 0x00, 0x61, 0x99, 0x52, 0x91, 0xa6, 0x7a, 0xf5, 0xb7, 0x7d, 0x49, 0x43, 0x15,
            0xae, 0x32, 0x2e, 0x1b, 0xed, 0x99, 0x40, 0x76, 0xcc, 0xd0, 0x22, 0x54, 0xd2, 0xcf,
            0x82, 0xfd, 0x2e, 0x92, 0x36, 0x53, 0xbb, 0x6a, 0xab, 0x0f, 0xd2, 0x50, 0x91, 0xff,
            0x7e, 0xa9, 0xe1, 0x0b, 0x61, 0xc6, 0x12, 0x52, 0xe3, 0xc7, 0x94, 0xd7, 0xe3, 0x92,
            0x04, 0x47, 0xbf, 0x42, 0x26, 0x7a, 0x40, 0x00, 0x03, 0x00, 0x05, 0x00, 0x01, 0x00,
            0xe0, 0xbc, 0x80, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x06, 0x00, 0x00, 0x00, 0x51, 0xd5,
            0xea, 0x1d, 0x0f, 0xe1, 0x9b, 0xc9, 0x4c, 0x13, 0x73, 0x19, 0xfc, 0x22, 0xdc, 0x23,
            0xff, 0xc9, 0xea, 0xc0, 0x1b, 0x50, 0x52, 0x2c, 0x3a, 0x6b, 0x2b, 0xad, 0x2c, 0x8a,
            0x1a, 0xa8, 0x83, 0xed, 0x76, 0x84, 0x33, 0xb6, 0x22, 0xd6, 0xb1, 0xe0, 0x22, 0x5b,
            0x64, 0x00, 0x0f, 0x00, 0x0f, 0x00, 0x01, 0x00, 0x00, 0x00, 0x20, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x4c, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x4a,
            0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x01, 0x01, 0x02,
            0x00, 0x20, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xc0, 0xfd, 0x02, 0x00, 0x3e, 0xa0, 0x01, 0x03, 0x50, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x00, 0x67, 0x36, 0x83, 0x19, 0x3a, 0x3c, 0xdd, 0x81, 0x07, 0x00,
            0x00, 0x00, 0xcc, 0x6b, 0xe9, 0x59, 0xe7, 0x02, 0xb2, 0xd0, 0x66, 0x45, 0x62, 0x59,
            0x43, 0x8d, 0x1e, 0x7e, 0x01, 0x9e, 0x8a, 0x4c, 0x6d, 0x4a, 0x89, 0x3a, 0x74, 0x98,
            0x88, 0x27, 0x26, 0x9f, 0xcf, 0x72, 0x05, 0x80, 0x8c, 0xc3, 0xc7, 0xa7, 0x72, 0xe0,
            0xc2, 0xd2, 0xee, 0x22, 0xaf, 0x95, 0x29, 0x59, 0x79, 0xd7, 0xdb, 0xd4, 0x4e, 0xaf,
            0x13, 0x75, 0x29, 0xcf, 0xc0, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataBlock::new();
        test_struct.read_data(&test_data, 0)?;

        assert_eq!(
            test_struct.volume_identifier.to_string(),
            "fbdde069-e6b1-4cf9-8064-6b68d5955171",
        );
        assert_eq!(
            test_struct.description,
            Ucs2String::from("TEST TestVolume 2026-09-04")
        );
        assert_eq!(test_struct.key_protectors.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = BdeMetadataBlock::new();
        let result = test_struct.read_data(&test_data, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = BdeMetadataBlock::new();
        test_struct.read_at_position(&data_stream, 1024, SeekFrom::Start(0))?;

        assert_eq!(
            test_struct.volume_identifier.to_string(),
            "fbdde069-e6b1-4cf9-8064-6b68d5955171",
        );
        assert_eq!(
            test_struct.description,
            Ucs2String::from("TEST TestVolume 2026-09-04")
        );
        assert_eq!(test_struct.key_protectors.len(), 1);

        Ok(())
    }
}
