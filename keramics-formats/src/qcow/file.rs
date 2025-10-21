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
use std::sync::{Arc, RwLock};

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::ByteString;

use crate::block_tree::BlockTree;

use super::block_range::{QcowBlockRange, QcowBlockRangeType};
use super::cluster_table::{QcowClusterTable, QcowClusterTableEntry};
use super::enums::{QcowCompressionMethod, QcowEncryptionMethod};
use super::file_header_common::QcowFileHeaderCommon;
use super::file_header_v1::QcowFileHeaderV1;
use super::file_header_v2::QcowFileHeaderV2;
use super::file_header_v3::QcowFileHeaderV3;

/// QEMU Copy-On-Write (QCOW) file.
pub struct QcowFile {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format version.
    pub format_version: u32,

    /// File header size.
    file_header_size: u32,

    /// Offset bit mask.
    offset_bit_mask: u64,

    /// Level 1 index bit shift.
    level1_index_bit_shift: u8,

    /// Level 1 cluster table.
    level1_cluster_table: QcowClusterTable,

    /// Level 2 index bit mask.
    level2_index_bit_mask: u64,

    /// Level 2 table number of references.
    level2_table_number_of_references: u64,

    /// Level 2 cluster table.
    level2_cluster_table: QcowClusterTable,

    /// Number of cluster block bits.
    number_of_cluster_block_bits: u8,

    /// Cluster block bit mask.
    cluster_block_bit_mask: u64,

    /// Cluster block size.
    cluster_block_size: u64,

    /// Compression bit shift.
    compression_bit_shift: u8,

    /// Compression bit mask.
    compression_bit_mask: u64,

    /// Compression flag bit mask.
    compression_flag_bit_mask: u64,

    /// Compression method.
    pub compression_method: QcowCompressionMethod,

    /// Encryption method.
    pub encryption_method: QcowEncryptionMethod,

    /// Block tree.
    block_tree: BlockTree<QcowBlockRange>,

    /// Backing file name.
    backing_file_name: Option<ByteString>,

    /// Backing file.
    backing_file: Option<Arc<RwLock<QcowFile>>>,

    /// Media size.
    pub media_size: u64,

    /// Media offset.
    media_offset: u64,
}

impl QcowFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
            format_version: 0,
            file_header_size: 0,
            offset_bit_mask: 0,
            level1_index_bit_shift: 0,
            level1_cluster_table: QcowClusterTable::new(),
            level2_index_bit_mask: 0,
            level2_table_number_of_references: 0,
            level2_cluster_table: QcowClusterTable::new(),
            number_of_cluster_block_bits: 0,
            cluster_block_bit_mask: 0,
            cluster_block_size: 0,
            compression_bit_shift: 0,
            compression_bit_mask: 0,
            compression_flag_bit_mask: 0,
            compression_method: QcowCompressionMethod::Zlib,
            encryption_method: QcowEncryptionMethod::None,
            block_tree: BlockTree::<QcowBlockRange>::new(0, 0, 0),
            backing_file_name: None,
            backing_file: None,
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Retrieves the backing file name.
    pub fn get_backing_file_name(&self) -> Option<&ByteString> {
        self.backing_file_name.as_ref()
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
        let mut data: [u8; 112] = [0; 112];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "QcowFileHeader data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        let mut file_header_common: QcowFileHeaderCommon = QcowFileHeaderCommon::new();

        match file_header_common.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read common file header");
                return Err(error);
            }
        }
        self.format_version = file_header_common.format_version;

        let backing_file_name_offset: u64;
        let backing_file_name_size: u32;
        let number_of_level2_table_bits: u8;
        let level1_table_offset: u64;

        let mut level1_table_number_of_references: u64 = 0;

        if self.format_version == 1 {
            let mut file_header_v1: QcowFileHeaderV1 = QcowFileHeaderV1::new();

            match file_header_v1.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read version 1 file header"
                    );
                    return Err(error);
                }
            }
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(QcowFileHeaderV1::debug_read_data(&data));
            }
            self.media_size = file_header_v1.media_size;
            self.number_of_cluster_block_bits = file_header_v1.number_of_cluster_block_bits;
            self.encryption_method = match file_header_v1.encryption_method {
                0 => QcowEncryptionMethod::None,
                1 => QcowEncryptionMethod::AesCbc128,
                2 => QcowEncryptionMethod::Luks,
                _ => QcowEncryptionMethod::Unknown,
            };
            self.file_header_size = 48;

            backing_file_name_offset = file_header_v1.backing_file_name_offset;
            backing_file_name_size = file_header_v1.backing_file_name_size;
            number_of_level2_table_bits = file_header_v1.number_of_level2_table_bits;
            level1_table_offset = file_header_v1.level1_table_offset;
        } else if self.format_version == 2 {
            let mut file_header_v2: QcowFileHeaderV2 = QcowFileHeaderV2::new();

            match file_header_v2.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read version 2 file header"
                    );
                    return Err(error);
                }
            }
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(QcowFileHeaderV2::debug_read_data(&data));
            }
            self.media_size = file_header_v2.media_size;
            self.number_of_cluster_block_bits = file_header_v2.number_of_cluster_block_bits as u8;
            self.encryption_method = match file_header_v2.encryption_method {
                0 => QcowEncryptionMethod::None,
                1 => QcowEncryptionMethod::AesCbc128,
                2 => QcowEncryptionMethod::Luks,
                _ => QcowEncryptionMethod::Unknown,
            };
            self.file_header_size = 72;

            backing_file_name_offset = file_header_v2.backing_file_name_offset;
            backing_file_name_size = file_header_v2.backing_file_name_size;
            number_of_level2_table_bits = self.number_of_cluster_block_bits - 3;
            level1_table_offset = file_header_v2.level1_table_offset;
            level1_table_number_of_references =
                file_header_v2.level1_table_number_of_references as u64;
        } else if self.format_version == 3 {
            let mut file_header_v3: QcowFileHeaderV3 = QcowFileHeaderV3::new();

            match file_header_v3.read_data(&data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read version 3 file header"
                    );
                    return Err(error);
                }
            }
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(QcowFileHeaderV3::debug_read_data(&data));
            }
            self.media_size = file_header_v3.media_size;
            self.number_of_cluster_block_bits = file_header_v3.number_of_cluster_block_bits as u8;
            self.encryption_method = match file_header_v3.encryption_method {
                0 => QcowEncryptionMethod::None,
                1 => QcowEncryptionMethod::AesCbc128,
                2 => QcowEncryptionMethod::Luks,
                _ => QcowEncryptionMethod::Unknown,
            };
            self.compression_method = match file_header_v3.compression_method {
                0 => QcowCompressionMethod::Zlib,
                _ => QcowCompressionMethod::Unknown,
            };
            self.file_header_size = file_header_v3.header_size;

            backing_file_name_offset = file_header_v3.backing_file_name_offset;
            backing_file_name_size = file_header_v3.backing_file_name_size;
            number_of_level2_table_bits = self.number_of_cluster_block_bits - 3;
            level1_table_offset = file_header_v3.level1_table_offset;
            level1_table_number_of_references =
                file_header_v3.level1_table_number_of_references as u64;
        } else {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format version: {}",
                self.format_version
            )));
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
            self.number_of_cluster_block_bits + number_of_level2_table_bits;

        if self.level1_index_bit_shift > 63 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid number of level 1 index bit shift: {} value out of bounds",
                self.level1_index_bit_shift
            )));
        }
        self.level2_index_bit_mask = !(u64::MAX << number_of_level2_table_bits);
        self.cluster_block_bit_mask = !(u64::MAX << self.number_of_cluster_block_bits);
        self.compression_bit_mask = !(u64::MAX << self.compression_bit_shift);
        self.cluster_block_size = 1 << self.number_of_cluster_block_bits;

        self.level2_table_number_of_references = 1 << number_of_level2_table_bits;

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
        if self.mediator.debug_output {
            self.mediator.debug_print(format!("QcowFile {{\n"));
            self.mediator.debug_print(format!(
                "    level1_table_number_of_references: {}\n",
                level1_table_number_of_references,
            ));
            self.mediator.debug_print(format!(
                "    level2_table_number_of_references: {}\n",
                self.level2_table_number_of_references,
            ));
            self.mediator.debug_print(format!(
                "    cluster_block_size: {}\n",
                self.cluster_block_size,
            ));
            self.mediator.debug_print(format!("}}\n\n"));
        }
        self.level1_cluster_table.set_range(
            level1_table_offset,
            level1_table_number_of_references as u32,
        );

        let block_tree_data_size: u64 =
            self.media_size.div_ceil(self.cluster_block_size) * self.cluster_block_size;
        self.block_tree = BlockTree::<QcowBlockRange>::new(
            block_tree_data_size,
            self.level2_table_number_of_references,
            self.cluster_block_size,
        );
        if backing_file_name_offset > 0 && backing_file_name_size > 0 {
            match self.read_backing_file_name(
                data_stream,
                backing_file_name_offset,
                backing_file_name_size,
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

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(backing_file_name_offset)
        );
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "QcowBackingFile data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.backing_file_name = Some(ByteString::from(&data));

        Ok(())
    }

    /// Reads a specific cluster block entry and fills the block tree.
    fn read_cluster_block_entry(&mut self, media_offset: u64) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let level1_table_index: u64 = media_offset >> self.level1_index_bit_shift;

        let level1_entry: QcowClusterTableEntry = match self
            .level1_cluster_table
            .read_entry(data_stream, level1_table_index as u32)
        {
            Ok(cluster_table_entry) => cluster_table_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read level 1 cluster table entry"
                );
                return Err(error);
            }
        };
        let level1_media_offset: u64 = level1_table_index << self.level1_index_bit_shift;
        let level2_table_offset: u64 = level1_entry.reference & self.offset_bit_mask;

        if level2_table_offset == 0 {
            let range_media_size: u64 = 1 << self.level1_index_bit_shift;

            let block_range: QcowBlockRange = QcowBlockRange::new(
                level1_media_offset,
                0,
                range_media_size,
                QcowBlockRangeType::Sparse,
            );
            match self
                .block_tree
                .insert_value(level1_media_offset, range_media_size, block_range)
            {
                Ok(_) => {}
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
        } else {
            self.level2_cluster_table.set_range(
                level2_table_offset,
                self.level2_table_number_of_references as u32,
            );
            let level2_table_index: u64 =
                (media_offset >> self.number_of_cluster_block_bits) & self.level2_index_bit_mask;

            let level2_entry: QcowClusterTableEntry = match self
                .level2_cluster_table
                .read_entry(data_stream, level2_table_index as u32)
            {
                Ok(cluster_table_entry) => cluster_table_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read level 2 cluster table entry"
                    );
                    return Err(error);
                }
            };
            let level2_media_offset: u64 =
                level1_media_offset + (level2_table_index * self.cluster_block_size);
            let block_data_offset: u64 = level2_entry.reference & self.offset_bit_mask;
            let range_type: QcowBlockRangeType = if block_data_offset == 0 {
                if self.backing_file_name.is_some() {
                    QcowBlockRangeType::InBackingFile
                } else {
                    QcowBlockRangeType::Sparse
                }
            } else {
                if (level2_entry.reference & self.compression_flag_bit_mask) == 0 {
                    QcowBlockRangeType::InFile
                } else {
                    if self.encryption_method != QcowEncryptionMethod::None {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported combined encryption and compression"
                        ));
                    }
                    QcowBlockRangeType::Compressed
                }
            };
            let block_range: QcowBlockRange = QcowBlockRange::new(
                level2_media_offset,
                block_data_offset,
                self.cluster_block_size,
                range_type,
            );
            match self.block_tree.insert_value(
                level2_media_offset,
                self.cluster_block_size,
                block_range,
            ) {
                Ok(_) => {}
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            };
        }
        Ok(())
    }

    /// Reads media data based on the level 1 and level 2 tables.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut media_offset: u64 = self.media_offset;

        while data_offset < read_size {
            if media_offset >= self.media_size {
                break;
            }
            let mut block_tree_value: Option<&QcowBlockRange> =
                self.block_tree.get_value(media_offset);

            if block_tree_value.is_none() {
                match self.read_cluster_block_entry(media_offset) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read cluster block entry"
                        );
                        return Err(error);
                    }
                }
                block_tree_value = self.block_tree.get_value(media_offset);
            }
            let block_range: &QcowBlockRange = match block_tree_value {
                Some(value) => value,
                None => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {}",
                        media_offset
                    )));
                }
            };
            let range_relative_offset: u64 = media_offset - block_range.media_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let mut range_read_size: usize = read_size - data_offset;

            if (range_read_size as u64) > range_remainder_size {
                range_read_size = range_remainder_size as usize;
            }
            let data_end_offset: usize = data_offset + range_read_size;
            let range_read_count: usize = match block_range.range_type {
                QcowBlockRangeType::Compressed => {
                    // TODO: add compression support.
                    todo!();
                }
                QcowBlockRangeType::InBackingFile => {
                    let backing_file: &Arc<RwLock<QcowFile>> = match self.backing_file.as_ref() {
                        Some(backing_file) => backing_file,
                        None => {
                            return Err(keramics_core::error_trace_new!("Missing backing file"));
                        }
                    };
                    let read_count: usize = keramics_core::data_stream_read_at_position!(
                        backing_file,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(media_offset)
                    );
                    read_count
                }
                QcowBlockRangeType::InFile => {
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
                QcowBlockRangeType::Sparse => {
                    data[data_offset..data_end_offset].fill(0);

                    range_read_size
                }
            };
            if range_read_count == 0 {
                break;
            }
            data_offset += range_read_count;
            media_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }

    /// Sets the backing file.
    pub fn set_backing_file(
        &mut self,
        backing_file: &Arc<RwLock<QcowFile>>,
    ) -> Result<(), ErrorTrace> {
        self.backing_file = Some(backing_file.clone());

        Ok(())
    }
}

impl DataStream for QcowFile {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.media_size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.media_offset >= self.media_size {
            return Ok(0);
        }
        let remaining_media_size: u64 = self.media_size - self.media_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_media_size {
            read_size = remaining_media_size as usize;
        }
        let read_count: usize = match self.read_data_from_blocks(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
                return Err(error);
            }
        };
        self.media_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.media_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.media_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.media_size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.media_offset)
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

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("qcow/ext2.qcow2").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    // TODO: add tests for get_backing_file_name

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = QcowFile::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("qcow/ext2.qcow2").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    #[test]
    fn test_read_file_header() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = QcowFile::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("qcow/ext2.qcow2").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_file_header(&data_stream)?;

        assert_eq!(file.media_size, 4194304);

        Ok(())
    }

    // TODO: add tests for read_backing_file_name
    // TODO: add tests for read_cluster_block_entry
    // TODO: add tests for read_data_from_blocks
    // TODO: add tests for set_backing_file

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, file.media_size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = get_file()?;

        let offset = file.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = file.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_media_size() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = get_file()?;

        let offset: u64 = file.seek(SeekFrom::End(512))?;
        assert_eq!(offset, file.media_size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut file: QcowFile = get_file()?;
        file.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
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
        let mut file: QcowFile = get_file()?;
        file.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = file.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
