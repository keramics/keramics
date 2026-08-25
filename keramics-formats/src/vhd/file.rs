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
use keramics_types::{Ucs2String, Uuid};

use crate::range_stream::RangeStream;

use super::block_reader::VhdBlockReader;
use super::block_stream::VhdBlockStream;
use super::constants::*;
use super::dynamic_disk_header::VhdDynamicDiskHeader;
use super::enums::VhdDiskType;
use super::file_footer::VhdFileFooter;

/// Virtual Hard Disk (VHD) file.
pub struct VhdFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Disk type.
    disk_type: VhdDiskType,

    /// Identifier.
    pub(super) identifier: Uuid,

    /// Parent identifier.
    pub(super) parent_identifier: Option<Uuid>,

    /// Parent name.
    parent_name: Option<Ucs2String>,

    /// Parent file.
    parent_file: Option<Arc<VhdFile>>,

    /// Bytes per sector.
    pub(super) bytes_per_sector: u16,

    /// Block size.
    block_size: u32,

    /// Block allocation table offset.
    block_allocation_table_offset: u64,

    /// Number of blocks.
    number_of_blocks: u32,

    /// Media size.
    pub(super) media_size: u64,
}

impl VhdFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            disk_type: VhdDiskType::Fixed,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
            parent_file: None,
            bytes_per_sector: 0,
            block_size: 0,
            block_allocation_table_offset: 0,
            number_of_blocks: 0,
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
                if self.disk_type == VhdDiskType::Fixed {
                    Some(Arc::new(RwLock::new(RangeStream::new(
                        data_stream,
                        0,
                        self.media_size,
                    ))))
                } else {
                    let parent_data_stream: Option<DataStreamReference> = match &self.parent_file {
                        Some(parent_file) => parent_file.get_data_stream(),
                        None => None,
                    };
                    Some(Arc::new(RwLock::new(VhdBlockStream::new(
                        VhdBlockReader::new(
                            data_stream,
                            &self.disk_type,
                            self.bytes_per_sector,
                            self.block_size,
                            self.block_allocation_table_offset,
                            self.number_of_blocks,
                            parent_data_stream,
                            self.media_size,
                        ),
                    ))))
                }
            }
            None => None,
        }
    }

    /// Retrieves the disk type.
    pub fn get_disk_type(&self) -> &VhdDiskType {
        &self.disk_type
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
            let blocks_data_size: u64 = (dynamic_disk_header.number_of_blocks as u64)
                * (dynamic_disk_header.block_size as u64);

            if file_footer.data_size > blocks_data_size {
                let calculated_number_of_blocks: u64 = file_footer
                    .data_size
                    .div_ceil(dynamic_disk_header.block_size as u64);
                return Err(keramics_core::error_trace_new!(format!(
                    "Number of blocks: {} in block allocation table too small for data size: {} ({} blocks)",
                    dynamic_disk_header.number_of_blocks,
                    file_footer.data_size,
                    calculated_number_of_blocks,
                )));
            }
            self.block_size = dynamic_disk_header.block_size;
            self.block_allocation_table_offset = dynamic_disk_header.block_table_offset;
            self.number_of_blocks = dynamic_disk_header.number_of_blocks;

            if !dynamic_disk_header.parent_identifier.is_nil() {
                self.parent_identifier = Some(dynamic_disk_header.parent_identifier);
                self.parent_name = Some(dynamic_disk_header.parent_name);
            }
        }
        Ok(())
    }

    /// Sets the parent file.
    pub fn set_parent(&mut self, parent_file: &Arc<VhdFile>) -> Result<(), ErrorTrace> {
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

    fn get_file() -> Result<VhdFile, ErrorTrace> {
        let mut file: VhdFile = VhdFile::new();

        let path_string: String = get_test_data_path("vhd/ext2.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let bytes_per_sector: u16 = file.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_disk_type() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let disk_type: &VhdDiskType = file.get_disk_type();
        assert_eq!(disk_type, &VhdDiskType::Dynamic);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let identifier: &Uuid = file.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "4f75d18f-d5ef-438e-b326-d60da6c9ed67"
        );
        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let media_size: u64 = file.get_media_size();
        assert_eq!(media_size, 4212736);

        Ok(())
    }

    #[test]
    fn test_get_parent_file_name() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = VhdFile::new();

        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let parent_file_name: Option<Ucs2String> = file.get_parent_file_name();
        assert_eq!(parent_file_name, Some(Ucs2String::from("ntfs-parent.vhd")));

        Ok(())
    }

    #[test]
    fn test_get_parent_identifier() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let parent_identifier: Option<&Uuid> = file.get_parent_identifier();
        assert!(parent_identifier.is_none());

        Ok(())
    }

    #[test]
    fn test_get_parent_name() -> Result<(), ErrorTrace> {
        let file: VhdFile = get_file()?;

        let parent_name: Option<&Ucs2String> = file.get_parent_name();
        assert!(parent_name.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: VhdFile = VhdFile::new();

        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
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
        let mut file: VhdFile = VhdFile::new();

        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
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

    // TODO: add test for set_parent
}
