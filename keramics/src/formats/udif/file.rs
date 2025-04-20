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

use std::io;
use std::io::{Read, Seek};
use std::sync::Arc;

use crate::compression::{AdcContext, Bzip2Context, LzfseContext, ZlibContext};
use crate::formats::plist::{PlistObject, XmlPlist};
use crate::mediator::{Mediator, MediatorReference};
use crate::types::{BlockTree, LruCache, SharedValue};
use crate::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

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
    data_stream: VfsDataStreamReference,

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
            data_stream: SharedValue::none(),
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

    /// Opens a file.
    pub fn open(&mut self, file_system: &Arc<VfsFileSystem>, path: &VfsPath) -> io::Result<()> {
        self.data_stream = match file_system.get_data_stream_by_path_and_name(path, None)? {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        };
        self.read_metadata()
    }

    /// Reads the file footer and XML plist.
    fn read_metadata(&mut self) -> io::Result<()> {
        let mut file_footer: UdifFileFooter = UdifFileFooter::new();

        file_footer.read_at_position(&self.data_stream, io::SeekFrom::End(-512))?;

        self.bytes_per_sector = 512;

        let data_fork_end_offset: u64 = file_footer.data_fork_offset + file_footer.data_fork_size;

        if file_footer.plist_size == 0 {
            self.data_fork_offset = file_footer.data_fork_offset;
            self.has_block_ranges = false;
            self.media_size = file_footer.data_fork_size;
        } else {
            let string: String = match self.data_stream.with_write_lock() {
                Ok(mut data_stream) => {
                    if file_footer.plist_size == 0 || file_footer.plist_size > 65536 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported plist data size",
                        ));
                    }
                    let mut data: Vec<u8> = vec![0; file_footer.plist_size as usize];

                    data_stream.read_at_position(
                        &mut data,
                        io::SeekFrom::Start(file_footer.plist_offset),
                    )?;

                    if self.mediator.debug_output {
                        self.mediator.debug_print(format!(
                            "Plist data of size: {} at offset: {} (0x{:08x})\n",
                            file_footer.plist_size,
                            file_footer.plist_offset,
                            file_footer.plist_offset,
                        ));
                        self.mediator.debug_print_data(&data, true);
                    }
                    match String::from_utf8(data) {
                        Ok(string) => string,
                        Err(error) => return Err(crate::error_to_io_error!(error)),
                    }
                }
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            let mut xml_plist: XmlPlist = XmlPlist::new();
            xml_plist.parse(string.as_str())?;

            let resource_fork_object: &PlistObject =
                match xml_plist.root_object.get_object_by_key("resource-fork") {
                    Some(string) => string,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unable to retrieve resource-fork value from plist",
                        ));
                    }
                };
            let blkx_array: &Vec<PlistObject> = match resource_fork_object.get_vector_by_key("blkx")
            {
                Some(string) => string,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unable to retrieve blkx value from plist",
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
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unable to retrieve Data value from blkx array entry: {}",
                                table_index
                            ),
                        ));
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
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported block table: {} start sector value out of bounds",
                            table_index,
                        ),
                    ));
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
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported block table: {} entry: {} start sector value out of bounds",
                                table_index,
                                entry_index,
                            ),
                        ));
                    }
                    if block_table_entry.number_of_sectors == 0
                        || block_table_entry.number_of_sectors > MAXIMUM_NUMBER_OF_SECTORS
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported block table: {} entry: {} number of sectors value out of bounds",
                                table_index,
                                entry_index,
                            ),
                        ));
                    }
                    if block_table_entry.data_offset < file_footer.data_fork_offset
                        || block_table_entry.data_offset >= data_fork_end_offset
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported block table: {} entry: {} data offset value out of bounds",
                                table_index,
                                entry_index,
                            ),
                        ));
                    }
                    if block_table_entry.data_size
                        > data_fork_end_offset - block_table_entry.data_offset
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported block table: {} entry: {} data size value out of bounds",
                                table_index,
                                entry_index,
                            ),
                        ));
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
                                return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Unsupported compressed block table: {} entry: {} number of sectors value out of bounds",
                                table_index,
                                entry_index,
                            ),
                        ));
                            }
                            if compressed_entry_type == 0 {
                                compressed_entry_type = block_table_entry.entry_type;
                            } else if block_table_entry.entry_type != compressed_entry_type {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Unsupported mixed compression methods",
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
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Unsupported block table entry type: 0x{:08x}",
                                    block_table_entry.entry_type
                                ),
                            ));
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
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                };
            }
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> io::Result<usize> {
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
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Missing block range for offset: {}", media_offset),
                    ));
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
                UdifBlockRangeType::InFile => match self.data_stream.with_write_lock() {
                    Ok(mut data_stream) => data_stream.read_at_position(
                        &mut data[data_offset..data_end_offset],
                        io::SeekFrom::Start(block_range.data_offset + range_relative_offset),
                    )?,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                },
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
    ) -> io::Result<()> {
        let mut compressed_data: Vec<u8> = vec![0; block_range.compressed_data_size as usize];

        match self.data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(
                &mut compressed_data,
                io::SeekFrom::Start(block_range.data_offset),
            )?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
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
                adc_context.decompress(&compressed_data, data)?;
            }
            UdifCompressionMethod::Bzip2 => {
                let mut bzip2_context: Bzip2Context = Bzip2Context::new();
                bzip2_context.decompress(&compressed_data, data)?;
            }
            UdifCompressionMethod::Lzfse => {
                let mut lzfse_context: LzfseContext = LzfseContext::new();
                lzfse_context.decompress(&compressed_data, data)?;
            }
            UdifCompressionMethod::Lzma => {
                // TODO: add support for UdifCompressionMethod::Lzma,
                todo!();
            }
            UdifCompressionMethod::Zlib => {
                let mut zlib_context: ZlibContext = ZlibContext::new();
                zlib_context.decompress(&compressed_data, data)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unsupported compression method",
                ));
            }
        };
        Ok(())
    }
}

impl Read for UdifFile {
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
        let read_count: usize = if self.has_block_ranges {
            self.read_data_from_blocks(&mut buf[..read_size])?
        } else {
            match self.data_stream.with_write_lock() {
                Ok(mut data_stream) => {
                    let data_fork_offset: u64 = self.data_fork_offset + self.media_offset;

                    data_stream.read_at_position(
                        &mut buf[0..read_size],
                        io::SeekFrom::Start(data_fork_offset),
                    )?
                }
                Err(error) => return Err(crate::error_to_io_error!(error)),
            }
        };
        self.media_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for UdifFile {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsPath};

    fn get_file() -> io::Result<UdifFile> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file: UdifFile = UdifFile::new();

        file.open(&vfs_file_system, &vfs_path)?;

        Ok(file)
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file: UdifFile = UdifFile::new();

        file.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 1964032);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;

        let offset = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;
        file.seek(io::SeekFrom::Start(1024))?;

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
    fn test_seek_and_read_beyond_media_size() -> io::Result<()> {
        let mut file: UdifFile = get_file()?;
        file.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
