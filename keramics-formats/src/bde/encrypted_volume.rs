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
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encryption::AesCcmContext;
use keramics_types::{Ucs2String, Uuid, bytes_to_u32_le};

use super::aes_ccm_encrypted_key::BdeAesCcmEncryptedKey;
use super::block_range::{BdeBlockRange, BdeBlockRangeType};
use super::block_reader::BdeBlockReader;
use super::block_stream::BdeBlockStream;
use super::boot_record_togo::BdeBootRecordToGo;
use super::boot_record_used_disk_space::BdeBootRecordUsedDiskSpace;
use super::boot_record_v1::BdeBootRecordV1;
use super::boot_record_v2::BdeBootRecordV2;
use super::constants::*;
use super::credential::BdeCredential;
use super::encryption::BdeEncryption;
use super::encryption_context::BdeEncryptionContext;
use super::encryption_type::BdeEncryptionType;
use super::enums::{BdeFormatVersion, BdeKeyProtectorType};
use super::eow_block_map::BdeEowBlockMap;
use super::eow_block_record::BdeEowBlockRecord;
use super::eow_descriptor::BdeEowDescriptor;
use super::eow_relocation_log::BdeEowRelocationLog;
use super::key_protector::BdeKeyProtector;
use super::metadata_block::BdeMetadataBlock;
use super::password::BdePassword;
use super::recovery_password::BdeRecoveryPassword;
use super::volume_master_key::BdeVolumeMasterKey;

/// BitLocker Drive Encryption (BDE) encrypted volume.
pub struct BdeEncryptedVolume {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version (or variant).
    format_version: BdeFormatVersion,

    /// Volume identifier.
    volume_identifier: Uuid,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Encryption type.
    encryption_type: BdeEncryptionType,

    /// Description.
    description: Option<Ucs2String>,

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
            format_version: BdeFormatVersion::NotSet,
            volume_identifier: Uuid::new(),
            bytes_per_sector: 0,
            encryption_type: BdeEncryptionType::new(0),
            description: None,
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

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> &BdeFormatVersion {
        &self.format_version
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
        let mut eow_descriptor_offset1: u64 = 0;
        let mut eow_descriptor_offset2: u64 = 0;

        let metadata_block_offset1: u64;
        let metadata_block_offset2: u64;
        let metadata_block_offset3: u64;
        let metadata_block_size: usize;

        if &data[160..176] == BDE_IDENTIFIER {
            keramics_core::debug_trace_structure!(BdeBootRecordV2::debug_read_data(&data));

            let mut boot_record: BdeBootRecordV2 = BdeBootRecordV2::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read boot record version 2 at offset: {} (0x{:08x})",
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
            self.format_version = BdeFormatVersion::Version2;
        } else if &data[160..176] == BDE_USED_DISK_SPACE_ONLY_IDENTIFIER {
            keramics_core::debug_trace_structure!(BdeBootRecordUsedDiskSpace::debug_read_data(
                &data
            ));
            let mut boot_record: BdeBootRecordUsedDiskSpace = BdeBootRecordUsedDiskSpace::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Used Disk Space Only boot record at offset: {} (0x{:08x})",
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
            eow_descriptor_offset1 = boot_record.eow_descriptor_offset1;
            eow_descriptor_offset2 = boot_record.eow_descriptor_offset2;

            self.bytes_per_sector = boot_record.bytes_per_sector;
            self.format_version = BdeFormatVersion::UsedDiskSpaceOnly;
        } else if &data[424..440] == BDE_IDENTIFIER {
            keramics_core::debug_trace_structure!(BdeBootRecordToGo::debug_read_data(&data));

            let mut boot_record: BdeBootRecordToGo = BdeBootRecordToGo::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read To Go boot record at offset: {} (0x{:08x})",
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
            self.format_version = BdeFormatVersion::ToGo;
        } else if &data[3..11] == BDE_FILE_SYSTEM_SIGNATURE {
            keramics_core::debug_trace_structure!(BdeBootRecordV1::debug_read_data(&data));

            let mut boot_record: BdeBootRecordV1 = BdeBootRecordV1::new();

            match boot_record.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read boot record version 1 at offset: {} (0x{:08x})",
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
            self.format_version = BdeFormatVersion::Version1;
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
        self.volume_identifier = metadata_block.volume_identifier;
        self.encryption_type = BdeEncryptionType::new(metadata_block.encryption_method);

        if !metadata_block.description.is_empty() {
            self.description = Some(metadata_block.description);
        }
        self.full_volume_encryption_key = metadata_block.full_volume_encryption_key;
        self.key_protectors = metadata_block.key_protectors;

        // Metadata ranges (boot record and metadata blocks).
        let mut metadata_ranges: Vec<BdeBlockRange> = Vec::new();

        if metadata_block.mft_mirror_cluster_block_number != 0 {
            // Block range to map the BDE boot record to an NTFS boot record.
            metadata_ranges.push(BdeBlockRange::new(
                0,
                metadata_block.mft_mirror_cluster_block_number,
                self.bytes_per_sector as u64,
                BdeBlockRangeType::V1BootSector,
            ));
        } else if let Some(metadata_area_descriptors) = &metadata_block.metadata_area_descriptors {
            if metadata_area_descriptors.boot_record_offset == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid metadata areas descriptor - missing boot record offset",
                ));
            }
            if metadata_area_descriptors.boot_record_size == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid metadata areas descriptor - missing boot record size",
                ));
            }
            if self.bytes_per_sector == 0 {
                let bytes_per_sector: u64 = if metadata_block.boot_record_number_of_sectors == 0 {
                    0
                } else {
                    metadata_area_descriptors.boot_record_size
                        / (metadata_block.boot_record_number_of_sectors as u64)
                };
                if bytes_per_sector != 512 && bytes_per_sector != 4096 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported bytes per sector: {}",
                        bytes_per_sector
                    )));
                }
                self.bytes_per_sector = bytes_per_sector as u16;
            }
            // Block range to map the encrypted boot record to the start of the unlocked volume.
            metadata_ranges.push(BdeBlockRange::new(
                0,
                metadata_area_descriptors.boot_record_offset,
                metadata_area_descriptors.boot_record_size,
                BdeBlockRangeType::Encrypted,
            ));
            // Block range to hide the encrypted boot record.
            metadata_ranges.push(BdeBlockRange::new(
                metadata_area_descriptors.boot_record_offset,
                0,
                metadata_area_descriptors.boot_record_size,
                BdeBlockRangeType::Sparse,
            ));
        } else {
            if metadata_block.boot_record_offset == 0 {
                return Err(keramics_core::error_trace_new!(
                    "Unable to determine boot record offset",
                ));
            }
            // TODO: fallback if there are no metadata area descriptors.
            todo!();
        }
        // Block range to hide the metadata block 1.
        metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset1,
            0,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        // Block range to hide the metadata block 2.
        metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset2,
            0,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        // Block range to hide the metadata block 3.
        metadata_ranges.push(BdeBlockRange::new(
            metadata_block.metadata_block_offset3,
            0,
            metadata_block_size as u64,
            BdeBlockRangeType::Sparse,
        ));
        if let Some(metadata_area_descriptors) = &metadata_block.metadata_area_descriptors {
            if metadata_area_descriptors.unknown_area_offset > 0
                && metadata_area_descriptors.unknown_area_size > 0
            {
                // Block range to hide the unknown metadata area.
                metadata_ranges.push(BdeBlockRange::new(
                    metadata_area_descriptors.unknown_area_offset,
                    0,
                    metadata_area_descriptors.unknown_area_size,
                    BdeBlockRangeType::Sparse,
                ));
            }
        }
        if volume_size == 0 {
            volume_size = metadata_block.volume_size;
        }
        if volume_size == 0 {
            volume_size = data_stream_size;
        }
        self.volume_size = volume_size;

        let mut unencrypted_ranges: Vec<BdeBlockRange> = Vec::new();

        if eow_descriptor_offset1 > 0 {
            match self.read_encrypt_on_write_data(
                data_stream,
                eow_descriptor_offset1,
                eow_descriptor_offset2,
                &mut metadata_ranges,
                &mut unencrypted_ranges,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read Encrypt-on-Write (EOW) data"
                    );
                    return Err(error);
                }
            }
        }
        metadata_ranges.sort_by_key(|block_range| block_range.logical_offset);
        unencrypted_ranges.sort_by_key(|block_range| block_range.logical_offset);

        let mut adjusted_ranges: Vec<BdeBlockRange> = Vec::new();

        for mut unencrypted_range in unencrypted_ranges.drain(..) {
            let unencrypted_range_end_offset: u64 =
                unencrypted_range.logical_offset + unencrypted_range.size;

            for metadata_block_range in metadata_ranges.iter() {
                if metadata_block_range.logical_offset < unencrypted_range.logical_offset {
                    continue;
                }
                if metadata_block_range.logical_offset > unencrypted_range_end_offset {
                    break;
                }
                let metadata_range_end_offset: u64 =
                    metadata_block_range.logical_offset + metadata_block_range.size;

                if unencrypted_range.logical_offset < metadata_block_range.logical_offset {
                    let range_size: u64 =
                        metadata_block_range.logical_offset - unencrypted_range.logical_offset;
                    let adjusted_range: BdeBlockRange = BdeBlockRange::new(
                        unencrypted_range.logical_offset,
                        unencrypted_range.physical_offset,
                        range_size,
                        BdeBlockRangeType::InFile,
                    );
                    adjusted_ranges.push(adjusted_range);
                }
                unencrypted_range.logical_offset = metadata_range_end_offset;
                unencrypted_range.physical_offset = metadata_range_end_offset;
                unencrypted_range.size = unencrypted_range_end_offset.saturating_sub(metadata_range_end_offset);
            }
            if unencrypted_range.size > 0 {
                adjusted_ranges.push(unencrypted_range);
            }
        }
        metadata_ranges.append(&mut adjusted_ranges);
        metadata_ranges.sort_by_key(|block_range| block_range.logical_offset);

        // TODO: handle pre-EOW unencrypted ranges.
        // TODO: merge successive (sparse) ranges.
        let mut volume_offset: u64 = 0;

        for metadata_block_range in metadata_ranges.drain(..) {
            if metadata_block_range.logical_offset < volume_offset
                || metadata_block_range.logical_offset > self.volume_size
            {
                return Err(keramics_core::error_trace_new!(
                    "Invalid metadata block offset value out of bounds"
                ));
            }
            if volume_offset < metadata_block_range.logical_offset {
                let range_size: u64 = metadata_block_range.logical_offset - volume_offset;

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
        self.data_stream = Some(data_stream.clone());

        // TODO: check for clear key and unlock volume

        Ok(())
    }

    /// Reads the Encrypt-on-Write data.
    pub fn read_encrypt_on_write_data(
        &mut self,
        data_stream: &DataStreamReference,
        eow_descriptor_offset1: u64,
        eow_descriptor_offset2: u64,
        metadata_ranges: &mut Vec<BdeBlockRange>,
        unencrypted_ranges: &mut Vec<BdeBlockRange>,
    ) -> Result<(), ErrorTrace> {
        let mut eow_descriptor: BdeEowDescriptor = BdeEowDescriptor::new();

        match eow_descriptor.read_at_position(data_stream, SeekFrom::Start(eow_descriptor_offset1))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read Encrypt-on-Write (EOW) descriptor 1 at offset: {} (0x{:08x})",
                        eow_descriptor_offset1, eow_descriptor_offset1
                    )
                );
                return Err(error);
            }
        }
        // Block range to hide the Encrypt-on-Write descriptor 1.
        metadata_ranges.push(BdeBlockRange::new(
            eow_descriptor_offset1,
            0,
            4096,
            BdeBlockRangeType::Sparse,
        ));
        // Note that Encrypt-on-Write (EOW) descriptor 2 contains a copy of descriptor 1.
        if eow_descriptor_offset2 > 0 {
            // Block range to hide the Encrypt-on-Write descriptor 2.
            metadata_ranges.push(BdeBlockRange::new(
                eow_descriptor_offset2,
                0,
                4096,
                BdeBlockRangeType::Sparse,
            ));
        }
        for eow_block_map_area_offset in eow_descriptor.block_map_area_offsets.iter() {
            let mut eow_block_map: BdeEowBlockMap = BdeEowBlockMap::new();

            match eow_block_map.read_at_position(
                data_stream,
                eow_descriptor.physical_sector_size as usize,
                SeekFrom::Start(*eow_block_map_area_offset),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Encrypt-on-Write (EOW) block map at offset: {} (0x{:08x})",
                            *eow_block_map_area_offset, *eow_block_map_area_offset
                        ),
                    );
                    return Err(error);
                }
            }
            // TODO: check this.
            let eow_block_map_area_size: u32 = eow_block_map.block_map_size.next_multiple_of(4096);

            // Block range to hide the Encrypt-on-Write block map area.
            metadata_ranges.push(BdeBlockRange::new(
                *eow_block_map_area_offset,
                0,
                eow_block_map_area_size as u64,
                BdeBlockRangeType::Sparse,
            ));
            if eow_block_map.relocation_log_area_offset > 0 {
                let relocation_log_area_size: u32 = eow_descriptor
                    .relocation_log_area_size
                    .next_multiple_of(4096);

                // Block range to hide the Encrypt-on-Write relocation log area.
                metadata_ranges.push(BdeBlockRange::new(
                    eow_block_map.relocation_log_area_offset,
                    0,
                    relocation_log_area_size as u64,
                    BdeBlockRangeType::Sparse,
                ));
            }
            let eow_block_record_offset1: u64 =
                *eow_block_map_area_offset + (eow_block_map.block_record_offset1 as u64);

            let mut eow_block_record1: BdeEowBlockRecord =
                BdeEowBlockRecord::new(eow_descriptor.relocation_block_size);

            match eow_block_record1.read_at_position(
                data_stream,
                eow_block_map.block_record_size as usize,
                SeekFrom::Start(eow_block_record_offset1),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Encrypt-on-Write (EOW) block record 1 at offset: {} (0x{:08x})",
                            eow_block_record_offset1, eow_block_record_offset1
                        )
                    );
                    return Err(error);
                }
            }
            let eow_block_record_offset2: u64 =
                *eow_block_map_area_offset + (eow_block_map.block_record_offset2 as u64);

            let mut eow_block_record2: BdeEowBlockRecord =
                BdeEowBlockRecord::new(eow_descriptor.relocation_block_size);

            match eow_block_record2.read_at_position(
                data_stream,
                eow_block_map.block_record_size as usize,
                SeekFrom::Start(eow_block_record_offset2),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Encrypt-on-Write (EOW) block record 2 at offset: {} (0x{:08x})",
                            eow_block_record_offset2, eow_block_record_offset2
                        )
                    );
                    return Err(error);
                }
            }
            let last_block_record: &BdeEowBlockRecord =
                if eow_block_record1.sequence_number > eow_block_record2.sequence_number {
                    &eow_block_record1
                } else if eow_block_record1.sequence_number < eow_block_record2.sequence_number {
                    &eow_block_record2
                } else {
                    return Err(keramics_core::error_trace_new!(
                        "Unable to determine most recent Encrypt-on-Write (EOW) block record"
                    ));
                };

            // Note that the bitmap ranges are relative to eow_block_map.volume_region_offset.
            for bitmap_range in last_block_record.ranges.iter() {
                if bitmap_range.start_offset >= eow_block_map.volume_region_size {
                    break;
                }
                if !bitmap_range.is_set {
                    let range_offset: u64 =
                        eow_block_map.volume_region_offset + bitmap_range.start_offset;
                    let range_size: u64 =
                        min(bitmap_range.end_offset, eow_block_map.volume_region_size)
                            - bitmap_range.start_offset;

                    // Block range of unencrypted region.
                    unencrypted_ranges.push(BdeBlockRange::new(
                        range_offset,
                        range_offset,
                        range_size,
                        BdeBlockRangeType::InFile,
                    ));
                }
            }
            // TODO: remove, currently only used for format analysis
            if eow_block_map.relocation_log_area_offset > 0 {
                let mut eow_relocation_log: BdeEowRelocationLog = BdeEowRelocationLog::new();

                match eow_relocation_log.read_at_position(
                    data_stream,
                    eow_descriptor.relocation_log_area_size as usize,
                    SeekFrom::Start(eow_block_map.relocation_log_area_offset),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read Encrypt-on-Write (EOW) relocation log area at offset: {} (0x{:08x})",
                                eow_block_map.relocation_log_area_offset,
                                eow_block_map.relocation_log_area_offset,
                            )
                        );
                        return Err(error);
                    }
                }
            }
        }
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
                    let password_hash: Vec<u8> = match BdePassword::calculate_hash(passphrase) {
                        Ok(password_hash) => password_hash,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to calculate password hash"
                            );
                            return Err(error);
                        }
                    };
                    for (key_protector_index, key_protector) in
                        self.key_protectors.iter().enumerate()
                    {
                        if key_protector.protector_type == BdeKeyProtectorType::Passphrase {
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
                            match volume_master_key.unlock_with_password_hash(&password_hash) {
                                Ok(true) => {
                                    vmk_key = volume_master_key.key;
                                    vmk_key_unlocked = true;
                                }
                                Ok(false) => {}
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to unlock volume master key: {} with password",
                                            key_protector_index
                                        ),
                                    );
                                    return Err(error);
                                }
                            }
                        }
                    }
                    if vmk_key_unlocked {
                        break;
                    }
                }
                BdeCredential::RecoveryPassword(recovery_password) => {
                    let password_hash: Vec<u8> =
                        match BdeRecoveryPassword::calculate_hash(recovery_password) {
                            Ok(password_hash) => password_hash,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to calculate recovery password hash"
                                );
                                return Err(error);
                            }
                        };
                    for (key_protector_index, key_protector) in
                        self.key_protectors.iter().enumerate()
                    {
                        if key_protector.protector_type == BdeKeyProtectorType::RecoveryPassword {
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
                            match volume_master_key.unlock_with_password_hash(&password_hash) {
                                Ok(true) => {
                                    vmk_key = volume_master_key.key;
                                    vmk_key_unlocked = true;
                                }
                                Ok(false) => {}
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to unlock volume master key: {} with recovery password",
                                            key_protector_index
                                        ),
                                    );
                                    return Err(error);
                                }
                            }
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
                    if aes_ccm_encrypted_key.tag == tag {
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

                        if (key_data_size as usize) != self.encryption_type.get_fvek_size() {
                            return Err(keramics_core::error_trace_new!(
                                "Invalid FVEK - unsupported data size",
                            ));
                        }
                        let encryption_context: BdeEncryptionContext =
                            match BdeEncryption::get_encryption_context(
                                self.bytes_per_sector,
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
                        // TODO: determine or check unencrypted volume size

                        self.encryption_context = Some(encryption_context);
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

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let format_version: &BdeFormatVersion = encrypted_volume.get_format_version();
        assert_eq!(format_version, &BdeFormatVersion::Version2);

        Ok(())
    }

    #[test]
    fn test_get_encryption_type() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let encryption_type: &BdeEncryptionType = encrypted_volume.get_encryption_type();
        assert_eq!(encryption_type.method, 0x8002);

        Ok(())
    }

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

    #[test]
    fn test_get_number_of_key_protectors() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let number_of_key_protectors: usize = encrypted_volume.get_number_of_key_protectors();
        assert_eq!(number_of_key_protectors, 1);

        Ok(())
    }

    #[test]
    fn test_get_volume_size() -> Result<(), ErrorTrace> {
        let encrypted_volume: BdeEncryptedVolume = get_encrypted_volume()?;

        let volume_size: u64 = encrypted_volume.get_volume_size();
        assert_eq!(volume_size, 65994752);

        Ok(())
    }

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
