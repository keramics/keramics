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

use std::cmp::min;
use std::io::SeekFrom;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;

use super::block_range::{NtfsBlockRange, NtfsBlockRangeType};
use super::data_run::NtfsDataRunType;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) (cluster) block stream.
pub struct NtfsBlockStream {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// Cluster block size.
    cluster_block_size: u32,

    /// Block tree.
    block_tree: BlockTree<NtfsBlockRange>,

    /// The current offset.
    current_offset: u64,

    /// The size.
    size: u64,

    /// The valid data size.
    valid_data_size: u64,
}

impl NtfsBlockStream {
    /// Creates a new block stream.
    pub(super) fn new(cluster_block_size: u32) -> Self {
        Self {
            data_stream: None,
            cluster_block_size: cluster_block_size,
            block_tree: BlockTree::<NtfsBlockRange>::new(0, 0, 0),
            current_offset: 0,
            size: 0,
            valid_data_size: 0,
        }
    }

    /// Opens a block stream.
    pub(super) fn open(
        &mut self,
        data_stream: &DataStreamReference,
        data_attribute: &NtfsMftAttribute,
    ) -> Result<(), ErrorTrace> {
        if data_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported resident $DATA attribute"
            ));
        }
        let block_tree_size: u64 = data_attribute
            .allocated_data_size
            .div_ceil(self.cluster_block_size as u64)
            * (self.cluster_block_size as u64);
        self.block_tree =
            BlockTree::<NtfsBlockRange>::new(block_tree_size, 0, self.cluster_block_size as u64);

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
                        data_run.number_of_blocks,
                        range_type,
                    );
                    match self.block_tree.insert_value(
                        virtual_cluster_offset,
                        range_size,
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
        self.data_stream = Some(data_stream.clone());

        if data_attribute.is_compressed() {
            self.size = data_attribute.allocated_data_size;
            self.valid_data_size = data_attribute.allocated_data_size;
        } else {
            self.size = data_attribute.data_size;
            self.valid_data_size = data_attribute.valid_data_size;
        };
        Ok(())
    }

    /// Reads data based on the block ranges.
    fn read_data_from_blocks(&mut self, data: &mut [u8]) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = self.current_offset;

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
                let block_range: &NtfsBlockRange = match self.block_tree.get_value(current_offset) {
                    Some(value) => value,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing block range for offset: {}",
                            current_offset
                        )));
                    }
                };
                let range_logical_offset: u64 = block_range.virtual_cluster_offset;
                let range_size: u64 =
                    block_range.number_of_blocks * (self.cluster_block_size as u64);

                let mut range_logical_end_offset: u64 = range_logical_offset + range_size;
                if range_logical_end_offset > self.valid_data_size {
                    range_logical_end_offset = self.valid_data_size;
                };
                let range_relative_offset: u64 = current_offset - range_logical_offset;
                let range_remainder_size: u64 =
                    (range_logical_end_offset - range_logical_offset) - range_relative_offset;
                let read_remainder_size: usize = read_size - data_offset;
                let range_read_size: usize =
                    min(read_remainder_size, range_remainder_size as usize);

                let data_end_offset: usize = data_offset + range_read_size;
                match block_range.range_type {
                    NtfsBlockRangeType::InFile => {
                        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
                            Some(data_stream) => data_stream,
                            None => {
                                return Err(keramics_core::error_trace_new!("Missing data stream"));
                            }
                        };
                        let range_physical_offset: u64 =
                            block_range.cluster_block_number * (self.cluster_block_size as u64);

                        let read_count: usize = keramics_core::data_stream_read_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(range_physical_offset + range_relative_offset)
                        );
                        read_count
                    }
                    NtfsBlockRangeType::Sparse => {
                        data[data_offset..data_end_offset].fill(0);

                        range_read_size
                    }
                }
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

impl DataStream for NtfsBlockStream {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = match self.read_data_from_blocks(&mut buf[..read_size]) {
            Ok(read_count) => read_count,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read data from blocks");
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
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.size as i64;
                end_offset += relative_offset;
                end_offset as u64
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

    fn get_test_mft_attribute_data() -> Vec<u8> {
        return vec![
            0x80, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x01, 0x00, 0x40, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5e, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x5e, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x03, 0xe9, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);

        block_stream.open(&data_stream, &data_attribute)?;

        Ok(())
    }

    // TODO: add tests for read_data_from_blocks

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        let offset: u64 = block_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, 11358 - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = block_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_beyond_file_size() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        let offset: u64 = block_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, 11358 + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        block_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x59, 0x6f, 0x75, 0x22, 0x20, 0x28, 0x6f,
            0x72, 0x20, 0x22, 0x59, 0x6f, 0x75, 0x72, 0x22, 0x29, 0x20, 0x73, 0x68, 0x61, 0x6c,
            0x6c, 0x20, 0x6d, 0x65, 0x61, 0x6e, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x64, 0x69,
            0x76, 0x69, 0x64, 0x75, 0x61, 0x6c, 0x20, 0x6f, 0x72, 0x20, 0x4c, 0x65, 0x67, 0x61,
            0x6c, 0x20, 0x45, 0x6e, 0x74, 0x69, 0x74, 0x79, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x65, 0x78, 0x65, 0x72, 0x63, 0x69, 0x73, 0x69, 0x6e, 0x67, 0x20, 0x70, 0x65,
            0x72, 0x6d, 0x69, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x73, 0x20, 0x67, 0x72, 0x61, 0x6e,
            0x74, 0x65, 0x64, 0x20, 0x62, 0x79, 0x20, 0x74, 0x68, 0x69, 0x73, 0x20, 0x4c, 0x69,
            0x63, 0x65, 0x6e, 0x73, 0x65, 0x2e, 0x0a, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x22, 0x53, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x22, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x20,
            0x73, 0x68, 0x61, 0x6c, 0x6c, 0x20, 0x6d, 0x65, 0x61, 0x6e, 0x20, 0x74, 0x68, 0x65,
            0x20, 0x70, 0x72, 0x65, 0x66, 0x65, 0x72, 0x72, 0x65, 0x64, 0x20, 0x66, 0x6f, 0x72,
            0x6d, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x6d, 0x61, 0x6b, 0x69, 0x6e, 0x67, 0x20, 0x6d,
            0x6f, 0x64, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x2c, 0x0a,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x69, 0x6e, 0x63, 0x6c, 0x75, 0x64, 0x69, 0x6e,
            0x67, 0x20, 0x62, 0x75, 0x74, 0x20, 0x6e, 0x6f, 0x74, 0x20, 0x6c, 0x69, 0x6d, 0x69,
            0x74, 0x65, 0x64, 0x20, 0x74, 0x6f, 0x20, 0x73, 0x6f, 0x66, 0x74, 0x77, 0x61, 0x72,
            0x65, 0x20, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x20, 0x63, 0x6f, 0x64, 0x65, 0x2c,
            0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e,
            0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x2c,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x75, 0x72, 0x61,
            0x74, 0x69, 0x6f, 0x6e, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x73, 0x2e, 0x0a, 0x0a, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x4f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x22, 0x20,
            0x66, 0x6f, 0x72, 0x6d, 0x20, 0x73, 0x68, 0x61, 0x6c, 0x6c, 0x20, 0x6d, 0x65, 0x61,
            0x6e, 0x20, 0x61, 0x6e, 0x79, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x20, 0x72, 0x65, 0x73,
            0x75, 0x6c, 0x74, 0x69, 0x6e, 0x67, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x6d, 0x65,
            0x63, 0x68, 0x61, 0x6e, 0x69, 0x63, 0x61, 0x6c, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x69, 0x6f,
            0x6e, 0x20, 0x6f, 0x72, 0x20, 0x74, 0x72, 0x61, 0x6e, 0x73, 0x6c, 0x61, 0x74, 0x69,
            0x6f, 0x6e, 0x20, 0x6f, 0x66, 0x20, 0x61, 0x20, 0x53, 0x6f, 0x75, 0x72, 0x63, 0x65,
            0x20, 0x66, 0x6f, 0x72, 0x6d, 0x2c, 0x20, 0x69, 0x6e, 0x63, 0x6c, 0x75, 0x64, 0x69,
            0x6e, 0x67, 0x20, 0x62, 0x75, 0x74, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x6e,
            0x6f, 0x74, 0x20, 0x6c, 0x69, 0x6d, 0x69, 0x74, 0x65, 0x64, 0x20, 0x74, 0x6f, 0x20,
            0x63, 0x6f, 0x6d, 0x70, 0x69, 0x6c, 0x65, 0x64, 0x20, 0x6f, 0x62, 0x6a, 0x65, 0x63,
            0x74, 0x20, 0x63, 0x6f, 0x64, 0x65, 0x2c, 0x20, 0x67, 0x65, 0x6e, 0x65, 0x72, 0x61,
            0x74, 0x65, 0x64, 0x20, 0x64, 0x6f, 0x63, 0x75, 0x6d, 0x65, 0x6e, 0x74, 0x61, 0x74,
            0x69, 0x6f, 0x6e, 0x2c, 0x0a, 0x20, 0x20, 0x20,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(4096);
        block_stream.open(&data_stream, &data_attribute)?;

        block_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    // TODO: add tests for get_size.
}
