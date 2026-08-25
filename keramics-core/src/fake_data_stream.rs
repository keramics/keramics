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
use std::sync::{Arc, RwLock};

use super::data_stream::{DataStream, DataStreamReference};
use super::errors::ErrorTrace;

/// Fake (or virtual) data stream.
pub struct FakeDataStream {
    /// The data.
    data: Vec<u8>,

    /// The data size.
    data_size: usize,

    /// The data offset.
    data_offset: u64,

    /// The current offset.
    current_offset: u64,

    /// The size.
    size: u64,
}

impl FakeDataStream {
    /// Creates a new data stream.
    pub fn new(data: &[u8], size: u64) -> Self {
        Self {
            data: data.to_vec(),
            data_size: data.len(),
            data_offset: 0,
            current_offset: 0,
            size,
        }
    }

    /// Sets the data offset.
    pub fn set_data_offset(&mut self, data_offset: u64) {
        self.data_offset = data_offset;
        self.size += data_offset;
    }
}

impl DataStream for FakeDataStream {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.current_offset)
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let mut current_offset: u64 = self.current_offset;
        let mut buf_offset: usize = 0;

        let remaining_size: u64 = self.size - current_offset;
        let read_size: usize = min(buf.len(), remaining_size as usize);
        let data_end_offset: u64 = self.data_offset + (self.data_size as u64);

        while buf_offset < read_size {
            let read_remainder_size: usize = read_size - buf_offset;

            let read_count: usize =
                if current_offset >= self.data_offset && current_offset < data_end_offset {
                    let data_offset: usize = (current_offset - self.data_offset) as usize;

                    let data_remainder_size: usize =
                        min(read_remainder_size, self.data_size - data_offset);

                    let data_end_offset: usize = data_offset + data_remainder_size;
                    let buf_end_offset: usize = buf_offset + data_remainder_size;

                    buf[buf_offset..buf_end_offset]
                        .copy_from_slice(&self.data[data_offset..data_end_offset]);

                    data_remainder_size
                } else {
                    let buf_end_offset: usize = buf_offset + read_remainder_size;

                    buf[buf_offset..buf_end_offset].fill(0);

                    read_remainder_size
                };
            buf_offset += read_count;
            current_offset += read_count as u64;
        }
        self.current_offset = current_offset;

        Ok(buf_offset)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                match self.current_offset.checked_add_signed(relative_offset) {
                    Some(offset) => offset,
                    None => {
                        return Err(ErrorTrace::new(format!(
                            "{}: Invalid offset value out of bounds",
                            crate::error_trace_function!(),
                        )));
                    }
                }
            }
            SeekFrom::End(relative_offset) => match self.size.checked_add_signed(relative_offset) {
                Some(offset) => offset,
                None => {
                    return Err(ErrorTrace::new(format!(
                        "{}: Invalid offset value out of bounds",
                        crate::error_trace_function!(),
                    )));
                }
            },
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

/// Opens a new fake data stream.
pub fn open_fake_data_stream(data: &[u8]) -> DataStreamReference {
    let data_size: u64 = data.len() as u64;
    let fake_data_stream: FakeDataStream = FakeDataStream::new(data, data_size);
    Arc::new(RwLock::new(fake_data_stream))
}

/// Opens a new fake data stream with offset.
pub fn open_fake_data_stream_with_offset(data: &[u8], data_offset: u64) -> DataStreamReference {
    let data_size: u64 = data.len() as u64;
    let mut fake_data_stream: FakeDataStream = FakeDataStream::new(data, data_size);
    fake_data_stream.set_data_offset(data_offset);
    Arc::new(RwLock::new(fake_data_stream))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ];
    }

    fn get_test_data_stream() -> FakeDataStream {
        let test_data: Vec<u8> = get_test_data();
        FakeDataStream::new(&test_data, 32768)
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        data_stream.seek(SeekFrom::Start(1024))?;

        let offset: u64 = data_stream.get_offset()?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let size: u64 = data_stream.get_size()?;
        assert_eq!(size, 32768);

        Ok(())
    }

    #[test]
    fn test_seek_from_start() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let offset: u64 = data_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let offset: u64 = data_stream.seek(SeekFrom::End(-512))?;
        assert_eq!(offset, data_stream.size - 512);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let offset = data_stream.seek(SeekFrom::Start(1024))?;
        assert_eq!(offset, 1024);

        let offset: u64 = data_stream.seek(SeekFrom::Current(-512))?;
        assert_eq!(offset, 512);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let result: Result<u64, ErrorTrace> = data_stream.seek(SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let offset: u64 = data_stream.seek(SeekFrom::End(512))?;
        assert_eq!(offset, data_stream.size + 512);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();
        data_stream.seek(SeekFrom::Start(128))?;

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 64);

        let expected_data: &str = concat!(
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        );
        assert_eq!(&data, &expected_data.as_bytes()[128..192]);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();
        data_stream.seek(SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_with_data_offset() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let mut data_stream: FakeDataStream = FakeDataStream::new(&test_data, 32768);

        data_stream.set_data_offset(128);

        let size: u64 = data_stream.get_size()?;
        assert_eq!(size, 32768 + 128);

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 64);
        assert_eq!(&data, &[0; 64]);

        data_stream.seek(SeekFrom::Start(128))?;

        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 64);

        let expected_data: &str = concat!(
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        );
        assert_eq!(&data, &expected_data.as_bytes()[0..64]);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_with_spanning_range() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let mut data_stream: FakeDataStream = FakeDataStream::new(&test_data, 32768);
        let data_size: u64 = test_data.len() as u64;

        data_stream.seek(SeekFrom::Start(data_size - 8))?;

        let mut data: Vec<u8> = vec![0; 32];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 32);

        let expected_data: &str = concat!(
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        );
        let test_data_offset: usize = (data_size as usize) - 8;
        assert_eq!(&data[0..8], &expected_data.as_bytes()[test_data_offset..]);
        assert_eq!(&data[8..24], &[0; 16]);

        Ok(())
    }

    #[test]
    fn test_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();
        data_stream.seek(SeekFrom::End(0))?;

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_read_with_empty_buffer() -> Result<(), ErrorTrace> {
        let mut data_stream: FakeDataStream = get_test_data_stream();

        let mut data: Vec<u8> = vec![];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }

    #[test]
    fn test_open_fake_data_stream() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut data_stream = data_stream.write().expect("unable to obtain write lock");
        let size: u64 = data_stream.get_size()?;
        assert_eq!(size, test_data.len() as u64);

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 64);
        assert_eq!(&data, &test_data[0..64]);

        Ok(())
    }

    #[test]
    fn test_open_fake_data_stream_with_offset() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_offset: u64 = 64;
        let data_stream: DataStreamReference =
            open_fake_data_stream_with_offset(&test_data, data_offset);

        let mut data_stream = data_stream.write().expect("unable to obtain write lock");
        let size: u64 = data_stream.get_size()?;
        assert_eq!(size, (test_data.len() as u64) + data_offset);

        data_stream.seek(SeekFrom::Start(data_offset))?;
        let mut data: Vec<u8> = vec![0; 32];
        let read_size: usize = data_stream.read(&mut data)?;
        assert_eq!(read_size, 32);
        assert_eq!(&data, &test_data[0..32]);

        Ok(())
    }
}
