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

use std::cmp::{Ordering, min};
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::BlockReader;

use super::block_range::{NtfsBlockRange, NtfsBlockRangeType};
use super::data_run::NtfsDataRunType;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) (cluster) block reader.
pub struct NtfsBlockReader {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Cluster block size.
    cluster_block_size: u32,

    /// Block ranges.
    block_ranges: Vec<NtfsBlockRange>,

    /// The size.
    size: u64,

    /// The valid data size.
    valid_data_size: u64,
}

impl NtfsBlockReader {
    /// Creates a new block stream.
    pub(super) fn new(data_stream: &DataStreamReference, cluster_block_size: u32) -> Self {
        Self {
            data_stream: data_stream.clone(),
            cluster_block_size,
            block_ranges: Vec::new(),
            size: 0,
            valid_data_size: 0,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(&mut self, data_attribute: &NtfsMftAttribute) -> Result<(), ErrorTrace> {
        if data_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported resident $DATA attribute"
            ));
        }
        if data_attribute.allocated_data_size > 0 {
            let mut virtual_cluster_number: u64 = 0;
            let mut virtual_cluster_offset: u64 = 0;

            for cluster_group in data_attribute.data_cluster_groups.iter() {
                if cluster_group.first_vcn != virtual_cluster_number {
                    return Err(keramics_core::error_trace_new!(format!(
                        "$DATA attribute cluster group first VNC: {} does not match expected value: {}",
                        cluster_group.first_vcn, virtual_cluster_number
                    )));
                }
                for data_run in cluster_group.data_runs.iter() {
                    let range_size: u64 =
                        data_run.number_of_blocks * (self.cluster_block_size as u64);

                    let range_type: NtfsBlockRangeType = match &data_run.run_type {
                        NtfsDataRunType::InFile => NtfsBlockRangeType::InFile,
                        NtfsDataRunType::Sparse => NtfsBlockRangeType::Sparse,
                        _ => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported data run type"
                            ));
                        }
                    };
                    let block_range: NtfsBlockRange = NtfsBlockRange::new(
                        virtual_cluster_offset,
                        data_run.block_number,
                        range_size,
                        range_type,
                    );
                    self.block_ranges.push(block_range);

                    virtual_cluster_number += data_run.number_of_blocks as u64;
                    virtual_cluster_offset += range_size;
                }
                if cluster_group.last_vcn != 0xffffffffffffffff
                    && cluster_group.last_vcn + 1 != virtual_cluster_number
                {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Cluster group last VNC: {} does not match expected value",
                        cluster_group.last_vcn
                    )));
                }
            }
        }
        if data_attribute.is_compressed() {
            self.size = data_attribute.allocated_data_size;
            self.valid_data_size = data_attribute.allocated_data_size;
        } else {
            self.size = data_attribute.data_size;
            self.valid_data_size = data_attribute.valid_data_size;
        }
        Ok(())
    }
}

impl BlockReader for NtfsBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        let mut range_index: usize = if current_offset >= self.valid_data_size {
            0
        } else {
            match self.block_ranges.binary_search_by(|block_range| {
                let range_end_offset: u64 = block_range.virtual_cluster_offset + block_range.size;

                if current_offset >= range_end_offset {
                    Ordering::Less
                } else if current_offset < block_range.virtual_cluster_offset {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }) {
                Ok(range_index) => range_index,
                Err(_) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing block range for offset: {} (0x{:08x})",
                        current_offset, current_offset
                    )));
                }
            }
        };
        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let read_count: usize = if current_offset >= self.valid_data_size {
                let range_remainder_size: u64 = self.size - current_offset;
                let read_remainder_size: usize = read_size - data_offset;
                let range_read_size: usize =
                    min(read_remainder_size, range_remainder_size as usize);
                let data_end_offset: usize = data_offset + range_read_size;

                data[data_offset..data_end_offset].fill(0);

                range_read_size
            } else {
                let block_range: &NtfsBlockRange = match self.block_ranges.get(range_index) {
                    Some(block_range) => block_range,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to retrieve block range: {} for offset: {} (0x{:08x})",
                            range_index, current_offset, current_offset,
                        )));
                    }
                };
                let range_logical_end_offset: u64 = min(
                    block_range.virtual_cluster_offset + block_range.size,
                    self.valid_data_size,
                );
                let range_relative_offset: u64 =
                    current_offset - block_range.virtual_cluster_offset;
                let range_remainder_size: u64 = (range_logical_end_offset
                    - block_range.virtual_cluster_offset)
                    - range_relative_offset;

                let range_read_size: usize =
                    min(read_size - data_offset, range_remainder_size as usize);
                let data_end_offset: usize = data_offset + range_read_size;

                match block_range.range_type {
                    NtfsBlockRangeType::InFile => {
                        let range_physical_offset: u64 =
                            block_range.cluster_block_number * (self.cluster_block_size as u64);

                        keramics_core::data_stream_read_exact_at_position!(
                            &self.data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(range_physical_offset + range_relative_offset)
                        );
                    }
                    NtfsBlockRangeType::Sparse => {
                        data[data_offset..data_end_offset].fill(0);
                    }
                }
                range_index += 1;

                range_read_size
            };
            if read_count == 0 {
                break;
            }
            data_offset += read_count;
            current_offset += read_count as u64;
        }
        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_test_mft_attribute_data() -> Vec<u8> {
        vec![
            0x80, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x01, 0x00, 0x40, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5e, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x5e, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x03, 0xe9, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_reader: NtfsBlockReader = NtfsBlockReader::new(&data_stream, 4096);
        block_reader.open(&data_attribute)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks
}
