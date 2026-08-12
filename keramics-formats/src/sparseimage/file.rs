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

use keramics_core::mediator::Mediator;
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u32_be;

use crate::block_tree::BlockTree;
use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionType};

use super::block_range::SparseImageBlockRange;
use super::file_header::SparseImageFileHeader;

/// Mac OS sparse image (.sparseimage) file.
pub struct SparseImageFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Block tree.
    block_tree: BlockTree<SparseImageBlockRange>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Band size.
    band_size: u32,

    /// Encryption type.
    encryption_type: Option<CdsaEncrEncryptionType>,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,

    /// The current offset.
    current_offset: u64,

    /// Media size.
    media_size: u64,
}

impl SparseImageFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            block_tree: BlockTree::<SparseImageBlockRange>::new(0, 0, 0),
            bytes_per_sector: 0,
            band_size: 0,
            encryption_type: None,
            is_locked: false,
            current_offset: 0,
            media_size: 0,
        }
    }

    /// Retrieves the block size.
    pub fn get_block_size(&self) -> u32 {
        self.band_size
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the encryption type.
    pub fn get_encryption_type(&self) -> Option<&CdsaEncrEncryptionType> {
        self.encryption_type.as_ref()
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Determines if the (encrypted) image is locked.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Reads a file from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
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
                        "Unable to open encrypted container",
                    );
                    return Err(error);
                }
            }
            self.encryption_type = Some(cdsaencr_container.get_encryption_type().clone());
            self.is_locked = true;
        }
        if !self.is_locked {
            match self.read_header_block(data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read header block");
                    return Err(error);
                }
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the header block containing the file header and bands array.
    fn read_header_block(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut data: [u8; 4096] = [0; 4096];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        keramics_core::debug_trace_data_and_structure!(
            "SparseImageFileHeader",
            0,
            &data[0..64],
            64,
            SparseImageFileHeader::debug_read_data(&data)
        );
        let mut file_header: SparseImageFileHeader = SparseImageFileHeader::new();

        match file_header.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        let number_of_bands: u32 = file_header
            .number_of_sectors
            .div_ceil(file_header.sectors_per_band);

        if number_of_bands > (4096 - 64) / 4 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of bands: {} value out of bounds",
                number_of_bands
            )));
        }
        let array_data_size: usize = (number_of_bands as usize) * 4;
        let array_data_end_offset: usize = 64 + array_data_size;

        keramics_core::debug_trace_data!(
            "SparseImageBandNumbersArray",
            64,
            &data[64..array_data_end_offset],
            array_data_size
        );
        if file_header.sectors_per_band > u32::MAX / 512 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid sectors per band: {} value out of bounds",
                file_header.sectors_per_band
            )));
        }
        self.bytes_per_sector = 512;
        self.band_size = file_header.sectors_per_band * (self.bytes_per_sector as u32);
        self.media_size = (file_header.number_of_sectors as u64) * (self.bytes_per_sector as u64);

        let block_tree_size: u64 = (number_of_bands as u64) * (self.band_size as u64);

        self.block_tree = BlockTree::<SparseImageBlockRange>::new(
            block_tree_size,
            file_header.sectors_per_band as u64,
            512,
        );
        let mut data_offset: usize = 64;

        let mediator = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print("SparseImageBandNumbersArray {\n");
            mediator.debug_print("    band_numbers: [\n");
        }
        for array_index in 0..number_of_bands {
            let band_number: u32 = bytes_to_u32_be!(data, data_offset);
            data_offset += 4;

            if mediator.debug_output {
                if array_index % 16 == 0 {
                    mediator.debug_print(format!("        {}", band_number));
                } else if array_index % 16 == 15 {
                    mediator.debug_print(format!(", {},\n", band_number));
                } else {
                    mediator.debug_print(format!(", {}", band_number));
                }
            }
            if band_number == 0 {
                continue;
            }
            let band_media_offset: u64 = ((band_number - 1) as u64) * (self.band_size as u64);
            let band_data_offset: u64 = 4096 + ((array_index as u64) * (self.band_size as u64));

            let block_range: SparseImageBlockRange = SparseImageBlockRange::new(band_data_offset);
            match self.block_tree.insert_value(
                band_media_offset,
                self.band_size as u64,
                block_range,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to insert block range into block tree"
                    );
                    return Err(error);
                }
            }
        }
        if mediator.debug_output {
            if number_of_bands % 16 != 0 {
                mediator.debug_print("\n");
            }
            mediator.debug_print("    ],\n");
            mediator.debug_print("}\n\n");
        }
        Ok(())
    }

    /// Reads media data based on the block ranges in the block tree.
    fn read_data_from_bands(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.current_offset;
        let band_number: u64 = media_offset / (self.band_size as u64);
        let band_offset: u64 = band_number * (self.band_size as u64);
        let mut range_relative_offset: u64 = media_offset - band_offset;
        let mut range_remainder_size: u64 = (self.band_size as u64) - range_relative_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;

            let range_read_count: usize = match self.block_tree.get_value(media_offset) {
                Ok(Some(block_range)) => {
                    let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                        Some(data_stream) => data_stream,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing data stream"));
                        }
                    };
                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(block_range.data_offset + range_relative_offset)
                    );
                    read_count
                }
                Ok(None) => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                            media_offset, media_offset
                        )
                    );
                    return Err(error);
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;

            range_relative_offset = 0;
            range_remainder_size = self.band_size as u64;
        }
        Ok(data_offset)
    }

    /// Unlocks a locked (encrypted) file.
    pub fn unlock(&mut self, credentials: &[CdsaEncrCredential]) -> Result<bool, ErrorTrace> {
        if !self.is_locked {
            return Ok(true);
        }
        let data_stream: &DataStreamReference = match &self.data_stream {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

        match cdsaencr_container.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open encrypted container",);
                return Err(error);
            }
        }
        let result: bool = match cdsaencr_container.unlock(credentials) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Failed to unlock encrypted container",
                );
                return Err(error);
            }
        };
        if result {
            let data_stream: DataStreamReference = Arc::new(RwLock::new(cdsaencr_container));

            match self.read_header_block(&data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read header block");
                    return Err(error);
                }
            }
            self.data_stream = Some(data_stream);
            self.is_locked = false;
        }
        Ok(!self.is_locked)
    }
}

impl DataStream for SparseImageFile {
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

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file(path_string: &str) -> Result<SparseImageFile, ErrorTrace> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let test_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_block_size() -> Result<(), ErrorTrace> {
        let file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let block_size: u32 = file.get_block_size();
        assert_eq!(block_size, 1048576);

        Ok(())
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let bytes_per_sector: u16 = file.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_encryption_type() -> Result<(), ErrorTrace> {
        let file: SparseImageFile = get_file("sparseimage/hfsplus_aes128.sparseimage")?;

        let encryption_type: &CdsaEncrEncryptionType = file.get_encryption_type().unwrap();
        assert_eq!(encryption_type.method, 0x80000001);
        assert_eq!(encryption_type.mode, 5);
        assert_eq!(encryption_type.key_size, 16);

        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let media_size: u64 = file.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_is_locked() -> Result<(), ErrorTrace> {
        let file: SparseImageFile = get_file("sparseimage/hfsplus_aes128.sparseimage")?;

        let is_locked: bool = file.is_locked();
        assert_eq!(is_locked, true);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.band_size, 1048576);
        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_header_block() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = SparseImageFile::new();

        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_header_block(&data_stream)?;

        assert_eq!(file.bytes_per_sector, 512);
        assert_eq!(file.band_size, 1048576);
        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    // TODO: add test for read_data_from_bands

    #[test]
    fn test_unlock() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus_aes128.sparseimage")?;

        assert_eq!(file.is_locked, true);

        let credentials: Vec<CdsaEncrCredential> =
            vec![CdsaEncrCredential::Passphrase(b"KeRaMiCs".to_vec())];
        file.unlock(&credentials)?;

        assert_eq!(file.is_locked, false);

        Ok(())
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        file.seek(SeekFrom::Start(1024))?;

        let offset: u64 = file.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let size: u64 = file.get_size()?;
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let offset: u64 = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let offset: u64 = file.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let offset = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let result: Result<u64, ErrorTrace> = file.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;

        let offset: u64 = file.seek(SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;
        file.seek(SeekFrom::Start(1024))?;

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
    fn test_seek_and_read_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut file: SparseImageFile = get_file("sparseimage/hfsplus.sparseimage")?;
        file.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
