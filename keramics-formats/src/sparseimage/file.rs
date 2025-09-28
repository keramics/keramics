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

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference};
use keramics_types::bytes_to_u32_be;

use crate::block_tree::BlockTree;

use super::block_range::SparseImageBlockRange;
use super::file_header::SparseImageFileHeader;

/// Mac OS sparse image (.sparseimage) file.
pub struct SparseImageFile {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Block tree.
    block_tree: BlockTree<SparseImageBlockRange>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Block size.
    pub block_size: u32,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl SparseImageFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
            block_tree: BlockTree::<SparseImageBlockRange>::new(0, 0, 0),
            bytes_per_sector: 0,
            block_size: 0,
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        self.read_header_block(data_stream)?;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the header block containing the file header and bands array.
    fn read_header_block(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        let mut data: [u8; 4096] = [0; 4096];

        match data_stream.write() {
            Ok(mut data_stream) => {
                data_stream.read_exact_at_position(&mut data, io::SeekFrom::Start(0))?
            }
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        let mut file_header: SparseImageFileHeader = SparseImageFileHeader::new();

        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "SparseImageFileHeader data of size: 64 at offset: 0 (0x00000000)\n",
            ));
            self.mediator.debug_print_data(&data[0..64], true);
            self.mediator
                .debug_print(SparseImageFileHeader::debug_read_data(&data[0..64]));
        }
        file_header.read_data(&data[0..64])?;

        let number_of_bands: u32 = file_header
            .number_of_sectors
            .div_ceil(file_header.sectors_per_band);

        if number_of_bands > (4096 - 64) / 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid number of bands: {} value out of bounds",
                    number_of_bands
                ),
            ));
        }
        if self.mediator.debug_output {
            let data_size: usize = (number_of_bands as usize) * 4;
            let data_end_offset: usize = 64 + data_size;

            self.mediator.debug_print(format!(
                "SparseImageBandNumbersArray data of size: {} at offset: 64 (0x00000040)\n",
                data_size,
            ));
            self.mediator
                .debug_print_data(&data[64..data_end_offset], true);
        }
        if file_header.sectors_per_band > u32::MAX / 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid sectors per band: {} value out of bounds",
                    file_header.sectors_per_band
                ),
            ));
        }
        self.bytes_per_sector = 512;
        self.block_size = file_header.sectors_per_band * (self.bytes_per_sector as u32);
        self.media_size = (file_header.number_of_sectors as u64) * (self.bytes_per_sector as u64);

        let block_tree_size: u64 = (number_of_bands as u64) * (self.block_size as u64);

        self.block_tree = BlockTree::<SparseImageBlockRange>::new(
            block_tree_size,
            file_header.sectors_per_band as u64,
            512,
        );
        let mut data_offset: usize = 64;

        if self.mediator.debug_output {
            self.mediator
                .debug_print(format!("SparseImageBandNumbersArray {{\n"));
            self.mediator.debug_print(format!("    band_numbers: [\n"));
        }
        for array_index in 0..number_of_bands {
            let band_number: u32 = bytes_to_u32_be!(data, data_offset);
            data_offset += 4;

            if self.mediator.debug_output {
                if array_index % 16 == 0 {
                    self.mediator
                        .debug_print(format!("        {}", band_number));
                } else if array_index % 16 == 15 {
                    self.mediator.debug_print(format!(", {},\n", band_number));
                } else {
                    self.mediator.debug_print(format!(", {}", band_number));
                }
            }
            if band_number == 0 {
                continue;
            }
            let block_media_offset: u64 = ((band_number - 1) as u64) * (self.block_size as u64);
            let band_data_offset: u64 = 4096 + ((array_index as u64) * (self.block_size as u64));

            let block_range: SparseImageBlockRange = SparseImageBlockRange::new(band_data_offset);
            match self.block_tree.insert_value(
                block_media_offset,
                self.block_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            };
        }
        if self.mediator.debug_output {
            if number_of_bands % 16 != 0 {
                self.mediator.debug_print(format!("\n"));
            }
            self.mediator.debug_print(format!("    ],\n"));
            self.mediator.debug_print(format!("}}\n\n"));
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_bands(&mut self, data: &mut [u8]) -> io::Result<usize> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;
        let block_number: u64 = media_offset / (self.block_size as u64);
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

            let block_tree_value: Option<&SparseImageBlockRange> =
                self.block_tree.get_value(media_offset);

            let range_read_count: usize = match block_tree_value {
                Some(block_range) => match self.data_stream.as_ref() {
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
                        ));
                    }
                },
                None => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;

            range_relative_offset = 0;
            range_remainder_size = self.block_size as u64;
        }
        Ok(data_offset)
    }
}

impl Read for SparseImageFile {
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

impl Seek for SparseImageFile {
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

impl DataStream for SparseImageFile {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.media_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_file() -> io::Result<SparseImageFile> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_read_data_stream() -> io::Result<()> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.block_size, 1048576);
        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_header_block() -> io::Result<()> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let data_stream: DataStreamReference =
            open_os_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        file.read_header_block(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.block_size, 1048576);
        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    // TODO: add test for read_data_from_bands

    #[test]
    fn test_seek_from_start() -> io::Result<()> {
        let mut file: SparseImageFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> io::Result<()> {
        let mut file: SparseImageFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> io::Result<()> {
        let mut file: SparseImageFile = get_file()?;

        let offset = file.seek(io::SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(io::SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> io::Result<()> {
        let mut file: SparseImageFile = get_file()?;

        let offset: u64 = file.seek(io::SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> io::Result<()> {
        let mut file: SparseImageFile = get_file()?;
        file.seek(io::SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x00, 0x53, 0x46, 0x48, 0x00, 0x00, 0xaa, 0x11, 0xaa, 0x11, 0x00, 0x30, 0x65, 0x43,
            0xec, 0xac, 0x48, 0x6f, 0x33, 0x32, 0x41, 0x86, 0x9c, 0x40, 0x86, 0x15, 0x80, 0x36,
            0xc8, 0xec, 0x25, 0x7b, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xd7, 0x1f,
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
        let mut file: SparseImageFile = get_file()?;
        file.seek(io::SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
