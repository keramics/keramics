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

use crate::mediator::{Mediator, MediatorReference};
use crate::types::{BlockTree, ByteString, SharedValue};
use crate::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

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
    data_stream: VfsDataStreamReference,

    /// Format version.
    pub format_version: u32,

    /// File header size.
    file_header_size: u32,

    /// Offset bit mask.
    offset_bit_mask: u64,

    /// Level 1 index bit shift.
    level1_index_bit_shift: u8,

    /// Level 1 table offset.
    level1_table_offset: u64,

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
    pub backing_file_name: Option<ByteString>,

    /// Backing file.
    backing_file: SharedValue<QcowFile>,

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
            data_stream: SharedValue::none(),
            format_version: 0,
            file_header_size: 0,
            offset_bit_mask: 0,
            level1_index_bit_shift: 0,
            level1_table_offset: 0,
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
            backing_file: SharedValue::none(),
            media_size: 0,
            media_offset: 0,
        }
    }

    /// Opens a file.
    pub fn open(&mut self, file_system: &dyn VfsFileSystem, path: &VfsPath) -> io::Result<()> {
        self.data_stream = file_system.open_data_stream(path, None)?;

        self.read_file_header()
    }

    /// Reads the file header.
    fn read_file_header(&mut self) -> io::Result<()> {
        let mut data: [u8; 112] = [0; 112];

        let offset: u64 = match self.data_stream.with_write_lock() {
            Ok(mut data_stream) => {
                data_stream.read_exact_at_position(&mut data, io::SeekFrom::Start(0))?
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
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
        file_header_common.read_data(&data)?;

        self.format_version = file_header_common.format_version;

        let backing_file_name_offset: u64;
        let backing_file_name_size: u32;
        let number_of_level2_table_bits: u8;
        let level1_table_offset: u64;

        let mut level1_table_number_of_references: u64 = 0;

        if self.format_version == 1 {
            let mut file_header_v1: QcowFileHeaderV1 = QcowFileHeaderV1::new();
            file_header_v1.read_data(&data)?;

            if self.mediator.debug_output {
                self.mediator
                    .debug_print(file_header_v1.debug_read_data(&data));
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
            file_header_v2.read_data(&data)?;

            if self.mediator.debug_output {
                self.mediator
                    .debug_print(file_header_v2.debug_read_data(&data));
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
            file_header_v3.read_data(&data)?;

            if self.mediator.debug_output {
                self.mediator
                    .debug_print(file_header_v3.debug_read_data(&data));
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported format version: {}", self.format_version),
            ));
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
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid number of level 1 index bit shift: {} value out of bounds",
                    self.level1_index_bit_shift
                ),
            ));
        }
        self.level2_index_bit_mask = !(u64::MAX << number_of_level2_table_bits);
        self.cluster_block_bit_mask = !(u64::MAX << self.number_of_cluster_block_bits);
        self.compression_bit_mask = !(u64::MAX << self.compression_bit_shift);
        self.cluster_block_size = 1 << self.number_of_cluster_block_bits;

        self.level2_table_number_of_references = 1 << number_of_level2_table_bits;

        if self.format_version == 1 {
            if self.cluster_block_size > u64::MAX / self.level2_table_number_of_references {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid level 2 table number of references: {} value out of bounds",
                        self.level2_table_number_of_references
                    ),
                ));
            }
            level1_table_number_of_references = self
                .media_size
                .div_ceil(self.cluster_block_size * self.level2_table_number_of_references);

            if level1_table_number_of_references > u32::MAX as u64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid level 1 table number of references: {} value out of bounds",
                        level1_table_number_of_references
                    ),
                ));
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
            self.read_backing_file_name(backing_file_name_offset, backing_file_name_size)?;
        }
        if self.encryption_method != QcowEncryptionMethod::None {
            // TODO: handle encryption
            todo!();
        }
        Ok(())
    }

    fn read_backing_file_name(
        &mut self,
        backing_file_name_offset: u64,
        backing_file_name_size: u32,
    ) -> io::Result<()> {
        if backing_file_name_offset < self.file_header_size as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported backing file name offset: {}",
                    backing_file_name_offset
                ),
            ));
        }
        if backing_file_name_size > 65536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported backing file name size: {}",
                    backing_file_name_size
                ),
            ));
        }
        let mut data: Vec<u8> = vec![0; backing_file_name_size as usize];

        let offset: u64 = match self.data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream
                .read_exact_at_position(&mut data, io::SeekFrom::Start(backing_file_name_offset))?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "QcowBackingFile data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.backing_file_name = Some(ByteString::from_bytes(&data));

        Ok(())
    }

    /// Reads a specific cluster block entry and fills the block tree.
    fn read_cluster_block_entry(&mut self, media_offset: u64) -> io::Result<()> {
        let level1_table_index: u64 = media_offset >> self.level1_index_bit_shift;

        let level1_entry: QcowClusterTableEntry = self
            .level1_cluster_table
            .read_entry(&self.data_stream, level1_table_index as u32)?;

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
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
        } else {
            self.level2_cluster_table.set_range(
                level2_table_offset,
                self.level2_table_number_of_references as u32,
            );
            let level2_table_index: u64 =
                (media_offset >> self.number_of_cluster_block_bits) & self.level2_index_bit_mask;

            let level2_entry: QcowClusterTableEntry = self
                .level2_cluster_table
                .read_entry(&self.data_stream, level2_table_index as u32)?;

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
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Unsupported combined encryption and compression",
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
                Err(error) => return Err(crate::error_to_io_error!(error)),
            };
        }
        Ok(())
    }

    /// Reads media data based on the level 1 and level 2 tables.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> io::Result<usize> {
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
                self.read_cluster_block_entry(media_offset)?;

                block_tree_value = self.block_tree.get_value(media_offset);
            }
            let block_range: &QcowBlockRange = match block_tree_value {
                Some(value) => value,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Missing block range for offset: {}", media_offset),
                    ));
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
                QcowBlockRangeType::InBackingFile => match self.backing_file.with_write_lock() {
                    Ok(mut backing_file) => {
                        backing_file.seek(io::SeekFrom::Start(media_offset))?;

                        backing_file.read(&mut data[data_offset..data_end_offset])?
                    }
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                },
                QcowBlockRangeType::InFile => match self.data_stream.with_write_lock() {
                    Ok(mut data_stream) => data_stream.read_at_position(
                        &mut data[data_offset..data_end_offset],
                        io::SeekFrom::Start(block_range.data_offset + range_relative_offset),
                    )?,
                    Err(error) => return Err(crate::error_to_io_error!(error)),
                },
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
    pub fn set_backing_file(&mut self, backing_file: &SharedValue<QcowFile>) -> io::Result<()> {
        self.backing_file = backing_file.clone();

        Ok(())
    }
}

impl Read for QcowFile {
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
        let read_count: usize = self.read_data_from_blocks(&mut buf[..read_size])?;

        self.media_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for QcowFile {
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

    use crate::vfs::{VfsFileSystemReference, VfsPathType, VfsResolver, VfsResolverReference};

    #[test]
    fn test_open() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference = vfs_resolver.open_file_system(&vfs_path)?;

        let mut file = QcowFile::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        match vfs_file_system.with_write_lock() {
            Ok(file_system) => file.open(file_system.as_ref(), &vfs_path)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert_eq!(file.media_size, 4194304);

        Ok(())
    }
}
