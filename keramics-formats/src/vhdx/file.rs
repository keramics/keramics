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

use keramics_core::{DataStreamReference, DebugTrace, ErrorTrace};
use keramics_types::{Ucs2String, Uuid, bytes_to_u32_le, bytes_to_u64_le};

use super::block_reader::VhdxBlockReader;
use super::block_stream::VhdxBlockStream;
use super::constants::*;
use super::enums::VhdxDiskType;
use super::file_header::VhdxFileHeader;
use super::image_header::VhdxImageHeader;
use super::metadata_table::VhdxMetadataTable;
use super::parent_locator::VhdxParentLocator;
use super::region_table::VhdxRegionTable;
use super::region_table_entry::VhdxRegionTableEntry;

/// Virtual Hard Disk version 2 (VHDX) file.
pub struct VhdxFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    format_version: u16,

    /// Disk type.
    disk_type: VhdxDiskType,

    /// Identifier.
    pub(super) identifier: Uuid,

    /// Parent identifier.
    pub(super) parent_identifier: Option<Uuid>,

    /// Parent name.
    parent_name: Option<Ucs2String>,

    /// Parent file.
    parent_file: Option<Arc<VhdxFile>>,

    /// Bytes per sector.
    pub(super) bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Block allocation table offset.
    block_allocation_table_offset: u64,

    /// Block allocation table size.
    block_allocation_table_size: u32,

    /// Number of entries per chunk;
    entries_per_chunk: u64,

    /// Sector bitmap size.
    sector_bitmap_size: u32,

    /// Media size.
    pub(super) media_size: u64,
}

impl VhdxFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            disk_type: VhdxDiskType::Fixed,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
            parent_file: None,
            bytes_per_sector: 0,
            block_size: 0,
            block_allocation_table_offset: 0,
            block_allocation_table_size: 0,
            entries_per_chunk: 0,
            sector_bitmap_size: 0,
            media_size: 0,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.data_stream {
            Some(data_stream) => {
                let parent_data_stream: Option<DataStreamReference> = match &self.parent_file {
                    Some(parent_file) => parent_file.get_data_stream(),
                    None => None,
                };
                Some(Arc::new(RwLock::new(VhdxBlockStream::new(
                    VhdxBlockReader::new(
                        data_stream,
                        &self.disk_type,
                        self.bytes_per_sector,
                        self.block_size,
                        self.block_allocation_table_offset,
                        self.block_allocation_table_size,
                        parent_data_stream,
                        self.media_size,
                    ),
                ))))
            }
            None => None,
        }
    }

    /// Retrieves the disk type.
    pub fn get_disk_type(&self) -> &VhdxDiskType {
        &self.disk_type
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u16 {
        self.format_version
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Retrieves the parent file name
    pub fn get_parent_file_name(&self) -> Option<Ucs2String> {
        match &self.parent_name {
            Some(parent_name) => {
                match parent_name
                    .elements
                    .iter()
                    .rposition(|value| *value == 0x005c)
                {
                    Some(value_index) => {
                        Some(Ucs2String::from(&parent_name.elements[value_index + 1..]))
                    }
                    None => Some(parent_name.clone()),
                }
            }
            None => None,
        }
    }

    /// Retrieves the parent identifier.
    pub fn get_parent_identifier(&self) -> Option<&Uuid> {
        self.parent_identifier.as_ref()
    }

    /// Retrieves the parent name.
    pub fn get_parent_name(&self) -> Option<&Ucs2String> {
        self.parent_name.as_ref()
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

    /// Reads the file header, image headers and region tables.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut file_header: VhdxFileHeader = VhdxFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        let mut primary_image_header: VhdxImageHeader = VhdxImageHeader::new();

        match primary_image_header.read_at_position(data_stream, SeekFrom::Start(65536)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read primary image header");
                return Err(error);
            }
        }
        let mut secondary_image_header: VhdxImageHeader = VhdxImageHeader::new();

        match secondary_image_header.read_at_position(data_stream, SeekFrom::Start(2 * 65536)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read secondary image header"
                );
                return Err(error);
            }
        }
        if primary_image_header.sequence_number > secondary_image_header.sequence_number {
            self.identifier = primary_image_header.data_write_identifier;
            self.format_version = primary_image_header.format_version;
        } else {
            self.identifier = secondary_image_header.data_write_identifier;
            self.format_version = secondary_image_header.format_version;
        }
        let mut primary_region_table: VhdxRegionTable = VhdxRegionTable::new();

        match primary_region_table.read_at_position(data_stream, SeekFrom::Start(3 * 65536)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read primary region table");
                return Err(error);
            }
        }
        let mut secondary_region_table: VhdxRegionTable = VhdxRegionTable::new();

        match secondary_region_table.read_at_position(data_stream, SeekFrom::Start(4 * 65536)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read secondary region table"
                );
                return Err(error);
            }
        }
        // TODO: compare primary region table with secondary

        let metadata_region: &VhdxRegionTableEntry =
            match primary_region_table.get_entry(&VHDX_METADATA_REGION_IDENTIFIER) {
                Some(region_table_entry) => {
                    if region_table_entry.data_size < 65536 {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported metadata region size: {}",
                            region_table_entry.data_size
                        )));
                    }
                    region_table_entry
                }
                None => {
                    return Err(keramics_core::error_trace_new!("Missing metadata region"));
                }
            };
        match self.read_metadata_values(data_stream, metadata_region) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata values");
                return Err(error);
            }
        }
        let block_allocation_table_region: &VhdxRegionTableEntry =
            match primary_region_table.get_entry(&VHDX_BLOCK_ALLOCATION_TABLE_REGION_IDENTIFIER) {
                Some(region_table_entry) => region_table_entry,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table region"
                    ));
                }
            };
        self.entries_per_chunk =
            ((1 << 23) * (self.bytes_per_sector as u64)) / (self.block_size as u64);
        self.sector_bitmap_size = 1048576 / (self.entries_per_chunk as u32);

        self.block_allocation_table_offset = block_allocation_table_region.data_offset;
        self.block_allocation_table_size = block_allocation_table_region.data_size;

        let number_of_entries: u32 = block_allocation_table_region.data_size / 8;
        let blocks_data_size: u64 = (number_of_entries as u64) * (self.block_size as u64);

        if self.media_size > blocks_data_size {
            let calculated_number_of_blocks: u64 = self.media_size.div_ceil(self.block_size as u64);
            return Err(keramics_core::error_trace_new!(format!(
                "Number of blocks: {} in block allocation table too small for virtual disk size: {} ({} blocks)",
                number_of_entries, self.media_size, calculated_number_of_blocks,
            )));
        }
        Ok(())
    }

    /// Reads the metadata values.
    fn read_metadata_values(
        &mut self,
        data_stream: &DataStreamReference,
        metadata_region: &VhdxRegionTableEntry,
    ) -> Result<(), ErrorTrace> {
        let mut metadata_table: VhdxMetadataTable = VhdxMetadataTable::new();

        match metadata_table
            .read_at_position(data_stream, SeekFrom::Start(metadata_region.data_offset))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata table");
                return Err(error);
            }
        }
        let file_parameters_flags: u32;

        match metadata_table.get_entry(&VHDX_FILE_PARAMETERS_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 8 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported file parameters metadata item size: {}",
                        metadata_table_entry.item_size
                    )));
                }
                let mut data: [u8; 8] = [0; 8];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                keramics_core::data_stream_read_at_position!(
                    data_stream,
                    &mut data,
                    SeekFrom::Start(metadata_item_offset)
                );
                file_parameters_flags = bytes_to_u32_le!(data, 4);

                self.block_size = bytes_to_u32_le!(data, 0);
                self.disk_type = match file_parameters_flags & 0x00000003 {
                    0 => VhdxDiskType::Fixed,
                    1 => VhdxDiskType::Dynamic,
                    2 => VhdxDiskType::Differential,
                    _ => VhdxDiskType::Unknown,
                };
                if self.block_size < 1024 * 1024 || self.block_size > 256 * 1024 * 1024 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid block size: {} value out of bounds",
                        self.block_size
                    )));
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing file parameters metadata item"
                ));
            }
        };
        match metadata_table.get_entry(&VHDX_VIRTUAL_DISK_SIZE_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 8 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported virtual disk size metadata item size: {}",
                        metadata_table_entry.item_size
                    )));
                }
                let mut data: [u8; 8] = [0; 8];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                keramics_core::data_stream_read_at_position!(
                    data_stream,
                    &mut data,
                    SeekFrom::Start(metadata_item_offset)
                );
                self.media_size = bytes_to_u64_le!(data, 0);
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing virtual disk size metadata item"
                ));
            }
        };
        match metadata_table.get_entry(&VHDX_LOGICAL_SECTOR_SIZE_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 4 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported logical sector size metadata item size: {}",
                        metadata_table_entry.item_size
                    )));
                }
                let mut data: [u8; 4] = [0; 4];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                keramics_core::data_stream_read_at_position!(
                    data_stream,
                    &mut data,
                    SeekFrom::Start(metadata_item_offset)
                );
                let logical_sector_size: u32 = bytes_to_u32_le!(data, 0);

                if logical_sector_size != 512 && logical_sector_size != 4096 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid logical sector size: {} value out of bounds",
                        logical_sector_size
                    )));
                }
                self.bytes_per_sector = logical_sector_size as u16;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing logical sector size metadata item"
                ));
            }
        };
        let mut physical_sector_size: u32 = 0;

        match metadata_table.get_entry(&VHDX_PHYSICAL_SECTOR_SIZE_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 4 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported physical sector size metadata item size: {}",
                        metadata_table_entry.item_size
                    )));
                }
                let mut data: [u8; 4] = [0; 4];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                keramics_core::data_stream_read_at_position!(
                    data_stream,
                    &mut data,
                    SeekFrom::Start(metadata_item_offset)
                );
                physical_sector_size = bytes_to_u32_le!(data, 0);

                if physical_sector_size != 512 && physical_sector_size != 4096 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid physical sector size: {} value out of bounds",
                        physical_sector_size
                    )));
                }
            }
            None => {}
        };
        let virtual_disk_identifier: Uuid;

        match metadata_table.get_entry(&VHDX_VIRTUAL_DISK_IDENTIFIER_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                if metadata_table_entry.item_size != 16 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported virtual disk identifier metadata item size: {}",
                        metadata_table_entry.item_size
                    )));
                }
                let mut data: [u8; 16] = [0; 16];
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                keramics_core::data_stream_read_at_position!(
                    data_stream,
                    &mut data,
                    SeekFrom::Start(metadata_item_offset)
                );
                virtual_disk_identifier = Uuid::from_le_bytes(&data);
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing virtual disk identifier metadata item"
                ));
            }
        }
        DebugTrace::static_scope(|debug_trace| {
            debug_trace.print_start("VhdxMetadataValues");
            debug_trace.print_field("file_parameters_block_size", self.block_size);
            debug_trace.print_field(
                "file_parameters_flags",
                format!("0x{:08x},", file_parameters_flags),
            );
            debug_trace.print_field("virtual_disk_size", self.media_size);
            debug_trace.print_field("logical_sector_size", self.bytes_per_sector);
            debug_trace.print_field("physical_sector_size", physical_sector_size);
            debug_trace.print_field("virtual_disk_identifier", virtual_disk_identifier);
            debug_trace.print_end();
        });
        match metadata_table.get_entry(&VHDX_PARENT_LOCATOR_METADATA_IDENTIFIER) {
            Some(metadata_table_entry) => {
                let mut parent_locator: VhdxParentLocator = VhdxParentLocator::new();
                let metadata_item_offset: u64 =
                    metadata_region.data_offset + metadata_table_entry.item_offset as u64;

                match parent_locator.read_at_position(
                    data_stream,
                    metadata_table_entry.item_size,
                    SeekFrom::Start(metadata_item_offset),
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read parent locator"
                        );
                        return Err(error);
                    }
                }
                match parent_locator.get_entry("parent_linkage") {
                    Some(ucs2_string) => {
                        // TODO: improve handling of invalid string.
                        let uuid_string: String = ucs2_string.to_string();

                        let parent_identifier: Uuid = match Uuid::from_string(uuid_string.as_str())
                        {
                            Ok(uuid) => uuid,
                            Err(error) => {
                                return Err(keramics_core::error_trace_new_with_error!(
                                    "Unable to parse parent identifier",
                                    error
                                ));
                            }
                        };
                        self.parent_identifier = Some(parent_identifier);
                    }
                    None => {}
                };
                match parent_locator.get_entry("absolute_win32_path") {
                    Some(ucs2_string) => {
                        self.parent_name = Some(ucs2_string.clone());
                    }
                    None => {}
                };
                if self.parent_name.is_none() {
                    match parent_locator.get_entry("volume_path") {
                        Some(ucs2_string) => {
                            self.parent_name = Some(ucs2_string.clone());
                        }
                        None => {}
                    };
                }
                if self.parent_name.is_none() {
                    match parent_locator.get_entry("relative_path") {
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

    /// Sets the parent file.
    pub fn set_parent(&mut self, parent_file: &Arc<VhdxFile>) -> Result<(), ErrorTrace> {
        let parent_identifier: &Uuid = match &self.parent_identifier {
            Some(parent_identifier) => parent_identifier,
            None => {
                return Err(keramics_core::error_trace_new!("Missing parent identifier"));
            }
        };
        if parent_identifier != &parent_file.identifier {
            return Err(keramics_core::error_trace_new!(format!(
                "Parent identifier: {} does not match identifier of parent file: {}",
                parent_identifier, parent_file.identifier,
            )));
        }
        self.parent_file = Some(parent_file.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<VhdxFile, ErrorTrace> {
        let mut file: VhdxFile = VhdxFile::new();

        let path_string: String = get_test_data_path("vhdx/ext2.vhdx");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let bytes_per_sector: u16 = file.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_disk_type() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let disk_type: &VhdxDiskType = file.get_disk_type();
        assert_eq!(disk_type, &VhdxDiskType::Fixed);

        Ok(())
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let format_version: u16 = file.get_format_version();
        assert_eq!(format_version, 1);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let identifier: &Uuid = file.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "ee10a932-6284-f448-aaab-ab839f90ddef"
        );
        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let media_size: u64 = file.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_parent_file_name() -> Result<(), ErrorTrace> {
        let mut file: VhdxFile = VhdxFile::new();

        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let parent_file_name: Option<Ucs2String> = file.get_parent_file_name();
        assert_eq!(parent_file_name, Some(Ucs2String::from("ntfs-parent.vhdx")));

        Ok(())
    }

    #[test]
    fn test_get_parent_identifier() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let parent_identifier: Option<&Uuid> = file.get_parent_identifier();
        assert!(parent_identifier.is_none());

        Ok(())
    }

    #[test]
    fn test_get_parent_name() -> Result<(), ErrorTrace> {
        let file: VhdxFile = get_file()?;

        let parent_name: Option<&Ucs2String> = file.get_parent_name();
        assert!(parent_name.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: VhdxFile = VhdxFile::new();

        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
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
            file.parent_name,
            Some(Ucs2String::from(
                "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhdx"
            ))
        );
        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file: VhdxFile = VhdxFile::new();

        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
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
            file.parent_name,
            Some(Ucs2String::from(
                "C:\\Projects\\dfvfs\\test_data\\ntfs-parent.vhdx"
            ))
        );
        Ok(())
    }

    // TODO: add tests for read_metadata_values
    // TODO: add tests for set_parent
}
