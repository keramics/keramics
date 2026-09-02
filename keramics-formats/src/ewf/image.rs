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

use std::collections::HashMap;
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

#[cfg(feature = "debug-trace")]
use keramics_core::DebugTrace;

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_range::{EwfBlockRange, EwfBlockRangeType};
use super::block_reader::EwfBlockReader;
use super::block_stream::EwfBlockStream;
use super::constants::*;
use super::digest::EwfDigest;
use super::enums::{EwfHeaderValueType, EwfMediaType, EwfNamingSchema};
use super::error2::EwfError2;
use super::file::EwfFile;
use super::hash::EwfHash;
use super::header::EwfHeader;
use super::header_value::EwfHeaderValue;
use super::header2::EwfHeader2;
use super::ltree_header::EwfLtreeHeader;
use super::section_header::EwfSectionHeader;
use super::segment_file::EwfSegmentFile;
use super::table::EwfTable;
use super::table_entry::EwfTableEntry;
use super::volume::{EwfE01Volume, EwfS01Volume};

/// Expert Witness Compression Format (EWF) image.
pub struct EwfImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Segment (file) set identifier.
    segment_set_identifier: Uuid,

    /// Name.
    name: String,

    /// Segment file naming schema.
    naming_schema: Option<EwfNamingSchema>,

    /// Segment file cache.
    segment_file_cache: LruCache<u16, EwfFile>,

    /// Number of chunks.
    number_of_chunks: u32,

    /// Sectors per chunk.
    sectors_per_chunk: u32,

    /// Bytes per sector.
    bytes_per_sector: u32,

    /// Number of sectors.
    number_of_sectors: u32,

    /// Chunk size.
    chunk_size: u32,

    /// Block ranges.
    block_ranges: Vec<EwfBlockRange>,

    /// Decompressed chunk cache.
    chunk_cache: LruCache<u64, Vec<u8>>,

    /// Error granularity.
    error_granularity: u32,

    /// Media type.
    media_type: EwfMediaType,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    media_size: u64,

    /// Values stored in header and header2 sections.
    header_values: HashMap<EwfHeaderValueType, EwfHeaderValue>,

    /// MD5 hash.
    md5_hash: [u8; 16],

    /// SHA1 hash.
    sha1_hash: [u8; 20],
}

impl EwfImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            segment_set_identifier: Uuid::new(),
            name: String::new(),
            naming_schema: None,
            segment_file_cache: LruCache::new(16),
            number_of_chunks: 0,
            sectors_per_chunk: 0,
            bytes_per_sector: 0,
            number_of_sectors: 0,
            chunk_size: 0,
            block_ranges: Vec::new(),
            chunk_cache: LruCache::new(64),
            error_granularity: 0,
            media_type: EwfMediaType::Unknown,
            current_offset: 0,
            media_size: 0,
            header_values: HashMap::new(),
            md5_hash: [0; 16],
            sha1_hash: [0; 20],
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        Arc::new(RwLock::new(EwfBlockStream::new(EwfBlockReader::new(
            &self.file_resolver,
            self.name.as_str(),
            self.naming_schema.as_ref(),
            self.chunk_size,
            &self.block_ranges,
            self.media_size,
        ))))
    }

    /// Retrieves the error granularity (in number of sectors).
    pub fn get_error_granularity(&self) -> u32 {
        self.error_granularity
    }

    /// Retrieves a header value.
    pub fn get_header_value(&self, value_type: &EwfHeaderValueType) -> Option<&EwfHeaderValue> {
        self.header_values.get(value_type)
    }

    /// Retrieves the MD51 hash.
    pub fn get_md5_hash(&self) -> &[u8] {
        &self.md5_hash
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Retrieves the mediat type.
    pub fn get_media_type(&self) -> &EwfMediaType {
        &self.media_type
    }

    /// Retrieves the number of sectors.
    pub fn get_number_of_sectors(&self) -> u32 {
        self.number_of_sectors
    }

    /// Retrieves the number of sectors per chunk.
    pub fn get_sectors_per_chunk(&self) -> u32 {
        self.sectors_per_chunk
    }

    /// Determines the segment file naming schema.
    fn get_segment_file_naming_schema(
        file_name: &PathComponent,
    ) -> Result<Option<EwfNamingSchema>, ErrorTrace> {
        let extension: String = match file_name.extension() {
            Ok(Some(extension)) => extension.to_string(),
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve extension of segment file: {}",
                        file_name,
                    )
                );
                return Err(error);
            }
        };
        let naming_schema: EwfNamingSchema = match extension.as_str() {
            "E01" => EwfNamingSchema::E01UpperCase,
            "L01" => EwfNamingSchema::L01UpperCase,
            "S01" => EwfNamingSchema::S01UpperCase,
            "e01" => EwfNamingSchema::E01LowerCase,
            "l01" => EwfNamingSchema::L01LowerCase,
            "s01" => EwfNamingSchema::S01LowerCase,
            _ => return Ok(None),
        };
        Ok(Some(naming_schema))
    }

    /// Retrieves the segment set identifier.
    pub fn get_segment_set_identifier(&self) -> &Uuid {
        &self.segment_set_identifier
    }

    /// Retrieves the SHA-1 hash.
    pub fn get_sha1_hash(&self) -> &[u8] {
        &self.sha1_hash
    }

    /// Opens a storage media image.
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
    fn open_segment_file(&self, segment_file_name: &String) -> Result<EwfFile, ErrorTrace> {
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
        let mut segment_file: EwfFile = EwfFile::new();

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

    /// Reads the sections of a segment file.
    fn read_sections(
        &mut self,
        segment_file: &EwfFile,
        segment_file_name: &String,
        data_stream: &DataStreamReference,
        block_media_offset: &mut u64,
        last_segment_file: &mut bool,
    ) -> Result<(), ErrorTrace> {
        let mut file_offset: u64 = 13;

        let mut last_sectors_section_header: Option<&EwfSectionHeader> = None;

        for section_header in &segment_file.sections {
            match section_header.section_type.as_slice() {
                EWF_SECTION_TYPE_DATA => {
                    let mut volume: EwfE01Volume = EwfE01Volume::new();

                    match volume.read_at_position(&data_stream, SeekFrom::Start(file_offset + 76)) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read data section"
                            );
                            return Err(error);
                        }
                    }
                    if self.segment_set_identifier != volume.segment_set_identifier {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Mismatch between segment set identifier in volume section: {} and data section: {}",
                            self.segment_set_identifier.to_string(),
                            volume.segment_set_identifier.to_string(),
                        )));
                    }
                }
                EWF_SECTION_TYPE_DIGEST => {
                    let mut digest: EwfDigest = EwfDigest::new();

                    match digest.read_at_position(&data_stream, SeekFrom::Start(file_offset + 76)) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read digest section"
                            );
                            return Err(error);
                        }
                    }
                    self.md5_hash.copy_from_slice(&digest.md5_hash);
                    self.sha1_hash.copy_from_slice(&digest.sha1_hash);
                }
                EWF_SECTION_TYPE_DISK | EWF_SECTION_TYPE_VOLUME => {
                    match self.read_volume_section(
                        segment_file,
                        segment_file_name,
                        data_stream,
                        file_offset,
                        section_header,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read disk or volume section"
                            );
                            return Err(error);
                        }
                    }
                }
                EWF_SECTION_TYPE_DONE => {
                    *last_segment_file = true;
                }
                EWF_SECTION_TYPE_ERROR2 => {
                    let mut error2: EwfError2 = EwfError2::new();

                    match error2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        SeekFrom::Start(file_offset + 76),
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read error2 section"
                            );
                            return Err(error);
                        }
                    }
                    // TODO: store entries
                }
                EWF_SECTION_TYPE_HASH => {
                    let mut hash: EwfHash = EwfHash::new();

                    match hash.read_at_position(&data_stream, SeekFrom::Start(file_offset + 76)) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read hash section"
                            );
                            return Err(error);
                        }
                    }
                    self.md5_hash.copy_from_slice(&hash.md5_hash);
                }
                EWF_SECTION_TYPE_HEADER => {
                    let mut header: EwfHeader = EwfHeader::new();

                    match header.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        SeekFrom::Start(file_offset + 76),
                        &mut self.header_values,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read header section"
                            );
                            return Err(error);
                        }
                    }
                }
                EWF_SECTION_TYPE_HEADER2 => {
                    if segment_file.segment_number != 1 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported header2 section found in segment file: {}",
                            segment_file_name
                        )));
                    }
                    let mut header2: EwfHeader2 = EwfHeader2::new();

                    match header2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        SeekFrom::Start(file_offset + 76),
                        &mut self.header_values,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read header2 section"
                            );
                            return Err(error);
                        }
                    }
                }
                EWF_SECTION_TYPE_LTREE => {
                    let mut ltree_header: EwfLtreeHeader = EwfLtreeHeader::new();

                    match ltree_header
                        .read_at_position(&data_stream, SeekFrom::Start(file_offset + 76))
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read ltree header section"
                            );
                            return Err(error);
                        }
                    }
                    if self.number_of_chunks == 0 {
                        // Correct the media size information for EWF-L01.
                        self.media_size = ltree_header.data_size;
                        self.number_of_sectors = 0;
                    }
                }
                // TODO: ltypes
                EWF_SECTION_TYPE_NEXT => {
                    *last_segment_file = false;
                }
                EWF_SECTION_TYPE_SECTORS => {
                    last_sectors_section_header = Some(section_header);
                }
                // TODO: session
                EWF_SECTION_TYPE_TABLE => {
                    match self.read_table_section(
                        segment_file,
                        data_stream,
                        file_offset,
                        section_header,
                        &last_sectors_section_header,
                        block_media_offset,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read table section"
                            );
                            return Err(error);
                        }
                    }
                }
                EWF_SECTION_TYPE_TABLE2 => {
                    let mut table2: EwfTable = EwfTable::new();

                    match table2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        SeekFrom::Start(file_offset + 76),
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read table2 section"
                            );
                            return Err(error);
                        }
                    }
                    // TODO: compare with table
                }
                // TODO: xhash
                // TODO: xheader
                _ => {}
            }
            file_offset += section_header.size;
        }
        Ok(())
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
        self.naming_schema = match Self::get_segment_file_naming_schema(file_name) {
            Ok(naming_schema) => naming_schema,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine naming schema from segment file: {}",
                        file_name,
                    )
                );
                return Err(error);
            }
        };
        let mut block_media_offset: u64 = 0;
        let mut last_segment_file: bool = false;
        let mut segment_number: u16 = 1;

        while !last_segment_file {
            let segment_file_name: String = match EwfSegmentFile::get_file_name(
                &self.name,
                segment_number,
                self.naming_schema.as_ref(),
            ) {
                Ok(name) => name,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to determine file name of segment number: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            };
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
            let mut segment_file: EwfFile = EwfFile::new();

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
            match segment_file.read_section_headers() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read section headers in segment file: {}",
                            segment_file_name
                        )
                    );
                    return Err(error);
                }
            }
            if segment_file.segment_number != segment_number {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment number: {} in segment file: {}",
                    segment_file.segment_number, segment_file_name
                )));
            }
            match self.read_sections(
                &segment_file,
                &segment_file_name,
                &data_stream,
                &mut block_media_offset,
                &mut last_segment_file,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read sections");
                    return Err(error);
                }
            }
            self.segment_file_cache.insert(segment_number, segment_file);

            segment_number += 1;
        }
        Ok(())
    }

    /// Reads a table section.
    fn read_table_section(
        &mut self,
        segment_file: &EwfFile,
        data_stream: &DataStreamReference,
        file_offset: u64,
        section_header: &EwfSectionHeader,
        last_sectors_section_header: &Option<&EwfSectionHeader>,
        block_media_offset: &mut u64,
    ) -> Result<(), ErrorTrace> {
        if self.chunk_size == 0 || self.media_size == 0 {
            return Err(keramics_core::error_trace_new!(
                "Missing disk or volume section"
            ));
        }
        let mut safe_block_media_offset: u64 = *block_media_offset;

        let mut table: EwfTable = EwfTable::new();

        match table.read_at_position(
            &data_stream,
            section_header.size - 76,
            SeekFrom::Start(file_offset + 76),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read table");
                return Err(error);
            }
        }
        let number_of_entries: usize = table.entries.len();

        if number_of_entries == 0 {
            return Err(keramics_core::error_trace_new!("Missing table entries"));
        }
        let mut table_entry: &EwfTableEntry = &table.entries[0];
        let mut chunk_data_offset_overflow: bool = false;

        for table_entry_index in 0..number_of_entries - 1 {
            let chunk_is_compressed: bool = if chunk_data_offset_overflow {
                false
            } else {
                table_entry.is_compressed()
            };
            let chunk_data_offset: u32 = if chunk_data_offset_overflow {
                table_entry.chunk_data_offset
            } else {
                table_entry.chunk_data_offset & 0x7fffffff
            };
            let next_table_entry: &EwfTableEntry = &table.entries[table_entry_index + 1];

            let next_chunk_data_offset: u32 = if chunk_data_offset_overflow {
                next_table_entry.chunk_data_offset
            } else {
                next_table_entry.chunk_data_offset & 0x7fffffff
            };
            let chunk_data_size: u32 = if chunk_data_offset < next_chunk_data_offset {
                next_chunk_data_offset - chunk_data_offset
            } else if chunk_data_offset < next_table_entry.chunk_data_offset {
                #[cfg(feature = "debug-trace")]
                DebugTrace::static_scope(|debug_trace| {
                    debug_trace.print(format!(
                        "EwfImage table entry: {} current offset: {} larger than next offset: {}",
                        table_entry_index, chunk_data_offset, next_chunk_data_offset
                    ));
                });
                next_table_entry.chunk_data_offset - chunk_data_offset
            } else {
                #[cfg(feature = "debug-trace")]
                DebugTrace::static_scope(|debug_trace| {
                    debug_trace.print(format!(
                        "EwfImage table entry: {} current offset: {} larger than next offset: {}",
                        table_entry_index, chunk_data_offset, next_chunk_data_offset
                    ));
                });
                0
            };
            let block_range_type: EwfBlockRangeType =
                if chunk_data_size == 0 || chunk_data_size > (i32::MAX as u32) {
                    EwfBlockRangeType::Corrupt
                } else if chunk_is_compressed {
                    EwfBlockRangeType::Compressed
                } else {
                    EwfBlockRangeType::InFile
                };
            let block_range: EwfBlockRange = EwfBlockRange::new(
                safe_block_media_offset,
                segment_file.segment_number,
                table.base_offset + (chunk_data_offset as u64),
                chunk_data_size,
                block_range_type,
            );
            self.block_ranges.push(block_range);

            safe_block_media_offset += self.chunk_size as u64;

            // Handle > 2 GiB segment file solution in EnCase 6.7 (chunk data offset overflow)
            if !chunk_data_offset_overflow
                && chunk_data_offset + chunk_data_size > (i32::MAX as u32)
            {
                #[cfg(feature = "debug-trace")]
                DebugTrace::static_scope(|debug_trace| {
                    debug_trace.print(format!(
                        "EwfImage table entry: {} chunk data offset overflow at: {}",
                        table_entry_index, chunk_data_offset
                    ));
                });
                chunk_data_offset_overflow = true;
            }
            table_entry = next_table_entry;
        }
        let chunk_is_compressed: bool = if chunk_data_offset_overflow {
            false
        } else {
            table_entry.is_compressed()
        };
        let chunk_data_offset: u32 = if chunk_data_offset_overflow {
            table_entry.chunk_data_offset
        } else {
            table_entry.chunk_data_offset & 0x7fffffff
        };
        // There is no indication how large the last chunk is, what is known
        // is where it starts. Hence the size of the last chunk is determined
        // by subtracting the last offset from the offset of the next section.

        let last_chunk_data_offset: u64 = table.base_offset + (chunk_data_offset as u64);

        let last_chunk_data_end_offset: u64 = match last_sectors_section_header {
            // The chunks are stored in the sectors section.
            Some(sectors_section_header) => sectors_section_header.next_offset,
            // The chunks are stored in the table section.
            None => section_header.next_offset,
        };
        let last_chunk_data_size: u32 =
            (last_chunk_data_end_offset - last_chunk_data_offset) as u32;

        let block_range_type: EwfBlockRangeType = if chunk_is_compressed {
            EwfBlockRangeType::Compressed
        } else {
            EwfBlockRangeType::InFile
        };
        let block_range: EwfBlockRange = EwfBlockRange::new(
            safe_block_media_offset,
            segment_file.segment_number,
            last_chunk_data_offset,
            last_chunk_data_size,
            block_range_type,
        );
        self.block_ranges.push(block_range);

        *block_media_offset = safe_block_media_offset + (self.chunk_size as u64);

        Ok(())
    }

    /// Reads a volume section.
    fn read_volume_section(
        &mut self,
        segment_file: &EwfFile,
        segment_file_name: &String,
        data_stream: &DataStreamReference,
        file_offset: u64,
        section_header: &EwfSectionHeader,
    ) -> Result<(), ErrorTrace> {
        if segment_file.segment_number != 1 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported disk or volume section found in segment file: {}",
                segment_file_name
            )));
        }
        if self.chunk_size != 0 || self.media_size != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Multipe disk or volume sections found in segment file: {}",
                segment_file_name
            )));
        }
        match section_header.size {
            170 => {
                let mut volume: EwfS01Volume = EwfS01Volume::new();

                match volume.read_at_position(&data_stream, SeekFrom::Start(file_offset + 76)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read volume");
                        return Err(error);
                    }
                }
                self.number_of_chunks = volume.number_of_chunks;
                self.sectors_per_chunk = volume.sectors_per_chunk;
                self.bytes_per_sector = volume.bytes_per_sector;
                self.number_of_sectors = volume.number_of_sectors;
            }
            1128 => {
                let mut volume: EwfE01Volume = EwfE01Volume::new();

                match volume.read_at_position(&data_stream, SeekFrom::Start(file_offset + 76)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read volume");
                        return Err(error);
                    }
                }
                self.media_type = match volume.media_type {
                    0x00 => EwfMediaType::RemoveableDisk,
                    0x01 => EwfMediaType::FixedDisk,
                    0x03 => EwfMediaType::OpticalDisk,
                    0x0e => EwfMediaType::LogicalEvidence,
                    0x10 => EwfMediaType::Memory,
                    _ => EwfMediaType::Unknown,
                };
                self.number_of_chunks = volume.number_of_chunks;
                self.sectors_per_chunk = volume.sectors_per_chunk;
                self.bytes_per_sector = volume.bytes_per_sector;
                self.number_of_sectors = volume.number_of_sectors;
                self.error_granularity = volume.error_granularity;
                self.segment_set_identifier = volume.segment_set_identifier;
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported volume section data size: {} found in segment file: {}",
                    section_header.size - 76,
                    segment_file_name
                )));
            }
        }
        self.chunk_size = self.sectors_per_chunk * self.bytes_per_sector;
        self.media_size = (self.number_of_sectors as u64) * (self.bytes_per_sector as u64);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    use crate::tests::get_test_data_path;

    fn get_image() -> Result<EwfImage, ErrorTrace> {
        let mut image: EwfImage = EwfImage::new();

        let path_string: String = get_test_data_path("ewf");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.E01");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: EwfImage = get_image()?;

        let bytes_per_sector: u32 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for get_header_value

    #[test]
    fn test_get_number_of_sectors() -> Result<(), ErrorTrace> {
        let image: EwfImage = get_image()?;

        let number_of_sectors: u32 = image.get_number_of_sectors();
        assert_eq!(number_of_sectors, 8192);

        Ok(())
    }

    #[test]
    fn test_get_segment_file_naming_schema() -> Result<(), ErrorTrace> {
        let file_name: PathComponent = PathComponent::from("image.E01");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, Some(EwfNamingSchema::E01UpperCase));

        let file_name: PathComponent = PathComponent::from("image.e01");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, Some(EwfNamingSchema::E01LowerCase));

        let file_name: PathComponent = PathComponent::from("image.S01");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, Some(EwfNamingSchema::S01UpperCase));

        let file_name: PathComponent = PathComponent::from("image.s01");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, Some(EwfNamingSchema::S01LowerCase));

        let file_name: PathComponent = PathComponent::from("image");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, None);

        let file_name: PathComponent = PathComponent::from("image.raw");
        let naming_schema: Option<EwfNamingSchema> =
            EwfImage::get_segment_file_naming_schema(&file_name)?;
        assert_eq!(naming_schema, None);

        Ok(())
    }

    #[test]
    fn test_get_segment_set_identifier() -> Result<(), ErrorTrace> {
        let image: EwfImage = get_image()?;

        let segment_set_identifier: &Uuid = image.get_segment_set_identifier();
        assert_eq!(
            segment_set_identifier.to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: EwfImage = EwfImage::new();

        let path_string: String = get_test_data_path("ewf");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.E01");
        image.open(&file_resolver, &file_name)?;

        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for open_segment_file
    // TODO: add tests for read_sections
    // TODO: add tests for read_table_section
    // TODO: add tests for read_volume_section
}
