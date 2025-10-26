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
use std::sync::{Arc, RwLock};

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::{Ucs2String, Uuid};

use crate::block_tree::BlockTree;

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
    data_stream: Option<DataStreamReference>,

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
    parent_file: Option<Arc<RwLock<VhdFile>>>,

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
            data_stream: None,
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

    /// Retrieves the parent file name
    pub fn get_parent_file_name(&self) -> Option<Ucs2String> {
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
        let mut parent_file_name: Ucs2String = Ucs2String::new();
        parent_file_name.elements = parent_name.elements[string_index..].to_vec();

        Some(parent_file_name)
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_metadata(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file footer and dynamic block header.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut file_footer: VhdFileFooter = VhdFileFooter::new();

        match file_footer.read_at_position(data_stream, SeekFrom::End(-512)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file footer");
                return Err(error);
            }
        }
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

            match dynamic_disk_header
                .read_at_position(data_stream, SeekFrom::Start(file_footer.next_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read dynamic disk header"
                    );
                    return Err(error);
                }
            }
            let block_size: u64 = dynamic_disk_header.block_size as u64;
            let block_tree_data_size: u64 =
                (dynamic_disk_header.number_of_blocks as u64) * block_size;

            if file_footer.data_size > block_tree_data_size {
                let calculated_number_of_blocks: u64 = file_footer.data_size.div_ceil(block_size);
                return Err(keramics_core::error_trace_new!(format!(
                    "Number of blocks: {} in block allocation table too small for data size: {} ({} blocks)",
                    dynamic_disk_header.number_of_blocks,
                    file_footer.data_size,
                    calculated_number_of_blocks,
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
    fn read_block_allocation_entry(&mut self, block_number: u64) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let block_allocation_table: &VhdBlockAllocationTable =
            self.block_allocation_table.as_ref().unwrap();
        let entry: VhdBlockAllocationTableEntry =
            match block_allocation_table.read_entry(data_stream, block_number as u32) {
                Ok(entry) => entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read block allocation table entry"
                    );
                    return Err(error);
                }
            };
        if entry.sector_number != 0xffffffff {
            match self.read_sector_bitmap(block_number, entry.sector_number) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read sector bitmap");
                    return Err(error);
                }
            }
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
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
        }
        Ok(())
    }

    /// Reads a specific sector bitmap and fills the block tree.
    fn read_sector_bitmap(
        &mut self,
        block_number: u64,
        sector_number: u32,
    ) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let sector_bitmap_offset: u64 = (sector_number as u64) * (self.bytes_per_sector as u64);

        let mut sector_bitmap: VhdSectorBitmap =
            VhdSectorBitmap::new(self.sector_bitmap_size as usize, self.bytes_per_sector);

        match sector_bitmap.read_at_position(data_stream, SeekFrom::Start(sector_bitmap_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read sector bitmap");
                return Err(error);
            }
        }
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
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
            range_media_offset += bitmap_range.size;
            range_data_offset += bitmap_range.size;
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
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
                match self.read_block_allocation_entry(block_number) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read block allocation entry"
                        );
                        return Err(error);
                    }
                }
                block_tree_value = self.block_tree.get_value(media_offset);
            }
            let block_range: &VhdBlockRange = match block_tree_value {
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
                VhdBlockRangeType::InFile => {
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
                VhdBlockRangeType::InParent => {
                    let parent_file: &Arc<RwLock<VhdFile>> = match self.parent_file.as_ref() {
                        Some(parent_file) => parent_file,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing parent file"));
                        }
                    };
                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        parent_file,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(media_offset)
                    );
                    read_count
                }
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
    pub fn set_parent(&mut self, parent_file: &Arc<RwLock<VhdFile>>) -> Result<(), ErrorTrace> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(keramics_core::error_trace_new!("Missing parent identifier"));
            }
        };
        match parent_file.read() {
            Ok(file) => {
                if *parent_identifier != file.identifier {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Parent identifier: {} does not match identifier of parent file: {}",
                        parent_identifier.to_string(),
                        file.identifier.to_string(),
                    )));
                }
            }
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on parent file",
                    error
                ));
            }
        }
        self.parent_file = Some(parent_file.clone());

        Ok(())
    }
}

impl DataStream for VhdFile {
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
        let read_count: usize = if self.disk_type != VhdDiskType::Fixed {
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
            let read_count: usize = keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut buf[0..read_size],
                SeekFrom::Start(self.media_offset)
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

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<VhdFile, ErrorTrace> {
        let mut file: VhdFile = VhdFile::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("vhd/ext2.vhd").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_parent_file_name() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = VhdFile::new();

        let path_buf: PathBuf =
            PathBuf::from(get_test_data_path("vhd/ntfs-differential.vhd").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let parent_file_name: Option<Ucs2String> = file.get_parent_file_name();
        assert_eq!(parent_file_name, Some(Ucs2String::from("ntfs-parent.vhd")));

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file = VhdFile::new();

        let path_buf: PathBuf =
            PathBuf::from(get_test_data_path("vhd/ntfs-differential.vhd").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

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
            file.parent_name,
            Some(Ucs2String::from(
                "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhd"
            ))
        );
        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file = VhdFile::new();

        let path_buf: PathBuf =
            PathBuf::from(get_test_data_path("vhd/ntfs-differential.vhd").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_metadata(&data_stream)?;

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
            file.parent_name,
            Some(Ucs2String::from(
                "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhd",
            ))
        );
        Ok(())
    }

    // TODO: add test for read_block_allocation_entry
    // TODO: add test for read_sector_bitmap
    // TODO: add test for read_data_from_blocks
    // TODO: add test for set_parent

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = get_file()?;

        let offset = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = get_file()?;
        file.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
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
        let mut file: VhdFile = get_file()?;
        file.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
