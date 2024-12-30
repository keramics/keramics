/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::formats::plist::XmlPlist;
use crate::mediator::{Mediator, MediatorReference};
use crate::types::SharedValue;
use crate::vfs::{
    VfsDataStreamReference, VfsFileSystem, VfsFileSystemReference, VfsPath, VfsPathReference,
    VfsPathType,
};

/// Mac OS sparse bundle (.sparsebundle) storage media image.
pub struct SparseBundleImage {
    /// Mediator.
    mediator: MediatorReference,

    /// File system.
    file_system: VfsFileSystemReference,

    /// Path.
    path: VfsPathReference,

    /// Block size.
    pub block_size: u32,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl SparseBundleImage {
    /// Creates a new storage media image.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            file_system: SharedValue::none(),
            path: VfsPath::new(VfsPathType::Fake, "/", None),
            block_size: 0,
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Opens a storage media image.
    pub fn open(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        match file_system.with_write_lock() {
            Ok(file_system) => {
                self.read_info_plist(&file_system, path)?;

                let directory_name: &str = file_system.get_directory_name(&path.location);
                self.path = VfsPath::new_from_path(&path, directory_name);
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.file_system = file_system.clone();

        Ok(())
    }

    /// Reads Info.plist or Info.bckup.
    fn read_info_plist(
        &mut self,
        file_system: &VfsFileSystem,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        let result: Option<VfsDataStreamReference> = file_system.open_data_stream(path, None)?;

        let data_stream: VfsDataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.location),
                ))
            }
        };
        let string: String = match data_stream.with_write_lock() {
            Ok(mut data_stream) => {
                let data_stream_size: u64 = data_stream.get_size()?;

                if data_stream_size == 0 || data_stream_size > 65536 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported Info.plist file size",
                    ));
                }
                let mut data: Vec<u8> = vec![0; data_stream_size as usize];
                data_stream.read_at_position(&mut data, io::SeekFrom::Start(0))?;

                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "Info.plist data of size: {} at offset: 0 (0x00000000)\n",
                        data_stream_size,
                    ));
                    self.mediator.debug_print_data(&data, true);
                }
                match String::from_utf8(data) {
                    Ok(string) => string,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                }
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut xml_plist: XmlPlist = XmlPlist::new();
        xml_plist.parse(string.as_str())?;

        let version: &String = match xml_plist
            .root_object
            .get_string_by_key("CFBundleInfoDictionaryVersion")
        {
            Some(string) => string,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to retrieve CFBundleInfoDictionaryVersion value from Info.plist",
                ));
            }
        };
        if version != "6.0" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported CFBundleInfoDictionaryVersion: {}", version),
            ));
        }
        let bundle_type: &String = match xml_plist
            .root_object
            .get_string_by_key("diskimage-bundle-type")
        {
            Some(string) => string,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to retrieve diskimage-bundle-type value from Info.plist",
                ));
            }
        };
        if bundle_type != "com.apple.diskimage.sparsebundle" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported diskimage-bundle-type: {}", bundle_type),
            ));
        }
        let band_size: &i64 = match xml_plist.root_object.get_integer_by_key("band-size") {
            Some(integer) => integer,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to retrieve band-size value from Info.plist",
                ));
            }
        };
        if *band_size <= 0 || *band_size > u32::MAX as i64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid band-size: {} value out of bounds", *band_size),
            ));
        }
        self.block_size = *band_size as u32;

        let size: &i64 = match xml_plist.root_object.get_integer_by_key("size") {
            Some(integer) => integer,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unable to retrieve size value from Info.plist",
                ));
            }
        };
        if *size <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid size: {} value out of bounds", *size),
            ));
        }
        self.media_size = *size as u64;

        Ok(())
    }

    /// Reads media data from the bands based on the block size.
    fn read_data_from_bands(&mut self, data: &mut [u8]) -> io::Result<usize> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;
        let mut block_number: u64 = media_offset / (self.block_size as u64);
        let block_offset: u64 = block_number * (self.block_size as u64);
        let mut range_relative_offset: u64 = media_offset - block_offset;
        let mut range_remainder_size: u64 = (self.block_size as u64) - range_relative_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            // TODO: add file_system.join function
            let band_file_location: String =
                format!("{}/bands/{:x}", self.path.location, block_number);
            let band_file_path: VfsPathReference =
                VfsPath::new_from_path(&self.path, band_file_location.as_str());

            let result: Option<VfsDataStreamReference> = match self.file_system.with_write_lock() {
                Ok(file_system) => file_system.open_data_stream(&band_file_path, None)?,
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            let data_stream: VfsDataStreamReference = match result {
                Some(data_stream) => data_stream,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("No such file: {}", band_file_path.location),
                    ))
                }
            };
            let range_read_count: usize = match data_stream.with_write_lock() {
                Ok(mut data_stream) => data_stream.read_at_position(
                    &mut data[data_offset..data_end_offset],
                    io::SeekFrom::Start(range_relative_offset),
                )?,
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;

            block_number += 1;
            range_relative_offset = 0;
            range_remainder_size = self.block_size as u64;
        }
        Ok(data_offset)
    }
}

impl Read for SparseBundleImage {
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
        let read_count: usize = self.read_data_from_bands(&mut buf[..read_size])?;

        self.media_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for SparseBundleImage {
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

    use crate::vfs::{VfsContext, VfsPathType};

    fn get_image() -> io::Result<SparseBundleImage> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut image: SparseBundleImage = SparseBundleImage::new();

        let vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/sparsebundle/hfsplus.sparsebundle/Info.plist",
            None,
        );
        image.open(&vfs_file_system, &vfs_path)?;

        Ok(image)
    }

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let mut image: SparseBundleImage = SparseBundleImage::new();

        let vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/sparsebundle/hfsplus.sparsebundle/Info.plist",
            None,
        );
        image.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(image.block_size, 8388608);
        assert_eq!(image.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, image.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;

        let offset = image.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = image.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;

        let offset: u64 = image.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, image.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;
        image.seek(io::SeekFrom::Start(1024))?;

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
    fn test_seek_and_read_beyond_media_size() -> io::Result<()> {
        let mut image: SparseBundleImage = get_image()?;
        image.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = image.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
