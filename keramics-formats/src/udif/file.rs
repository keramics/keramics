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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{Uuid, bytes_to_u32_be};

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{
    CdsaEncrContainerFooter, CdsaEncrContainerHeader, CdsaEncrCredential, CdsaEncrEncryption,
    CdsaEncrEncryptionContext, CdsaEncrEncryptionType, CdsaEncrHmacContext, CdsaEncrKeyProtector,
    CdsaEncrKeyProtectorType, CdsaEncrPassphraseWrappedKey, CdsaEncrPublicKeyWrappedKey,
};
use crate::lru_cache::LruCache;
use crate::plist::{PlistObject, XmlPlist};

use super::block_table::UdifBlockTable;
use super::block_table_reader::UdifBlockTableReader;
use super::file_footer::UdifFileFooter;
use super::resource_fork_header::UdifResourceForkHeader;
use super::resource_map::UdifResourceMap;
use super::resource_map_item::UdifResourceMapItem;

/// Universal Disk Image Format (UDIF) file.
pub struct UdifFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    pub(super) format_version: u32,

    /// Segment offset.
    pub(super) segment_offset: u64,

    /// Segment number.
    pub(super) segment_number: u32,

    /// Number of segments.
    pub(super) number_of_segments: u32,

    /// Segment set identifier.
    pub(super) segment_set_identifier: Uuid,

    /// Number of sectors.
    pub(super) number_of_sectors: u64,

    /// Data fork offset.
    pub(super) data_fork_offset: u64,

    /// Data fork size.
    pub(super) data_fork_size: u64,

    /// Resource fork offset.
    pub(super) resource_fork_offset: u64,

    /// Resource fork size.
    pub(super) resource_fork_size: u64,

    /// Plist offset.
    pub(super) plist_offset: u64,

    /// Plist size.
    pub(super) plist_size: u64,

    /// Block size.
    pub(super) block_size: u32,

    /// Value to indicate the (encrypted) file is locked.
    pub(super) is_locked: bool,

    /// Encryption type.
    pub(super) encryption_type: CdsaEncrEncryptionType,

    /// Initialization vector size.
    pub(super) initialization_vector_size: usize,

    /// HMAC method.
    pub(super) hmac_method: u32,

    /// HMAC method.
    pub(super) hmac_key_size: usize,

    /// Key key_protectors.
    pub(super) key_protectors: Vec<CdsaEncrKeyProtector>,

    /// Credentials.
    pub(super) credentials: Vec<CdsaEncrCredential>,

    /// Encryption context.
    encryption_context: CdsaEncrEncryptionContext,

    /// HMAC context.
    hmac_context: CdsaEncrHmacContext,

    /// Decrypted block cache.
    block_cache: LruCache<u64, Vec<u8>>,
}

impl UdifFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            segment_offset: 0,
            segment_number: 0,
            number_of_segments: 0,
            segment_set_identifier: Uuid::new(),
            number_of_sectors: 0,
            data_fork_offset: 0,
            data_fork_size: 0,
            resource_fork_offset: 0,
            resource_fork_size: 0,
            plist_offset: 0,
            plist_size: 0,
            block_size: 0,
            is_locked: false,
            encryption_type: CdsaEncrEncryptionType::new(),
            initialization_vector_size: 0,
            hmac_method: 0,
            hmac_key_size: 0,
            key_protectors: Vec::new(),
            credentials: Vec::new(),
            encryption_context: CdsaEncrEncryptionContext::None,
            hmac_context: CdsaEncrHmacContext::None,
            block_cache: LruCache::new(64),
        }
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u32 {
        self.format_version
    }

    /// Retrieves the segment set identifier.
    pub fn get_segment_set_identifier(&self) -> &Uuid {
        &self.segment_set_identifier
    }

    /// Retrieves the segment number.
    pub fn get_segment_number(&self) -> u32 {
        self.segment_number
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut signature,
            SeekFrom::Start(0)
        );
        if &signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE {
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
            self.data_fork_offset = encrypted_container_header.data_fork_offset;
            self.data_fork_size = encrypted_container_header.data_fork_size;
            self.block_size = encrypted_container_header.block_size;
            self.is_locked = true;
            self.encryption_type = encrypted_container_header.encryption_type;
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
            let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut signature,
                SeekFrom::End(-8)
            );
            if &signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE {
                let mut encrypted_container_footer: CdsaEncrContainerFooter =
                    CdsaEncrContainerFooter::new();

                match encrypted_container_footer.read_at_position(data_stream, SeekFrom::End(-1276))
                {
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
                self.data_fork_offset = encrypted_container_footer.data_fork_offset as u64;
                self.data_fork_size = encrypted_container_footer.data_fork_size as u64;
                self.block_size = encrypted_container_footer.block_size;
                self.is_locked = true;
                self.encryption_type = encrypted_container_footer.encryption_type;
                self.initialization_vector_size =
                    encrypted_container_footer.initialization_vector_size as usize;
                self.hmac_method = encrypted_container_footer.hmac_method;
                self.hmac_key_size = (encrypted_container_footer.hmac_key_size / 8) as usize;

                let key_protector: CdsaEncrKeyProtector = CdsaEncrKeyProtector::new(
                    CdsaEncrKeyProtectorType::PassphraseWrappedKey,
                    (offset + 8) - 1276,
                    1276,
                );
                self.key_protectors.push(key_protector);
            } else {
                let mut file_footer: UdifFileFooter = UdifFileFooter::new();

                match file_footer.read_at_position(data_stream, SeekFrom::End(-512)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read file footer");
                        return Err(error);
                    }
                }
                self.format_version = file_footer.format_version;
                self.segment_offset = file_footer.segment_offset;
                self.segment_number = file_footer.segment_number;
                self.number_of_segments = file_footer.number_of_segments;
                self.segment_set_identifier = file_footer.segment_set_identifier;
                self.number_of_sectors = file_footer.number_of_sectors;
                self.data_fork_offset = file_footer.data_fork_offset;
                self.data_fork_size = file_footer.data_fork_size;
                self.resource_fork_offset = file_footer.resource_fork_offset;
                self.resource_fork_size = file_footer.resource_fork_size;
                self.plist_offset = file_footer.plist_offset;
                self.plist_size = file_footer.plist_size;
                self.block_size = 512;
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads an exact amount of data at a specific position.
    pub(super) fn read_exact_at_position(
        &mut self,
        data: &mut [u8],
        segment_offset: u64,
        read_metadata: bool,
    ) -> Result<usize, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("File is locked"));
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut file_offset: u64 = segment_offset;

        if !read_metadata {
            if segment_offset < self.segment_offset
                || segment_offset >= self.segment_offset + self.data_fork_size
            {
                return Err(keramics_core::error_trace_new!(
                    "Invalid segment offset value out of bounds"
                ));
            }
            file_offset -= self.segment_offset;
        }
        if self.encryption_type.mode == 0 {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                data,
                SeekFrom::Start(file_offset)
            );
            Ok(data.len())
        } else {
            let read_size: usize = data.len();
            let mut data_offset: usize = 0;

            let mut block_number: u64 = file_offset / (self.block_size as u64);
            let mut block_offset: u64 = block_number * (self.block_size as u64);

            if block_number > u32::MAX as u64 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid block number value out of bounds"
                ));
            }
            while data_offset < read_size {
                if !read_metadata && file_offset >= self.data_fork_size {
                    break;
                }
                let range_relative_offset: u64 = file_offset - block_offset;
                let range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

                let range_read_size: usize =
                    min(read_size - data_offset, range_remainder_size as usize);

                if range_read_size == 0 {
                    break;
                }
                if !self.block_cache.contains(&block_number) {
                    let block_data_offset: u64 = self.data_fork_offset + block_offset;

                    let mut encrypted_data: Vec<u8> = vec![0; self.block_size as usize];

                    keramics_core::data_stream_read_exact_at_position!(
                        data_stream,
                        &mut encrypted_data,
                        SeekFrom::Start(block_data_offset)
                    );
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
                file_offset += range_read_size as u64;
                block_offset += self.block_size as u64;
                block_number += 1;
            }
            Ok(data_offset)
        }
    }

    /// Reads metadata from the resource fork.
    pub(super) fn read_resource_fork(
        &mut self,
        block_table_reader: &mut UdifBlockTableReader,
    ) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut resource_fork_header: UdifResourceForkHeader = UdifResourceForkHeader::new();

        match resource_fork_header
            .read_at_position(data_stream, SeekFrom::Start(self.resource_fork_offset))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read resource fork header");
                return Err(error);
            }
        }
        let offset: u64 =
            self.resource_fork_offset + (resource_fork_header.resource_map_offset as u64);

        let mut resource_map: UdifResourceMap = UdifResourceMap::new();

        match resource_map.read_at_position(
            data_stream,
            resource_fork_header.resource_map_size,
            SeekFrom::Start(offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read resource map");
                return Err(error);
            }
        }
        let mut lookup_item: Option<&UdifResourceMapItem> = None;

        for resource_map_item in resource_map.items.iter() {
            if resource_map_item.name == "blkx" {
                lookup_item = Some(resource_map_item);
                break;
            }
        }
        let blkx_item: &UdifResourceMapItem = match lookup_item {
            Some(resource_map_item) => resource_map_item,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve blkx item from resource map"
                ));
            }
        };
        let mut data: [u8; 4] = [0; 4];

        for blkx_value in blkx_item.values.iter() {
            let offset: u64 = self.resource_fork_offset
                + (resource_fork_header.resource_data_offset as u64)
                + (blkx_value.data_offset as u64);

            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(offset)
            );
            let block_table_data_size: u32 = bytes_to_u32_be!(data, 0);

            let mut block_table = UdifBlockTable::new();

            match block_table.read_at_position(
                data_stream,
                block_table_data_size,
                SeekFrom::Start(offset + 4),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read block table");
                    return Err(error);
                }
            }
            match block_table_reader.process_block_table(&block_table) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to process block table");
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads metadata from the XML plist.
    pub(super) fn read_xml_plist(
        &mut self,
        block_table_reader: &mut UdifBlockTableReader,
    ) -> Result<(), ErrorTrace> {
        // Note that 16777216 is an arbitrary chosen limit.
        if self.plist_size > 16777216 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let mut data: Vec<u8> = vec![0; self.plist_size as usize];

        match self.read_exact_at_position(&mut data, self.plist_offset, true) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read plist data");
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!(
            "UdifFileXmlPlist",
            self.plist_offset,
            &data,
            self.plist_size
        );
        let string: String = match String::from_utf8(data) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert plist data into UTF-8 string",
                    error
                ));
            }
        };
        let mut xml_plist: XmlPlist = XmlPlist::new();

        match xml_plist.parse(string.as_str()) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse plist",
                    error
                ));
            }
        }
        let resource_fork_object: &PlistObject =
            match xml_plist.root_object.get_object_by_key("resource-fork") {
                Some(string) => string,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unable to retrieve resource-fork value from plist"
                    ));
                }
            };
        let blkx_item: &[PlistObject] = match resource_fork_object.get_slice_by_key("blkx") {
            Some(string) => string,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve blkx item from plist"
                ));
            }
        };
        for (value_index, blkx_value) in blkx_item.iter().enumerate() {
            let data: &[u8] = match blkx_value.get_bytes_by_key("Data") {
                Some(data) => data,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve Data value from blkx value: {}",
                        value_index
                    )));
                }
            };
            // TODO: determine data offset relative to start of plist
            keramics_core::debug_trace_data!("UdifBlockTable", 0, &data, data.len());

            let mut block_table: UdifBlockTable = UdifBlockTable::new();

            match block_table.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read block table");
                    return Err(error);
                }
            }
            match block_table_reader.process_block_table(&block_table) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to process block table");
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Unlocks a locked (encrypted) file.
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
                        let mut file_footer: CdsaEncrContainerFooter =
                            CdsaEncrContainerFooter::new();

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
                            match file_footer.unlock(credential) {
                                Ok(result) => {
                                    if result {
                                        let block_key_size: usize = self.encryption_type.key_size;
                                        block_key =
                                            file_footer.block_key_data[0..block_key_size].to_vec();

                                        let hmac_key_size: usize = self.hmac_key_size;
                                        hmac_key =
                                            file_footer.hmac_key_data[0..hmac_key_size].to_vec();

                                        keys_unlocked = true;

                                        self.credentials.push(credential.clone());

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

                                        self.credentials.push(credential.clone());

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
                            "UdifEncryptedKeyProtector",
                            key_protector.offset,
                            &data,
                            key_protector.size,
                        );
                    }
                }
            }
        }
        if keys_unlocked {
            keramics_core::debug_trace_data!("UdifBlockKey", 0, &block_key, block_key.len());
            keramics_core::debug_trace_data!("UdifBlockHmacKey", 0, &hmac_key, hmac_key.len());

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;
    use crate::udif::segment_range::UdifSegmentRange;

    fn get_file(path_string: &str) -> Result<UdifFile, ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file: UdifFile = get_file("udif/hfsplus_zlib.dmg")?;

        let format_version: u32 = file.get_format_version();
        assert_eq!(format_version, 4);

        Ok(())
    }

    #[test]
    fn test_get_segment_set_identifier() -> Result<(), ErrorTrace> {
        let file: UdifFile = get_file("udif/hfsplus_zlib.dmg")?;

        let segment_set_identifier: &Uuid = file.get_segment_set_identifier();
        assert_eq!(
            segment_set_identifier.to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
        Ok(())
    }

    #[test]
    fn test_get_segment_number() -> Result<(), ErrorTrace> {
        let file: UdifFile = get_file("udif/hfsplus_zlib.dmg")?;

        let segment_number: u32 = file.get_segment_number();
        assert_eq!(segment_number, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.format_version, 4);

        Ok(())
    }

    #[test]
    fn test_read_resource_fork() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_rsrc.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let segment_range: UdifSegmentRange = UdifSegmentRange::new(0, 1, file.data_fork_size);
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(512, file.data_fork_size);
        file.read_resource_fork(&mut block_table_reader)?;

        assert!(block_table_reader.has_block_ranges());

        Ok(())
    }

    #[test]
    fn test_read_xml_plist() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let segment_range: UdifSegmentRange = UdifSegmentRange::new(0, 1, file.data_fork_size);
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(512, file.data_fork_size);
        file.read_xml_plist(&mut block_table_reader)?;

        assert!(block_table_reader.has_block_ranges());

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_aes256.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.is_locked, true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        file.unlock(&credentials)?;

        assert_eq!(file.is_locked, false);

        Ok(())
    }
}
