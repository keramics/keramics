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

use std::cmp::{Ordering, min};
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionType};
use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::block_reader::UdifBlockReader;
use super::block_stream::UdifBlockStream;
use super::block_table_reader::UdifBlockTableReader;
use super::enums::UdifCompressionMethod;
use super::file::UdifFile;
use super::segment_range::UdifSegmentRange;

/// Universal Disk Image Format (UDIF) file.
pub struct UdifImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Segment file set identifier.
    segment_set_identifier: Uuid,

    /// Number of segments.
    number_of_segments: u32,

    /// Segment size.
    segments_size: u64,

    /// Name.   
    name: String,

    /// Segment ranges.
    segment_ranges: Vec<UdifSegmentRange>,

    /// Segments data stream.
    segments_data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Number of sectors.
    number_of_sectors: u64,

    /// Block ranges.
    block_ranges: Vec<UdifBlockRange>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Compression method.
    compression_method: UdifCompressionMethod,

    /// Encryption type.
    encryption_type: Option<CdsaEncrEncryptionType>,

    /// Credentials.
    credentials: Vec<CdsaEncrCredential>,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    media_size: u64,
}

impl UdifImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            segment_set_identifier: Uuid::new(),
            number_of_segments: 0,
            segments_size: 0,
            name: String::new(),
            segment_ranges: Vec::new(),
            segments_data_stream: None,
            bytes_per_sector: 0,
            number_of_sectors: 0,
            block_ranges: Vec::new(),
            block_cache: LruCache::new(64),
            compression_method: UdifCompressionMethod::None,
            encryption_type: None,
            credentials: Vec::new(),
            is_locked: false,
            current_offset: 0,
            media_size: 0,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the compression method.
    pub fn get_compression_method(&self) -> &UdifCompressionMethod {
        &self.compression_method
    }

    /// Retrieves the encryption type.
    pub fn get_encryption_type(&self) -> Option<&CdsaEncrEncryptionType> {
        self.encryption_type.as_ref()
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Retrieves the number of segments.
    pub fn get_number_of_segments(&self) -> u32 {
        self.number_of_segments
    }

    /// Determines the segment file name.
    fn get_segment_file_name(name: &String, segment_number: u32) -> String {
        if segment_number == 1 {
            format!("{}.dmg", name)
        } else {
            format!("{}.{:03}.dmgpart", name, segment_number)
        }
    }

    /// Retrieves the segment set identifier.
    pub fn get_segment_set_identifier(&self) -> &Uuid {
        &self.segment_set_identifier
    }

    /// Determines if the (encrypted) image is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Opens a storage media image.
    pub fn open(
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
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to open encrypted container of segment file: {}",
                            file_name
                        ),
                    );
                    return Err(error);
                }
            }
            self.encryption_type = Some(cdsaencr_container.get_encryption_type().clone());
            self.is_locked = true;
        }
        self.bytes_per_sector = 512;
        self.file_resolver = file_resolver.clone();

        if !self.is_locked {
            match self.read_segment_files(&data_stream, file_name) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                    return Err(error);
                }
            }
            let block_table_reader: UdifBlockTableReader =
                match self.read_metadata(self.bytes_per_sector) {
                    Ok(block_table_reader) => block_table_reader,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                        return Err(error);
                    }
                };
            self.media_size = block_table_reader.get_media_size();
            self.compression_method = block_table_reader.get_compression_method();
            self.block_ranges = block_table_reader.block_ranges;

            if self.media_size > (self.number_of_sectors * (self.bytes_per_sector as u64)) {
                return Err(keramics_core::error_trace_new!(
                    "Number of sectors value out of bounds",
                ));
            }
            self.segments_data_stream = Some(Arc::new(RwLock::new(UdifBlockStream::new(
                UdifBlockReader::new(
                    &self.file_resolver,
                    self.name.as_str(),
                    &self.segment_ranges,
                    &self.credentials,
                    self.segments_size,
                ),
            ))));
        }
        Ok(())
    }

    /// Opens a segment file.
    fn open_segment_file(&self, segment_number: u32) -> Result<UdifFile, ErrorTrace> {
        let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

        let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

        let mut data_stream: DataStreamReference =
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
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to open encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            match cdsaencr_container.unlock(&self.credentials) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to unlock encrypted container of segment file: {}",
                        segment_file_name,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Failed to unlock encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            data_stream = match cdsaencr_container.get_data_stream() {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing encrypted container data stream",
                    ));
                }
            };
        }
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

    /// Decompressed a block.
    fn decompress_block(&self, compressed_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        match self.compression_method {
            UdifCompressionMethod::Adc => {
                let mut adc_context: AdcContext = AdcContext::new();

                match adc_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress ADC data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Bzip2 => {
                let mut bzip2_context: Bzip2Context = Bzip2Context::new();

                match bzip2_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress bzip2 data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Lzfse => {
                let mut lzfse_context: LzfseContext = LzfseContext::new();

                match lzfse_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress LZFSE data"
                        );
                        return Err(error);
                    }
                }
            }
            UdifCompressionMethod::Lzma => {
                // TODO: add support for UdifCompressionMethod::Lzma,
                todo!();
            }
            UdifCompressionMethod::Zlib => {
                _ = crate::zlib_decompress!(
                    &compressed_data,
                    data,
                    "Unable to decompress zlib data"
                );
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported compression method"
                ));
            }
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = self.current_offset;

        let mut range_index: usize = match self.block_ranges.binary_search_by(|block_range| {
            let range_end_offset: u64 = block_range.media_offset + block_range.size;

            if current_offset >= range_end_offset {
                Ordering::Less
            } else if current_offset < block_range.media_offset {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(range_index) => range_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing block range for offset: {} (0x{:08x})",
                    current_offset, current_offset
                )));
            }
        };
        while data_offset < read_size {
            if current_offset >= self.media_size {
                break;
            }
            let block_range: &UdifBlockRange = match self.block_ranges.get(range_index) {
                Some(block_range) => block_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                        range_index, current_offset, current_offset,
                    )));
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.media_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            match block_range.range_type {
                UdifBlockRangeType::Compressed => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    if !self.block_cache.contains(&block_range.data_offset) {
                        let mut compressed_data: Vec<u8> =
                            vec![0; block_range.compressed_data_size as usize];

                        match self.segments_data_stream.as_ref() {
                            Some(data_stream) => {
                                keramics_core::data_stream_read_exact_at_position!(
                                    data_stream,
                                    &mut compressed_data,
                                    SeekFrom::Start(block_range.data_offset),
                                );
                            }
                            None => {
                                return Err(keramics_core::error_trace_new!("Missing data stream"));
                            }
                        }
                        let mut data: Vec<u8> = vec![0; block_range.size as usize];

                        match self.decompress_block(&compressed_data, &mut data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to decompress block at offset: {} (0x{:08x})",
                                        block_range.data_offset, block_range.data_offset
                                    )
                                );
                                return Err(error);
                            }
                        }
                        self.block_cache.insert(block_range.data_offset, data);
                    }
                    let range_data: &[u8] = match self.block_cache.get(&block_range.data_offset) {
                        Some(data) => data,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unable to retrieve data from cache"
                            )));
                        }
                    };
                    if range_data.len() != (block_range.size as usize) {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve block range data",
                        ));
                    }
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);
                }
                UdifBlockRangeType::InFile => {
                    let range_data_offset: u64 = block_range.data_offset + range_relative_offset;

                    match self.segments_data_stream.as_ref() {
                        Some(data_stream) => {
                            keramics_core::data_stream_read_exact_at_position!(
                                data_stream,
                                &mut data[data_offset..data_end_offset],
                                SeekFrom::Start(range_data_offset),
                            );
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing data stream"));
                        }
                    }
                }
                UdifBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
            range_index += 1;
        }
        Ok(data_offset)
    }

    /// Reads the metadata in the XML plist or resource fork in the first segment file.
    fn read_metadata(&mut self, bytes_per_sector: u16) -> Result<UdifBlockTableReader, ErrorTrace> {
        let segment_number: u32 = 1;

        let mut segment_file: UdifFile = match self.open_segment_file(segment_number) {
            Ok(udif_file) => udif_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open segment file: {}", segment_number)
                );
                return Err(error);
            }
        };
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(bytes_per_sector, self.segments_size);

        if segment_file.plist_size == 0 && segment_file.resource_fork_size == 0 {
            block_table_reader.media_offset = self.segments_size;
        } else if segment_file.plist_size == 0 {
            match segment_file.read_resource_fork(&mut block_table_reader) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read resource fork from segment file: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            }
        } else {
            match segment_file.read_xml_plist(&mut block_table_reader) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read XML plist from segment file: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(block_table_reader)
    }

    /// Reads the successive segment files.
    pub fn read_segment_files(
        &mut self,
        data_stream: &DataStreamReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
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
        self.segments_size = 0;

        let segment_number: u32 = segment_file.segment_number;

        if self.number_of_segments == 0 && segment_number == 0 {
            self.number_of_segments = 1
        } else if self.number_of_segments != 0 && segment_number != 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment number value out of bounds"
            ));
        }
        if segment_file.segment_offset != self.segments_size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment offset value out of bounds"
            ));
        }
        self.segments_size += segment_file.data_fork_size;

        let segment_range: UdifSegmentRange =
            UdifSegmentRange::new(1, segment_file.segment_offset, segment_file.data_fork_size);
        self.segment_ranges.push(segment_range);

        self.segment_set_identifier = segment_file.segment_set_identifier.clone();

        for segment_number in 2..=self.number_of_segments {
            let segment_file: UdifFile = match self.open_segment_file(segment_number) {
                Ok(udif_file) => udif_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_number)
                    );
                    return Err(error);
                }
            };
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
            if segment_file.segment_offset != self.segments_size {
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
            self.segments_size += segment_file.data_fork_size;

            let segment_range: UdifSegmentRange = UdifSegmentRange::new(
                segment_number,
                segment_file.segment_offset,
                segment_file.data_fork_size,
            );
            self.segment_ranges.push(segment_range);
        }
        Ok(())
    }

    /// Unlocks a locked (encrypted) volume.
    pub fn unlock(&mut self, credentials: &[CdsaEncrCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let segment_number: u32 = 1;

        let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

        let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

        let mut data_stream: DataStreamReference =
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
        let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

        match cdsaencr_container.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to open encrypted container of segment file: {}",
                        segment_file_name
                    ),
                );
                return Err(error);
            }
        }
        let result: bool = match cdsaencr_container.unlock(credentials) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Failed to unlock encrypted container of segment file: {}",
                        segment_file_name
                    ),
                );
                return Err(error);
            }
        };
        if result {
            self.credentials = credentials.to_vec();

            data_stream = match cdsaencr_container.get_data_stream() {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing encrypted container data stream",
                    ));
                }
            };
            let path_component: PathComponent = PathComponent::from(&segment_file_name);

            match self.read_segment_files(&data_stream, &path_component) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                    return Err(error);
                }
            }
            let block_table_reader: UdifBlockTableReader =
                match self.read_metadata(self.bytes_per_sector) {
                    Ok(block_table_reader) => block_table_reader,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                        return Err(error);
                    }
                };
            self.media_size = block_table_reader.get_media_size();
            self.compression_method = block_table_reader.get_compression_method();
            self.block_ranges = block_table_reader.block_ranges;

            if self.media_size > (self.number_of_sectors * (self.bytes_per_sector as u64)) {
                return Err(keramics_core::error_trace_new!(
                    "Number of sectors value out of bounds",
                ));
            }
            self.is_locked = false;
            self.segments_data_stream = Some(Arc::new(RwLock::new(UdifBlockStream::new(
                UdifBlockReader::new(
                    &self.file_resolver,
                    self.name.as_str(),
                    &self.segment_ranges,
                    &self.credentials,
                    self.segments_size,
                ),
            ))));
        }
        Ok(!self.is_locked)
    }
}

impl DataStream for UdifImage {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Image is locked"));
        }
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Image is locked"));
        }
        if self.current_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = if self.block_ranges.is_empty() {
            match self.segments_data_stream.as_ref() {
                Some(data_stream) => {
                    keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut buf[..read_size],
                        SeekFrom::Start(self.current_offset)
                    )
                }
                None => {
                    return Err(keramics_core::error_trace_new!("Missing data stream"));
                }
            }
        } else {
            match self.read_data_from_blocks(&mut buf[..read_size]) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                    return Err(error);
                }
            }
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        if self.is_locked {
            return Err(keramics_core::error_trace_new!("Image is locked"));
        }
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
            SeekFrom::End(relative_offset) => {
                match self.media_size.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
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

    fn get_image(file_name: &str) -> Result<UdifImage, ErrorTrace> {
        let mut image: UdifImage = UdifImage::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from(file_name);
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let bytes_per_sector: u16 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_compression_method() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let compression_method: &UdifCompressionMethod = image.get_compression_method();
        assert_eq!(compression_method, &UdifCompressionMethod::Zlib);

        Ok(())
    }

    #[test]
    fn test_get_encryption_type() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_aes256.dmg")?;

        let encryption_type: &CdsaEncrEncryptionType = image.get_encryption_type().unwrap();
        assert_eq!(encryption_type.method, 0x80000001);
        assert_eq!(encryption_type.mode, 5);
        assert_eq!(encryption_type.key_size, 32);

        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let media_size: u64 = image.get_media_size();
        assert_eq!(media_size, 1964032);

        Ok(())
    }

    #[test]
    fn test_get_number_of_segments() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let number_of_segments: u32 = image.get_number_of_segments();
        assert_eq!(number_of_segments, 2);

        Ok(())
    }

    #[test]
    fn test_get_segment_file_name() {
        let base_name: String = String::from("image");

        let name: String = UdifImage::get_segment_file_name(&base_name, 1);
        assert_eq!(name, "image.dmg");

        let name: String = UdifImage::get_segment_file_name(&base_name, 9);
        assert_eq!(name, "image.009.dmgpart");

        let name: String = UdifImage::get_segment_file_name(&base_name, 1234);
        assert_eq!(name, "image.1234.dmgpart");
    }

    #[test]
    fn test_get_segment_set_identifier() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let segment_set_identifier: &Uuid = image.get_segment_set_identifier();
        assert_eq!(
            segment_set_identifier.to_string(),
            "cd0a02d0-648c-49ec-b7e8-1c5d1e6d6281"
        );
        Ok(())
    }

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image("hfsplus_aes256.dmg")?;

        let is_locked: bool = image.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = UdifImage::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("hfsplus_zlib_segments.dmg");
        image.open(&file_resolver, &file_name)?;

        assert_eq!(image.media_size, 1964032);

        Ok(())
    }

    // TODO: add tests for open_segment_file
    // TODO: add tests for read_data_from_blocks
    // TODO: add tests for read_metadata
    // TODO: add tests for read_segment_files

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_aes128.dmg")?;

        assert_eq!(image.is_locked, true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        image.unlock(&credentials)?;

        assert_eq!(image.is_locked, false);

        Ok(())
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        image.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let size: u64 = image.get_size()?;
        assert_eq!(size, 1964032);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let offset: u64 = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let offset: u64 = image.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let offset = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let result: Result<u64, ErrorTrace> = image.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;

        let offset: u64 = image.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;
        image.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
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
    fn test_seek_and_read_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image("hfsplus_zlib_segments.dmg")?;
        image.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
