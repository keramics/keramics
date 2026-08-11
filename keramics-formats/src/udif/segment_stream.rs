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

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_table_reader::UdifBlockTableReader;
use super::credential::{UdifCredential, UdifCredentialType};
use super::encrypted_file_footer::UdifEncryptedFileFooter;
use super::encryption::{UdifEncryption, UdifEncryptionContext, UdifHmacContext};
use super::encryption_type::UdifEncryptionType;
use super::enums::UdifKeyProtectorType;
use super::file::UdifFile;
use super::key_protector::UdifKeyProtector;
use super::passphrase_wrapped_key::UdifPassphraseWrappedKey;
use super::public_key_wrapped_key::UdifPublicKeyWrappedKey;
use super::segment_range::UdifSegmentRange;

/// Universal Disk Image Format (UDIF) segment stream.
pub struct UdifSegmentStream {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Segment file set identifier.
    pub segment_set_identifier: Uuid,

    /// Number of segments.
    pub number_of_segments: u32,

    /// Name.
    name: String,

    /// Segment ranges.
    segment_ranges: Vec<UdifSegmentRange>,

    /// Segment file cache.
    segment_file_cache: LruCache<u32, UdifFile>,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Block size.
    pub block_size: u32,

    /// Decrypted block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Value to indicate the (encrypted) image is locked.
    pub is_locked: bool,

    /// Encryption type.
    pub encryption_type: UdifEncryptionType,

    /// Key key_protectors.
    key_protectors: Vec<UdifKeyProtector>,

    /// Encryption context.
    encryption_context: UdifEncryptionContext,

    /// HMAC context.
    hmac_context: UdifHmacContext,

    /// The current offset.
    current_offset: u64,

    /// The size.
    pub size: u64,
}

impl UdifSegmentStream {
    /// Creates a new segment stream.
    pub(super) fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            segment_set_identifier: Uuid::new(),
            number_of_segments: 0,
            name: String::new(),
            segment_ranges: Vec::new(),
            segment_file_cache: LruCache::new(16),
            number_of_sectors: 0,
            block_size: 0,
            block_cache: LruCache::new(64),
            is_locked: false,
            encryption_type: UdifEncryptionType::new(),
            key_protectors: Vec::new(),
            encryption_context: UdifEncryptionContext::None,
            hmac_context: UdifHmacContext::None,
            current_offset: 0,
            size: 0,
        }
    }

    /// Determines the segment file name.
    fn get_segment_file_name(name: &String, segment_number: u32) -> String {
        if segment_number == 1 {
            format!("{}.dmg", name)
        } else {
            format!("{}.{:03}.dmgpart", name, segment_number)
        }
    }

    /// Opens a segment stream.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        match self.read_segment_files(&file_resolver, file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Opens a segment file.
    fn open_segment_file(&self, segment_file_name: &String) -> Result<UdifFile, ErrorTrace> {
        let path_components: [PathComponent; 1] = [PathComponent::from(segment_file_name)];

        let data_stream: DataStreamReference =
            match self.file_resolver.get_data_stream(&path_components) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing segment file: {}",
                        segment_file_name
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
        let mut segment_file: UdifFile = UdifFile::new();

        match segment_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read segment file: {}", segment_file_name)
                );
                return Err(error);
            }
        }
        Ok(segment_file)
    }

    /// Reads the segment files.
    fn read_segment_files(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        self.name = match file_name.file_stem() {
            Ok(Some(file_stem)) => file_stem.to_string(),
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing file stem in segment file: {}",
                    file_name,
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file stem of segment file: {}",
                        file_name,
                    )
                );
                return Err(error);
            }
        };
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing segment file: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open segment file: {}", file_name)
                );
                return Err(error);
            }
        };
        let mut segment_file: UdifFile = UdifFile::new();

        match segment_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read segment file: {}", file_name)
                );
                return Err(error);
            }
        }
        self.number_of_sectors = segment_file.number_of_sectors;
        self.number_of_segments = segment_file.number_of_segments;

        let segment_number: u32 = segment_file.segment_number;

        if self.number_of_segments == 0 && segment_number == 0 {
            self.number_of_segments = 1
        } else if self.number_of_segments != 0 && segment_number != 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment number value out of bounds"
            ));
        }
        if segment_file.plist_size != 0 && segment_file.resource_fork_size != 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - both XML plist and resource fork in use"
            ));
        }
        if segment_file.segment_offset != self.size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment offset value out of bounds"
            ));
        }
        self.size += segment_file.data_fork_size;

        let segment_range: UdifSegmentRange = UdifSegmentRange::new(
            segment_file.segment_offset,
            1,
            segment_file.data_fork_offset,
            segment_file.data_fork_size,
        );
        self.segment_ranges.push(segment_range);

        self.segment_set_identifier = segment_file.segment_set_identifier.clone();
        self.block_size = segment_file.block_size;
        self.is_locked = segment_file.is_locked;
        self.encryption_type = segment_file.encryption_type.clone();
        self.key_protectors = segment_file.key_protectors;
        segment_file.key_protectors = Vec::new();

        self.segment_file_cache.insert(1, segment_file);

        for segment_number in 2..=self.number_of_segments {
            let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);
            let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing segment file: {}",
                            segment_file_name
                        )));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open segment file: {}", segment_file_name)
                        );
                        return Err(error);
                    }
                };
            let mut segment_file: UdifFile = UdifFile::new();

            match segment_file.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            }
            if segment_file.segment_number != segment_number {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment number value out of bounds",
                    segment_number
                )));
            }
            if &segment_file.segment_set_identifier != &self.segment_set_identifier {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment set identifier mismatch",
                    segment_number
                )));
            }
            if segment_file.plist_size != 0 || segment_file.resource_fork_size != 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - XML plist and/or resource fork in use",
                    segment_number
                )));
            }
            if segment_file.segment_offset != self.size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment offset value out of bounds",
                    segment_number
                )));
            }
            if segment_file.number_of_sectors != self.number_of_sectors {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - number of sectors value out of bounds",
                    segment_number
                )));
            }
            self.size += segment_file.data_fork_size;

            let segment_range: UdifSegmentRange = UdifSegmentRange::new(
                segment_file.segment_offset,
                segment_number,
                segment_file.data_fork_offset,
                segment_file.data_fork_size,
            );
            self.segment_ranges.push(segment_range);
        }
        Ok(())
    }

    /// Reads the metadata in the XML plist or resource fork in the first segment file.
    pub(super) fn read_metadata(
        &mut self,
        bytes_per_sector: u16,
    ) -> Result<UdifBlockTableReader, ErrorTrace> {
        let segment_number: u32 = 1;

        if !self.segment_file_cache.contains(&segment_number) {
            let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

            let segment_file: UdifFile = match self.open_segment_file(&segment_file_name) {
                Ok(udif_file) => udif_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
            self.segment_file_cache.insert(segment_number, segment_file);
        }
        let first_segment_file: &mut UdifFile =
            match self.segment_file_cache.get_mut(&segment_number) {
                Some(segment_file) => segment_file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_number
                    )));
                }
            };
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(bytes_per_sector, self.size);

        if first_segment_file.plist_size == 0 && first_segment_file.resource_fork_size == 0 {
            block_table_reader.media_offset = self.size;
        } else {
            match first_segment_file.read_metadata(&mut block_table_reader) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read metadata from segment file: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(block_table_reader)
    }

    /// Reads media data based on the segment files.
    fn read_data_from_segments(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut segment_offset: u64 = self.current_offset;

        let mut segment_index: usize = self.segment_ranges.partition_point(|segment_range| {
            let segment_end_offset: u64 = segment_range.segment_offset + segment_range.size;
            segment_offset >= segment_end_offset
        });
        while data_offset < read_size {
            if segment_offset >= self.size {
                break;
            }
            let segment_range: &UdifSegmentRange = match self.segment_ranges.get(segment_index) {
                Some(segment_range) => segment_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment range for offset: {} (0x{:08x})",
                        segment_offset, segment_offset
                    )));
                }
            };
            let range_relative_offset: u64 = segment_offset - segment_range.segment_offset;
            let range_remainder_size: u64 = segment_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            if !self
                .segment_file_cache
                .contains(&segment_range.segment_number)
            {
                let segment_file_name: String =
                    Self::get_segment_file_name(&self.name, segment_range.segment_number);

                let segment_file: UdifFile = match self.open_segment_file(&segment_file_name) {
                    Ok(udif_file) => udif_file,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open segment file: {}", segment_file_name)
                        );
                        return Err(error);
                    }
                };
                self.segment_file_cache
                    .insert(segment_range.segment_number, segment_file);
            }
            let segment_file: &mut UdifFile = match self
                .segment_file_cache
                .get_mut(&segment_range.segment_number)
            {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_range.segment_number
                    )));
                }
            };
            let segment_range_offset: u64 = segment_range.data_offset + range_relative_offset;

            match segment_file.read_exact_at_position(
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(segment_range_offset),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read from segment file: {} at offset: {} (0x{:08x})",
                            segment_range.segment_number,
                            segment_range_offset,
                            segment_range_offset
                        )
                    );
                    return Err(error);
                }
            }
            data_offset += range_read_size;
            segment_offset += range_read_size as u64;
            segment_index += 1;
        }
        Ok(data_offset)
    }

    /// Reads media data from encrypted blocks.
    pub fn read_data_from_encrypted_blocks(
        &mut self,
        data: &mut [u8],
    ) -> Result<usize, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Image is locked"));
        }
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut segment_offset: u64 = self.current_offset;

        let mut block_number: u64 = segment_offset / (self.block_size as u64);
        let mut block_offset: u64 = block_number * (self.block_size as u64);

        if block_number > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(
                "Invalid block number value out of bounds"
            ));
        }
        let mut segment_index: usize = self.segment_ranges.partition_point(|segment_range| {
            let segment_end_offset: u64 = segment_range.segment_offset + segment_range.size;
            segment_offset >= segment_end_offset
        });
        while data_offset < read_size {
            if segment_offset >= self.size {
                break;
            }
            let segment_range: &UdifSegmentRange = match self.segment_ranges.get(segment_index) {
                Some(segment_range) => segment_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment range for offset: {} (0x{:08x})",
                        segment_offset, segment_offset
                    )));
                }
            };
            if !self
                .segment_file_cache
                .contains(&segment_range.segment_number)
            {
                let segment_file_name: String =
                    Self::get_segment_file_name(&self.name, segment_range.segment_number);

                let segment_file: UdifFile = match self.open_segment_file(&segment_file_name) {
                    Ok(udif_file) => udif_file,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open segment file: {}", segment_file_name)
                        );
                        return Err(error);
                    }
                };
                self.segment_file_cache
                    .insert(segment_range.segment_number, segment_file);
            }
            let segment_file: &mut UdifFile = match self
                .segment_file_cache
                .get_mut(&segment_range.segment_number)
            {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_range.segment_number
                    )));
                }
            };
            let segment_range_end_offset: u64 = segment_range.segment_offset + segment_range.size;

            while segment_offset < segment_range_end_offset {
                let range_relative_offset: u64 = segment_offset - block_offset;
                let range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

                let range_read_size: usize =
                    min(read_size - data_offset, range_remainder_size as usize);

                if range_read_size == 0 {
                    break;
                }
                if !self.block_cache.contains(&block_number) {
                    let block_data_offset: u64 = segment_range.data_offset + block_offset;

                    let mut encrypted_data: Vec<u8> = vec![0; self.block_size as usize];

                    match segment_file.read_exact_at_position(
                        &mut encrypted_data,
                        SeekFrom::Start(block_data_offset),
                    ) {
                        Ok(read_count) => self.block_size as usize,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read encrypted block: {} from segment file: {} at offset: {} (0x{:08x})",
                                    block_number,
                                    segment_file.segment_number,
                                    block_data_offset,
                                    block_data_offset,
                                )
                            );
                            return Err(error);
                        }
                    };
                    let block_number_data: [u8; 4] = (block_number as u32).to_be_bytes();

                    let mut initialization_vector: Vec<u8> = match self
                        .hmac_context
                        .calculate_hmac(&block_number_data)
                    {
                        Ok(data) => data,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to HMAC initialization vector of encrypted block: {}",
                                    block_number
                                )
                            );
                            return Err(error);
                        }
                    };
                    let mut block_data: Vec<u8> = vec![0; self.block_size as usize];

                    match self.encryption_context.decrypt_cbc(
                        &mut initialization_vector,
                        &encrypted_data,
                        &mut block_data,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to decrypt encrypted block: {}", block_number)
                            );
                            return Err(error);
                        }
                    }
                    keramics_core::debug_trace_data!(
                        "BlockData",
                        block_data_offset,
                        &block_data,
                        self.block_size,
                    );
                    self.block_cache.insert(block_number, block_data);
                }
                let range_data: &[u8] = match self.block_cache.get(&block_number) {
                    Some(data) => data,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve encrypted block: {} data from cache",
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
                if data_offset >= read_size {
                    break;
                }
                segment_offset += range_read_size as u64;
                block_offset += self.block_size as u64;
                block_number += 1;
            }
            segment_index += 1;
        }
        Ok(data_offset)
    }

    /// Unlocks a locked (encrypted) segment stream.
    pub fn unlock(&mut self, credentials: &[UdifCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let segment_number: u32 = 1;

        if !self.segment_file_cache.contains(&segment_number) {
            let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

            let segment_file: UdifFile = match self.open_segment_file(&segment_file_name) {
                Ok(udif_file) => udif_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
            self.segment_file_cache.insert(segment_number, segment_file);
        }
        let first_segment_file: &mut UdifFile =
            match self.segment_file_cache.get_mut(&segment_number) {
                Some(segment_file) => segment_file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_number
                    )));
                }
            };
        for key_protector in self.key_protectors.iter() {
            // Note that 65536 is an arbitrary chosen limit.
            if key_protector.size > 65536 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported key protector size: {} value out of bounds",
                    key_protector.size
                )));
            }
            let mut data: Vec<u8> = vec![0; key_protector.size as usize];

            match first_segment_file
                .read_exact_at_position(&mut data, SeekFrom::Start(key_protector.offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read from segment file: {} at offset: {} (0x{:08x})",
                            segment_number, key_protector.offset, key_protector.offset
                        )
                    );
                    return Err(error);
                }
            }
            match key_protector.protector_type {
                UdifKeyProtectorType::PassphraseWrappedKey => {
                    let mut block_key: Vec<u8> = Vec::new();
                    let mut hmac_key: Vec<u8> = Vec::new();

                    if first_segment_file.format_version == 1 {
                        keramics_core::debug_trace_data_and_structure!(
                            "UdifEncryptedFileFooter",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            UdifEncryptedFileFooter::debug_read_data(&data)
                        );
                        let mut file_footer: UdifEncryptedFileFooter =
                            UdifEncryptedFileFooter::new();

                        match file_footer.read_data(&data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read file footer"
                                );
                                return Err(error);
                            }
                        }
                        for credential in credentials.iter() {
                            if credential.credential_type == UdifCredentialType::Passphrase {
                                match file_footer.unlock(credential) {
                                    Ok(result) => {
                                        if result {
                                            let block_key_size: usize =
                                                first_segment_file.encryption_type.key_size;
                                            block_key = file_footer.block_key_data
                                                [0..block_key_size]
                                                .to_vec();

                                            let hmac_key_size: usize =
                                                first_segment_file.hmac_key_size;
                                            hmac_key = file_footer.hmac_key_data[0..hmac_key_size]
                                                .to_vec();

                                            self.is_locked = false;
                                            break;
                                        }
                                    }
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to unlock file footer"
                                        );
                                        return Err(error);
                                    }
                                }
                            }
                        }
                    } else if first_segment_file.format_version == 2 {
                        keramics_core::debug_trace_data_and_structure!(
                            "UdifPasspraseWrappedKey",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            UdifPassphraseWrappedKey::debug_read_data(&data)
                        );
                        let mut wrapped_key: UdifPassphraseWrappedKey =
                            UdifPassphraseWrappedKey::new();

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
                            if credential.credential_type == UdifCredentialType::Passphrase {
                                match wrapped_key.unlock(credential) {
                                    Ok(result) => {
                                        if result {
                                            let block_key_size: usize =
                                                first_segment_file.encryption_type.key_size;
                                            block_key =
                                                wrapped_key.key_data[0..block_key_size].to_vec();

                                            let hmac_key_size: usize =
                                                first_segment_file.hmac_key_size;
                                            let data_end_offset: usize =
                                                block_key_size + hmac_key_size;
                                            hmac_key = wrapped_key.key_data
                                                [block_key_size..data_end_offset]
                                                .to_vec();

                                            self.is_locked = false;
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
                    }
                    if !self.is_locked {
                        keramics_core::debug_trace_data!(
                            "UdifBlockKey",
                            0,
                            &block_key,
                            block_key.len(),
                        );
                        keramics_core::debug_trace_data!(
                            "UdifBlockHmacKey",
                            0,
                            &hmac_key,
                            hmac_key.len(),
                        );
                        self.encryption_context = match UdifEncryption::get_encryption_context(
                            &first_segment_file.encryption_type,
                            &block_key,
                        ) {
                            Ok(Some(context)) => context,
                            Ok(None) => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unsupported encryption type: {}",
                                    first_segment_file.encryption_type
                                )));
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to retrieve encryption type: {}",
                                        first_segment_file.encryption_type
                                    )
                                );
                                return Err(error);
                            }
                        };
                        self.hmac_context = match UdifEncryption::get_hmac_context(
                            first_segment_file.hmac_method,
                            &hmac_key,
                        ) {
                            Ok(Some(context)) => context,
                            Ok(None) => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unsupported HMAC method: {}",
                                    first_segment_file.hmac_method
                                )));
                            }
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to retrieve HMAC context for method: {}",
                                        first_segment_file.hmac_method
                                    )
                                );
                                return Err(error);
                            }
                        };
                    }
                }
                UdifKeyProtectorType::PublicKeyWrappedKey => {
                    if first_segment_file.format_version == 2 {
                        keramics_core::debug_trace_data_and_structure!(
                            "UdifPublicKeyrappedKey",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                            UdifPublicKeyWrappedKey::debug_read_data(&data)
                        );
                        let mut wrapped_key: UdifPublicKeyWrappedKey =
                            UdifPublicKeyWrappedKey::new();

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
                    if first_segment_file.format_version == 2 {
                        keramics_core::debug_trace_data!(
                            "UdifEncryptedKeyProtector",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                        );
                    }
                }
            }
        }
        Ok(!self.is_locked)
    }
}

impl DataStream for UdifSegmentStream {
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
        let read_count: usize = if self.encryption_type.mode == 0 {
            match self.read_data_from_segments(&mut buf[..read_size]) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read data from segments"
                    );
                    return Err(error);
                }
            }
        } else {
            match self.read_data_from_encrypted_blocks(&mut buf[..read_size]) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read data from encrypted blocks"
                    );
                    return Err(error);
                }
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

    use crate::os_file_resolver::open_os_file_resolver;
    use crate::tests::get_test_data_path;

    fn get_segment_stream(file_name_string: &str) -> Result<UdifSegmentStream, ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = UdifSegmentStream::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from(file_name_string);
        segment_stream.open(&file_resolver, &file_name)?;

        Ok(segment_stream)
    }

    #[test]
    fn test_get_segment_file_name() {
        let base_name: String = String::from("image");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 1);
        assert_eq!(name, "image.dmg");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 9);
        assert_eq!(name, "image.009.dmgpart");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 1234);
        assert_eq!(name, "image.1234.dmgpart");
    }

    // TODO: add tests for open_segment_file

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = UdifSegmentStream::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("hfsplus_segments.dmg");
        segment_stream.open(&file_resolver, &file_name)?;

        assert_eq!(segment_stream.size, 1955840);

        Ok(())
    }

    // TODO: add tests for read_segment_files
    // TODO: add tests for read_data_from_segments

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        segment_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = segment_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let size: u64 = segment_stream.get_size()?;
        assert_eq!(size, 1955840);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, segment_stream.size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset = segment_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = segment_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let result: Result<u64, ErrorTrace> = segment_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, segment_stream.size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;
        segment_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = segment_stream.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x53, 0x46, 0x48, 0x00, 0x00, 0xaa, 0x11, 0xaa, 0x11, 0x00, 0x30, 0x65, 0x43,
            0xec, 0xac, 0xb2, 0xb3, 0x80, 0x60, 0xbe, 0x78, 0xa9, 0x4d, 0x8b, 0x19, 0x2f, 0xcc,
            0x48, 0x39, 0xca, 0x2d, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd7, 0x0e,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x64, 0x00, 0x69, 0x00, 0x73, 0x00, 0x6b, 0x00, 0x20, 0x00, 0x69, 0x00, 0x6d, 0x00,
            0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;
        segment_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = segment_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_zlib_aes128.dmg")?;

        assert_eq!(segment_stream.is_locked, true);

        let mut credentials: Vec<UdifCredential> = Vec::new();
        credentials.push(UdifCredential::new(
            UdifCredentialType::Passphrase,
            b"KeRaMiCs",
        ));
        segment_stream.unlock(&credentials)?;

        assert_eq!(segment_stream.is_locked, false);

        Ok(())
    }
}
