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

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext};
use keramics_core::{DataStream, ErrorTrace};
use keramics_types::Uuid;

use crate::block_tree::BlockTree;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::block_table_reader::UdifBlockTableReader;
use super::credential::UdifCredential;
use super::encryption_type::UdifEncryptionType;
use super::enums::UdifCompressionMethod;
use super::segment_stream::UdifSegmentStream;

/// Universal Disk Image Format (UDIF) file.
pub struct UdifImage {
    /// The segment (data) stream.
    segment_stream: Arc<RwLock<UdifSegmentStream>>,

    /// Segment file set identifier.
    segment_set_identifier: Uuid,

    /// Number of segments.
    number_of_segments: u32,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Value to indicate the image has block ranges.
    has_block_ranges: bool,

    /// Block tree.
    block_tree: BlockTree<UdifBlockRange>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Compression method.
    compression_method: UdifCompressionMethod,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,

    /// Encryption type.
    encryption_type: Option<UdifEncryptionType>,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    media_size: u64,
}

impl UdifImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            segment_stream: Arc::new(RwLock::new(UdifSegmentStream::new())),
            segment_set_identifier: Uuid::new(),
            number_of_segments: 0,
            bytes_per_sector: 0,
            has_block_ranges: false,
            block_tree: BlockTree::<UdifBlockRange>::new(0, 0, 0),
            block_cache: LruCache::new(64),
            compression_method: UdifCompressionMethod::None,
            is_locked: false,
            encryption_type: None,
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
    pub fn get_encryption_type(&self) -> Option<&UdifEncryptionType> {
        self.encryption_type.as_ref()
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> Option<u64> {
        if self.is_locked {
            None
        } else {
            Some(self.media_size)
        }
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
        match self.segment_stream.write() {
            Ok(mut segment_stream) => {
                match segment_stream.open(&file_resolver, file_name) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open segment stream"
                        );
                        return Err(error);
                    }
                }
                self.bytes_per_sector = 512;
                self.segment_set_identifier = segment_stream.segment_set_identifier.clone();
                self.number_of_segments = segment_stream.number_of_segments;
                let mut block_table_reader: UdifBlockTableReader =
                    match segment_stream.read_metadata(self.bytes_per_sector) {
                        Ok(block_table_reader) => block_table_reader,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                            return Err(error);
                        }
                    };
                self.has_block_ranges = block_table_reader.has_block_ranges();

                if self.has_block_ranges {
                    self.block_tree = match block_table_reader.get_block_tree() {
                        Ok(block_tree) => block_tree,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to determine block tree"
                            );
                            return Err(error);
                        }
                    };
                }
                self.media_size = block_table_reader.get_media_size();
                self.compression_method = block_table_reader.get_compression_method();
                self.is_locked = segment_stream.is_locked;

                if self.is_locked {
                    self.encryption_type = Some(segment_stream.encryption_type.clone());
                } else {
                    if self.media_size
                        > (segment_stream.number_of_sectors * (self.bytes_per_sector as u64))
                    {
                        return Err(keramics_core::error_trace_new!(
                            "Number of sectors value out of bounds",
                        ));
                    }
                }
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on segment stream",
                    error
                ));
            }
        }
        Ok(())
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
        let mut media_offset: u64 = self.current_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let block_range: &UdifBlockRange = match self.block_tree.get_value(media_offset) {
                Ok(Some(block_range)) => block_range,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {} (0x{:08x})",
                        media_offset, media_offset
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                            media_offset, media_offset
                        )
                    );
                    return Err(error);
                }
            };
            let range_relative_offset: u64 = media_offset - block_range.media_offset;
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

                        keramics_core::data_stream_read_exact_at_position!(
                            &self.segment_stream,
                            &mut compressed_data,
                            SeekFrom::Start(block_range.data_offset),
                        );
                        keramics_core::debug_trace_data!(
                            "UdifCompressedBlock",
                            block_range.data_offset,
                            &compressed_data,
                            block_range.compressed_data_size
                        );
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
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);
                }
                UdifBlockRangeType::InFile => {
                    let range_data_offset: u64 = block_range.data_offset + range_relative_offset;

                    keramics_core::data_stream_read_exact_at_position!(
                        &self.segment_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(range_data_offset),
                    );
                }
                UdifBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);
                }
            }
            data_offset += range_read_size;
            media_offset += range_read_size as u64;
        }
        Ok(data_offset)
    }

    /// Unlocks a locked (encrypted) volume.
    pub fn unlock(&mut self, credentials: &[UdifCredential]) -> Result<bool, ErrorTrace> {
        match self.segment_stream.write() {
            Ok(mut segment_stream) => match segment_stream
                .unlock(self.bytes_per_sector, credentials)
            {
                Ok(true) => {
                    self.segment_set_identifier = segment_stream.segment_set_identifier.clone();
                    self.number_of_segments = segment_stream.number_of_segments;

                    let mut block_table_reader: UdifBlockTableReader = match segment_stream
                        .read_metadata(self.bytes_per_sector)
                    {
                        Ok(block_table_reader) => block_table_reader,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                            return Err(error);
                        }
                    };
                    self.has_block_ranges = block_table_reader.has_block_ranges();

                    if self.has_block_ranges {
                        self.block_tree = match block_table_reader.get_block_tree() {
                            Ok(block_tree) => block_tree,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to determine block tree"
                                );
                                return Err(error);
                            }
                        };
                    }
                    self.media_size = block_table_reader.get_media_size();
                    self.compression_method = block_table_reader.get_compression_method();

                    if self.media_size
                        > (segment_stream.number_of_sectors * (self.bytes_per_sector as u64))
                    {
                        return Err(keramics_core::error_trace_new!(
                            "Number of sectors value out of bounds",
                        ));
                    }
                    self.is_locked = false;
                }
                Ok(false) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock segment stream");
                    return Err(error);
                }
            },
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on segment stream",
                    error
                ));
            }
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
        let read_count: usize = if self.has_block_ranges {
            match self.read_data_from_blocks(&mut buf[..read_size]) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                    return Err(error);
                }
            }
        } else {
            keramics_core::data_stream_read_at_position!(
                &self.segment_stream,
                &mut buf[..read_size],
                SeekFrom::Start(self.current_offset)
            )
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

    fn get_image() -> Result<UdifImage, ErrorTrace> {
        let mut image: UdifImage = UdifImage::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("hfsplus_zlib_segments.dmg");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image()?;

        let bytes_per_sector: u16 = image.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_compression_method() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image()?;

        let compression_method: &UdifCompressionMethod = image.get_compression_method();
        assert_eq!(compression_method, &UdifCompressionMethod::Zlib);

        Ok(())
    }

    // TODO: add tests for get_encryption_type

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let image: UdifImage = get_image()?;

        let media_size: Option<u64> = image.get_media_size();
        assert_eq!(media_size, Some(1964032));

        Ok(())
    }

    // TODO: add tests for get_number_of_segments
    // TODO: add tests for get_segment_set_identifier

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

    // TODO: add tests for read_data_from_blocks

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        image.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let size: u64 = image.get_size()?;
        assert_eq!(size, 1964032);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let offset = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let result: Result<u64, ErrorTrace> = image.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image: UdifImage = get_image()?;
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
        let mut image: UdifImage = get_image()?;
        image.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
