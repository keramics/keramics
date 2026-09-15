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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_stream::BlockStream;

use super::block_reader::NtfsBlockReader;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) (cluster) block stream.
pub type NtfsBlockStream = BlockStream<NtfsBlockReader>;

impl NtfsBlockStream {
    /// Opens a block stream.
    #[allow(dead_code)]
    pub(super) fn open(
        data_stream: &DataStreamReference,
        data_attribute: &NtfsMftAttribute,
        cluster_block_size: u32,
    ) -> Result<Self, ErrorTrace> {
        Self::open_with_flags(data_stream, data_attribute, cluster_block_size, false)
    }

    /// Opens a block stream with flags.
    pub(super) fn open_with_flags(
        data_stream: &DataStreamReference,
        data_attribute: &NtfsMftAttribute,
        cluster_block_size: u32,
        ignore_valid_data_size: bool,
    ) -> Result<Self, ErrorTrace> {
        let mut block_reader: NtfsBlockReader =
            NtfsBlockReader::new(data_stream, cluster_block_size);
        match block_reader.open_with_flags(data_attribute, ignore_valid_data_size) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open block reader");
                return Err(error);
            }
        }
        Ok(Self::new(block_reader))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;
    use std::path::PathBuf;

    use keramics_core::{DataStream, DataStreamReference, ErrorTrace, open_os_data_stream};

    use crate::ntfs::mft_attribute::NtfsMftAttribute;
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

    fn get_block_stream() -> Result<NtfsBlockStream, ErrorTrace> {
        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        NtfsBlockStream::open(&data_stream, &data_attribute, 4096)
    }

    // TODO: add tests for get_offset.

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let size: u64 = block_stream.get_size()?;
        assert_eq!(size, 11358);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, 11358 - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = block_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let result: Result<u64, ErrorTrace> = block_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, 11358 + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut block_stream: NtfsBlockStream = get_block_stream()?;
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
        let mut block_stream: NtfsBlockStream = get_block_stream()?;

        block_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_open_with_flags_ignore_valid_data_size() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let test_mft_attribute_data: Vec<u8> = get_test_mft_attribute_data();
        let mut data_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        data_attribute.read_data(&test_mft_attribute_data)?;

        let mut block_stream_default: NtfsBlockStream =
            NtfsBlockStream::open_with_flags(&data_stream, &data_attribute, 4096, false)?;
        assert_eq!(block_stream_default.get_size()?, 11358);

        let mut block_stream_ignore_vdl: NtfsBlockStream =
            NtfsBlockStream::open_with_flags(&data_stream, &data_attribute, 4096, true)?;
        assert_eq!(block_stream_ignore_vdl.get_size()?, 12288);

        block_stream_ignore_vdl.seek(SeekFrom::Start(11358))?;
        let mut slack_data: Vec<u8> = vec![0; 930];
        let read_size: usize = block_stream_ignore_vdl.read(&mut slack_data)?;
        assert_eq!(read_size, 930);

        Ok(())
    }

    #[test]
    fn test_open_with_flags_does_not_zero_fill_beyond_valid_data_size() -> Result<(), ErrorTrace> {
        let cluster_size: usize = 4096;
        let total_clusters: usize = 3;
        let mut raw_cluster_data: Vec<u8> = vec![0xaa; cluster_size * total_clusters];
        for byte in raw_cluster_data[11358..12288].iter_mut() {
            *byte = 0x5a;
        }
        let fake_data_stream: DataStreamReference =
            keramics_core::open_fake_data_stream(&raw_cluster_data);

        let mut mock_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        mock_attribute.non_resident_flag = 0x01;
        mock_attribute.allocated_data_size = 12288;
        mock_attribute.data_size = 11358;
        mock_attribute.valid_data_size = 11358;
        let mut cluster_group: crate::ntfs::cluster_group::NtfsClusterGroup =
            crate::ntfs::cluster_group::NtfsClusterGroup::new(0, 2);
        cluster_group.data_runs.push(crate::ntfs::data_run::NtfsDataRun {
            number_of_blocks: 3,
            block_number: 0,
            run_type: crate::ntfs::data_run::NtfsDataRunType::InFile,
        });
        mock_attribute.data_cluster_groups.push(cluster_group);

        let mut stream_normal: NtfsBlockStream =
            NtfsBlockStream::open_with_flags(&fake_data_stream, &mock_attribute, 4096, false)?;
        assert_eq!(stream_normal.get_size()?, 11358);
        stream_normal.seek(SeekFrom::Start(11358))?;
        let mut read_buf: Vec<u8> = vec![0xff; 512];
        let bytes_read: usize = stream_normal.read(&mut read_buf)?;
        assert_eq!(bytes_read, 0);

        let mut stream_ignore: NtfsBlockStream =
            NtfsBlockStream::open_with_flags(&fake_data_stream, &mock_attribute, 4096, true)?;
        assert_eq!(stream_ignore.get_size()?, 12288);
        stream_ignore.seek(SeekFrom::Start(11358))?;
        let mut slack_buf: Vec<u8> = vec![0; 930];
        let slack_read: usize = stream_ignore.read(&mut slack_buf)?;
        assert_eq!(slack_read, 930);
        assert_eq!(slack_buf, vec![0x5a; 930]);

        Ok(())
    }
}
