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

use std::cmp::min;
use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::Uuid;

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionType};
use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::block_table_reader::UdifBlockTableReader;
use super::file::UdifFile;
use super::segment_range::UdifSegmentRange;

/// Universal Disk Image Format (UDIF) segment stream.
pub struct UdifSegmentStream {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Segment file set identifier.
    pub segment_set_identifier: Uuid,

    /// Number of segments.
    pub number_of_segments: u32,

    /// Name.
    name: String,

    /// Segment ranges.
    segment_ranges: Vec<UdifSegmentRange>,

    /// Segment file cache.
    segment_file_cache: LruCache<u32, UdifFile>,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// Encryption type.
    pub encryption_type: CdsaEncrEncryptionType,

    /// Credentials.
    credentials: Vec<CdsaEncrCredential>,

    /// Value to indicate the (encrypted) image is locked.
    pub is_locked: bool,

    /// The current offset.
    current_offset: u64,

    /// The size.
    pub size: u64,
}

impl UdifSegmentStream {
    /// Creates a new segment stream.
    pub(super) fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            segment_set_identifier: Uuid::new(),
            number_of_segments: 0,
            name: String::new(),
            segment_ranges: Vec::new(),
            segment_file_cache: LruCache::new(16),
            number_of_sectors: 0,
            encryption_type: CdsaEncrEncryptionType::new(),
            credentials: Vec::new(),
            is_locked: false,
            current_offset: 0,
            size: 0,
        }
    }

    /// Determines the segment file name.
    fn get_segment_file_name(name: &String, segment_number: u32) -> String {
        if segment_number == 1 {
            format!("{}.dmg", name)
        } else {
            format!("{}.{:03}.dmgpart", name, segment_number)
        }
    }

    /// Opens a segment stream.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        self.name = match file_name.file_stem() {
            Ok(Some(file_stem)) => file_stem.to_string(),
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing file stem in segment file: {}",
                    file_name,
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file stem of segment file: {}",
                        file_name,
                    )
                );
                return Err(error);
            }
        };
        let path_components: [PathComponent; 1] = [file_name.clone()];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing segment file: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open segment file: {}", file_name)
                );
                return Err(error);
            }
        };
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to open encrypted container of segment file: {}",
                            file_name
                        ),
                    );
                    return Err(error);
                }
            }
            self.encryption_type = cdsaencr_container.get_encryption_type().clone();
            self.is_locked = true;
        }
        self.file_resolver = file_resolver.clone();

        if !self.is_locked {
            match self.read_first_segment_file(&data_stream, file_name) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read first segment file"
                    );
                    return Err(error);
                }
            }
            match self.read_segment_files() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Opens a segment file.
    fn open_segment_file(&self, segment_number: u32) -> Result<UdifFile, ErrorTrace> {
        let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

        let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

        let mut data_stream: DataStreamReference =
            match self.file_resolver.get_data_stream(&path_components) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing segment file: {}",
                        segment_file_name
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
        let mut footer_signature: [u8; 8] = [0; 8];
        let mut header_signature: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut header_signature,
            SeekFrom::Start(0)
        );
        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut footer_signature,
            SeekFrom::End(-8)
        );
        if &header_signature == CDSAENCR_CONTAINER_HEADER_SIGNATURE
            || &footer_signature == CDSAENCR_CONTAINER_FOOTER_SIGNATURE
        {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to open encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            match cdsaencr_container.unlock(&self.credentials) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to unlock encrypted container of segment file: {}",
                        segment_file_name,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Failed to unlock encrypted container of segment file: {}",
                            segment_file_name
                        ),
                    );
                    return Err(error);
                }
            }
            data_stream = Arc::new(RwLock::new(cdsaencr_container));
        }
        let mut segment_file: UdifFile = UdifFile::new();

        match segment_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read segment file: {}", segment_file_name)
                );
                return Err(error);
            }
        }
        Ok(segment_file)
    }

    /// Reads the first segment file.
    fn read_first_segment_file(
        &mut self,
        data_stream: &DataStreamReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let mut segment_file: UdifFile = UdifFile::new();

        match segment_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to read segment file: {}", file_name)
                );
                return Err(error);
            }
        }
        self.number_of_sectors = segment_file.number_of_sectors;
        self.number_of_segments = segment_file.number_of_segments;
        self.size = 0;

        let segment_number: u32 = segment_file.segment_number;

        if self.number_of_segments == 0 && segment_number == 0 {
            self.number_of_segments = 1
        } else if self.number_of_segments != 0 && segment_number != 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment number value out of bounds"
            ));
        }
        if segment_file.plist_size != 0 && segment_file.resource_fork_size != 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - both XML plist and resource fork in use"
            ));
        }
        if segment_file.segment_offset != self.size {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment file: 1 - segment offset value out of bounds"
            ));
        }
        self.size += segment_file.data_fork_size;

        let segment_range: UdifSegmentRange =
            UdifSegmentRange::new(segment_file.segment_offset, 1, segment_file.data_fork_size);
        self.segment_ranges.push(segment_range);

        self.segment_set_identifier = segment_file.segment_set_identifier.clone();

        self.segment_file_cache.insert(1, segment_file);

        Ok(())
    }

    /// Reads the successive segment files.
    fn read_segment_files(&mut self) -> Result<(), ErrorTrace> {
        for segment_number in 2..=self.number_of_segments {
            let segment_file: UdifFile = match self.open_segment_file(segment_number) {
                Ok(udif_file) => udif_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_number)
                    );
                    return Err(error);
                }
            };
            if segment_file.segment_number != segment_number {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment number value out of bounds",
                    segment_number
                )));
            }
            if &segment_file.segment_set_identifier != &self.segment_set_identifier {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment set identifier mismatch",
                    segment_number
                )));
            }
            if segment_file.plist_size != 0 || segment_file.resource_fork_size != 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - XML plist and/or resource fork in use",
                    segment_number
                )));
            }
            if segment_file.segment_offset != self.size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - segment offset value out of bounds",
                    segment_number
                )));
            }
            if segment_file.number_of_sectors != self.number_of_sectors {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported segment file: {} - number of sectors value out of bounds",
                    segment_number
                )));
            }
            self.size += segment_file.data_fork_size;

            let segment_range: UdifSegmentRange = UdifSegmentRange::new(
                segment_file.segment_offset,
                segment_number,
                segment_file.data_fork_size,
            );
            self.segment_ranges.push(segment_range);
        }
        Ok(())
    }

    /// Reads the metadata in the XML plist or resource fork in the first segment file.
    pub(super) fn read_metadata(
        &mut self,
        bytes_per_sector: u16,
    ) -> Result<UdifBlockTableReader, ErrorTrace> {
        let segment_number: u32 = 1;

        if !self.segment_file_cache.contains(&segment_number) {
            let segment_file: UdifFile = match self.open_segment_file(segment_number) {
                Ok(udif_file) => udif_file,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_number)
                    );
                    return Err(error);
                }
            };
            self.segment_file_cache.insert(segment_number, segment_file);
        }
        let segment_file: &mut UdifFile = match self.segment_file_cache.get_mut(&segment_number) {
            Some(segment_file) => segment_file,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve segment file: {} from cache",
                    segment_number
                )));
            }
        };
        let mut block_table_reader: UdifBlockTableReader =
            UdifBlockTableReader::new(bytes_per_sector, self.size);

        if segment_file.plist_size == 0 && segment_file.resource_fork_size == 0 {
            block_table_reader.media_offset = self.size;
        } else if segment_file.plist_size == 0 {
            match segment_file.read_resource_fork(&mut block_table_reader) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read resource fork from segment file: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            }
        } else {
            match segment_file.read_xml_plist(&mut block_table_reader) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read XML plist from segment file: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(block_table_reader)
    }

    /// Reads media data based on the segment files.
    fn read_data_from_segments(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut segment_offset: u64 = self.current_offset;

        let mut segment_index: usize = self.segment_ranges.partition_point(|segment_range| {
            let segment_end_offset: u64 = segment_range.segment_offset + segment_range.size;
            segment_offset >= segment_end_offset
        });
        while data_offset < read_size {
            if segment_offset >= self.size {
                break;
            }
            let segment_range: &UdifSegmentRange = match self.segment_ranges.get(segment_index) {
                Some(segment_range) => segment_range,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment range for offset: {} (0x{:08x})",
                        segment_offset, segment_offset
                    )));
                }
            };
            let range_relative_offset: u64 = segment_offset - segment_range.segment_offset;
            let range_remainder_size: u64 = segment_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);

            if range_read_size == 0 {
                break;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            if !self
                .segment_file_cache
                .contains(&segment_range.segment_number)
            {
                let segment_file: UdifFile =
                    match self.open_segment_file(segment_range.segment_number) {
                        Ok(udif_file) => udif_file,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to open segment file: {}",
                                    segment_range.segment_number
                                )
                            );
                            return Err(error);
                        }
                    };
                self.segment_file_cache
                    .insert(segment_range.segment_number, segment_file);
            }
            let segment_file: &mut UdifFile = match self
                .segment_file_cache
                .get_mut(&segment_range.segment_number)
            {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve segment file: {} from cache",
                        segment_range.segment_number
                    )));
                }
            };
            match segment_file
                .read_exact_at_position(&mut data[data_offset..data_end_offset], segment_offset)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read from segment file: {} at offset: {} (0x{:08x})",
                            segment_range.segment_number, segment_offset, segment_offset
                        )
                    );
                    return Err(error);
                }
            }
            data_offset += range_read_size;
            segment_offset += range_read_size as u64;
            segment_index += 1;
        }
        Ok(data_offset)
    }

    /// Unlocks a locked (encrypted) segment stream.
    pub fn unlock(&mut self, credentials: &[CdsaEncrCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let segment_number: u32 = 1;

        let segment_file_name: String = Self::get_segment_file_name(&self.name, segment_number);

        let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

        let mut data_stream: DataStreamReference =
            match self.file_resolver.get_data_stream(&path_components) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing segment file: {}",
                        segment_file_name
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to open segment file: {}", segment_file_name)
                    );
                    return Err(error);
                }
            };
        let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

        match cdsaencr_container.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to open encrypted container of segment file: {}",
                        segment_file_name
                    ),
                );
                return Err(error);
            }
        }
        let result: bool = match cdsaencr_container.unlock(credentials) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Failed to unlock encrypted container of segment file: {}",
                        segment_file_name
                    ),
                );
                return Err(error);
            }
        };
        if result {
            self.credentials = credentials.to_vec();

            data_stream = Arc::new(RwLock::new(cdsaencr_container));

            let path_component: PathComponent = PathComponent::from(&segment_file_name);
            match self.read_first_segment_file(&data_stream, &path_component) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read first segment file"
                    );
                    return Err(error);
                }
            }
            match self.read_segment_files() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                    return Err(error);
                }
            }
            self.is_locked = false;
        }
        Ok(!self.is_locked)
    }
}

impl DataStream for UdifSegmentStream {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = match self.read_data_from_segments(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from segments");
                return Err(error);
            }
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
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid offset value out of bounds"
                    ));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::os_file_resolver::open_os_file_resolver;
    use crate::tests::get_test_data_path;

    fn get_segment_stream(file_name_string: &str) -> Result<UdifSegmentStream, ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = UdifSegmentStream::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from(file_name_string);
        segment_stream.open(&file_resolver, &file_name)?;

        Ok(segment_stream)
    }

    #[test]
    fn test_get_segment_file_name() {
        let base_name: String = String::from("image");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 1);
        assert_eq!(name, "image.dmg");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 9);
        assert_eq!(name, "image.009.dmgpart");

        let name: String = UdifSegmentStream::get_segment_file_name(&base_name, 1234);
        assert_eq!(name, "image.1234.dmgpart");
    }

    // TODO: add tests for open_segment_file

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = UdifSegmentStream::new();

        let path_string: String = get_test_data_path("udif");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("hfsplus_segments.dmg");
        segment_stream.open(&file_resolver, &file_name)?;

        assert_eq!(segment_stream.size, 1955840);

        Ok(())
    }

    // TODO: add tests for read_first_segment_file
    // TODO: add tests for read_segment_files
    // TODO: add tests for read_data_from_segments

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        segment_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = segment_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let size: u64 = segment_stream.get_size()?;
        assert_eq!(size, 1955840);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, segment_stream.size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset = segment_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = segment_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let result: Result<u64, ErrorTrace> = segment_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;

        let offset: u64 = segment_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, segment_stream.size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;
        segment_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = segment_stream.read(&mut data)?;
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
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_segments.dmg")?;
        segment_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = segment_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut segment_stream: UdifSegmentStream = get_segment_stream("hfsplus_aes256.dmg")?;

        assert_eq!(segment_stream.is_locked, true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        segment_stream.unlock(&credentials)?;

        assert_eq!(segment_stream.is_locked, false);

        Ok(())
    }
}
