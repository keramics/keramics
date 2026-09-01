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
use keramics_types::bytes_to_u32_be;

#[cfg(feature = "debug-trace")]
use {keramics_core::DebugTrace, keramics_core::formatters::debug_format_array};

use crate::cdsaencr::constants::*;
use crate::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential, CdsaEncrEncryptionType};

use super::block_range::{SparseImageBlockRange, SparseImageBlockRangeType};
use super::block_reader::SparseImageBlockReader;
use super::block_stream::SparseImageBlockStream;
use super::file_header::SparseImageFileHeader;

/// Mac OS sparse image (.sparseimage) file.
pub struct SparseImageFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Band size.
    band_size: u32,

    /// Block ranges.
    block_ranges: Vec<SparseImageBlockRange>,

    /// Encryption type.
    encryption_type: Option<CdsaEncrEncryptionType>,

    /// Value to indicate the (encrypted) image is locked.
    is_locked: bool,

    /// Media size.
    media_size: u64,
}

impl SparseImageFile {
    /// Creates a file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            band_size: 0,
            block_ranges: Vec::new(),
            encryption_type: None,
            is_locked: false,
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

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.data_stream {
            Some(data_stream) => Some(Arc::new(RwLock::new(SparseImageBlockStream::new(
                SparseImageBlockReader::new(
                    data_stream,
                    self.band_size,
                    &self.block_ranges,
                    self.media_size,
                ),
            )))),
            None => None,
        }
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
            array_data_size,
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

        #[cfg(feature = "debug-trace")]
        DebugTrace::static_scope(|debug_trace| {
            let band_numbers: Vec<u32> = data[64..array_data_end_offset]
                .chunks_exact(4)
                .map(|chunk| u32::from_be_bytes(chunk.try_into().unwrap()))
                .collect();

            debug_trace.print_start("SparseImageBandNumbersArray");
            debug_trace.print_field(
                "band_numbers",
                debug_format_array(
                    &band_numbers
                        .iter()
                        .map(|&element| element.to_string())
                        .collect::<Vec<String>>()
                        .as_slice(),
                ),
            );
            debug_trace.print_end();
        });
        let mut block_ranges: Vec<SparseImageBlockRange> = Vec::new();

        for (array_index, chunk) in data[64..array_data_end_offset].chunks_exact(4).enumerate() {
            let band_number: u32 = bytes_to_u32_be!(chunk, 0);

            if band_number == 0 {
                continue;
            }
            let band_logical_offset: u64 = ((band_number - 1) as u64) * (self.band_size as u64);

            block_ranges.push(SparseImageBlockRange::new(
                band_logical_offset,
                array_index as u32,
                1,
                SparseImageBlockRangeType::InFile,
            ));
        }
        block_ranges.sort_by_key(|block_range| block_range.logical_offset);

        let mut media_offset: u64 = 0;

        for block_range in block_ranges.drain(..) {
            if media_offset < block_range.logical_offset {
                let range_size: u64 = block_range.logical_offset - media_offset;
                let number_of_bands: u64 = range_size / (self.band_size as u64);

                self.block_ranges.push(SparseImageBlockRange::new(
                    media_offset,
                    0,
                    number_of_bands as u32,
                    SparseImageBlockRangeType::Sparse,
                ));
                media_offset += range_size;
            }
            self.block_ranges.push(block_range);

            media_offset += self.band_size as u64;
        }
        Ok(())
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
                keramics_core::error_trace_add_frame!(error, "Unable to open encrypted container");
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

    // TODO: add tests for get_data_stream

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
}
