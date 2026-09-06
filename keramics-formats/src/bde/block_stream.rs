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

use super::block_reader::BdeBlockReader;

/// BitLocker disk encryption (BDE) block stream.
pub type BdeBlockStream = BlockStream<BdeBlockReader>;

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::{DataStream, DataStreamReference, ErrorTrace, open_os_data_stream};

    use crate::RangeStream;
    use crate::bde::block_range::{BdeBlockRange, BdeBlockRangeType};
    use crate::bde::encryption::{BdeCipherContext, BdeEncryption};
    use crate::bde::encryption_context::BdeEncryptionContext;
    use crate::bde::encryption_type::BdeEncryptionType;
    use crate::tests::get_test_data_path;
    use crate::vhd::VhdFile;

    fn get_block_stream() -> Result<BdeBlockStream, ErrorTrace> {
        let path_string: String = get_test_data_path("bde/bde_aes128.vhd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        let block_ranges: [BdeBlockRange; 8] = [
            BdeBlockRange {
                logical_offset: 0,
                physical_offset: 35651584,
                size: 8192,
                range_type: BdeBlockRangeType::Encrypted,
            },
            BdeBlockRange {
                logical_offset: 8192,
                physical_offset: 8192,
                size: 35577856,
                range_type: BdeBlockRangeType::Encrypted,
            },
            BdeBlockRange {
                logical_offset: 35586048,
                physical_offset: 35586048,
                size: 65536,
                range_type: BdeBlockRangeType::Sparse,
            },
            BdeBlockRange {
                logical_offset: 35651584,
                physical_offset: 35651584,
                size: 7626752,
                range_type: BdeBlockRangeType::Encrypted,
            },
            BdeBlockRange {
                logical_offset: 43278336,
                physical_offset: 43278336,
                size: 65536,
                range_type: BdeBlockRangeType::Sparse,
            },
            BdeBlockRange {
                logical_offset: 43343872,
                physical_offset: 43343872,
                size: 7622656,
                range_type: BdeBlockRangeType::Encrypted,
            },
            BdeBlockRange {
                logical_offset: 50966528,
                physical_offset: 50966528,
                size: 65536,
                range_type: BdeBlockRangeType::Sparse,
            },
            BdeBlockRange {
                logical_offset: 51032064,
                physical_offset: 51032064,
                size: 14962688,
                range_type: BdeBlockRangeType::Encrypted,
            },
        ];
        let encryption_type: BdeEncryptionType = BdeEncryptionType::new(0x8002);
        let fvek_key: [u8; 16] = [
            0x15, 0xbc, 0x71, 0x55, 0xa3, 0x8f, 0xa1, 0x61, 0xc7, 0x2a, 0x9e, 0xeb, 0xe1, 0x20,
            0x25, 0xe6,
        ];
        let cipher_context: BdeCipherContext =
            BdeEncryption::get_cipher_context(&encryption_type, &fvek_key)?.unwrap();
        let encryption_context: BdeEncryptionContext =
            BdeEncryptionContext::new(512, cipher_context);

        Ok(BdeBlockStream::new(BdeBlockReader::new(
            &data_stream,
            512,
            &block_ranges,
            &encryption_context,
            65994752,
        )))
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;

        block_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = block_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;

        let size: u64 = block_stream.get_size()?;
        assert_eq!(size, 65994752);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;

        let offset: u64 = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;
        let size: u64 = block_stream.get_size()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;

        let offset = block_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = block_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;

        let result: Result<u64, ErrorTrace> = block_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;
        let size: u64 = block_stream.get_size()?;

        let offset: u64 = block_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let mut block_stream: BdeBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::Start(0))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 512);

        let expected_data: Vec<u8> = vec![
            0xeb, 0x52, 0x90, 0x4e, 0x54, 0x46, 0x53, 0x20, 0x20, 0x20, 0x20, 0x00, 0x02, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x00, 0x00, 0x3f, 0x00, 0xff, 0x00,
            0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x80, 0x00, 0x7f, 0xf7,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfa, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x16, 0x45, 0x12, 0xee, 0x56, 0x12, 0xee, 0x40, 0x00, 0x00, 0x00, 0x00,
            0xfa, 0x33, 0xc0, 0x8e, 0xd0, 0xbc, 0x00, 0x7c, 0xfb, 0x68, 0xc0, 0x07, 0x1f, 0x1e,
            0x68, 0x66, 0x00, 0xcb, 0x88, 0x16, 0x0e, 0x00, 0x66, 0x81, 0x3e, 0x03, 0x00, 0x4e,
            0x54, 0x46, 0x53, 0x75, 0x15, 0xb4, 0x41, 0xbb, 0xaa, 0x55, 0xcd, 0x13, 0x72, 0x0c,
            0x81, 0xfb, 0x55, 0xaa, 0x75, 0x06, 0xf7, 0xc1, 0x01, 0x00, 0x75, 0x03, 0xe9, 0xdd,
            0x00, 0x1e, 0x83, 0xec, 0x18, 0x68, 0x1a, 0x00, 0xb4, 0x48, 0x8a, 0x16, 0x0e, 0x00,
            0x8b, 0xf4, 0x16, 0x1f, 0xcd, 0x13, 0x9f, 0x83, 0xc4, 0x18, 0x9e, 0x58, 0x1f, 0x72,
            0xe1, 0x3b, 0x06, 0x0b, 0x00, 0x75, 0xdb, 0xa3, 0x0f, 0x00, 0xc1, 0x2e, 0x0f, 0x00,
            0x04, 0x1e, 0x5a, 0x33, 0xdb, 0xb9, 0x00, 0x20, 0x2b, 0xc8, 0x66, 0xff, 0x06, 0x11,
            0x00, 0x03, 0x16, 0x0f, 0x00, 0x8e, 0xc2, 0xff, 0x06, 0x16, 0x00, 0xe8, 0x4b, 0x00,
            0x2b, 0xc8, 0x77, 0xef, 0xb8, 0x00, 0xbb, 0xcd, 0x1a, 0x66, 0x23, 0xc0, 0x75, 0x2d,
            0x66, 0x81, 0xfb, 0x54, 0x43, 0x50, 0x41, 0x75, 0x24, 0x81, 0xf9, 0x02, 0x01, 0x72,
            0x1e, 0x16, 0x68, 0x07, 0xbb, 0x16, 0x68, 0x52, 0x11, 0x16, 0x68, 0x09, 0x00, 0x66,
            0x53, 0x66, 0x53, 0x66, 0x55, 0x16, 0x16, 0x16, 0x68, 0xb8, 0x01, 0x66, 0x61, 0x0e,
            0x07, 0xcd, 0x1a, 0x33, 0xc0, 0xbf, 0x0a, 0x13, 0xb9, 0xf6, 0x0c, 0xfc, 0xf3, 0xaa,
            0xe9, 0xfe, 0x01, 0x90, 0x90, 0x66, 0x60, 0x1e, 0x06, 0x66, 0xa1, 0x11, 0x00, 0x66,
            0x03, 0x06, 0x1c, 0x00, 0x1e, 0x66, 0x68, 0x00, 0x00, 0x00, 0x00, 0x66, 0x50, 0x06,
            0x53, 0x68, 0x01, 0x00, 0x68, 0x10, 0x00, 0xb4, 0x42, 0x8a, 0x16, 0x0e, 0x00, 0x16,
            0x1f, 0x8b, 0xf4, 0xcd, 0x13, 0x66, 0x59, 0x5b, 0x5a, 0x66, 0x59, 0x66, 0x59, 0x1f,
            0x0f, 0x82, 0x16, 0x00, 0x66, 0xff, 0x06, 0x11, 0x00, 0x03, 0x16, 0x0f, 0x00, 0x8e,
            0xc2, 0xff, 0x0e, 0x16, 0x00, 0x75, 0xbc, 0x07, 0x1f, 0x66, 0x61, 0xc3, 0xa1, 0xf6,
            0x01, 0xe8, 0x09, 0x00, 0xa1, 0xfa, 0x01, 0xe8, 0x03, 0x00, 0xf4, 0xeb, 0xfd, 0x8b,
            0xf0, 0xac, 0x3c, 0x00, 0x74, 0x09, 0xb4, 0x0e, 0xbb, 0x07, 0x00, 0xcd, 0x10, 0xeb,
            0xf2, 0xc3, 0x0d, 0x0a, 0x41, 0x20, 0x64, 0x69, 0x73, 0x6b, 0x20, 0x72, 0x65, 0x61,
            0x64, 0x20, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x20, 0x6f, 0x63, 0x63, 0x75, 0x72, 0x72,
            0x65, 0x64, 0x00, 0x0d, 0x0a, 0x42, 0x4f, 0x4f, 0x54, 0x4d, 0x47, 0x52, 0x20, 0x69,
            0x73, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x72, 0x65, 0x73, 0x73, 0x65, 0x64, 0x00, 0x0d,
            0x0a, 0x50, 0x72, 0x65, 0x73, 0x73, 0x20, 0x43, 0x74, 0x72, 0x6c, 0x2b, 0x41, 0x6c,
            0x74, 0x2b, 0x44, 0x65, 0x6c, 0x20, 0x74, 0x6f, 0x20, 0x72, 0x65, 0x73, 0x74, 0x61,
            0x72, 0x74, 0x0d, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x01,
            0xa7, 0x01, 0xbf, 0x01, 0x00, 0x00, 0x55, 0xaa,
        ];
        assert_eq!(data, expected_data);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut block_stream: BdeBlockStream = get_block_stream()?;
        block_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 512];
        let read_size: usize = block_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
