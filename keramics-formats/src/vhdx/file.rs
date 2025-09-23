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
use std::sync::{Arc, RwLock};

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference};
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le, Ucs2String, Uuid};

use crate::block_tree::BlockTree;

use super::block_allocation_table::{VhdxBlockAllocationTable, VhdxBlockAllocationTableEntry};
use super::block_range::{VhdxBlockRange, VhdxBlockRangeType};
use super::constants::*;
use super::enums::VhdxDiskType;
use super::file_header::VhdxFileHeader;
use super::image_header::VhdxImageHeader;
use super::metadata_table::VhdxMetadataTable;
use super::parent_locator::VhdxParentLocator;
use super::region_table::VhdxRegionTable;
use super::region_table_entry::VhdxRegionTableEntry;
use super::sector_bitmap::VhdxSectorBitmap;

/// Virtual Hard Disk version 2 (VHDX) file.
pub struct VhdxFile {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    pub format_version: u16,

    /// Block allocation table.
    block_allocation_table: Option<VhdxBlockAllocationTable>,

    /// Block tree.
    block_tree: BlockTree<VhdxBlockRange>,

    /// Disk type.
    pub disk_type: VhdxDiskType,

    /// Identifier.
    pub identifier: Uuid,

    /// Parent identifier.
    pub parent_identifier: Option<Uuid>,

    /// Parent name.
    pub parent_name: Option<Ucs2String>,

    /// Parent file.
    parent_file: Option<Arc<RwLock<VhdxFile>>>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Block size.
    pub block_size: u32,

    /// Number of entries per chunk;
    entries_per_chunk: u64,

    /// Sector bitmap size.
    sector_bitmap_size: u32,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl VhdxFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
            format_version: 0,
            block_allocation_table: None,
            block_tree: BlockTree::<VhdxBlockRange>::new(0, 0, 0),
            disk_type: VhdxDiskType::Fixed,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
            parent_file: None,
            bytes_per_sector: 0,
            block_size: 0,
            entries_per_chunk: 0,
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
    pub fn read_data_stream(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        self.read_metadata(data_stream)?;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file header, image headers and region tables.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        let mut file_header: VhdxFileHeader = VhdxFileHeader::new();
        file_header.read_at_position(data_stream, io::SeekFrom::Start(0))?;

        let mut primary_image_header: VhdxImageHeader = VhdxImageHeader::new();

        primary_image_header.read_at_position(data_stream, io::SeekFrom::Start(65536))?;

        let mut secondary_image_header: VhdxImageHeader = VhdxImageHeader::new();

        secondary_image_header.read_at_position(data_stream, io::SeekFrom::Start(2 * 65536))?;

        if primary_image_header.sequence_number > secondary_image_header.sequence_number {
            self.identifier = primary_image_header.data_write_identifier;
            self.format_version = primary_image_header.format_version;
        } else {
            self.identifier = secondary_image_header.data_write_identifier;
            self.format_version = secondary_image_header.format_version;
        }
        let mut primary_region_table: VhdxRegionTable = VhdxRegionTable::new();

        primary_region_table.read_at_position(data_stream, io::SeekFrom::Start(3 * 65536))?;

        let mut secondary_region_table: VhdxRegionTable = VhdxRegionTable::new();

        secondary_region_table.read_at_position(data_stream, io::SeekFrom::Start(4 * 65536))?;

        // TODO: compare primary region table with secondary

        let metadata_region: &VhdxRegionTableEntry = match primary_region_table
            .entries
            .get(&VHDX_METADATA_REGION_IDENTIFIER)
        {
            Some(region_table_entry) => {
                if region_table_entry.data_size < 65536 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported metadata region size: {}",
                            region_table_entry.data_size
                        ),
                    ));
                }
                region_table_entry
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing metadata region",
                ))
            }
        };
        self.read_metadata_values(data_stream, metadata_region)?;

        let block_allocation_table_region: &VhdxRegionTableEntry = match primary_region_table
            .entries
            .get(&VHDX_BLOCK_ALLOCATION_TABLE_REGION_IDENTIFIER)
        {
            Some(region_table_entry) => region_table_entry,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing block allocation table region",
                ))
            }
        };
        self.entries_per_chunk =
            ((1 << 23) * (self.bytes_per_sector as u64)) / (self.block_size as u64);
        self.sector_bitmap_size = 1048576 / (self.entries_per_chunk as u32);

        let number_of_entries: u32 = block_allocation_table_region.data_size / 8;

        self.block_allocation_table = Some(VhdxBlockAllocationTable::new(
            block_allocation_table_region.data_offset,
            number_of_entries,
        ));
        let block_tree_data_size: u64 = (number_of_entries as u64) * (self.block_size as u64);
        let sectors_per_block: u32 = self.block_size / (self.bytes_per_sector as u32);

        if self.media_size > block_tree_data_size {
            let calculated_number_of_blocks: u64 = self.media_size.div_ceil(self.block_size as u64);
            return Err(io::Error::new(
            io::ErrorKind::InvalidData,
                    format!(
                "Number of blocks: {} in block allocation table too small for virtual disk size: {} ({} blocks)",
                number_of_entries, self.media_size, calculated_number_of_blocks,
            )));
        }
        self.block_tree = BlockTree::<VhdxBlockRange>::new(
            block_tree_data_size,
            sectors_per_block as u64,
            self.bytes_per_sector as u64,
        );
        Ok(())
    }

    /// Reads the metadata values.
    fn read_metadata_values(
        &mut self,
        data_stream: &DataStreamReference,
        metadata_region: &VhdxRegionTableEntry,
    ) -> io::Result<()> {
        let mut metadata_table: VhdxMetadataTable = VhdxMetadataTable::new();

        metadata_table.read_at_position(
            data_stream,
            io::SeekFrom::Start(metadata_region.data_offset),
        )?;
        if self.mediator.debug_output {
            self.mediator
                .debug_print(format!("VhdxMetadataValues {{\n"));
        }
        match metadata_table
            .entries
            .get(&VHDX_FILE_PARAMETERS_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported file parameters metadata item size: {}",
                            metadata_table_entry.item_size
                        ),
                    ));
                }
                let mut data: [u8; 8] = [0; 8];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match data_stream.write() {
                    Ok(mut data_stream) => data_stream
                        .read_at_position(&mut data, io::SeekFrom::Start(metadata_item_offset))?,
                    Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                };
                let file_parameters_flags: u32 = bytes_to_u32_le!(data, 4);

                self.block_size = bytes_to_u32_le!(data, 0);
                self.disk_type = match file_parameters_flags & 0x00000003 {
                    0 => VhdxDiskType::Fixed,
                    1 => VhdxDiskType::Dynamic,
                    2 => VhdxDiskType::Differential,
                    _ => VhdxDiskType::Unknown,
                };
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "    file_parameters_block_size: {},\n",
                        self.block_size
                    ));
                    self.mediator.debug_print(format!(
                        "    file_parameters_flags: 0x{:08x},\n",
                        file_parameters_flags
                    ));
                }
                if self.block_size < 1024 * 1024 || self.block_size > 256 * 1024 * 1024 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid block size: {} value out of bounds",
                            self.block_size
                        ),
                    ));
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing file parameters metadata item",
                ))
            }
        };
        match metadata_table
            .entries
            .get(&VHDX_VIRTUAL_DISK_SIZE_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported virtual disk size metadata item size: {}",
                            metadata_table_entry.item_size
                        ),
                    ));
                }
                let mut data: [u8; 8] = [0; 8];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match data_stream.write() {
                    Ok(mut data_stream) => data_stream
                        .read_at_position(&mut data, io::SeekFrom::Start(metadata_item_offset))?,
                    Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                };
                self.media_size = bytes_to_u64_le!(data, 0);

                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    virtual_disk_size: {},\n", self.media_size));
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing virtual disk size metadata item",
                ))
            }
        };
        match metadata_table
            .entries
            .get(&VHDX_LOGICAL_SECTOR_SIZE_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported logical sector size metadata item size: {}",
                            metadata_table_entry.item_size
                        ),
                    ));
                }
                let mut data: [u8; 4] = [0; 4];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match data_stream.write() {
                    Ok(mut data_stream) => data_stream
                        .read_at_position(&mut data, io::SeekFrom::Start(metadata_item_offset))?,
                    Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                };
                let logical_sector_size: u32 = bytes_to_u32_le!(data, 0);

                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "    logical_sector_size: {},\n",
                        logical_sector_size
                    ));
                }
                if logical_sector_size != 512 && logical_sector_size != 4096 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid logical sector size: {} value out of bounds",
                            logical_sector_size
                        ),
                    ));
                }
                self.bytes_per_sector = logical_sector_size as u16;
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing logical sector size metadata item",
                ))
            }
        };
        match metadata_table
            .entries
            .get(&VHDX_PHYSICAL_SECTOR_SIZE_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported physical sector size metadata item size: {}",
                            metadata_table_entry.item_size
                        ),
                    ));
                }
                let mut data: [u8; 4] = [0; 4];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match data_stream.write() {
                    Ok(mut data_stream) => data_stream
                        .read_at_position(&mut data, io::SeekFrom::Start(metadata_item_offset))?,
                    Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                };
                let physical_sector_size: u32 = bytes_to_u32_le!(data, 0);

                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "    physical_sector_size: {},\n",
                        physical_sector_size
                    ));
                }
                if physical_sector_size != 512 && physical_sector_size != 4096 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid physical sector size: {} value out of bounds",
                            physical_sector_size
                        ),
                    ));
                }
            }
            None => {}
        };
        match metadata_table
            .entries
            .get(&VHDX_VIRTUAL_DISK_IDENTIFIER_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 16 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported virtual disk identifier metadata item size: {}",
                            metadata_table_entry.item_size
                        ),
                    ));
                }
                let mut data: [u8; 16] = [0; 16];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match data_stream.write() {
                    Ok(mut data_stream) => data_stream
                        .read_at_position(&mut data, io::SeekFrom::Start(metadata_item_offset))?,
                    Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                };
                let virtual_disk_identifier: Uuid = Uuid::from_le_bytes(&data);

                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "    virtual_disk_identifier: {},\n",
                        virtual_disk_identifier.to_string()
                    ));
                }
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing virtual disk identifier metadata item",
                ))
            }
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("}}\n\n"));
        }
        match metadata_table
            .entries
            .get(&VHDX_PARENT_LOCATOR_METADATA_IDENTIFIER)
        {
            Some(metadata_table_entry) => {
                let mut parent_locator: VhdxParentLocator = VhdxParentLocator::new();
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                parent_locator.read_at_position(
                    data_stream,
                    metadata_table_entry.item_size,
                    io::SeekFrom::Start(metadata_item_offset),
                )?;
                match parent_locator.entries.get("parent_linkage") {
                    Some(ucs2_string) => {
                        // TODO: improve handling of invalid string.
                        let uuid_string: String = ucs2_string.to_string();
                        let mut parent_identifier: Uuid = Uuid::new();

                        match parent_identifier.from_string(uuid_string.as_str()) {
                            Ok(_) => {}
                            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                        }
                        self.parent_identifier = Some(parent_identifier);
                    }
                    None => {}
                };
                match parent_locator.entries.get("absolute_win32_path") {
                    Some(ucs2_string) => {
                        self.parent_name = Some(ucs2_string.clone());
                    }
                    None => {}
                };
                if self.parent_name.is_none() {
                    match parent_locator.entries.get("volume_path") {
                        Some(ucs2_string) => {
                            self.parent_name = Some(ucs2_string.clone());
                        }
                        None => {}
                    };
                }
                if self.parent_name.is_none() {
                    match parent_locator.entries.get("relative_path") {
                        Some(ucs2_string) => {
                            self.parent_name = Some(ucs2_string.clone());
                        }
                        None => {}
                    };
                }
            }
            None => {}
        };
        Ok(())
    }

    /// Reads a specific block allocation entry and fills the block tree.
    fn read_block_allocation_entry(&mut self, block_number: u64) -> io::Result<()> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing data stream",
                ))
            }
        };
        let table_entry: u64 = if self.disk_type == VhdxDiskType::Fixed {
            block_number
        } else {
            ((block_number / self.entries_per_chunk) * (self.entries_per_chunk + 1))
                + (block_number % self.entries_per_chunk)
        };
        let block_allocation_table: &VhdxBlockAllocationTable =
            self.block_allocation_table.as_ref().unwrap();
        let entry: VhdxBlockAllocationTableEntry =
            block_allocation_table.read_entry(data_stream, table_entry as u32)?;

        if self.disk_type == VhdxDiskType::Differential && entry.block_state != 6 {
            self.read_sector_bitmap(block_number, entry.block_offset)?;
        } else {
            let block_media_offset: u64 = block_number * (self.block_size as u64);

            let block_range: VhdxBlockRange = if entry.block_state < 6 {
                VhdxBlockRange::new(
                    block_media_offset,
                    0,
                    self.block_size as u64,
                    VhdxBlockRangeType::Sparse,
                )
            } else {
                VhdxBlockRange::new(
                    block_media_offset,
                    entry.block_offset,
                    self.block_size as u64,
                    VhdxBlockRangeType::InFile,
                )
            };
            match self.block_tree.insert_value(
                block_media_offset,
                self.block_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            };
        }
        Ok(())
    }

    /// Reads a specific sector bitmap and fills the block tree.
    fn read_sector_bitmap(&mut self, block_number: u64, block_offset: u64) -> io::Result<()> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing data stream",
                ))
            }
        };
        let table_entry: u64 =
            (1 + (block_number / self.entries_per_chunk)) * (self.entries_per_chunk + 1) - 1;

        let block_allocation_table: &VhdxBlockAllocationTable =
            self.block_allocation_table.as_ref().unwrap();
        let entry: VhdxBlockAllocationTableEntry =
            block_allocation_table.read_entry(data_stream, table_entry as u32)?;

        let sector_bitmap_offset: u64 = entry.block_offset
            + ((block_number % self.entries_per_chunk) * self.sector_bitmap_size as u64);

        let mut sector_bitmap: VhdxSectorBitmap =
            VhdxSectorBitmap::new(self.sector_bitmap_size as usize, self.bytes_per_sector);
        sector_bitmap.read_at_position(data_stream, io::SeekFrom::Start(sector_bitmap_offset))?;

        let mut range_media_offset: u64 = block_number * (self.block_size as u64);
        let mut range_data_offset: u64 = block_offset;

        for bitmap_range in sector_bitmap.ranges.iter() {
            let block_range: VhdxBlockRange = if bitmap_range.is_set {
                VhdxBlockRange::new(
                    range_media_offset,
                    range_data_offset,
                    bitmap_range.size,
                    VhdxBlockRangeType::InFile,
                )
            } else {
                VhdxBlockRange::new(
                    range_media_offset,
                    0,
                    bitmap_range.size,
                    VhdxBlockRangeType::InParent,
                )
            };
            match self
                .block_tree
                .insert_value(range_media_offset, bitmap_range.size, block_range)
            {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
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
            let mut block_tree_value: Option<&VhdxBlockRange> =
                self.block_tree.get_value(media_offset);

            if block_tree_value.is_none() {
                self.read_block_allocation_entry(block_number)?;

                block_tree_value = self.block_tree.get_value(media_offset);
            }
            let block_range: &VhdxBlockRange = match block_tree_value {
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
                VhdxBlockRangeType::InFile => match self.data_stream.as_ref() {
                    Some(data_stream) => match data_stream.write() {
                        Ok(mut data_stream) => data_stream.read_at_position(
                            &mut data[data_offset..data_end_offset],
                            io::SeekFrom::Start(block_range.data_offset + range_relative_offset),
                        )?,
                        Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                    },
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing data stream",
                        ))
                    }
                },
                VhdxBlockRangeType::InParent => match &self.parent_file {
                    Some(parent_file) => match parent_file.write() {
                        Ok(mut file) => {
                            file.seek(io::SeekFrom::Start(media_offset))?;

                            file.read(&mut data[data_offset..data_end_offset])?
                        }
                        Err(error) => return Err(keramics_core::error_to_io_error!(error)),
                    },
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing parent file",
                        ));
                    }
                },
                VhdxBlockRangeType::Sparse => {
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
    pub fn set_parent(&mut self, parent_file: &Arc<RwLock<VhdxFile>>) -> io::Result<()> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Missing parent identifier",
                ))
            }
        };
        match parent_file.read() {
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
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        }
        self.parent_file = Some(parent_file.clone());

        Ok(())
    }
}

impl Read for VhdxFile {
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

impl Seek for VhdxFile {
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

impl DataStream for VhdxFile {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.media_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_file() -> io::Result<VhdxFile> {
        let mut file: VhdxFile = VhdxFile::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/vhdx/ext2.vhdx")?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_parent_file_name() -> io::Result<()> {
        let mut file: VhdxFile = VhdxFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
        file.read_data_stream(&data_stream)?;

        let parent_file_name: Option<Ucs2String> = file.get_parent_file_name();
        assert_eq!(parent_file_name.unwrap().to_string(), "ntfs-parent.vhdx");

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> io::Result<()> {
        let mut file: VhdxFile = VhdxFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 4194304);
        assert_eq!(
            file.identifier.to_string(),
            "305abbc8-cef4-45ea-aee8-42ee5c891b06"
        );
        assert_eq!(
            file.parent_identifier.unwrap().to_string(),
            "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
        );
        assert_eq!(
            file.parent_name.unwrap().to_string(),
            "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhdx",
        );
        Ok(())
    }

    #[test]
    fn test_read_metadata() -> io::Result<()> {
        let mut file: VhdxFile = VhdxFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
        file.read_metadata(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.media_size, 4194304);
        assert_eq!(
            file.identifier.to_string(),
            "305abbc8-cef4-45ea-aee8-42ee5c891b06"
        );
        assert_eq!(
            file.parent_identifier.unwrap().to_string(),
            "7584f8fb-36d3-4091-afb5-b1afe587bfa8"
        );
        assert_eq!(
            file.parent_name.unwrap().to_string(),
            "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhdx",
        );
        Ok(())
    }

    // TODO: add tests for read_metadata_values
    // TODO: add tests for read_block_allocation_entry
    // TODO: add tests for read_sector_bitmap
    // TODO: add tests for read_data_from_blocks
    // TODO: add tests for set_parent

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;

        let offset = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;
        file.seek(io::SeekFrom::Start(1024))?;

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
    fn test_seek_and_read_beyond_media_size() -> io::Result<()> {
        let mut file: VhdxFile = get_file()?;
        file.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
