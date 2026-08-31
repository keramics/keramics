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

use crate::block_stream::BlockStream;

use super::block_reader::DecmpfsBlockReader;

/// Apple File System Compression (decmpfs) data stream.
pub type DecmpfsDataStream = BlockStream<DecmpfsBlockReader>;

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStream, DataStreamReference, ErrorTrace, open_fake_data_stream};

    use crate::decmpfs::DecmpfsCompressionMethod;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x66, 0x70, 0x6d, 0x63, 0x07, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe0, 0x03, 0x4d, 0x79, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73,
            0x73, 0x65, 0x64, 0x20, 0x66, 0x69, 0x6c, 0x65, 0x0a, 0x06, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00,
        ]
    }

    fn get_decmpfs_stream() -> Result<DecmpfsDataStream, ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut block_reader: DecmpfsBlockReader =
            DecmpfsBlockReader::new(&data_stream, DecmpfsCompressionMethod::Lzvn);
        block_reader.open(19)?;

        Ok(DecmpfsDataStream::new(block_reader))
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let size: u64 = decmpfs_stream.get_size()?;
        assert_eq!(size, 19);

        Ok(())
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let offset: u64 = decmpfs_stream.get_offset()?;
        assert_eq!(offset, 0);

        decmpfs_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = decmpfs_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let offset: u64 = decmpfs_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let offset: u64 = decmpfs_stream.seek(SeekFrom::End(-8))?;
        assert_eq!(offset, 11);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let offset: u64 = decmpfs_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = decmpfs_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let result: Result<u64, ErrorTrace> = decmpfs_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        let offset: u64 = decmpfs_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, 19 + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;
        decmpfs_stream.seek(SeekFrom::Start(1024))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = decmpfs_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut decmpfs_stream: DecmpfsDataStream = get_decmpfs_stream()?;

        decmpfs_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = decmpfs_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
