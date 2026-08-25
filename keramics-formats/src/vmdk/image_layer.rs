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
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::block_reader::VmdkBlockReader;
use super::block_stream::VmdkBlockStream;
use super::constants::*;
use super::descriptor_extent::VmdkDescriptorExtent;
use super::descriptor_storage::VmdkDescriptorStorage;
use super::enums::{VmdkCompressionMethod, VmdkDescriptorExtentType, VmdkDiskType, VmdkFileType};
use super::sparse_cowd_file::VmdkSparseCowdFile;
use super::sparse_file::VmdkSparseFile;
use super::sparse_file_header::VmdkSparseFileHeader;

/// VMware Virtual Disk (VMDK) image layer.
pub struct VmdkImageLayer {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Disk type.
    disk_type: VmdkDiskType,

    /// Sectors per grain.
    sectors_per_grain: u64,

    /// Bytes per sector.
    pub(super) bytes_per_sector: u16,

    /// Compression method.
    compression_method: VmdkCompressionMethod,

    /// Content identifier.
    content_identifier: u32,

    /// Parent content identifier.
    pub(super) parent_content_identifier: Option<u32>,

    /// Parent name.
    pub(super) parent_name: Option<ByteString>,

    /// Extents.
    extents: Vec<VmdkDescriptorExtent>,

    /// Parent layer.
    parent_layer: Option<Arc<VmdkImageLayer>>,

    /// Media size.
    pub(super) media_size: u64,
}

impl VmdkImageLayer {
    /// Creates a new image layer.
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
            parent_layer: None,
            media_size: 0,
        }
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        let parent_data_stream: Option<DataStreamReference> = match &self.parent_layer {
            Some(parent_layer) => Some(parent_layer.get_data_stream()),
            None => None,
        };
        Arc::new(RwLock::new(VmdkBlockStream::new(VmdkBlockReader::new(
            &self.file_resolver,
            self.bytes_per_sector,
            &self.extents,
            parent_data_stream,
            self.media_size,
        ))))
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the compression method.
    pub fn get_compression_method(&self) -> &VmdkCompressionMethod {
        &self.compression_method
    }

    /// Retrieves the content identifier.
    pub fn get_content_identifier(&self) -> u32 {
        self.content_identifier
    }

    /// Retrieves the disk type.
    pub fn get_disk_type(&self) -> &VmdkDiskType {
        &self.disk_type
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Retrieves the parent content identifier.
    pub fn get_parent_content_identifier(&self) -> Option<u32> {
        self.parent_content_identifier
    }

    /// Retrieves the parent name.
    pub fn get_parent_name(&self) -> Option<&ByteString> {
        self.parent_name.as_ref()
    }

    /// Retrieves the sectors per grain.
    pub fn get_sectors_per_grain(&self) -> u64 {
        self.sectors_per_grain
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
        let mut media_start_sector: u64 = 0;

        while let Some(line) = descriptor_storage.next_line() {
            let trimmed_line: &[u8] = VmdkDescriptorStorage::trim(line);
            last_line = VmdkDescriptorStorage::to_ascii_lowercase(trimmed_line);

            if last_line == b"# change tracking file" || last_line == b"# the disk data base" {
                break;
            }
            if last_line.is_empty() || last_line[0] == b'#' {
                continue;
            }
            let mut extent: VmdkDescriptorExtent =
                match VmdkDescriptorStorage::parse_extent(trimmed_line, &self.character_encoding) {
                    Some(extent) => extent,
                    None => {
                        return Err(keramics_core::error_trace_new!("Unsupported extent value"));
                    }
                };
            extent.media_start_sector = media_start_sector;
            media_start_sector += extent.number_of_sectors;
            extent.media_end_sector = media_start_sector;

            keramics_core::debug_trace_structure!(format!("{:#?}", extent));

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
        // TODO: check if extents align

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
                    // TODO: compare file media size with size of extent
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
                    // TODO: compare file media size with size of extent
                }
                VmdkDescriptorExtentType::Flat | VmdkDescriptorExtentType::VmfsFlat => {
                    // TODO: compare file size with size of extent
                }
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

    /// Sets the parent layer.
    pub fn set_parent(&mut self, parent_layer: &Arc<VmdkImageLayer>) -> Result<(), ErrorTrace> {
        let parent_content_identifier: &u32 = match &self.parent_content_identifier {
            Some(content_identifier) => content_identifier,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent content identifier"
                ));
            }
        };
        if parent_content_identifier != &parent_layer.content_identifier {
            return Err(keramics_core::error_trace_new!(format!(
                "Parent content identifier: {} does not match content identifier of parent layer: {}",
                parent_content_identifier, parent_layer.content_identifier,
            )));
        }
        self.parent_layer = Some(parent_layer.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;

    use crate::tests::get_test_data_path;

    fn get_image_layer() -> Result<VmdkImageLayer, ErrorTrace> {
        let mut image_layer: VmdkImageLayer = VmdkImageLayer::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image_layer.open(&file_resolver, &file_name)?;

        Ok(image_layer)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image_layer: VmdkImageLayer = get_image_layer()?;

        let bytes_per_sector: u16 = image_layer.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for get_compression_method

    #[test]
    fn test_get_content_identifier() -> Result<(), ErrorTrace> {
        let image_layer: VmdkImageLayer = get_image_layer()?;

        let content_identifier: u32 = image_layer.get_content_identifier();
        assert_eq!(content_identifier, 0x4c069322);

        Ok(())
    }

    // TODO: add tests for get_disk_type
    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let image_layer: VmdkImageLayer = get_image_layer()?;

        let media_size: u64 = image_layer.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_parent_content_identifier() -> Result<(), ErrorTrace> {
        let image_layer: VmdkImageLayer = get_image_layer()?;

        let parent_content_identifier: Option<u32> = image_layer.get_parent_content_identifier();
        assert_eq!(parent_content_identifier, None);

        Ok(())
    }

    // TODO: add tests for get_parent_name

    #[test]
    fn test_get_sectors_per_grain() -> Result<(), ErrorTrace> {
        let image_layer: VmdkImageLayer = get_image_layer()?;

        let sectors_per_grain: u64 = image_layer.get_sectors_per_grain();
        assert_eq!(sectors_per_grain, 128);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image_layer: VmdkImageLayer = VmdkImageLayer::new();

        let path_string: String = get_test_data_path("vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.vmdk");
        image_layer.open(&file_resolver, &file_name)?;

        Ok(())
    }

    // TODO: add tests for read_descriptor
    // TODO: add tests for read_extent_files
    // TODO: add tests for read_file_header
    // TODO: add tests for set_parent
}
