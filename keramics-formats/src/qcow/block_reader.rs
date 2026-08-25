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
use crate::traits::BlockReader;

use super::block_range::{QcowBlockRange, QcowBlockRangeType};
use super::cluster_table::{QcowClusterTable, QcowClusterTableEntry};
use super::enums::QcowEncryptionMethod;

/// QEMU Copy-On-Write (QCOW) block reader.
pub struct QcowBlockReader {
    /// Data stream.
    data_stream: DataStreamReference,

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
    cluster_block_size: u64,

    /// Compression flag bit mask.
    compression_flag_bit_mask: u64,

    /// Encryption method.
    encryption_method: QcowEncryptionMethod,

    /// Block tree.
    block_tree: BlockTree<QcowBlockRange>,

    /// Backing file data stream.
    backing_file_data_stream: Option<DataStreamReference>,

    /// Size.
    size: u64,
}

impl QcowBlockReader {
    /// Creates a new file.
    pub fn new(
        data_stream: &DataStreamReference,
        offset_bit_mask: u64,
        level1_index_bit_shift: u32,
        level1_table_offset: u64,
        level1_table_number_of_references: u32,
        level2_index_bit_mask: u64,
        level2_table_number_of_references: u64,
        number_of_cluster_block_bits: u32,
        cluster_block_size: u64,
        compression_flag_bit_mask: u64,
        backing_file_data_stream: Option<DataStreamReference>,
        size: u64,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
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
            compression_flag_bit_mask,
            encryption_method: QcowEncryptionMethod::None,
            block_tree: BlockTree::<QcowBlockRange>::new(
                size,
                level2_table_number_of_references,
                cluster_block_size,
            ),
            backing_file_data_stream,
            size,
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
                level1_media_offset + (level2_table_index * self.cluster_block_size);
            let block_data_offset: u64 = level2_entry.reference & self.offset_bit_mask;
            let range_type: QcowBlockRangeType = if block_data_offset == 0 {
                if self.backing_file_data_stream.is_some() {
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
                    // TODO: add compression support.
                    todo!();
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
