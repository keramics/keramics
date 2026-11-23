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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;
use crate::lru_cache::LruCache;
use crate::traits::BlockReader;

use super::block_range::{QcowBlockRange, QcowBlockRangeType};
use super::cluster_table::{QcowClusterTable, QcowClusterTableEntry};
use super::encryption::QcowEncryptionContext;
use super::enums::QcowCompressionMethod;

/// QEMU Copy-On-Write (QCOW) block reader.
pub struct QcowBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

    /// Data stream size.
    data_stream_size: u64,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Offset bit mask.
    offset_bit_mask: u64,

    /// Level 1 index bit shift.
    level1_index_bit_shift: u32,

    /// Level 1 cluster table.
    level1_cluster_table: QcowClusterTable,

    /// Level 2 index bit mask.
    level2_index_bit_mask: u64,

    /// Level 2 table number of references.
    level2_table_number_of_references: u64,

    /// Level 2 cluster table.
    level2_cluster_table: QcowClusterTable,

    /// Number of cluster block bits.
    number_of_cluster_block_bits: u32,

    /// Cluster block size.
    cluster_block_size: u32,

    /// Compression bit mask.
    compression_bit_mask: u64,

    /// Compression flag bit mask.
    compression_flag_bit_mask: u64,

    /// Compression method.
    compression_method: QcowCompressionMethod,

    /// Encryption context.
    encryption_context: Option<QcowEncryptionContext>,

    /// Block tree.
    block_tree: BlockTree<QcowBlockRange>,

    /// Decrypted and/or decompressed block cache.
    block_cache: LruCache<u64, Vec<u8>>,

    /// Backing file data stream.
    backing_file_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl QcowBlockReader {
    /// Creates a new file.
    pub fn new(
        data_stream: &DataStreamReference,
        data_stream_size: u64,
        bytes_per_sector: u16,
        offset_bit_mask: u64,
        level1_index_bit_shift: u32,
        level1_table_offset: u64,
        level1_table_number_of_references: u32,
        level2_index_bit_mask: u64,
        level2_table_number_of_references: u64,
        number_of_cluster_block_bits: u32,
        cluster_block_size: u32,
        compression_bit_mask: u64,
        compression_flag_bit_mask: u64,
        compression_method: &QcowCompressionMethod,
        encryption_context: Option<&QcowEncryptionContext>,
        backing_file_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            data_stream_size,
            bytes_per_sector,
            offset_bit_mask,
            level1_index_bit_shift,
            level1_cluster_table: QcowClusterTable::new(
                level1_table_offset,
                level1_table_number_of_references,
            ),
            level2_index_bit_mask,
            level2_table_number_of_references,
            level2_cluster_table: QcowClusterTable::new(0, 0),
            number_of_cluster_block_bits,
            cluster_block_size,
            compression_bit_mask,
            compression_flag_bit_mask,
            compression_method: compression_method.clone(),
            encryption_context: encryption_context.cloned(),
            block_tree: BlockTree::<QcowBlockRange>::new(
                size,
                level2_table_number_of_references,
                cluster_block_size as u64,
            ),
            block_cache: LruCache::new(64),
            backing_file_data_stream,
            size,
        }
    }

    /// Decompressed a block.
    fn decompress_block(&self, compressed_data: &[u8], data: &mut [u8]) -> Result<(), ErrorTrace> {
        match self.compression_method {
            QcowCompressionMethod::Deflate => {
                _ = crate::deflate_decompress!(
                    &compressed_data,
                    data,
                    "Unable to decompress DEFLATE data"
                );
            }
            QcowCompressionMethod::Zstd => {
                todo!();
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported compression method"
                ));
            }
        }
        Ok(())
    }

    /// Reads and decrypts a cluster block.
    fn read_cluster_block(
        &self,
        block_data_offset: u64,
        block_data_size: usize,
    ) -> Result<Vec<u8>, ErrorTrace> {
        let mut block_data: Vec<u8> = vec![0; block_data_size];

        keramics_core::data_stream_read_exact_at_position!(
            &self.data_stream,
            &mut block_data,
            SeekFrom::Start(block_data_offset)
        );
        match &self.encryption_context {
            Some(encryption_context) => {
                let mut sector_number: u64 = block_data_offset / (self.bytes_per_sector as u64);

                let mut data: Vec<u8> = vec![0; block_data_size];
                let mut data_offset: usize = 0;

                while data_offset < block_data_size {
                    let data_end_offset: usize = data_offset + (self.bytes_per_sector as usize);

                    match encryption_context.decrypt_sector(
                        sector_number,
                        &block_data[data_offset..data_end_offset],
                        &mut data[data_offset..data_end_offset],
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to decrypt sector: {}", sector_number)
                            );
                            return Err(error);
                        }
                    }
                    data_offset = data_end_offset;
                    sector_number += 1;
                }
                Ok(data)
            }
            None => Ok(block_data),
        }
    }

    /// Reads a specific cluster block entry and fills the block tree.
    fn read_cluster_block_entry(&mut self, media_offset: u64) -> Result<(), ErrorTrace> {
        let level1_table_index: u64 = media_offset >> self.level1_index_bit_shift;

        let level1_entry: QcowClusterTableEntry = match self
            .level1_cluster_table
            .read_entry(&self.data_stream, level1_table_index as u32)
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
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to insert block range into block tree"
                    );
                    return Err(error);
                }
            }
        } else {
            self.level2_cluster_table.set_range(
                level2_table_offset,
                self.level2_table_number_of_references as u32,
            );
            let level2_table_index: u64 =
                (media_offset >> self.number_of_cluster_block_bits) & self.level2_index_bit_mask;

            let level2_entry: QcowClusterTableEntry = match self
                .level2_cluster_table
                .read_entry(&self.data_stream, level2_table_index as u32)
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
                level1_media_offset + (level2_table_index * (self.cluster_block_size as u64));
            let block_data_offset: u64 = level2_entry.reference & self.offset_bit_mask;
            let range_type: QcowBlockRangeType = if block_data_offset == 0 {
                if self.backing_file_data_stream.is_some() {
                    QcowBlockRangeType::InBackingFile
                } else {
                    QcowBlockRangeType::Sparse
                }
            } else {
                if (level2_entry.reference & self.compression_flag_bit_mask) != 0 {
                    QcowBlockRangeType::Compressed
                } else if self.encryption_context.is_some() {
                    QcowBlockRangeType::Encrypted
                } else {
                    QcowBlockRangeType::InFile
                }
            };
            let block_range: QcowBlockRange = QcowBlockRange::new(
                level2_media_offset,
                block_data_offset,
                self.cluster_block_size as u64,
                range_type,
            );
            match self.block_tree.insert_value(
                level2_media_offset,
                self.cluster_block_size as u64,
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
        Ok(())
    }
}

impl BlockReader for QcowBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads media data based on the level 1 and level 2 tables.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let mut result: Result<Option<&QcowBlockRange>, ErrorTrace> =
                self.block_tree.get_value(current_offset);

            if result == Ok(None) {
                match self.read_cluster_block_entry(current_offset) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read cluster block entry"
                        );
                        return Err(error);
                    }
                }
                result = self.block_tree.get_value(current_offset);
            }
            let block_range: &QcowBlockRange = match result {
                Ok(Some(block_range)) => block_range,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {} (0x{:08x})",
                        current_offset, current_offset
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block range for offset: {} (0x{:08x})",
                            current_offset, current_offset,
                        )
                    );
                    return Err(error);
                }
            };
            let range_relative_offset: u64 = current_offset - block_range.media_offset;
            let range_remainder_size: u64 = block_range.size - range_relative_offset;

            let range_read_size: usize =
                min(read_size - data_offset, range_remainder_size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            let range_read_count: usize = match block_range.range_type {
                QcowBlockRangeType::Compressed => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    let block_data_offset: u64 =
                        block_range.data_offset & self.compression_bit_mask;

                    if block_data_offset > self.data_stream_size {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid block data offset value out of bounds",
                        ));
                    }
                    let block_data_size: usize = min(
                        self.cluster_block_size as usize,
                        (self.data_stream_size - block_data_offset) as usize,
                    );
                    if !self.block_cache.contains(&block_data_offset) {
                        let compressed_data: Vec<u8> =
                            match self.read_cluster_block(block_data_offset, block_data_size) {
                                Ok(block_data) => block_data,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to read block at offset: {} (0x{:08x})",
                                            block_data_offset, block_data_offset
                                        )
                                    );
                                    return Err(error);
                                }
                            };
                        let mut block_data: Vec<u8> = vec![0; self.cluster_block_size as usize];

                        match self.decompress_block(&compressed_data, &mut block_data) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to decompress block at offset: {} (0x{:08x})",
                                        block_data_offset, block_data_offset
                                    )
                                );
                                return Err(error);
                            }
                        }
                        self.block_cache.insert(block_data_offset, block_data);
                    }
                    let range_data: &[u8] = match self.block_cache.get(&block_data_offset) {
                        Some(data) => data,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Unable to retrieve data from cache"
                            )));
                        }
                    };
                    if range_data.len() != (block_range.size as usize) {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve block range data",
                        ));
                    }
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);

                    range_read_size
                }
                QcowBlockRangeType::Encrypted => {
                    let range_data_offset: usize = range_relative_offset as usize;
                    let range_data_end_offset: usize = range_data_offset + range_read_size;

                    let block_data_offset: u64 = block_range.data_offset;

                    if block_data_offset > self.data_stream_size {
                        return Err(keramics_core::error_trace_new!(
                            "Invalid block data offset value out of bounds",
                        ));
                    }
                    let block_data_size: usize = min(
                        self.cluster_block_size as usize,
                        (self.data_stream_size - block_data_offset) as usize,
                    );
                    if !self.block_cache.contains(&block_data_offset) {
                        let block_data: Vec<u8> =
                            match self.read_cluster_block(block_data_offset, block_data_size) {
                                Ok(block_data) => block_data,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to read block at offset: {} (0x{:08x})",
                                            block_data_offset, block_data_offset
                                        )
                                    );
                                    return Err(error);
                                }
                            };
                        self.block_cache.insert(block_data_offset, block_data);
                    }
                    let range_data: &[u8] = match self.block_cache.get(&block_data_offset) {
                        Some(data) => data,
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to retrieve data from cache"
                            ));
                        }
                    };
                    if range_data.len() != (block_range.size as usize) {
                        return Err(keramics_core::error_trace_new!(
                            "Unable to retrieve block range data",
                        ));
                    }
                    data[data_offset..data_end_offset]
                        .copy_from_slice(&range_data[range_data_offset..range_data_end_offset]);

                    range_read_size
                }
                QcowBlockRangeType::InBackingFile => match &self.backing_file_data_stream {
                    Some(data_stream) => {
                        keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(current_offset)
                        )
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing backing file"));
                    }
                },
                QcowBlockRangeType::InFile => {
                    keramics_core::data_stream_read_at_position!(
                        &self.data_stream,
                        &mut data[data_offset..data_end_offset],
                        SeekFrom::Start(block_range.data_offset + range_relative_offset)
                    )
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
            current_offset += range_read_count as u64;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::qcow::encryption::QcowEncryption;
    use crate::qcow::encryption_type::QcowEncryptionType;

    #[test]
    fn test_read_data_from_blocks_plain_sparse_backing_and_compressed() -> Result<(), ErrorTrace> {
        // Cluster size is 512 bytes (9 bits) with 4 entries per L2 table (L1 shift is 11).
        // Level 1 table at offset 0 maps entry 0 to the L2 table at offset 512 and leaves entry 1 sparse.
        // Level 2 table at offset 512 maps entry 0 to a plain cluster at offset 1024, entries 1 and 3
        // to the backing file or sparse zeros, and entry 2 to a compressed cluster at offset 1536.
        let mut image: Vec<u8> = vec![0; 2560];
        // L1 entry 0 = 512.
        image[0..8].copy_from_slice(&512_u64.to_be_bytes());
        // L1 entry 1 = 0.
        // L2 entry 0 = 1024.
        image[512..520].copy_from_slice(&1024_u64.to_be_bytes());
        // L2 entry 1 = 0.
        // L2 entry 2 = compressed flag (bit 62) | 1536.
        let compressed_reference: u64 = (1 << 62) | 1536;
        image[528..536].copy_from_slice(&compressed_reference.to_be_bytes());

        // Plain cluster at offset 1024.
        image[1024..1536].fill(0xaa);

        // Raw DEFLATE stored block (BFINAL=1, BTYPE=00, LEN=16, NLEN=0xffef, 16 bytes of 0xcc).
        let mut deflate_data: Vec<u8> = vec![0x01, 0x10, 0x00, 0xef, 0xff];
        deflate_data.extend_from_slice(&[0xcc; 16]);
        image[1536..1536 + deflate_data.len()].copy_from_slice(&deflate_data);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut backing_data: Vec<u8> = vec![0; 4096];
        backing_data[512..1024].fill(0xbb);
        let backing_stream: DataStreamReference = open_fake_data_stream(&backing_data);

        let mut reader: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            2,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            None,
            Some(backing_stream),
            4096,
        );
        let size: u64 = reader.get_size();
        assert_eq!(size, 4096);

        // Read across plain (0..512), backing (512..1024), compressed (1024..1536),
        // backing (1536..2048), and sparse L1 (2048..2560).
        let mut buffer: Vec<u8> = vec![0xff; 2560];
        let read_count: usize = reader.read_data_from_blocks(&mut buffer, 0)?;
        assert_eq!(read_count, 2560);
        assert_eq!(&buffer[0..512], &[0xaa; 512]);
        assert_eq!(&buffer[512..1024], &[0xbb; 512]);
        assert_eq!(&buffer[1024..1040], &[0xcc; 16]);
        assert_eq!(&buffer[1040..1536], &[0x00; 496]);
        assert_eq!(&buffer[2048..2560], &[0x00; 512]);

        // Read compressed cluster again to hit block_cache.
        let mut cached: [u8; 16] = [0; 16];
        let read_count: usize = reader.read_data_from_blocks(&mut cached, 1024)?;
        assert_eq!(read_count, 16);
        assert_eq!(cached, [0xcc; 16]);

        // Read at offset >= size returns 0.
        let read_count: usize = reader.read_data_from_blocks(&mut cached, 4096)?;
        assert_eq!(read_count, 0);

        // Without backing file, L2 entry 1 is sparse.
        let mut reader_no_backing: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            2,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            None,
            None,
            4096,
        );
        let mut sparse_buffer: [u8; 512] = [0xff; 512];
        let read_count: usize = reader_no_backing.read_data_from_blocks(&mut sparse_buffer, 512)?;
        assert_eq!(read_count, 512);
        assert_eq!(sparse_buffer, [0x00; 512]);

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_encrypted() -> Result<(), ErrorTrace> {
        let mut image: Vec<u8> = vec![0; 1536];
        // L1 entry 0 -> L2 table at offset 512.
        image[0..8].copy_from_slice(&512_u64.to_be_bytes());
        // L2 entry 0 -> encrypted cluster at offset 1024.
        image[512..520].copy_from_slice(&1024_u64.to_be_bytes());
        image[1024..1536].fill(0x22);

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(1);
        let encryption_context: QcowEncryptionContext =
            QcowEncryption::get_encryption_context(&encryption_type, &[0x11; 16])?.unwrap();

        let mut reader: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            1,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            Some(&encryption_context),
            None,
            2048,
        );
        let mut buffer: [u8; 512] = [0; 512];
        let read_count: usize = reader.read_data_from_blocks(&mut buffer, 0)?;
        assert_eq!(read_count, 512);
        assert_ne!(buffer, [0x22; 512]);

        // Second read hits block_cache.
        let mut cached: [u8; 16] = [0; 16];
        let read_count: usize = reader.read_data_from_blocks(&mut cached, 0)?;
        assert_eq!(read_count, 16);
        assert_eq!(&cached, &buffer[0..16]);

        Ok(())
    }

    #[test]
    fn test_read_data_from_blocks_with_errors() -> Result<(), ErrorTrace> {
        // Test unsupported compression method.
        let mut image: Vec<u8> = vec![0; 2048];
        image[0..8].copy_from_slice(&512_u64.to_be_bytes());
        let compressed_reference: u64 = (1 << 62) | 1024;
        image[512..520].copy_from_slice(&compressed_reference.to_be_bytes());
        // Invalid compressed block offset out of bounds at L2 entry 1.
        let out_of_bounds_compressed_reference: u64 = (1 << 62) | 4096;
        image[520..528].copy_from_slice(&out_of_bounds_compressed_reference.to_be_bytes());
        // Out of bounds encrypted block offset at L2 entry 2.
        image[528..536].copy_from_slice(&4096_u64.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let mut reader: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            1,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Unknown,
            None,
            None,
            2048,
        );
        let mut buffer: [u8; 512] = [0; 512];
        let result: Result<usize, ErrorTrace> = reader.read_data_from_blocks(&mut buffer, 0);
        assert!(result.is_err());

        // Compressed offset out of bounds.
        let result: Result<usize, ErrorTrace> = reader.read_data_from_blocks(&mut buffer, 512);
        assert!(result.is_err());

        // Invalid DEFLATE data error.
        let mut reader_deflate: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            1,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            None,
            None,
            2048,
        );
        let result: Result<usize, ErrorTrace> =
            reader_deflate.read_data_from_blocks(&mut buffer, 0);
        assert!(result.is_err());

        // Encrypted offset out of bounds.
        let encryption_type: QcowEncryptionType = QcowEncryptionType::new(1);
        let encryption_context: QcowEncryptionContext =
            QcowEncryption::get_encryption_context(&encryption_type, &[0x11; 16])?.unwrap();
        let mut reader_encrypted: QcowBlockReader = QcowBlockReader::new(
            &data_stream,
            image.len() as u64,
            512,
            0x00ffffffffffffff,
            11,
            0,
            1,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            Some(&encryption_context),
            None,
            2048,
        );
        let result: Result<usize, ErrorTrace> =
            reader_encrypted.read_data_from_blocks(&mut buffer, 1024);
        assert!(result.is_err());

        // Truncated stream fails L1 cluster table entry read.
        let short_stream: DataStreamReference = open_fake_data_stream(&[0; 4]);
        let mut reader_short: QcowBlockReader = QcowBlockReader::new(
            &short_stream,
            4,
            512,
            0x00ffffffffffffff,
            11,
            0,
            1,
            0x03,
            4,
            9,
            512,
            0x000000ffffffffff,
            1 << 62,
            &QcowCompressionMethod::Deflate,
            None,
            None,
            2048,
        );
        let result: Result<usize, ErrorTrace> = reader_short.read_data_from_blocks(&mut buffer, 0);
        assert!(result.is_err());

        Ok(())
    }
}
