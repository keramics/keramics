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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{Uuid, bytes_to_u32_be};

use crate::plist::{PlistObject, XmlPlist};

use super::block_table::UdifBlockTable;
use super::block_table_reader::UdifBlockTableReader;
use super::constants::*;
use super::encrypted_file_footer::UdifEncryptedFileFooter;
use super::encrypted_file_header::UdifEncryptedFileHeader;
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

    /// Segment offset.
    pub(super) segment_offset: u64,

    /// Segment number.
    pub(super) segment_number: u32,

    /// Number of segments.
    pub(super) number_of_segments: u32,

    /// Segment set identifier.
    pub(super) segment_set_identifier: Uuid,

    /// Data fork offset.
    pub(super) data_fork_offset: u64,

    /// Data fork size.
    pub(super) data_fork_size: u64,

    /// Resource fork offset.
    pub(super) resource_fork_offset: u64,

    /// Resource fork size.
    pub(super) resource_fork_size: u64,

    /// Plist offset.
    pub(super) plist_offset: u64,

    /// Plist size.
    pub(super) plist_size: u64,
}

impl UdifFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            segment_offset: 0,
            segment_number: 0,
            number_of_segments: 0,
            segment_set_identifier: Uuid::new(),
            data_fork_offset: 0,
            data_fork_size: 0,
            resource_fork_offset: 0,
            resource_fork_size: 0,
            plist_offset: 0,
            plist_size: 0,
        }
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u32 {
        self.format_version
    }

    /// Retrieves the segment set identifier.
    pub fn get_segment_set_identifier(&self) -> &Uuid {
        &self.segment_set_identifier
    }

    /// Retrieves the segment number.
    pub fn get_segment_number(&self) -> u32 {
        self.segment_number
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
                let mut file_footer: UdifFileFooter = UdifFileFooter::new();

                match file_footer.read_at_position(data_stream, SeekFrom::End(-512)) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read file footer");
                        return Err(error);
                    }
                }
                self.format_version = file_footer.format_version;
                self.segment_offset = file_footer.segment_offset;
                self.segment_number = file_footer.segment_number;
                self.number_of_segments = file_footer.number_of_segments;
                self.segment_set_identifier = file_footer.segment_set_identifier;
                self.data_fork_offset = file_footer.data_fork_offset;
                self.data_fork_size = file_footer.data_fork_size;
                self.resource_fork_offset = file_footer.resource_fork_offset;
                self.resource_fork_size = file_footer.resource_fork_size;
                self.plist_offset = file_footer.plist_offset;
                self.plist_size = file_footer.plist_size;
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads an exact amount of data at a specific position.
    pub(super) fn read_exact_at_position(
        &mut self,
        data: &mut [u8],
        position: SeekFrom,
    ) -> Result<u64, ErrorTrace> {
        match self.data_stream.as_ref() {
            Some(data_stream) => {
                let offset: u64 =
                    keramics_core::data_stream_read_exact_at_position!(data_stream, data, position);

                Ok(offset)
            }
            None => Err(keramics_core::error_trace_new!("Missing data stream")),
        }
    }

    /// Reads the metadata in the XML plist or resource fork.
    pub(super) fn read_metadata(
        &mut self,
        block_table_reader: &mut UdifBlockTableReader,
    ) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        if self.plist_size == 0 {
            let mut resource_fork_header: UdifResourceForkHeader = UdifResourceForkHeader::new();

            match resource_fork_header
                .read_at_position(data_stream, SeekFrom::Start(self.resource_fork_offset))
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read resource fork header"
                    );
                    return Err(error);
                }
            }
            let offset: u64 =
                self.resource_fork_offset + (resource_fork_header.resource_map_offset as u64);

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
            let mut data: [u8; 4] = [0; 4];

            for blkx_value in blkx_item.values.iter() {
                let offset: u64 = self.resource_fork_offset
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
        } else {
            // Note that 16777216 is an arbitrary chosen limit.
            if self.plist_size > 16777216 {
                return Err(keramics_core::error_trace_new!("Unsupported data size"));
            }
            let mut data: Vec<u8> = vec![0; self.plist_size as usize];

            keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(self.plist_offset)
            );
            keramics_core::debug_trace_data!(
                "UdifFileXmlPlist",
                self.plist_offset,
                &data,
                self.plist_size
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;
    use crate::udif::segment_range::UdifSegmentRange;

    // TODO add tests for get_format_version
    // TODO add tests for get_segment_set_identifier
    // TODO add tests for get_segment_number

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.format_version, 4);

        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file: UdifFile = UdifFile::new();

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        let segment_range: UdifSegmentRange =
            UdifSegmentRange::new(0, 1, file.data_fork_offset, file.data_fork_size);
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(512, file.data_fork_size);
        file.read_metadata(&mut block_table_reader)?;

        assert!(block_table_reader.has_block_ranges());

        Ok(())
    }
}
