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
use keramics_types::ByteString;

use super::block_reader::QcowBlockReader;
use super::block_stream::QcowBlockStream;
use super::enums::{QcowCompressionMethod, QcowEncryptionMethod};
use super::file_header::QcowFileHeader;

/// QEMU Copy-On-Write (QCOW) file.
pub struct QcowFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    format_version: u32,

    /// Bytes per sector.
    pub(super) bytes_per_sector: u16,

    /// File header size.
    file_header_size: u32,

    /// Offset bit mask.
    offset_bit_mask: u64,

    /// Level 1 index bit shift.
    level1_index_bit_shift: u32,

    /// Level 1 table number of references.
    level1_table_number_of_references: u32,

    /// Level 1 table offset.
    level1_table_offset: u64,

    /// Level 2 index bit mask.
    level2_index_bit_mask: u64,

    /// Level 2 table number of references.
    level2_table_number_of_references: u64,

    /// Number of cluster block bits.
    number_of_cluster_block_bits: u32,

    /// Cluster block bit mask.
    cluster_block_bit_mask: u64,

    /// Cluster block size.
    cluster_block_size: u64,

    /// Compression bit shift.
    compression_bit_shift: u32,

    /// Compression bit mask.
    compression_bit_mask: u64,

    /// Compression flag bit mask.
    compression_flag_bit_mask: u64,

    /// Compression method.
    compression_method: QcowCompressionMethod,

    /// Encryption method.
    encryption_method: QcowEncryptionMethod,

    /// Backing file name.
    backing_file_name: Option<ByteString>,

    /// Backing file.
    backing_file: Option<Arc<QcowFile>>,

    /// Media size.
    pub(super) media_size: u64,
}

impl QcowFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format_version: 0,
            bytes_per_sector: 0,
            file_header_size: 0,
            offset_bit_mask: 0,
            level1_index_bit_shift: 0,
            level1_table_number_of_references: 0,
            level1_table_offset: 0,
            level2_index_bit_mask: 0,
            level2_table_number_of_references: 0,
            number_of_cluster_block_bits: 0,
            cluster_block_bit_mask: 0,
            cluster_block_size: 0,
            compression_bit_shift: 0,
            compression_bit_mask: 0,
            compression_flag_bit_mask: 0,
            compression_method: QcowCompressionMethod::Zlib,
            encryption_method: QcowEncryptionMethod::None,
            backing_file_name: None,
            backing_file: None,
            media_size: 0,
        }
    }

    /// Retrieves the backing file name.
    pub fn get_backing_file_name(&self) -> Option<&ByteString> {
        self.backing_file_name.as_ref()
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the compression method.
    pub fn get_compression_method(&self) -> &QcowCompressionMethod {
        &self.compression_method
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> Option<DataStreamReference> {
        match &self.data_stream {
            Some(data_stream) => {
                let backing_file_data_stream: Option<DataStreamReference> = match &self.backing_file
                {
                    Some(backing_file) => backing_file.get_data_stream(),
                    None => None,
                };
                Some(Arc::new(RwLock::new(QcowBlockStream::new(
                    QcowBlockReader::new(
                        data_stream,
                        self.offset_bit_mask,
                        self.level1_index_bit_shift,
                        self.level1_table_offset,
                        self.level1_table_number_of_references,
                        self.level2_index_bit_mask,
                        self.level2_table_number_of_references,
                        self.number_of_cluster_block_bits,
                        self.cluster_block_size,
                        self.compression_flag_bit_mask,
                        backing_file_data_stream,
                        self.media_size,
                    ),
                ))))
            }
            None => None,
        }
    }

    /// Retrieves the encryption method.
    pub fn get_encryption_method(&self) -> &QcowEncryptionMethod {
        &self.encryption_method
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> u32 {
        self.format_version
    }

    /// Retrieves the media size.
    pub fn get_media_size(&self) -> u64 {
        self.media_size
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_file_header(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file header.
    fn read_file_header(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut file_header: QcowFileHeader = QcowFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        self.format_version = file_header.format_version;
        self.bytes_per_sector = 512;
        self.file_header_size = file_header.header_size;
        self.number_of_cluster_block_bits = file_header.number_of_cluster_block_bits;
        self.media_size = file_header.media_size;

        self.encryption_method = match file_header.encryption_method {
            0 => QcowEncryptionMethod::None,
            1 => QcowEncryptionMethod::AesCbc128,
            2 => QcowEncryptionMethod::Luks,
            _ => QcowEncryptionMethod::Unknown,
        };
        if self.format_version == 3 {
            self.compression_method = match file_header.compression_method {
                0 => QcowCompressionMethod::Zlib,
                _ => QcowCompressionMethod::Unknown,
            };
        }
        if self.format_version == 1 {
            self.offset_bit_mask = 0x7fffffffffffffff;
            self.compression_flag_bit_mask = 1 << 63;
            self.compression_bit_shift = 63 - self.number_of_cluster_block_bits;
        } else {
            self.offset_bit_mask = 0x3fffffffffffffff;
            self.compression_flag_bit_mask = 1 << 62;
            self.compression_bit_shift = 62 - self.number_of_cluster_block_bits;
        }
        self.level1_index_bit_shift =
            self.number_of_cluster_block_bits + file_header.number_of_level2_table_bits;

        if self.level1_index_bit_shift > 63 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of level 1 index bit shift: {} value out of bounds",
                self.level1_index_bit_shift
            )));
        }
        self.level1_table_offset = file_header.level1_table_offset;

        self.level2_index_bit_mask =
            !(u64::MAX << (file_header.number_of_level2_table_bits as u64));
        self.cluster_block_bit_mask = !(u64::MAX << self.number_of_cluster_block_bits);
        self.compression_bit_mask = !(u64::MAX << self.compression_bit_shift);
        self.cluster_block_size = 1 << self.number_of_cluster_block_bits;

        self.level2_table_number_of_references =
            1 << (file_header.number_of_level2_table_bits as u64);

        let mut level1_table_number_of_references: u64 =
            file_header.level1_table_number_of_references as u64;

        if self.format_version == 1 {
            if self.cluster_block_size > u64::MAX / self.level2_table_number_of_references {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid level 2 table number of references: {} value out of bounds",
                    self.level2_table_number_of_references
                )));
            }
            level1_table_number_of_references = self
                .media_size
                .div_ceil(self.cluster_block_size * self.level2_table_number_of_references);

            if level1_table_number_of_references > u32::MAX as u64 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid level 1 table number of references: {} value out of bounds",
                    level1_table_number_of_references
                )));
            }
        }
        keramics_core::debug_trace_structure!(format!(
            concat!(
                "QcowFile {{\n",
                "    level1_table_number_of_references: {},\n",
                "    level2_table_number_of_references: {},\n",
                "    cluster_block_size: {},\n",
                "}}\n\n"
            ),
            level1_table_number_of_references,
            self.level2_table_number_of_references,
            self.cluster_block_size,
        ));
        self.level1_table_number_of_references = level1_table_number_of_references as u32;

        if file_header.backing_file_name_offset > 0 && file_header.backing_file_name_size > 0 {
            match self.read_backing_file_name(
                data_stream,
                file_header.backing_file_name_offset,
                file_header.backing_file_name_size,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read backing file name"
                    );
                    return Err(error);
                }
            }
        }
        if self.encryption_method != QcowEncryptionMethod::None {
            // TODO: handle encryption
            return Err(keramics_core::error_trace_new!(
                "Unsupported encryption method"
            ));
        }
        Ok(())
    }

    /// Reads the backing file name.
    fn read_backing_file_name(
        &mut self,
        data_stream: &DataStreamReference,
        backing_file_name_offset: u64,
        backing_file_name_size: u32,
    ) -> Result<(), ErrorTrace> {
        if backing_file_name_offset < self.file_header_size as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported backing file name offset: {}",
                backing_file_name_offset
            )));
        }
        if backing_file_name_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported backing file name size: {}",
                backing_file_name_size
            )));
        }
        let mut data: Vec<u8> = vec![0; backing_file_name_size as usize];

        keramics_core::data_stream_read_exact_at_position_with_debug_trace_data!(
            "QcowBackingFile",
            data_stream,
            &mut data,
            backing_file_name_size,
            SeekFrom::Start(backing_file_name_offset)
        );
        self.backing_file_name = Some(ByteString::from(data.as_slice()));

        Ok(())
    }

    /// Sets the backing file.
    pub fn set_backing_file(&mut self, backing_file: &Arc<QcowFile>) -> Result<(), ErrorTrace> {
        self.backing_file = Some(backing_file.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<QcowFile, ErrorTrace> {
        let mut file: QcowFile = QcowFile::new();

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_backing_file_name() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let backing_file_name: Option<&ByteString> = file.get_backing_file_name();
        assert_eq!(backing_file_name, None);

        Ok(())
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let bytes_per_sector: u16 = file.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_compression_method() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let compression_method: &QcowCompressionMethod = file.get_compression_method();
        assert_eq!(compression_method, &QcowCompressionMethod::Zlib);

        Ok(())
    }

    #[test]
    fn test_get_encryption_method() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let encryption_method: &QcowEncryptionMethod = file.get_encryption_method();
        assert_eq!(encryption_method, &QcowEncryptionMethod::None);

        Ok(())
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let format_version: u32 = file.get_format_version();
        assert_eq!(format_version, 3);

        Ok(())
    }

    #[test]
    fn test_get_media_size() -> Result<(), ErrorTrace> {
        let file: QcowFile = get_file()?;

        let media_size: u64 = file.get_media_size();
        assert_eq!(media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = QcowFile::new();

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_file_header() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = QcowFile::new();

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_file_header(&data_stream)?;

        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for read_backing_file_name
    // TODO: add tests for set_backing_file
}
