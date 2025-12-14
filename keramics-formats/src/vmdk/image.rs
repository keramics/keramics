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

use std::cmp::min;
use std::io::SeekFrom;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_range::{VmdkBlockRange, VmdkBlockRangeType};
use super::constants::*;
use super::descriptor_extent::VmdkDescriptorExtent;
use super::descriptor_storage::VmdkDescriptorStorage;
use super::enums::{VmdkCompressionMethod, VmdkDescriptorExtentType, VmdkDiskType, VmdkFileType};
use super::extent_file::VmdkExtentFile;
use super::sparse_cowd_file::VmdkSparseCowdFile;
use super::sparse_file::VmdkSparseFile;
use super::sparse_file_header::VmdkSparseFileHeader;

/// VMware Virtual Disk (VMDK) storage media image.
pub struct VmdkImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Disk type.
    pub disk_type: VmdkDiskType,

    /// Sectors per grain.
    pub sectors_per_grain: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Compression method.
    pub compression_method: VmdkCompressionMethod,

    /// Content identifier.
    pub content_identifier: u32,

    /// Parent content identifier.
    pub parent_content_identifier: Option<u32>,

    /// Parent name.
    pub parent_name: Option<ByteString>,

    /// Extents.
    extents: Vec<VmdkDescriptorExtent>,

    /// Extent file cache.
    extent_file_cache: LruCache<u64, VmdkExtentFile>,

    /// Decompressed grain cache.
    grain_cache: LruCache<u64, Vec<u8>>,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    pub media_size: u64,
}

impl VmdkImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            character_encoding: CharacterEncoding::Utf8,
            disk_type: VmdkDiskType::Unknown,
            sectors_per_grain: 0,
            bytes_per_sector: 0,
            compression_method: VmdkCompressionMethod::None,
            content_identifier: 0,
            parent_content_identifier: None,
            parent_name: None,
            extents: Vec::new(),
            extent_file_cache: LruCache::new(16),
            grain_cache: LruCache::new(64),
            current_offset: 0,
            media_size: 0,
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open file: {}", file_name)
                );
                return Err(error);
            }
        };
        let file_type: VmdkFileType = match self.read_file_header(&data_stream) {
            Ok(file_type) => file_type,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        };
        self.bytes_per_sector = 512;

        match &file_type {
            VmdkFileType::DescriptorFile => {
                let file_size: u64 = keramics_core::data_stream_get_size!(data_stream);

                match self.read_descriptor(&data_stream, 0, file_size) {
                    Ok(()) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read descriptor file"
                        );
                        return Err(error);
                    }
                }
            }
            VmdkFileType::VmdkSparseFile => {
                let mut file_header: VmdkSparseFileHeader = VmdkSparseFileHeader::new();

                match file_header.read_at_position(&data_stream, SeekFrom::Start(0)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                        return Err(error);
                    }
                }
                if file_header.descriptor_start_sector == 0
                    || file_header.descriptor_start_sector
                        > u64::MAX / (self.bytes_per_sector as u64)
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid descriptor start sector value out of bounds"
                    )));
                }
                if file_header.descriptor_size == 0
                    || file_header.descriptor_size > u64::MAX / (self.bytes_per_sector as u64)
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid descriptor size value out of bounds"
                    )));
                }
                let descriptor_offset: u64 =
                    file_header.descriptor_start_sector * (self.bytes_per_sector as u64);
                let descriptor_size: u64 =
                    file_header.descriptor_size * (self.bytes_per_sector as u64);

                match self.read_descriptor(&data_stream, descriptor_offset, descriptor_size) {
                    Ok(()) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read descriptor from sparse file"
                        );
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported file type"
                )));
            }
        }
        match self.read_extent_files(file_resolver, file_name) {
            Ok(()) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read extent files");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads media data based on the extent files.
    fn read_data_from_extents(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.current_offset;

        let mut extent_index: usize = 0;
        let mut extent_offset: u64 = self.current_offset;

        // TODO: optimize extent lookup
        for extent in self.extents.iter() {
            let extent_size: u64 = extent.number_of_sectors * (self.bytes_per_sector as u64);

            if extent_offset < extent_size {
                break;
            }
            extent_index += 1;
            extent_offset -= extent_size;
        }
        if extent_index >= self.extents.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid media offset: {} (0x{:08x}) value out of bounds",
                media_offset, media_offset
            )));
        }
        let extent: &VmdkDescriptorExtent = match self.extents.get(extent_index) {
            Some(extent) => extent,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for offset: {} (0x{:08x})",
                    media_offset, media_offset
                )));
            }
        };
        let mut extent_size: u64 = extent.number_of_sectors * (self.bytes_per_sector as u64);

        while data_offset < read_size {
            let extent_remainder_size: u64 = extent_size - extent_offset;
            let extent_read_size: usize =
                min(read_size - data_offset, extent_remainder_size as usize);

            let range_read_count: usize = match &extent.extent_type {
                VmdkDescriptorExtentType::Flat
                | VmdkDescriptorExtentType::Sparse
                | VmdkDescriptorExtentType::VmfsFlat
                | VmdkDescriptorExtentType::VmfsSparse => {
                    let lookup_extent_index: u64 = extent_index as u64;

                    if !self.extent_file_cache.contains(&lookup_extent_index) {
                        let extent_file_name: &ByteString = match extent.file_name.as_ref() {
                            Some(file_name) => file_name,
                            None => {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Missing extent file: {} name",
                                    extent_index
                                )));
                            }
                        };
                        let path_components: [PathComponent; 1] =
                            [PathComponent::from(extent_file_name)];
                        let data_stream: DataStreamReference =
                            match self.file_resolver.get_data_stream(&path_components) {
                                Ok(Some(data_stream)) => data_stream,
                                Ok(None) => {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Missing extent file: {}",
                                        extent_file_name
                                    )));
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!("Unable to open extent file: {}", extent_file_name)
                                    );
                                    return Err(error);
                                }
                            };
                        let extent_file: VmdkExtentFile = match &extent.extent_type {
                            VmdkDescriptorExtentType::Sparse => {
                                let mut sparse_file: VmdkSparseFile = VmdkSparseFile::new();

                                match sparse_file.read_data_stream(&data_stream) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to open sparse VMDK file"
                                        );
                                        return Err(error);
                                    }
                                }
                                VmdkExtentFile::SparseVmdk(sparse_file)
                            }
                            VmdkDescriptorExtentType::VmfsSparse => {
                                let mut sparse_file: VmdkSparseCowdFile = VmdkSparseCowdFile::new();

                                match sparse_file.read_data_stream(&data_stream) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to open sparse COWD file"
                                        );
                                        return Err(error);
                                    }
                                }
                                VmdkExtentFile::SparseCowd(sparse_file)
                            }
                            _ => VmdkExtentFile::Raw(data_stream),
                        };
                        self.extent_file_cache
                            .insert(lookup_extent_index, extent_file);
                    }
                    match self.extent_file_cache.get_mut(&lookup_extent_index) {
                        Some(VmdkExtentFile::Raw(data_stream)) => {
                            let data_end_offset: usize = data_offset + extent_read_size;

                            keramics_core::data_stream_read_at_position!(
                                data_stream,
                                &mut data[data_offset..data_end_offset],
                                SeekFrom::Start(extent_offset)
                            )
                        }
                        Some(VmdkExtentFile::SparseCowd(sparse_file)) => {
                            // TODO: read grain from sparse extent file or parent image
                            todo!();
                        }
                        Some(VmdkExtentFile::SparseVmdk(sparse_file)) => {
                            let mut result: Result<Option<&VmdkBlockRange>, ErrorTrace> =
                                sparse_file.block_tree.get_value(extent_offset);

                            if result == Ok(None) {
                                match sparse_file.read_grain_table_entry(extent_offset) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to read grain table entry"
                                        );
                                        return Err(error);
                                    }
                                }
                                result = sparse_file.block_tree.get_value(extent_offset);
                            }
                            let block_range: &VmdkBlockRange = match result {
                                Ok(Some(block_range)) => block_range,
                                Ok(None) => {
                                    return Err(keramics_core::error_trace_new!(format!(
                                        "Missing block range for offset: {} (0x{:08x})",
                                        extent_offset, extent_offset
                                    )));
                                }
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                                            extent_offset, extent_offset
                                        )
                                    );
                                    return Err(error);
                                }
                            };
                            let range_relative_offset: u64 =
                                extent_offset - block_range.extent_offset;
                            let range_remainder_size: u64 =
                                block_range.size - range_relative_offset;
                            let range_read_size: usize =
                                min(extent_read_size, range_remainder_size as usize);
                            let data_end_offset: usize = data_offset + range_read_size;

                            match block_range.range_type {
                                VmdkBlockRangeType::Compressed => {
                                    let grain_media_offset: u64 = (media_offset
                                        / sparse_file.grain_size)
                                        * sparse_file.grain_size;

                                    if !self.grain_cache.contains(&grain_media_offset) {
                                        let compressed_grain_offset: u64 = block_range.data_offset;

                                        let mut block_data: Vec<u8> =
                                            vec![0; sparse_file.grain_size as usize];

                                        match sparse_file.read_compressed_grain(
                                            compressed_grain_offset,
                                            &mut block_data,
                                        ) {
                                            Ok(_) => {}
                                            Err(mut error) => {
                                                keramics_core::error_trace_add_frame!(
                                                    error,
                                                    format!(
                                                        "Unable to read compressed grain from extent file: {} at offset: {} (0x{:08x})",
                                                        extent_index,
                                                        compressed_grain_offset,
                                                        compressed_grain_offset
                                                    )
                                                );
                                                return Err(error);
                                            }
                                        }
                                        self.grain_cache.insert(grain_media_offset, block_data);
                                    }
                                    let range_data: &[u8] =
                                        match self.grain_cache.get(&grain_media_offset) {
                                            Some(data) => data,
                                            None => {
                                                return Err(keramics_core::error_trace_new!(
                                                    "Unable to retrieve data from cache"
                                                ));
                                            }
                                        };
                                    let range_data_offset: usize = range_relative_offset as usize;
                                    let range_data_end_offset: usize =
                                        range_data_offset + range_read_size;

                                    data[data_offset..data_end_offset].copy_from_slice(
                                        &range_data[range_data_offset..range_data_end_offset],
                                    );

                                    range_read_size
                                }
                                VmdkBlockRangeType::InFile => {
                                    match sparse_file.data_stream.as_ref() {
                                        Some(data_stream) => {
                                            keramics_core::data_stream_read_at_position!(
                                                data_stream,
                                                &mut data[data_offset..data_end_offset],
                                                SeekFrom::Start(
                                                    block_range.data_offset + range_relative_offset
                                                )
                                            )
                                        }
                                        None => {
                                            return Err(keramics_core::error_trace_new!(format!(
                                                "Missing extent file: {} data stream",
                                                extent_index
                                            )));
                                        }
                                    }
                                }
                                VmdkBlockRangeType::InParentOrSparse => {
                                    match self.parent_content_identifier {
                                        Some(_) => todo!(),
                                        None => {
                                            data[data_offset..data_end_offset].fill(0);

                                            range_read_size
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unable to retrieve extent file: {} from cache",
                                extent_index
                            )));
                        }
                    }
                }
                VmdkDescriptorExtentType::Zero => {
                    let data_end_offset: usize = data_offset + extent_read_size;

                    data[data_offset..data_end_offset].fill(0);

                    extent_read_size
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported extent type"
                    )));
                }
            };
            data_offset += range_read_count;
            extent_offset += range_read_count as u64;
            media_offset += range_read_count as u64;

            if media_offset >= self.media_size {
                break;
            }
            if extent_offset >= extent_size {
                extent_index += 1;

                let extent: &VmdkDescriptorExtent = match self.extents.get(extent_index) {
                    Some(extent) => extent,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing extent for offset: {} (0x{:08x})",
                            media_offset, media_offset
                        )));
                    }
                };
                extent_offset = 0;
                extent_size = extent.number_of_sectors * (self.bytes_per_sector as u64);
            }
        }
        Ok(data_offset)
    }

    /// Reads the descriptor
    fn read_descriptor(
        &mut self,
        data_stream: &DataStreamReference,
        descriptor_offset: u64,
        descriptor_size: u64,
    ) -> Result<(), ErrorTrace> {
        // Note that 16777216 is an arbitrary chosen limit.
        if descriptor_size < 21 || descriptor_size > 16777216 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported descriptor size: {} value out of bounds",
                descriptor_size
            )));
        }
        let mut data: Vec<u8> = vec![0; descriptor_size as usize];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(descriptor_offset)
        );
        let mut descriptor_storage: VmdkDescriptorStorage = VmdkDescriptorStorage::new(&data);

        match descriptor_storage.next_line().as_deref() {
            Some(line) => {
                match VmdkDescriptorStorage::to_ascii_lowercase(VmdkDescriptorStorage::trim(line))
                    .as_slice()
                {
                    b"# disk descriptorfile" => {}
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Invalid descriptor data - unsupported signature"
                        )));
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid descriptor data - missing signature"
                )));
            }
        }
        let mut last_line: Vec<u8> = Vec::new();

        while let Some(line) = descriptor_storage.next_line() {
            last_line =
                VmdkDescriptorStorage::to_ascii_lowercase(VmdkDescriptorStorage::trim(line));

            if last_line == b"# extent description" {
                break;
            }
            if last_line.is_empty() || last_line[0] == b'#' {
                continue;
            }
            let (key, value): (&[u8], &[u8]) =
                match VmdkDescriptorStorage::parse_key_value_pair(&last_line) {
                    Some((key, value)) => (key, value),
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid descriptor data - unsupported key-value pair"
                        ));
                    }
                };
            match key {
                b"cid" => match VmdkDescriptorStorage::parse_content_identifier_value(value) {
                    Some(value_32bit) => self.content_identifier = value_32bit,
                    None => {
                        return Err(keramics_core::error_trace_new!("Unsupported CID value"));
                    }
                },
                b"createtype" => match VmdkDescriptorStorage::parse_disk_type_value(value) {
                    Some(disk_type) => self.disk_type = disk_type,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported createType value"
                        ));
                    }
                },
                b"encoding" => match VmdkDescriptorStorage::parse_encoding_value(value) {
                    Some(character_encoding) => self.character_encoding = character_encoding,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported encoding value"
                        ));
                    }
                },
                b"parentcid" => {
                    match VmdkDescriptorStorage::parse_content_identifier_value(value) {
                        Some(value_32bit) => {
                            if value_32bit != 0xffffffff {
                                self.parent_content_identifier = Some(value_32bit);
                            }
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported parentCID value"
                            ));
                        }
                    }
                }
                b"parentfilenamehint" => {
                    match VmdkDescriptorStorage::parse_file_name(line, &self.character_encoding) {
                        Some(file_name) => self.parent_name = Some(file_name),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported parentFileNameHint value"
                            ));
                        }
                    }
                }
                b"version" => match VmdkDescriptorStorage::parse_integer_value(value) {
                    Some(_) => {}
                    None => {
                        return Err(keramics_core::error_trace_new!("Unsupported version value"));
                    }
                },
                _ => {}
            }
        }
        if last_line != b"# extent description" {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid descriptor data - missing extent description section"
            )));
        }
        while let Some(line) = descriptor_storage.next_line() {
            let trimmed_line: &[u8] = VmdkDescriptorStorage::trim(line);
            last_line = VmdkDescriptorStorage::to_ascii_lowercase(trimmed_line);

            if last_line == b"# change tracking file" || last_line == b"# the disk data base" {
                break;
            }
            if last_line.is_empty() || last_line[0] == b'#' {
                continue;
            }
            let extent: VmdkDescriptorExtent =
                match VmdkDescriptorStorage::parse_extent(trimmed_line, &self.character_encoding) {
                    Some(extent) => extent,
                    None => {
                        return Err(keramics_core::error_trace_new!("Unsupported extent value"));
                    }
                };
            match &extent.extent_type {
                VmdkDescriptorExtentType::Flat => match &self.disk_type {
                    VmdkDiskType::Device
                    | VmdkDiskType::DevicePartitioned
                    | VmdkDiskType::Flat2GbExtent
                    | VmdkDiskType::MonolithicFlat => {}
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported extent type"
                        )));
                    }
                },
                VmdkDescriptorExtentType::Sparse => match &self.disk_type {
                    VmdkDiskType::Sparse2GbExtent
                    | VmdkDiskType::MonolithicSparse
                    | VmdkDiskType::StreamOptimized => {}
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported extent type"
                        )));
                    }
                },
                VmdkDescriptorExtentType::VmfsFlat => match &self.disk_type {
                    VmdkDiskType::VmfsFlat
                    | VmdkDiskType::VmfsFlatPreAllocated
                    | VmdkDiskType::VmfsFlatZeroed => {}
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported extent type"
                        )));
                    }
                },
                VmdkDescriptorExtentType::VmfsSparse => match &self.disk_type {
                    VmdkDiskType::VmfsSparse | VmdkDiskType::VmfsSparseThin => {}
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported extent type"
                        )));
                    }
                },
                _ => {}
            }
            self.media_size += extent.number_of_sectors * (self.bytes_per_sector as u64);

            self.extents.push(extent);
        }
        if last_line == b"# change tracking file" {
            while let Some(line) = descriptor_storage.next_line() {
                last_line =
                    VmdkDescriptorStorage::to_ascii_lowercase(VmdkDescriptorStorage::trim(line));

                if last_line == b"# the disk data base" {
                    break;
                }
                if last_line.is_empty() || last_line[0] == b'#' {
                    continue;
                }
                match VmdkDescriptorStorage::parse_key_value_pair(&last_line) {
                    Some(_) => {}
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid descriptor data - unsupported change tracking file key-value pair"
                        ));
                    }
                }
            }
        }
        if last_line != b"# the disk data base" {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid descriptor data - missing disk data base section"
            )));
        }
        while let Some(line) = descriptor_storage.next_line() {
            last_line =
                VmdkDescriptorStorage::to_ascii_lowercase(VmdkDescriptorStorage::trim(line));

            if last_line.is_empty() || last_line[0] == b'#' {
                continue;
            }
            match VmdkDescriptorStorage::parse_key_value_pair(&last_line) {
                Some(_) => {}
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid descriptor data - unsupported disk data base key-value pair"
                    ));
                }
            }
        }
        Ok(())
    }

    /// Reads the extent files.
    pub fn read_extent_files(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        match &self.disk_type {
            VmdkDiskType::Custom
            | VmdkDiskType::Device
            | VmdkDiskType::DevicePartitioned
            | VmdkDiskType::Unknown
            | VmdkDiskType::VmfsRdm
            | VmdkDiskType::VmfsRdmp => {
                return Ok(());
            }
            _ => {}
        }
        let number_of_extents: usize = self.extents.len();

        for (extent_index, extent) in self.extents.iter().enumerate() {
            if extent.extent_type == VmdkDescriptorExtentType::Zero {
                continue;
            }
            let extent_file_name: &ByteString = match extent.file_name.as_ref() {
                Some(file_name) => file_name,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing extent: {} file name",
                        extent_index
                    )));
                }
            };
            // TODO: improve path handling for more complex scenarios.
            let path_components: [PathComponent; 1] = [PathComponent::from(extent_file_name)];

            let result: Option<DataStreamReference> =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => Some(data_stream),
                    Ok(None) => {
                        if number_of_extents != 1
                            || extent.extent_type != VmdkDescriptorExtentType::Sparse
                        {
                            None
                        } else {
                            // Handle a renamed single monolithic sparse or stream optimized image file.
                            let path_components: [PathComponent; 1] = [file_name.clone()];

                            match file_resolver.get_data_stream(&path_components) {
                                Ok(result) => result,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to open extent: {} file: {}",
                                            extent_index, file_name
                                        )
                                    );
                                    return Err(error);
                                }
                            }
                        }
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to open extent: {} file: {}",
                                extent_index, extent_file_name
                            )
                        );
                        return Err(error);
                    }
                };
            let data_stream: DataStreamReference = match result {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing extent: {} data stream: {}",
                        extent_index, extent_file_name
                    )));
                }
            };
            match &extent.extent_type {
                VmdkDescriptorExtentType::Sparse => {
                    let mut sparse_file: VmdkSparseFile = VmdkSparseFile::new();

                    match sparse_file.read_data_stream(&data_stream) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read extent: {} sparse VMDK file", extent_index)
                            );
                            return Err(error);
                        }
                    }
                    if self.disk_type != VmdkDiskType::StreamOptimized {
                        // TODO: check if extent file is compressed
                    }
                    if self.sectors_per_grain == 0 {
                        self.sectors_per_grain = sparse_file.sectors_per_grain;
                        self.compression_method = sparse_file.compression_method.clone();
                    } else if self.sectors_per_grain != sparse_file.sectors_per_grain {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Mismatch in sectors per grain"
                        )));
                    }
                    self.extent_file_cache
                        .insert(extent_index as u64, VmdkExtentFile::SparseVmdk(sparse_file));
                }
                VmdkDescriptorExtentType::VmfsSparse => {
                    let mut sparse_file: VmdkSparseCowdFile = VmdkSparseCowdFile::new();

                    match sparse_file.read_data_stream(&data_stream) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read extent: {} sparse COWD file", extent_index)
                            );
                            return Err(error);
                        }
                    }
                    if self.sectors_per_grain == 0 {
                        self.sectors_per_grain = sparse_file.sectors_per_grain as u64;
                    } else if self.sectors_per_grain != sparse_file.sectors_per_grain as u64 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Mismatch in sectors per grain"
                        )));
                    }
                    self.extent_file_cache
                        .insert(extent_index as u64, VmdkExtentFile::SparseCowd(sparse_file));
                }
                VmdkDescriptorExtentType::Flat | VmdkDescriptorExtentType::VmfsFlat => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported extent: {} type",
                        extent_index
                    )));
                }
            }
        }
        Ok(())
    }

    /// Reads the file header and determines the file type.
    fn read_file_header(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<VmdkFileType, ErrorTrace> {
        let mut data: [u8; 32] = [0; 32];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        if &data[0..4] == VMDK_SPARSE_COWD_FILE_HEADER_SIGNATURE {
            return Ok(VmdkFileType::CowdSparseFile);
        }
        if &data[0..4] == VMDK_SPARSE_FILE_HEADER_SIGNATURE {
            return Ok(VmdkFileType::VmdkSparseFile);
        }
        let lowercase_data: Vec<u8> = data
            .iter()
            .take(21)
            .map(|byte| {
                if *byte >= b'A' && *byte <= b'Z' {
                    *byte + 32
                } else {
                    *byte
                }
            })
            .collect::<Vec<u8>>();

        if &lowercase_data == b"# disk descriptorfile" {
            return Ok(VmdkFileType::DescriptorFile);
        }
        Ok(VmdkFileType::Unknown)
    }
}

impl DataStream for VmdkImage {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        match &self.disk_type {
            VmdkDiskType::Custom
            | VmdkDiskType::Device
            | VmdkDiskType::DevicePartitioned
            | VmdkDiskType::Unknown
            | VmdkDiskType::VmfsRdm
            | VmdkDiskType::VmfsRdmp => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported disk type"
                )));
            }
            _ => {}
        }
        if self.current_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = match self.read_data_from_extents(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from extents");
                return Err(error);
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

    fn get_image() -> Result<VmdkImage, ErrorTrace> {
        let mut image: VmdkImage = VmdkImage::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = VmdkImage::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image.open(&file_resolver, &file_name)?;

        Ok(())
    }

    // TODO: add tests for read_descriptor
    // TODO: add tests for read_extent_files
    // TODO: add tests for read_file_header

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        image.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let size: u64 = image.get_size()?;
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let offset = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let result: Result<u64, ErrorTrace> = image.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;
        image.seek(SeekFrom::Start(1024))?;

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
    fn test_seek_and_read_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut image: VmdkImage = get_image()?;
        image.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
