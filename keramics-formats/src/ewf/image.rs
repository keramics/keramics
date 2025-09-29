/* Copyright 2024-2025 Joachim Metz <joachim.metz@gmail.com>
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
use std::io;
use std::io::{Read, Seek};

use keramics_compression::ZlibContext;
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference, FakeFileResolver, FileResolverReference};
use keramics_types::Uuid;

use crate::block_tree::BlockTree;
use crate::lru_cache::LruCache;

use super::block_range::{EwfBlockRange, EwfBlockRangeType};
use super::constants::*;
use super::digest::EwfDigest;
use super::enums::{EwfHeaderValueType, EwfMediaType, EwfNamingSchema};
use super::error2::EwfError2;
use super::file::EwfFile;
use super::hash::EwfHash;
use super::header::EwfHeader;
use super::header_value::EwfHeaderValue;
use super::header2::EwfHeader2;
use super::section_header::EwfSectionHeader;
use super::table::EwfTable;
use super::table_entry::EwfTableEntry;
use super::volume::{EwfE01Volume, EwfS01Volume};

/// Expert Witness Compression Format (EWF) image.
pub struct EwfImage {
    /// Mediator.
    mediator: MediatorReference,

    /// File resolver.
    file_resolver: FileResolverReference,

    /// Segment file set identifier.
    pub set_identifier: Uuid,

    /// Segment file cache.
    segment_file_cache: LruCache<u16, EwfFile>,

    /// Number of chunks.
    number_of_chunks: u32,

    /// Sectors per chunk.
    pub sectors_per_chunk: u32,

    /// Bytes per sector.
    pub bytes_per_sector: u32,

    /// Number of sectors.
    pub number_of_sectors: u32,

    /// Block (or chunk) size.
    block_size: u32,

    /// Block tree.
    block_tree: BlockTree<EwfBlockRange>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Error granularity.
    pub error_granularity: u32,

    /// Media type.
    pub media_type: EwfMediaType,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,

    /// Values stored in header and header2 sections.
    header_values: HashMap<EwfHeaderValueType, EwfHeaderValue>,

    /// MD5 hash.
    pub md5_hash: [u8; 16],

    /// SHA1 hash.
    pub sha1_hash: [u8; 20],
}

impl EwfImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            set_identifier: Uuid::new(),
            segment_file_cache: LruCache::new(16),
            number_of_chunks: 0,
            sectors_per_chunk: 0,
            bytes_per_sector: 0,
            number_of_sectors: 0,
            block_size: 0,
            block_tree: BlockTree::<EwfBlockRange>::new(0, 0, 0),
            block_cache: LruCache::new(64),
            error_granularity: 0,
            media_type: EwfMediaType::Unknown,
            media_size: 0,
            media_offset: 0,
            header_values: HashMap::new(),
            md5_hash: [0; 16],
            sha1_hash: [0; 20],
        }
    }

    /// Retrieves a header value.
    pub fn get_header_value(&self, value_type: &EwfHeaderValueType) -> Option<&EwfHeaderValue> {
        self.header_values.get(value_type)
    }

    /// Determines the segment file extension for a given segment number.
    fn get_segment_file_extension(
        &self,
        segment_number: u16,
        naming_schema: &EwfNamingSchema,
    ) -> io::Result<String> {
        if segment_number == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported segment number: 0",
            ));
        }
        let mut extension: [u32; 3] = [0; 3];

        let first_character: u32 = match naming_schema {
            EwfNamingSchema::E01UpperCase => 0x45, // 'E'
            EwfNamingSchema::S01UpperCase => 0x53, // 'S'
            EwfNamingSchema::E01LowerCase => 0x65, // 'e'
            EwfNamingSchema::S01LowerCase => 0x73, // 's'
        };
        if segment_number < 100 {
            extension[2] = 0x30 + (segment_number % 10) as u32;
            extension[1] = 0x30 + (segment_number / 10) as u32;
            extension[0] = first_character;
        } else {
            let base_character: u32 = match naming_schema {
                EwfNamingSchema::E01UpperCase | EwfNamingSchema::S01UpperCase => 0x41, // 'A'
                EwfNamingSchema::E01LowerCase | EwfNamingSchema::S01LowerCase => 0x61, // 'a'
            };
            let mut extension_segment_number: u32 = (segment_number as u32) - 100;

            extension[2] = base_character + (extension_segment_number % 26) as u32;
            extension_segment_number /= 26;

            extension[1] = base_character + (extension_segment_number % 26) as u32;
            extension_segment_number /= 26;

            extension[0] = first_character + extension_segment_number;
        }
        let last_character: u32 = match naming_schema {
            EwfNamingSchema::E01UpperCase | EwfNamingSchema::S01UpperCase => 0x5a, // 'Z'
            EwfNamingSchema::E01LowerCase | EwfNamingSchema::S01LowerCase => 0x7a, // 'z'
        };
        if extension[0] > last_character {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Unsupported segment number: {} value exceeds maximum for naming schema",
                    segment_number,
                ),
            ));
        }
        let segment_extension: String = extension
            .iter()
            .map(|value| std::char::from_u32(*value).unwrap())
            .collect::<String>();
        Ok(segment_extension)
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &str,
    ) -> io::Result<()> {
        self.read_segment_files(&file_resolver, file_name)?;

        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads media data based on the chunk tables.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> io::Result<usize> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let block_tree_value: Option<&EwfBlockRange> = self.block_tree.get_value(media_offset);

            let block_range: &EwfBlockRange = match block_tree_value {
                Some(value) => value,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Missing block range for offset: {}", media_offset),
                    ));
                }
            };
            if !self
                .segment_file_cache
                .contains(&block_range.segment_number)
            {
                let file: EwfFile = EwfFile::new();

                self.segment_file_cache
                    .insert(block_range.segment_number, file);
            }
            let segment_file: &mut EwfFile =
                match self.segment_file_cache.get_mut(&block_range.segment_number) {
                    Some(file) => file,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Unable to retrieve segment file: {} from cache.",
                                block_range.segment_number
                            ),
                        ));
                    }
                };
            let range_relative_offset: u64 = media_offset - block_range.media_offset;
            let range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;
            let range_read_count: usize = match block_range.range_type {
                EwfBlockRangeType::Compressed => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    if !self.block_cache.contains(&block_range.data_offset) {
                        let mut compressed_data: Vec<u8> = vec![0; block_range.data_size as usize];

                        segment_file.read_exact_at_position(
                            &mut compressed_data,
                            io::SeekFrom::Start(block_range.data_offset),
                        )?;
                        if self.mediator.debug_output {
                            self.mediator.debug_print(format!(
                                "Compressed data of size: {} at offset: {} (0x{:08x})\n",
                                block_range.data_size,
                                block_range.data_offset,
                                block_range.data_offset,
                            ));
                            self.mediator.debug_print_data(&compressed_data, true);
                        }
                        let mut block_data: Vec<u8> = vec![0; self.block_size as usize];

                        let mut zlib_context: ZlibContext = ZlibContext::new();
                        zlib_context.decompress(&compressed_data, &mut block_data)?;

                        self.block_cache.insert(block_range.data_offset, block_data);
                    }
                    let range_data: &Vec<u8> = match self.block_cache.get(&block_range.data_offset)
                    {
                        Some(data) => data,
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Unable to retrieve data from cache."),
                            ));
                        }
                    };
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);

                    range_read_size
                }
                EwfBlockRangeType::InFile => {
                    segment_file.read_exact_at_position(
                        &mut data[data_offset..data_end_offset],
                        io::SeekFrom::Start(block_range.data_offset + range_relative_offset),
                    )?;
                    // TODO: calculate and compare checksum

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }

    /// Reads the sections of a segment file.
    fn read_sections(
        &mut self,
        segment_file: &EwfFile,
        segment_file_name: &String,
        data_stream: &DataStreamReference,
        block_media_offset: &mut u64,
        last_segment_file: &mut bool,
    ) -> io::Result<()> {
        let mut file_offset: u64 = 13;

        let mut last_sectors_section_header: Option<&EwfSectionHeader> = None;

        for section_header in &segment_file.sections {
            match &section_header.section_type {
                &EWF_SECTION_TYPE_DATA => {
                    let mut volume: EwfE01Volume = EwfE01Volume::new();

                    volume.read_at_position(&data_stream, io::SeekFrom::Start(file_offset + 76))?;

                    if self.set_identifier != volume.set_identifier {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Mismatch between set identifier in volume section: {} and data section: {}",
                                self.set_identifier.to_string(),
                                volume.set_identifier.to_string(),
                            ),
                        ));
                    }
                }
                &EWF_SECTION_TYPE_DIGEST => {
                    let mut digest: EwfDigest = EwfDigest::new();

                    digest.read_at_position(&data_stream, io::SeekFrom::Start(file_offset + 76))?;

                    self.md5_hash.copy_from_slice(&digest.md5_hash);
                    self.sha1_hash.copy_from_slice(&digest.sha1_hash);
                }
                &EWF_SECTION_TYPE_DISK | &EWF_SECTION_TYPE_VOLUME => {
                    self.read_volume_section(
                        segment_file,
                        segment_file_name,
                        data_stream,
                        file_offset,
                        section_header,
                    )?;
                }
                &EWF_SECTION_TYPE_DONE => {
                    *last_segment_file = true;
                }
                &EWF_SECTION_TYPE_ERROR2 => {
                    let mut error2: EwfError2 = EwfError2::new();

                    error2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        io::SeekFrom::Start(file_offset + 76),
                    )?;
                    // TODO: store entries
                }
                &EWF_SECTION_TYPE_HASH => {
                    let mut hash: EwfHash = EwfHash::new();

                    hash.read_at_position(&data_stream, io::SeekFrom::Start(file_offset + 76))?;

                    self.md5_hash.copy_from_slice(&hash.md5_hash);
                }
                &EWF_SECTION_TYPE_HEADER => {
                    if segment_file.segment_number != 1 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported header section found in segment file: {}",
                                segment_file_name
                            ),
                        ));
                    }
                    let mut header: EwfHeader = EwfHeader::new();
                    header.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        io::SeekFrom::Start(file_offset + 76),
                        &mut self.header_values,
                    )?;
                }
                &EWF_SECTION_TYPE_HEADER2 => {
                    if segment_file.segment_number != 1 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported header2 section found in segment file: {}",
                                segment_file_name
                            ),
                        ));
                    }
                    let mut header2: EwfHeader2 = EwfHeader2::new();
                    header2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        io::SeekFrom::Start(file_offset + 76),
                        &mut self.header_values,
                    )?;
                }
                // TODO: ltree
                // TODO: ltypes
                &EWF_SECTION_TYPE_NEXT => {
                    *last_segment_file = false;
                }
                &EWF_SECTION_TYPE_SECTORS => {
                    last_sectors_section_header = Some(section_header);
                }
                // TODO: session
                &EWF_SECTION_TYPE_TABLE => {
                    self.read_table_section(
                        segment_file,
                        data_stream,
                        file_offset,
                        section_header,
                        &last_sectors_section_header,
                        block_media_offset,
                    )?;
                }
                &EWF_SECTION_TYPE_TABLE2 => {
                    let mut table2: EwfTable = EwfTable::new();

                    table2.read_at_position(
                        &data_stream,
                        section_header.size - 76,
                        io::SeekFrom::Start(file_offset + 76),
                    )?;
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
        file_name: &str,
    ) -> io::Result<()> {
        let (name, extension): (&str, &str) = match file_name.rsplit_once(".") {
            Some(components) => (components.0, components.1),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Extension missing in segment file: {}", file_name),
                ));
            }
        };
        let naming_schema: EwfNamingSchema = match extension {
            "E01" => EwfNamingSchema::E01UpperCase,
            "S01" => EwfNamingSchema::S01UpperCase,
            "e01" => EwfNamingSchema::E01LowerCase,
            "s01" => EwfNamingSchema::S01LowerCase,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Unsupported extension in segment file: {}", file_name),
                ));
            }
        };
        let mut block_media_offset: u64 = 0;
        let mut last_segment_file: bool = false;
        let mut segment_number: u16 = 1;

        while !last_segment_file {
            let segment_extension: String =
                self.get_segment_file_extension(segment_number, &naming_schema)?;
            let segment_file_name: String = format!("{}.{}", name, segment_extension);

            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&mut vec![segment_file_name.as_str()])? {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such segment file: {}", segment_file_name),
                        ));
                    }
                };
            let mut segment_file: EwfFile = EwfFile::new();
            segment_file.read_data_stream(&data_stream)?;

            if segment_file.segment_number != segment_number {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Unsupported segment file: {} with segment number: {}",
                        segment_file_name, segment_file.segment_number
                    ),
                ));
            }
            self.read_sections(
                &segment_file,
                &segment_file_name,
                &data_stream,
                &mut block_media_offset,
                &mut last_segment_file,
            )?;
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
    ) -> io::Result<()> {
        if self.block_size == 0 || self.media_size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing disk or volume section",
            ));
        }
        let mut safe_block_media_offset: u64 = *block_media_offset;

        let mut table: EwfTable = EwfTable::new();

        table.read_at_position(
            &data_stream,
            section_header.size - 76,
            io::SeekFrom::Start(file_offset + 76),
        )?;
        let number_of_entries: usize = table.entries.len();

        if number_of_entries == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing table entries",
            ));
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
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "EwfImage table entry: {} current offset: {} larger than next offset: {}",
                        table_entry_index, chunk_data_offset, next_chunk_data_offset
                    ));
                }
                next_table_entry.chunk_data_offset - chunk_data_offset
            } else {
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "EwfImage table entry: {} current offset: {} larger than next offset: {}",
                        table_entry_index, chunk_data_offset, next_chunk_data_offset
                    ));
                }
                // TODO: handle corrupted table entry
                todo!();
            };
            let block_range_type: EwfBlockRangeType = if chunk_is_compressed {
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
            match self.block_tree.insert_value(
                safe_block_media_offset,
                self.block_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            };
            safe_block_media_offset += self.block_size as u64;

            // handle > 2 GiB segment file solution in EnCase 6.7 (chunk data offset
            // overflow)
            if !chunk_data_offset_overflow
                && chunk_data_offset + chunk_data_size > (i32::MAX as u32)
            {
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "EwfImage table entry: {} chunk data offset overflow at: {}",
                        table_entry_index, chunk_data_offset
                    ));
                }
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
        match self.block_tree.insert_value(
            safe_block_media_offset,
            self.block_size as u64,
            block_range,
        ) {
            Ok(_) => {}
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        *block_media_offset = safe_block_media_offset + (self.block_size as u64);

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
    ) -> io::Result<()> {
        if segment_file.segment_number != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported disk or volume section found in segment file: {}",
                    segment_file_name
                ),
            ));
        }
        if self.block_size != 0 || self.media_size != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Multipe disk or volume sections found in segment file: {}",
                    segment_file_name
                ),
            ));
        }
        match section_header.size {
            170 => {
                let mut volume: EwfS01Volume = EwfS01Volume::new();

                volume.read_at_position(&data_stream, io::SeekFrom::Start(file_offset + 76))?;
                self.number_of_chunks = volume.number_of_chunks;
                self.sectors_per_chunk = volume.sectors_per_chunk;
                self.bytes_per_sector = volume.bytes_per_sector;
                self.number_of_sectors = volume.number_of_sectors;
            }
            1128 => {
                let mut volume: EwfE01Volume = EwfE01Volume::new();

                volume.read_at_position(&data_stream, io::SeekFrom::Start(file_offset + 76))?;

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
                self.set_identifier = volume.set_identifier;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Unsupported volume section data size: {} found in segment file: {}",
                        section_header.size - 76,
                        segment_file_name
                    ),
                ));
            }
        }
        self.block_size = self.sectors_per_chunk * self.bytes_per_sector;
        self.media_size = (self.number_of_sectors as u64) * (self.bytes_per_sector as u64);

        let block_tree_data_size: u64 = (self.number_of_chunks as u64) * (self.block_size as u64);

        self.block_tree = BlockTree::<EwfBlockRange>::new(
            block_tree_data_size,
            self.sectors_per_chunk as u64,
            self.bytes_per_sector as u64,
        );
        Ok(())
    }
}

impl Read for EwfImage {
    /// Reads media data.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.media_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.media_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = self.read_data_from_blocks(&mut buf[..read_size])?;

        self.media_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for EwfImage {
    /// Sets the current position of the media data.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.media_offset = match pos {
            io::SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.media_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            io::SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.media_size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            io::SeekFrom::Start(offset) => offset,
        };
        Ok(self.media_offset)
    }
}

impl DataStream for EwfImage {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.media_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_file_resolver;

    fn get_image() -> io::Result<EwfImage> {
        let mut image: EwfImage = EwfImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/ewf")?;
        image.open(&file_resolver, "ext2.E01")?;

        Ok(image)
    }

    #[test]
    fn test_get_segment_file_extension() -> io::Result<()> {
        let image: EwfImage = EwfImage::new();

        let extension: String =
            image.get_segment_file_extension(1, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "E01");

        let extension: String =
            image.get_segment_file_extension(99, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "E99");

        let extension: String =
            image.get_segment_file_extension(100, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EAA");

        let extension: String =
            image.get_segment_file_extension(125, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EAZ");

        let extension: String =
            image.get_segment_file_extension(126, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EBA");

        let extension: String =
            image.get_segment_file_extension(776, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "FAA");

        let extension: String =
            image.get_segment_file_extension(14296, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "ZAA");

        let extension: String =
            image.get_segment_file_extension(14971, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "ZZZ");

        let result = image.get_segment_file_extension(14972, &EwfNamingSchema::E01UpperCase);
        assert!(result.is_err());

        let extension: String =
            image.get_segment_file_extension(1, &EwfNamingSchema::S01UpperCase)?;
        assert_eq!(extension, "S01");

        let extension: String =
            image.get_segment_file_extension(1, &EwfNamingSchema::E01LowerCase)?;
        assert_eq!(extension, "e01");

        let extension: String =
            image.get_segment_file_extension(1, &EwfNamingSchema::S01LowerCase)?;
        assert_eq!(extension, "s01");

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut image: EwfImage = EwfImage::new();

        let file_resolver: FileResolverReference = open_os_file_resolver("../test_data/ewf")?;
        image.open(&file_resolver, "ext2.E01")?;

        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for read_sections
    // TODO: add tests for read_table_section
    // TODO: add tests for read_volume_section

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;

        let offset = image.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;
        image.seek(io::SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x04, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x43, 0x0f,
            0x00, 0x00, 0xe3, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x0a, 0xea, 0x78, 0x67, 0x0a, 0xea, 0x78, 0x67, 0x02, 0x00, 0xff, 0xff,
            0x53, 0xef, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x09, 0xea, 0x78, 0x67, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0b, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x57, 0x1e, 0x25, 0x97, 0x42, 0xa1, 0x4d, 0x6a,
            0xad, 0xa9, 0xcd, 0xb1, 0x19, 0x1b, 0x5d, 0xea, 0x65, 0x78, 0x74, 0x32, 0x5f, 0x74,
            0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x6d, 0x6e, 0x74,
            0x2f, 0x6b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2a, 0x43,
            0x11, 0xae, 0xbe, 0xdb, 0x40, 0x41, 0xa4, 0xb6, 0xf5, 0x6b, 0x15, 0x34, 0xd6, 0x66,
            0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0xea,
            0x78, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2e, 0x00,
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
    fn test_seek_and_read_beyond_media_size() -> io::Result<()> {
        let mut image: EwfImage = get_image()?;
        image.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
