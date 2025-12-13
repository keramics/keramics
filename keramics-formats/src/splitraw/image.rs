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
use std::iter::Rev;
use std::str::Chars;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;

use super::enums::SplitRawNamingSchema;

/// Split raw storage media image.
pub struct SplitRawImage {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Name.
    name: String,

    /// Segment file naming schema.
    naming_schema: SplitRawNamingSchema,

    /// Name first segment number.
    name_first_segment_number: u16,

    /// Name suffix size.
    name_suffix_size: usize,

    /// Segment size.
    segment_size: u64,

    /// Number of segment files.
    number_of_segment_files: u16,

    /// Segment file cache.
    segment_file_cache: LruCache<u16, DataStreamReference>,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    pub media_size: u64,
}

impl SplitRawImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            name: String::new(),
            naming_schema: SplitRawNamingSchema::Numeric,
            name_first_segment_number: 0,
            name_suffix_size: 0,
            segment_size: 0,
            number_of_segment_files: 0,
            segment_file_cache: LruCache::new(16),
            current_offset: 0,
            media_size: 0,
        }
    }

    /// Retrieves the number of segments.
    pub fn get_number_of_segments(&self) -> u16 {
        self.number_of_segment_files
    }

    /// Determines the segment file name.
    fn get_segment_file_name(
        name: &String,
        mut segment_number: u16,
        number_of_segment_files: u16,
        naming_schema: &SplitRawNamingSchema,
        name_first_segment_number: u16,
        name_suffix_size: usize,
    ) -> Result<String, ErrorTrace> {
        if segment_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment number: 0"
            ));
        }
        match naming_schema {
            SplitRawNamingSchema::Alphabetic => {
                let mut segment_suffix: Vec<char> = Vec::new();

                segment_number = (segment_number - 1) + name_first_segment_number;
                while segment_number > 0 {
                    let remainder: u16 = segment_number % 26;
                    segment_number /= 26;

                    match char::from_u32((remainder + 0x61) as u32) {
                        Some(character) => segment_suffix.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                if segment_suffix.len() > name_suffix_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid segment suffix value exceeds size"
                    ));
                }
                while segment_suffix.len() < name_suffix_size {
                    segment_suffix.push('a');
                }
                Ok(format!(
                    "{}{}",
                    name,
                    segment_suffix.iter().rev().collect::<String>()
                ))
            }
            SplitRawNamingSchema::Numeric => {
                let mut segment_suffix: Vec<char> = Vec::new();

                // TODO: add hexadecimal support
                segment_number = (segment_number - 1) + name_first_segment_number;
                while segment_number > 0 {
                    let remainder: u16 = segment_number % 10;
                    segment_number /= 10;

                    match char::from_u32((remainder + 0x30) as u32) {
                        Some(character) => segment_suffix.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                if segment_suffix.len() > name_suffix_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid segment suffix value exceeds size"
                    ));
                }
                while segment_suffix.len() < name_suffix_size {
                    segment_suffix.push('0');
                }
                Ok(format!(
                    "{}{}",
                    name,
                    segment_suffix.iter().rev().collect::<String>()
                ))
            }
            SplitRawNamingSchema::XOfN => Ok(format!(
                "{}{}of{}",
                name, segment_number, number_of_segment_files
            )),
        }
    }

    /// Determines the segment file naming schema.
    fn get_segment_file_naming_schema(file_name_string: &str) -> Option<SplitRawNamingSchema> {
        match file_name_string.rfind("1of") {
            Some(string_index) => {
                match u16::from_str_radix(&file_name_string[string_index + 3..], 10) {
                    Ok(_) => Some(SplitRawNamingSchema::XOfN),
                    Err(_) => None,
                }
            }
            None => {
                if file_name_string.ends_with("aa") {
                    Some(SplitRawNamingSchema::Alphabetic)
                } else if file_name_string.ends_with("0") || file_name_string.ends_with("1") {
                    Some(SplitRawNamingSchema::Numeric)
                } else {
                    None
                }
            }
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        match self.read_segment_files(&file_resolver, file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read segment files");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads the segment files.
    fn read_segment_files(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &PathComponent,
    ) -> Result<(), ErrorTrace> {
        let file_name_string: String = file_name.to_string();

        self.naming_schema = match Self::get_segment_file_naming_schema(file_name_string.as_str()) {
            Some(naming_schema) => naming_schema,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to determine naming schema from segment file: {}",
                    file_name,
                )));
            }
        };
        match &self.naming_schema {
            SplitRawNamingSchema::Alphabetic => {
                let mut characters: Rev<Chars> = file_name_string.chars().rev();

                self.name_suffix_size = match characters.position(|value| value != 'a') {
                    Some(value_index) => {
                        // Note that value_index is relative to end of the string.
                        value_index
                    }
                    None => 0,
                };
                let name_size: usize = file_name_string.len() - self.name_suffix_size;

                self.name = file_name_string[0..name_size].to_string();
                self.name_first_segment_number = 0;
                self.number_of_segment_files = 0;
            }
            SplitRawNamingSchema::Numeric => {
                let mut characters: Rev<Chars> = file_name_string.chars().rev();

                self.name_first_segment_number = match characters.next() {
                    Some('0') => 0,
                    Some('1') => 1,
                    _ => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to determine first segment number from segment file: {}",
                            file_name,
                        )));
                    }
                };
                self.name_suffix_size = match characters.position(|value| value != '0') {
                    Some(value_index) => {
                        // Note that value_index is relative to last character in the string.
                        value_index + 1
                    }
                    None => 1,
                };
                let name_size: usize = file_name_string.len() - self.name_suffix_size;

                self.name = file_name_string[0..name_size].to_string();
                self.number_of_segment_files = 0;
            }
            SplitRawNamingSchema::XOfN => {
                let string_index: usize = match file_name_string.rfind("1of") {
                    Some(string_index) => string_index,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to determine number of segment files from segment file: {}",
                            file_name,
                        )));
                    }
                };
                self.number_of_segment_files = match u16::from_str_radix(
                    &file_name_string[string_index + 3..],
                    10,
                ) {
                    Ok(number_of_segment_files) => number_of_segment_files,
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!(
                                "Unable to determine number of segment files from segment file: {}",
                                file_name,
                            ),
                            error
                        ));
                    }
                };
                self.name = file_name_string[0..string_index].to_string();
                self.name_first_segment_number = 0;
            }
        }
        let mut segment_number: u16 = 1;
        let mut last_segment_file_name: String = String::new();
        let mut last_segment_file_size: u64 = 0;

        loop {
            let segment_file_name: String = match Self::get_segment_file_name(
                &self.name,
                segment_number,
                self.number_of_segment_files,
                &self.naming_schema,
                self.name_first_segment_number,
                self.name_suffix_size,
            ) {
                Ok(name) => name,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to determine file name of segment number: {}",
                            segment_number
                        )
                    );
                    return Err(error);
                }
            };
            let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];

            let data_stream: DataStreamReference =
                match file_resolver.get_data_stream(&path_components) {
                    Ok(Some(data_stream)) => data_stream,
                    Ok(None) => break,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open segment file: {}", segment_file_name)
                        );
                        return Err(error);
                    }
                };
            if last_segment_file_size > 0 && last_segment_file_size != self.segment_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch in size of segment file: {}",
                    last_segment_file_name
                )));
            }
            let segment_file_size: u64 = keramics_core::data_stream_get_size!(data_stream);

            if self.segment_size == 0 {
                self.segment_size = segment_file_size;
            }
            segment_number += 1;

            self.media_size += segment_file_size;

            last_segment_file_name = segment_file_name;
            last_segment_file_size = segment_file_size;
        }
        if self.number_of_segment_files == 0 {
            self.number_of_segment_files = segment_number - self.name_first_segment_number;
        }
        Ok(())
    }

    /// Reads media data based on the segment files.
    fn read_data_from_segment(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.current_offset;
        let safe_segment_number: u64 = media_offset / self.segment_size;
        let segment_offset: u64 = safe_segment_number * self.segment_size;
        let mut range_relative_offset: u64 = media_offset - segment_offset;
        let mut range_remainder_size: u64 = self.segment_size - range_relative_offset;

        if safe_segment_number >= u16::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid segment number: {} value out of bounds",
                safe_segment_number
            )));
        }
        let mut segment_number: u16 = (safe_segment_number + 1) as u16;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            if !self.segment_file_cache.contains(&segment_number) {
                let segment_file_name: String = match Self::get_segment_file_name(
                    &self.name,
                    segment_number,
                    self.number_of_segment_files,
                    &self.naming_schema,
                    self.name_first_segment_number,
                    self.name_suffix_size,
                ) {
                    Ok(name) => name,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to determine file name of segment number: {}",
                                segment_number
                            )
                        );
                        return Err(error);
                    }
                };
                let path_components: [PathComponent; 1] = [PathComponent::from(&segment_file_name)];
                let data_stream: DataStreamReference =
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
                self.segment_file_cache.insert(segment_number, data_stream);
            }
            let data_stream: &DataStreamReference =
                match self.segment_file_cache.get(&segment_number) {
                    Some(file) => file,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve segment file: {} from cache",
                            segment_number
                        )));
                    }
                };
            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            let read_count: usize = keramics_core::data_stream_read_at_position!(
                data_stream,
                &mut data[data_offset..data_end_offset],
                SeekFrom::Start(range_relative_offset)
            );
            if read_count == 0 {
                break;
            }
            data_offset += read_count;
            media_offset += read_count as u64;

            segment_number += 1;
            range_relative_offset = 0;
            range_remainder_size = self.segment_size;
        }
        Ok(data_offset)
    }
}

impl DataStream for SplitRawImage {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = match self.read_data_from_segment(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from segment");
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
            SeekFrom::End(relative_offset) => {
                match self.media_size.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid offset value out of bounds"
                        ));
                    }
                }
            }
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

    fn get_image() -> Result<SplitRawImage, ErrorTrace> {
        let mut image: SplitRawImage = SplitRawImage::new();

        let path_string: String = get_test_data_path("splitraw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.raw.000");
        image.open(&file_resolver, &file_name)?;

        Ok(image)
    }

    #[test]
    fn test_get_segment_file_name() -> Result<(), ErrorTrace> {
        let name: String = SplitRawImage::get_segment_file_name(
            &String::from("image"),
            1,
            99,
            &SplitRawNamingSchema::Alphabetic,
            0,
            2,
        )?;
        assert_eq!(name, "imageaa");

        let name: String = SplitRawImage::get_segment_file_name(
            &String::from("image"),
            1,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            1,
        )?;
        assert_eq!(name, "image1");

        let name: String = SplitRawImage::get_segment_file_name(
            &String::from("image."),
            1,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            3,
        )?;
        assert_eq!(name, "image.001");

        let name: String = SplitRawImage::get_segment_file_name(
            &String::from("image."),
            1,
            99,
            &SplitRawNamingSchema::XOfN,
            1,
            1,
        )?;
        assert_eq!(name, "image.1of99");

        let result: Result<String, ErrorTrace> = SplitRawImage::get_segment_file_name(
            &String::from("image"),
            0,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            1,
        );
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_segment_file_naming_schema() {
        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("imageaa");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Alphabetic));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image1");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image001");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.1of5");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::XOfN));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.0");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.1");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.000");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.001");
        assert_eq!(naming_schema, Some(SplitRawNamingSchema::Numeric));

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image");
        assert_eq!(naming_schema, None);

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("imageab");
        assert_eq!(naming_schema, None);

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image2");
        assert_eq!(naming_schema, None);

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.raw");
        assert_eq!(naming_schema, None);

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.2of5");
        assert_eq!(naming_schema, None);

        let naming_schema: Option<SplitRawNamingSchema> =
            SplitRawImage::get_segment_file_naming_schema("image.002");
        assert_eq!(naming_schema, None);
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = SplitRawImage::new();

        let path_string: String = get_test_data_path("splitraw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        let file_name: PathComponent = PathComponent::from("ext2.raw.000");
        image.open(&file_resolver, &file_name)?;

        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for read_segment_files
    // TODO: add tests for read_data_from_segment

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        image.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let size: u64 = image.get_size()?;
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let offset = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let result: Result<u64, ErrorTrace> = image.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image: SplitRawImage = get_image()?;
        image.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
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
        let mut image: SplitRawImage = get_image()?;
        image.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
