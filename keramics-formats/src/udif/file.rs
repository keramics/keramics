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

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u32_be;

use crate::block_tree::BlockTree;
use crate::lru_cache::LruCache;
use crate::plist::{PlistObject, XmlPlist};

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::block_table::UdifBlockTable;
use super::block_table_reader::UdifBlockTableReader;
use super::constants::*;
use super::encrypted_file_footer::UdifEncryptedFileFooter;
use super::encrypted_file_header::UdifEncryptedFileHeader;
use super::enums::UdifCompressionMethod;
use super::file_footer::UdifFileFooter;
use super::resource_fork_header::UdifResourceForkHeader;
use super::resource_map::UdifResourceMap;
use super::resource_map_item::UdifResourceMapItem;

/// Universal Disk Image Format (UDIF) file.
pub struct UdifFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    format_version: u32,

    /// Number of segments.
    number_of_segments: u32,

    /// Data fork offset.
    data_fork_offset: u64,

    /// Value to indicate the file has block ranges.
    has_block_ranges: bool,

    /// Block tree.
    block_tree: BlockTree<UdifBlockRange>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Compression method.
    compression_method: UdifCompressionMethod,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    media_size: u64,
}

impl UdifFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            number_of_segments: 0,
            data_fork_offset: 0,
            has_block_ranges: false,
            block_tree: BlockTree::<UdifBlockRange>::new(0, 0, 0),
            block_cache: LruCache::new(64),
            bytes_per_sector: 0,
            compression_method: UdifCompressionMethod::None,
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

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u32 {
        self.format_version
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        // TODO: read first 8 bytes to check for encrypted v2 header.
        let mut signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut signature,
            SeekFrom::Start(0)
        );
        if &signature == UDIF_ENCRYPTED_FILE_HEADER_SIGNATURE {
            let mut encrypted_file_header: UdifEncryptedFileHeader = UdifEncryptedFileHeader::new();

            match encrypted_file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read encrypted file header"
                    );
                    return Err(error);
                }
            }
            self.format_version = encrypted_file_header.format_version;
            // self.block_size = encrypted_file_header.block_size;
        } else {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut signature,
                SeekFrom::End(-8)
            );
            if &signature == UDIF_ENCRYPTED_FILE_FOOTER_SIGNATURE {
                let mut encrypted_file_footer: UdifEncryptedFileFooter =
                    UdifEncryptedFileFooter::new();

                match encrypted_file_footer.read_at_position(data_stream, SeekFrom::End(-1276)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read encrypted file footer"
                        );
                        return Err(error);
                    }
                }
                self.format_version = encrypted_file_footer.format_version;
                // self.block_size = encrypted_file_footer.block_size;
            } else {
                match self.read_metadata(data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                        return Err(error);
                    }
                }
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file footer and resource fork or XML plist.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut file_footer: UdifFileFooter = UdifFileFooter::new();

        match file_footer.read_at_position(data_stream, SeekFrom::End(-512)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file footer");
                return Err(error);
            }
        }
        self.format_version = file_footer.format_version;
        self.number_of_segments = file_footer.number_of_segments;
        self.bytes_per_sector = 512;

        let data_fork_end_offset: u64 = file_footer.data_fork_offset + file_footer.data_fork_size;

        if file_footer.plist_size == 0 && file_footer.resource_fork_size == 0 {
            self.data_fork_offset = file_footer.data_fork_offset;
            self.has_block_ranges = false;
            self.media_size = file_footer.data_fork_size;
        } else if file_footer.plist_size == 0 {
            let mut resource_fork_header: UdifResourceForkHeader = UdifResourceForkHeader::new();

            match resource_fork_header.read_at_position(
                data_stream,
                SeekFrom::Start(file_footer.resource_fork_offset),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read resource fork header"
                    );
                    return Err(error);
                }
            }
            let offset: u64 = file_footer.resource_fork_offset
                + (resource_fork_header.resource_map_offset as u64);

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
            let mut block_table_reader: UdifBlockTableReader = UdifBlockTableReader::new(
                self.bytes_per_sector,
                file_footer.data_fork_offset,
                file_footer.data_fork_size,
            );
            let mut data: [u8; 4] = [0; 4];

            for blkx_value in blkx_item.values.iter() {
                let offset: u64 = file_footer.resource_fork_offset
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
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to process block table"
                        );
                        return Err(error);
                    }
                }
            }
            self.compression_method = block_table_reader.get_compression_method();
            self.media_size = block_table_reader.get_media_size();

            self.block_tree = match block_table_reader.get_block_tree() {
                Ok(block_tree) => block_tree,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to determine block tree");
                    return Err(error);
                }
            };
            self.has_block_ranges = true;
        } else {
            // Note that 16777216 is an arbitrary chosen limit.
            if file_footer.plist_size > 16777216 {
                return Err(keramics_core::error_trace_new!("Unsupported data size"));
            }
            let mut data: Vec<u8> = vec![0; file_footer.plist_size as usize];

            keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(file_footer.plist_offset)
            );
            keramics_core::debug_trace_data!(
                "UdifFileXmlPlist",
                file_footer.plist_offset,
                &data,
                file_footer.plist_size
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
            let mut block_table_reader: UdifBlockTableReader = UdifBlockTableReader::new(
                self.bytes_per_sector,
                file_footer.data_fork_offset,
                file_footer.data_fork_size,
            );
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

                let mut block_table = UdifBlockTable::new();

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
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to process block table"
                        );
                        return Err(error);
                    }
                }
            }
            self.compression_method = block_table_reader.get_compression_method();
            self.media_size = block_table_reader.get_media_size();

            self.block_tree = match block_table_reader.get_block_tree() {
                Ok(block_tree) => block_tree,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to determine block tree");
                    return Err(error);
                }
            };
            self.has_block_ranges = true;
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
                Ok(Some(value)) => value,
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

            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;
            let range_read_count: usize = match block_range.range_type {
                UdifBlockRangeType::Compressed => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    if !self.block_cache.contains(&block_range.data_offset) {
                        let mut data: Vec<u8> = vec![0; block_range.size as usize];

                        match self.read_compressed_block(block_range, &mut data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to read compressed block"
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

                    range_read_size
                }
                UdifBlockRangeType::InFile => {
                    let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                        Some(data_stream) => data_stream,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing data stream"));
                        }
                    };
                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(block_range.data_offset + range_relative_offset)
                    );
                    read_count
                }
                UdifBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

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

    /// Reads a compressed block range.
    fn read_compressed_block(
        &self,
        block_range: &UdifBlockRange,
        data: &mut Vec<u8>,
    ) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut compressed_data: Vec<u8> = vec![0; block_range.compressed_data_size as usize];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut compressed_data,
            SeekFrom::Start(block_range.data_offset)
        );
        keramics_core::debug_trace_data!(
            "UdifCompressedBlock",
            block_range.data_offset,
            &compressed_data,
            block_range.compressed_data_size
        );
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
        };
        Ok(())
    }
}

impl DataStream for UdifFile {
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
            let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                Some(data_stream) => data_stream,
                None => {
                    return Err(keramics_core::error_trace_new!("Missing data stream"));
                }
            };
            let data_fork_offset: u64 = self.data_fork_offset + self.current_offset;
            let read_count: usize = keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut buf[0..read_size],
                SeekFrom::Start(data_fork_offset)
            );
            read_count
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

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<UdifFile, ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    // TODO add tests for bytes_per_sector
    // TODO add tests for get_compression_method
    // TODO add tests for get_format_version
    // TODO add tests for get_media_size

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 1964032);

        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_metadata(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 1964032);

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
    // TODO: add tests for read_compressed_block

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        file.seek(SeekFrom::Start(1024))?;

        let offset: u64 = file.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let size: u64 = file.get_size()?;
        assert_eq!(size, 1964032);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let offset = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let result: Result<u64, ErrorTrace> = file.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = get_file()?;
        file.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
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
        let mut file: UdifFile = get_file()?;
        file.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
