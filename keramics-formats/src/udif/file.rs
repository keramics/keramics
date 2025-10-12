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

use std::io::SeekFrom;

use keramics_compression::{AdcContext, Bzip2Context, LzfseContext, ZlibContext};
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;
use crate::lru_cache::LruCache;
use crate::plist::{PlistObject, XmlPlist};

use super::block_range::{UdifBlockRange, UdifBlockRangeType};
use super::block_table::UdifBlockTable;
use super::enums::UdifCompressionMethod;
use super::file_footer::UdifFileFooter;

const MAXIMUM_NUMBER_OF_SECTORS: u64 = u64::MAX / 512;

/// Universal Disk Image Format (UDIF) file.
pub struct UdifFile {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Data fork offset.
    data_fork_offset: u64,

    /// Value to indicate the file has block ranges.
    has_block_ranges: bool,

    /// Block tree.
    block_tree: BlockTree<UdifBlockRange>,

    /// Decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Compression method.
    pub compression_method: UdifCompressionMethod,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl UdifFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
            data_fork_offset: 0,
            has_block_ranges: false,
            block_tree: BlockTree::<UdifBlockRange>::new(0, 0, 0),
            block_cache: LruCache::new(64),
            bytes_per_sector: 0,
            compression_method: UdifCompressionMethod::None,
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        self.read_metadata(data_stream)?;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file footer and XML plist.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut file_footer: UdifFileFooter = UdifFileFooter::new();

        file_footer.read_at_position(data_stream, SeekFrom::End(-512))?;

        self.bytes_per_sector = 512;

        let data_fork_end_offset: u64 = file_footer.data_fork_offset + file_footer.data_fork_size;

        if file_footer.plist_size == 0 {
            self.data_fork_offset = file_footer.data_fork_offset;
            self.has_block_ranges = false;
            self.media_size = file_footer.data_fork_size;
        } else {
            if file_footer.plist_size == 0 || file_footer.plist_size > 65536 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported plist data size"
                ));
            }
            let mut data: Vec<u8> = vec![0; file_footer.plist_size as usize];

            keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(file_footer.plist_offset)
            );
            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "Plist data of size: {} at offset: {} (0x{:08x})\n",
                    file_footer.plist_size, file_footer.plist_offset, file_footer.plist_offset,
                ));
                self.mediator.debug_print_data(&data, true);
            }
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
            let blkx_array: &Vec<PlistObject> = match resource_fork_object.get_vector_by_key("blkx")
            {
                Some(string) => string,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unable to retrieve blkx value from plist"
                    ));
                }
            };
            let mut block_ranges: Vec<UdifBlockRange> = Vec::new();
            let mut media_sector: u64 = 0;
            let mut media_offset: u64 = 0;
            let mut compressed_entry_type: u32 = 0;

            for (table_index, blkx_array_entry) in blkx_array.iter().enumerate() {
                let data: &[u8] = match blkx_array_entry.get_bytes_by_key("Data") {
                    Some(data) => data,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve Data value from blkx array entry: {}",
                            table_index
                        )));
                    }
                };
                if self.mediator.debug_output {
                    // TODO: determine data offset relative to start of plist
                    self.mediator.debug_print(format!(
                        "UdifBlockTable data of size: {} at offset: {} (0x{:08x})\n",
                        data.len(),
                        0,
                        0,
                    ));
                    self.mediator.debug_print_data(&data, true);
                }
                let mut block_table = UdifBlockTable::new();
                block_table.read_data(&data)?;

                if block_table.start_sector != media_sector {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported block table: {} start sector value out of bounds",
                        table_index,
                    )));
                }
                for (entry_index, block_table_entry) in block_table.entries.iter().enumerate() {
                    if block_table_entry.entry_type == 0xffffffff {
                        break;
                    }
                    if block_table_entry.entry_type == 0x7ffffffe {
                        continue;
                    }
                    if block_table_entry.start_sector
                        > MAXIMUM_NUMBER_OF_SECTORS - block_table.start_sector
                        || block_table.start_sector + block_table_entry.start_sector != media_sector
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported block table: {} entry: {} start sector value out of bounds",
                            table_index, entry_index,
                        )));
                    }
                    if block_table_entry.number_of_sectors == 0
                        || block_table_entry.number_of_sectors > MAXIMUM_NUMBER_OF_SECTORS
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported block table: {} entry: {} number of sectors value out of bounds",
                            table_index, entry_index,
                        )));
                    }
                    if block_table_entry.data_offset < file_footer.data_fork_offset
                        || block_table_entry.data_offset >= data_fork_end_offset
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported block table: {} entry: {} data offset value out of bounds",
                            table_index, entry_index,
                        )));
                    }
                    if block_table_entry.data_size
                        > data_fork_end_offset - block_table_entry.data_offset
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported block table: {} entry: {} data size value out of bounds",
                            table_index, entry_index,
                        )));
                    }
                    let media_size: u64 =
                        block_table_entry.number_of_sectors * self.bytes_per_sector as u64;

                    let block_range: UdifBlockRange = match block_table_entry.entry_type {
                        0x00000000 | 0x00000002 => UdifBlockRange::new(
                            media_offset,
                            0,
                            media_size,
                            0,
                            UdifBlockRangeType::Sparse,
                        ),
                        0x00000001 => UdifBlockRange::new(
                            media_offset,
                            block_table_entry.data_offset,
                            media_size,
                            0,
                            UdifBlockRangeType::InFile,
                        ),
                        0x80000004..0x80000008 => {
                            if block_table_entry.number_of_sectors > 2048 {
                                return Err(keramics_core::error_trace_new!(format!(
                                    "Unsupported compressed block table: {} entry: {} number of sectors value out of bounds",
                                    table_index, entry_index,
                                )));
                            }
                            if compressed_entry_type == 0 {
                                compressed_entry_type = block_table_entry.entry_type;
                            } else if block_table_entry.entry_type != compressed_entry_type {
                                return Err(keramics_core::error_trace_new!(
                                    "Unsupported mixed compression methods"
                                ));
                            }
                            UdifBlockRange::new(
                                media_offset,
                                block_table_entry.data_offset,
                                media_size,
                                block_table_entry.data_size as u32,
                                UdifBlockRangeType::Compressed,
                            )
                        }
                        _ => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unsupported block table entry type: 0x{:08x}",
                                block_table_entry.entry_type
                            )));
                        }
                    };
                    block_ranges.push(block_range);

                    media_offset += media_size;
                    media_sector += block_table_entry.number_of_sectors;
                }
            }
            self.compression_method = match compressed_entry_type {
                0x80000004 => UdifCompressionMethod::Adc,
                0x80000005 => UdifCompressionMethod::Zlib,
                0x80000006 => UdifCompressionMethod::Bzip2,
                0x80000007 => UdifCompressionMethod::Lzfse,
                0x80000008 => UdifCompressionMethod::Lzma,
                _ => UdifCompressionMethod::None,
            };
            self.has_block_ranges = true;
            self.media_size = media_offset;

            let block_tree_data_size: u64 =
                media_sector.div_ceil(16384) * 16384 * self.bytes_per_sector as u64;

            self.block_tree = BlockTree::<UdifBlockRange>::new(
                block_tree_data_size,
                16384,
                self.bytes_per_sector as u64,
            );
            while let Some(block_range) = block_ranges.pop() {
                match self.block_tree.insert_value(
                    block_range.media_offset,
                    block_range.size,
                    block_range,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            "Unable to insert block range into block tree",
                            error
                        ));
                    }
                };
            }
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let block_tree_value: Option<&UdifBlockRange> = self.block_tree.get_value(media_offset);

            let block_range: &UdifBlockRange = match block_tree_value {
                Some(value) => value,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {}",
                        media_offset
                    )));
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

                        self.read_compressed_block(block_range, &mut data)?;

                        self.block_cache.insert(block_range.data_offset, data);
                    }
                    let range_data: &Vec<u8> = match self.block_cache.get(&block_range.data_offset)
                    {
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
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "Compressed data of size: {} at offset: {} (0x{:08x})\n",
                block_range.compressed_data_size, block_range.data_offset, block_range.data_offset,
            ));
            self.mediator.debug_print_data(&compressed_data, true);
        }
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
                let mut zlib_context: ZlibContext = ZlibContext::new();

                match zlib_context.decompress(&compressed_data, data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to decompress zlib data"
                        );
                        return Err(error);
                    }
                }
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
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.media_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.media_offset;
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
            let data_fork_offset: u64 = self.data_fork_offset + self.media_offset;
            let read_count: usize = keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut buf[0..read_size],
                SeekFrom::Start(data_fork_offset)
            );
            read_count
        };
        self.media_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.media_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.media_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.media_size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.media_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    fn get_file() -> Result<UdifFile, ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 1964032);

        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_metadata(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 1964032);

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
    // TODO: add tests for read_compressed_block

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
    fn test_seek_beyond_media_size() -> Result<(), ErrorTrace> {
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
