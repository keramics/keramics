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

use crate::range_stream::RangeStream;

use super::block_reader::VdiBlockReader;
use super::block_stream::VdiBlockStream;
use super::constants::*;
use super::file_header::VdiFileHeader;

/// Virtual Disk Image (VDI) file.
pub struct VdiFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Major format version
    major_format_version: u16,

    /// Minor format version
    minor_format_version: u16,

    /// Identifier.
    pub(super) identifier: Uuid,

    /// Parent identifier.
    pub(super) parent_identifier: Option<Uuid>,

    /// Parent file.
    parent_file: Option<Arc<VdiFile>>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Image type.
    image_type: u32,

    /// Block map offset.
    block_map_offset: u32,

    /// Data offset.
    data_offset: u32,

    /// Block size.
    block_size: u32,

    /// Number of blocks.
    number_of_blocks: u32,

    /// Media size.
    pub(super) media_size: u64,
}

impl VdiFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            major_format_version: 0,
            minor_format_version: 0,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_file: None,
            bytes_per_sector: 0,
            image_type: 0,
            block_map_offset: 0,
            data_offset: 0,
            block_size: 0,
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
                if self.image_type == VDI_IMAGE_TYPE_FIXED {
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
                    Some(Arc::new(RwLock::new(VdiBlockStream::new(
                        VdiBlockReader::new(
                            data_stream,
                            self.block_map_offset as u64,
                            self.data_offset as u64,
                            self.block_size as u64,
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

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> (u16, u16) {
        (self.major_format_version, self.minor_format_version)
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Retrieves the parent identifier.
    pub fn get_parent_identifier(&self) -> Option<&Uuid> {
        self.parent_identifier.as_ref()
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut file_header: VdiFileHeader = VdiFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        self.major_format_version = file_header.major_format_version;
        self.minor_format_version = file_header.minor_format_version;
        self.identifier = file_header.identifier;
        self.parent_identifier = file_header.parent_identifier;
        self.bytes_per_sector = 512;
        self.image_type = file_header.image_type;
        self.block_map_offset = file_header.block_map_offset;
        self.data_offset = file_header.data_offset;
        self.block_size = file_header.block_size;
        self.number_of_blocks = file_header.number_of_blocks;
        self.media_size = file_header.data_size;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Sets the parent file.
    pub fn set_parent(&mut self, parent_file: &Arc<VdiFile>) -> Result<(), ErrorTrace> {
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

    fn get_file() -> Result<VdiFile, ErrorTrace> {
        let mut file: VdiFile = VdiFile::new();

        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file: VdiFile = get_file()?;

        let bytes_per_sector: u16 = file.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file: VdiFile = get_file()?;

        let format_version: (u16, u16) = file.get_format_version();
        assert_eq!(format_version, (1, 1));

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let file: VdiFile = get_file()?;

        let identifier: &Uuid = file.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "88437ae8-9631-4c48-81c3-e787cc586501"
        );
        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let file: VdiFile = get_file()?;

        let media_size: u64 = file.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        use std::io::SeekFrom;

        let file: VdiFile = get_file()?;

        let data_stream: DataStreamReference = match file.get_data_stream() {
            Some(data_stream) => data_stream,
            None => return Err(keramics_core::error_trace_new!("Missing data stream")),
        };
        let mut data: Vec<u8> = vec![0; 512];

        keramics_core::data_stream_read_at_position!(
            &data_stream,
            &mut data,
            SeekFrom::Start(0x000000)
        );

        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut expected_data: Vec<u8> = vec![0; 512];

        keramics_core::data_stream_read_at_position!(
            &file_data_stream,
            &mut expected_data,
            SeekFrom::Start(0x000400)
        );
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_get_parent_identifier() -> Result<(), ErrorTrace> {
        let file: VdiFile = get_file()?;

        let parent_identifier: Option<&Uuid> = file.get_parent_identifier();
        assert!(parent_identifier.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let mut file: VdiFile = VdiFile::new();

        let path_string: String = get_test_data_path("vdi/ext2.vdi");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.media_size, 4194304);
        assert_eq!(
            file.identifier.to_string(),
            "88437ae8-9631-4c48-81c3-e787cc586501"
        );
        assert_eq!(file.parent_identifier, None,);

        Ok(())
    }

    // TODO: add test for set_parent
}
