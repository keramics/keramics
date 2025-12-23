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

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use crate::fake_file_resolver::FakeFileResolver;
use crate::file_resolver::FileResolverReference;
use crate::lru_cache::LruCache;
use crate::path_component::PathComponent;
use crate::plist::XmlPlist;

/// Mac OS sparse bundle (.sparsebundle) storage media image.
pub struct SparseBundleImage {
    /// Mediator.
    mediator: MediatorReference,

    /// File resolver.
    file_resolver: FileResolverReference,

    /// Block size.
    pub block_size: u32,

    /// Band file cache.
    band_file_cache: LruCache<u64, DataStreamReference>,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    pub media_size: u64,
}

impl SparseBundleImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            file_resolver: FileResolverReference::new(Box::new(FakeFileResolver::new())),
            block_size: 0,
            band_file_cache: LruCache::new(16),
            current_offset: 0,
            media_size: 0,
        }
    }

    /// Opens a storage media image.
    pub fn open(&mut self, file_resolver: &FileResolverReference) -> Result<(), ErrorTrace> {
        match self.read_info_plist(&file_resolver, "Info.plist") {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read Info.plist");
                return Err(error);
            }
        }
        self.file_resolver = file_resolver.clone();

        Ok(())
    }

    /// Reads an Info.plist or Info.bckup file.
    fn read_info_plist(
        &mut self,
        file_resolver: &FileResolverReference,
        file_name: &str,
    ) -> Result<(), ErrorTrace> {
        let path_components: [PathComponent; 1] = [PathComponent::from(file_name)];

        let data_stream: DataStreamReference = match file_resolver.get_data_stream(&path_components)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    file_name
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open file: {}", file_name)
                );
                return Err(error);
            }
        };
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        if data_stream_size == 0 || data_stream_size > 65536 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut data: Vec<u8> = vec![0; data_stream_size as usize];

        keramics_core::data_stream_read_at_position!(data_stream, &mut data, SeekFrom::Start(0));

        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "XML plist data of size: {} at offset: 0 (0x00000000)\n",
                data_stream_size,
            ));
            self.mediator.debug_print_data(&data, true);
        }
        let string: String = match String::from_utf8(data) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert XML plist data into UTF-8 string",
                    error
                ));
            }
        };
        let mut xml_plist: XmlPlist = XmlPlist::new();

        match xml_plist.parse(string.as_str()) {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to parse XML plist",
                    error
                ));
            }
        }
        match xml_plist
            .root_object
            .get_string_by_key("CFBundleInfoDictionaryVersion")
        {
            Some(string) => {
                if string != "6.0" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported CFBundleInfoDictionaryVersion: {}",
                        string
                    )));
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve CFBundleInfoDictionaryVersion value"
                ));
            }
        }
        match xml_plist
            .root_object
            .get_string_by_key("diskimage-bundle-type")
        {
            Some(string) => {
                if string != "com.apple.diskimage.sparsebundle" {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported diskimage-bundle-type: {}",
                        string
                    )));
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve diskimage-bundle-type value"
                ));
            }
        }
        match xml_plist.root_object.get_integer_by_key("band-size") {
            Some(integer) => {
                if *integer == 0 || *integer > u32::MAX as i64 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid band-size: {} value out of bounds",
                        *integer
                    )));
                }
                self.block_size = *integer as u32;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve band-size value"
                ));
            }
        }
        match xml_plist.root_object.get_integer_by_key("size") {
            Some(integer) => {
                if *integer == 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid size: {} value out of bounds",
                        *integer
                    )));
                }
                self.media_size = *integer as u64;
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to retrieve size value"
                ));
            }
        }
        Ok(())
    }

    /// Reads media data from the bands based on the block size.
    fn read_data_from_bands(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.current_offset;
        let mut block_number: u64 = media_offset / (self.block_size as u64);
        let block_offset: u64 = block_number * (self.block_size as u64);
        let mut range_relative_offset: u64 = media_offset - block_offset;
        let mut range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            if !self.band_file_cache.contains(&block_number) {
                let band_file_name: String = format!("{:x}", block_number);

                let path_components: [PathComponent; 2] = [
                    PathComponent::from("bands"),
                    PathComponent::from(&band_file_name),
                ];
                let data_stream: DataStreamReference =
                    match self.file_resolver.get_data_stream(&path_components) {
                        Ok(Some(data_stream)) => data_stream,
                        Ok(None) => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Missing band file: {}",
                                band_file_name
                            )));
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open band file: {}", band_file_name)
                            );
                            return Err(error);
                        }
                    };
                self.band_file_cache.insert(block_number, data_stream);
            }
            let data_stream: &DataStreamReference = match self.band_file_cache.get(&block_number) {
                Some(file) => file,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unable to retrieve band file: bands/{:x} from cache",
                        block_number
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

            block_number += 1;
            range_relative_offset = 0;
            range_remainder_size = self.block_size as u64;
        }
        Ok(data_offset)
    }
}

impl DataStream for SparseBundleImage {
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
        let read_count: usize = match self.read_data_from_bands(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from bands");
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

    fn get_image() -> Result<SparseBundleImage, ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.open(&file_resolver)?;

        Ok(image)
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.open(&file_resolver)?;

        assert_eq!(image.block_size, 8388608);
        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_info_plist() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = SparseBundleImage::new();

        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;
        image.read_info_plist(&file_resolver, "Info.plist")?;

        assert_eq!(image.block_size, 8388608);
        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for read_data_from_bands

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        image.seek(SeekFrom::Start(1024))?;

        let offset: u64 = image.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let size: u64 = image.get_size()?;
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let offset = image.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let result: Result<u64, ErrorTrace> = image.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;
        image.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x53, 0x46, 0x48, 0x00, 0x00, 0xaa, 0x11, 0xaa, 0x11, 0x00, 0x30, 0x65, 0x43,
            0xec, 0xac, 0x89, 0xc9, 0xaf, 0xca, 0xee, 0xbd, 0x3f, 0x4a, 0xb3, 0xa6, 0x12, 0x85,
            0x86, 0x38, 0xf8, 0xa6, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd7, 0x1f,
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
    fn test_seek_and_read_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut image: SparseBundleImage = get_image()?;
        image.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
