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
use keramics_types::Uuid;

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionType};
use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::path_component::PathComponent;

use super::block_range::UdifBlockRange;
use super::block_reader::UdifBlockReader;
use super::block_stream::UdifBlockStream;
use super::block_table_reader::UdifBlockTableReader;
use super::enums::UdifCompressionMethod;
use super::file::UdifFile;
use super::segment_file::UdifSegmentFile;
use super::segment_range::UdifSegmentRange;
use super::segments_block_reader::UdifSegmentsBlockReader;
use super::segments_block_stream::UdifSegmentsBlockStream;

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

    /// Compression method.
    compression_method: UdifCompressionMethod,

    /// Encryption type.
    encryption_type: Option<CdsaEncrEncryptionType>,

    /// Credentials.
    credentials: Vec<CdsaEncrCredential>,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,

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
            compression_method: UdifCompressionMethod::None,
            encryption_type: None,
            credentials: Vec::new(),
            is_locked: false,
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

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.segments_data_stream {
            Some(data_stream) => {
                if self.block_ranges.is_empty() {
                    Some(data_stream.clone())
                } else {
                    Some(Arc::new(RwLock::new(UdifBlockStream::new(
                        UdifBlockReader::new(
                            data_stream,
                            &self.block_ranges,
                            &self.compression_method,
                            self.media_size,
                        ),
                    ))))
                }
            }
            None => None,
        }
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
            self.segments_data_stream = Some(Arc::new(RwLock::new(UdifSegmentsBlockStream::new(
                UdifSegmentsBlockReader::new(
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
        let segment_file_name: String = UdifSegmentFile::get_file_name(&self.name, segment_number);

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

        let segment_file_name: String = UdifSegmentFile::get_file_name(&self.name, segment_number);

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
            self.segments_data_stream = Some(Arc::new(RwLock::new(UdifSegmentsBlockStream::new(
                UdifSegmentsBlockReader::new(
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
}
