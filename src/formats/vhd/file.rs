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

use std::cell::RefCell;
use std::io;
use std::io::{Read, Seek};
use std::rc::Rc;
use std::sync::Arc;

use crate::types::{BlockTree, SharedValue, Ucs2String, Uuid};
use crate::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

use super::block_allocation_table::{VhdBlockAllocationTable, VhdBlockAllocationTableEntry};
use super::block_range::{VhdBlockRange, VhdBlockRangeType};
use super::constants::*;
use super::dynamic_disk_header::VhdDynamicDiskHeader;
use super::enums::VhdDiskType;
use super::file_footer::VhdFileFooter;
use super::sector_bitmap::VhdSectorBitmap;

/// Virtual Hard Disk (VHD) file.
pub struct VhdFile {
    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Block allocation table.
    block_allocation_table: Option<VhdBlockAllocationTable>,

    /// Block tree.
    block_tree: BlockTree<VhdBlockRange>,

    /// Disk type.
    pub disk_type: VhdDiskType,

    /// Identifier.
    pub identifier: Uuid,

    /// Parent identifier.
    pub parent_identifier: Option<Uuid>,

    /// Parent name.
    pub parent_name: Option<Ucs2String>,

    /// Parent file.
    parent_file: Option<Rc<RefCell<VhdFile>>>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Block size.
    pub block_size: u32,

    /// Sector bitmap size.
    sector_bitmap_size: u32,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl VhdFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: SharedValue::none(),
            block_allocation_table: None,
            block_tree: BlockTree::<VhdBlockRange>::new(0, 0, 0),
            disk_type: VhdDiskType::Fixed,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
            parent_file: None,
            bytes_per_sector: 0,
            block_size: 0,
            sector_bitmap_size: 0,
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Retrieves the parent filename
    pub fn get_parent_filename(&self) -> Option<Ucs2String> {
        let parent_name: &Ucs2String = match &self.parent_name {
            Some(parent_name) => parent_name,
            None => return None,
        };
        let mut string_index: usize = 0;

        // Look for the last backslash character.
        for (index, character) in parent_name.elements.iter().enumerate().rev() {
            if *character == 0x5c {
                string_index = index + 1;
                break;
            }
        }
        let mut parent_filename: Ucs2String = Ucs2String::new();
        parent_filename.elements = parent_name.elements[string_index..].to_vec();

        Some(parent_filename)
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

    /// Reads the file footer and dynamic block header.
    fn read_metadata(&mut self) -> io::Result<()> {
        let mut file_footer: VhdFileFooter = VhdFileFooter::new();

        file_footer.read_at_position(&self.data_stream, io::SeekFrom::End(-512))?;

        self.disk_type = match file_footer.disk_type {
            VHD_DISK_TYPE_FIXED => VhdDiskType::Fixed,
            VHD_DISK_TYPE_DYNAMIC => VhdDiskType::Dynamic,
            VHD_DISK_TYPE_DIFFERENTIAL => VhdDiskType::Differential,
            _ => VhdDiskType::Unknown,
        };
        self.bytes_per_sector = 512;
        self.media_size = file_footer.data_size;

        if !file_footer.identifier.is_nil() {
            self.identifier = file_footer.identifier;
        }
        if self.disk_type != VhdDiskType::Fixed {
            let mut dynamic_disk_header: VhdDynamicDiskHeader = VhdDynamicDiskHeader::new();

            dynamic_disk_header.read_at_position(
                &self.data_stream,
                io::SeekFrom::Start(file_footer.next_offset),
            )?;
            let block_size: u64 = dynamic_disk_header.block_size as u64;
            let block_tree_data_size: u64 =
                (dynamic_disk_header.number_of_blocks as u64) * block_size;

            if file_footer.data_size > block_tree_data_size {
                let calculated_number_of_blocks: u64 = file_footer.data_size.div_ceil(block_size);
                return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                        format!(
                    "Number of blocks: {} in block allocation table too small for data size: {} ({} blocks)",
                    dynamic_disk_header.number_of_blocks, file_footer.data_size, calculated_number_of_blocks,
                )));
            }
            self.block_size = dynamic_disk_header.block_size;

            if !dynamic_disk_header.parent_identifier.is_nil() {
                self.parent_identifier = Some(dynamic_disk_header.parent_identifier);
                self.parent_name = Some(dynamic_disk_header.parent_name);
            }
            let sectors_ber_block: u32 = dynamic_disk_header.block_size / 512;
            self.sector_bitmap_size = (sectors_ber_block / 8).div_ceil(512) * 512;

            self.block_allocation_table = Some(VhdBlockAllocationTable::new(
                dynamic_disk_header.block_table_offset,
                dynamic_disk_header.number_of_blocks,
            ));
            self.block_tree = BlockTree::<VhdBlockRange>::new(
                block_tree_data_size,
                sectors_ber_block as u64,
                512,
            );
        }
        Ok(())
    }

    /// Reads a specific block allocation entry and fills the block tree.
    fn read_block_allocation_entry(&mut self, block_number: u64) -> io::Result<()> {
        let block_allocation_table: &VhdBlockAllocationTable =
            self.block_allocation_table.as_ref().unwrap();
        let entry: VhdBlockAllocationTableEntry =
            block_allocation_table.read_entry(&self.data_stream, block_number as u32)?;

        if entry.sector_number != 0xffffffff {
            self.read_sector_bitmap(block_number, entry.sector_number)?;
        } else {
            let block_media_offset: u64 = block_number * (self.block_size as u64);

            let block_range: VhdBlockRange = if self.disk_type == VhdDiskType::Dynamic {
                VhdBlockRange::new(
                    block_media_offset,
                    0,
                    self.block_size as u64,
                    VhdBlockRangeType::Sparse,
                )
            } else {
                VhdBlockRange::new(
                    block_media_offset,
                    0,
                    self.block_size as u64,
                    VhdBlockRangeType::InParent,
                )
            };
            match self.block_tree.insert_value(
                block_media_offset,
                self.block_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
        }
        Ok(())
    }

    /// Reads a specific sector bitmap and fills the block tree.
    fn read_sector_bitmap(&mut self, block_number: u64, sector_number: u32) -> io::Result<()> {
        let sector_bitmap_offset: u64 = (sector_number as u64) * (self.bytes_per_sector as u64);

        let mut sector_bitmap: VhdSectorBitmap =
            VhdSectorBitmap::new(self.sector_bitmap_size as usize, self.bytes_per_sector);
        sector_bitmap
            .read_at_position(&self.data_stream, io::SeekFrom::Start(sector_bitmap_offset))?;

        let mut range_media_offset: u64 = block_number * (self.block_size as u64);
        let mut range_data_offset: u64 = sector_bitmap_offset + (self.sector_bitmap_size as u64);

        for bitmap_range in sector_bitmap.ranges.iter() {
            let block_range: VhdBlockRange = if bitmap_range.is_set {
                VhdBlockRange::new(
                    range_media_offset,
                    range_data_offset,
                    bitmap_range.size,
                    VhdBlockRangeType::InFile,
                )
            } else if self.disk_type == VhdDiskType::Dynamic {
                VhdBlockRange::new(
                    range_media_offset,
                    0,
                    bitmap_range.size,
                    VhdBlockRangeType::Sparse,
                )
            } else {
                VhdBlockRange::new(
                    range_media_offset,
                    0,
                    bitmap_range.size,
                    VhdBlockRangeType::InParent,
                )
            };
            match self
                .block_tree
                .insert_value(range_media_offset, bitmap_range.size, block_range)
            {
                Ok(_) => {}
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            range_media_offset += bitmap_range.size;
            range_data_offset += bitmap_range.size;
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> io::Result<usize> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;
        let mut block_number: u64 = media_offset / (self.block_size as u64);

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let mut block_tree_value: Option<&VhdBlockRange> =
                self.block_tree.get_value(media_offset);

            if block_tree_value.is_none() {
                self.read_block_allocation_entry(block_number)?;

                block_tree_value = self.block_tree.get_value(media_offset);
            }
            let block_range: &VhdBlockRange = match block_tree_value {
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
                VhdBlockRangeType::InFile => match self.data_stream.with_write_lock() {
                    Ok(mut data_stream) => data_stream.read_at_position(
                        &mut data[data_offset..data_end_offset],
                        io::SeekFrom::Start(block_range.data_offset + range_relative_offset),
                    )?,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                },
                VhdBlockRangeType::InParent => match &self.parent_file {
                    Some(parent_file) => match parent_file.try_borrow_mut() {
                        Ok(mut file) => {
                            file.seek(io::SeekFrom::Start(media_offset))?;

                            file.read(&mut data[data_offset..data_end_offset])?
                        }
                        Err(error) => return Err(crate::error_to_io_error!(error)),
                    },
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing parent file",
                        ));
                    }
                },
                VhdBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;

            block_number += 1;
        }
        Ok(data_offset)
    }

    /// Sets the parent file.
    pub fn set_parent(&mut self, parent_file: &Rc<RefCell<VhdFile>>) -> io::Result<()> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing parent identifier",
                ))
            }
        };
        match parent_file.try_borrow() {
            Ok(file) => {
                if *parent_identifier != file.identifier {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Parent identifier: {} does not match identifier of parent file: {}",
                            parent_identifier.to_string(),
                            file.identifier.to_string(),
                        ),
                    ));
                }
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        }
        self.parent_file = Some(parent_file.clone());

        Ok(())
    }
}

impl Read for VhdFile {
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
        let read_count: usize = if self.disk_type != VhdDiskType::Fixed {
            self.read_data_from_blocks(&mut buf[..read_size])?
        } else {
            match self.data_stream.with_write_lock() {
                Ok(mut data_stream) => data_stream.read_at_position(
                    &mut buf[0..read_size],
                    io::SeekFrom::Start(self.media_offset),
                )?,
                Err(error) => return Err(crate::error_to_io_error!(error)),
            }
        };
        self.media_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for VhdFile {
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

    fn get_file() -> io::Result<VhdFile> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/vhd/ext2.vhd".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file: VhdFile = VhdFile::new();

        file.open(&vfs_file_system, &vfs_path)?;

        Ok(file)
    }

    #[test]
    fn test_get_parent_filename() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file: VhdFile = VhdFile::new();

        file.open(&vfs_file_system, &vfs_path)?;

        let parent_filename: Option<Ucs2String> = file.get_parent_filename();
        assert_eq!(parent_filename.unwrap().to_string(), "ntfs-parent.vhd");

        Ok(())
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file = VhdFile::new();

        file.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(file.media_size, 4194304);
        assert_eq!(
            file.identifier.to_string(),
            "722fa4e2-59c4-c645-8456-ddb430ac4a19"
        );
        assert_eq!(
            file.parent_identifier.unwrap().to_string(),
            "e7ea9200-8493-954e-a816-9572339be931"
        );
        assert_eq!(
            file.parent_name.unwrap().to_string(),
            "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhd",
        );
        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut file: VhdFile = get_file()?;

        let offset = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut file: VhdFile = get_file()?;
        file.seek(io::SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x04, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x58, 0x0f,
            0x00, 0x00, 0xf0, 0x03, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0xa3, 0x7b, 0xf9, 0x60, 0xa4, 0x7b, 0xf9, 0x60, 0x01, 0x00, 0xff, 0xff,
            0x53, 0xef, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0xa3, 0x7b, 0xf9, 0x60, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0b, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0xf0, 0x00, 0x50, 0xbb, 0x07, 0xee, 0x46, 0xa3,
            0x83, 0xa6, 0xa4, 0x05, 0xee, 0x0d, 0xb5, 0x1f, 0x65, 0x78, 0x74, 0x32, 0x5f, 0x74,
            0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x6d, 0x6e, 0x74,
            0x2f, 0x64, 0x66, 0x76, 0x66, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa6, 0x0d,
            0x1b, 0x0a, 0x17, 0xa0, 0x4e, 0x3e, 0x8a, 0x1f, 0x7f, 0x4f, 0x89, 0x7e, 0x46, 0x4e,
            0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa3, 0x7b,
            0xf9, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00,
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
        let mut file: VhdFile = get_file()?;
        file.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
