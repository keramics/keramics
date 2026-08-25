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

use std::io::SeekFrom;
use std::sync::{Arc, RwLock};

use super::errors::ErrorTrace;

pub type DataStreamReference = Arc<RwLock<dyn DataStream>>;

/// Data stream trait.
pub trait DataStream: Send + Sync {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace>;

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace>;

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace>;

    /// Reads data at a specific position.
    #[cfg_attr(feature = "no-inline", inline(never))]
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn read_at_position(&mut self, buf: &mut [u8], pos: SeekFrom) -> Result<usize, ErrorTrace> {
        self.seek(pos)?;
        self.read(buf)
    }

    /// Reads an exact amount of data at the current position.
    #[cfg_attr(feature = "no-inline", inline(never))]
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ErrorTrace> {
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(crate::error_trace_new!("Unable to read the exact amount"));
        }
        Ok(())
    }

    /// Reads an exact amount of data at a specific position.
    #[cfg_attr(feature = "no-inline", inline(never))]
    #[cfg_attr(not(feature = "no-inline"), inline(always))]
    fn read_exact_at_position(&mut self, buf: &mut [u8], pos: SeekFrom) -> Result<u64, ErrorTrace> {
        let offset: u64 = self.seek(pos)?;
        let read_size: usize = buf.len();
        let read_count: usize = self.read(buf)?;

        if read_count != read_size {
            return Err(crate::error_trace_new!(format!(
                "Unable to read the exact amount at offset: {} (0x{:08x}) (requested: {}, read: {})",
                offset, offset, read_size, read_count
            )));
        }
        Ok(offset)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test data stream.
    struct TestDataStream {
        /// Data.
        data: Vec<u8>,

        /// The current offset.
        current_offset: u64,

        /// The size.
        size: u64,

        /// Value to indicate the test data stream is allowed to seek.
        allow_seek: bool,
    }

    impl TestDataStream {
        /// Creates a new test data stream.
        fn new(data: Vec<u8>) -> Self {
            let size: u64 = data.len() as u64;
            Self {
                data,
                current_offset: 0,
                size,
                allow_seek: true,
            }
        }

        /// Disables seek.
        fn disable_seek(&mut self) {
            self.allow_seek = false;
        }
    }

    impl DataStream for TestDataStream {
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
            let data_offset: usize = self.current_offset as usize;
            let read_count: usize = buf.len().min(self.data.len() - data_offset);

            buf[0..read_count].copy_from_slice(&self.data[data_offset..data_offset + read_count]);

            self.current_offset += read_count as u64;

            Ok(read_count)
        }

        /// Sets the current position of the data.
        fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
            if !self.allow_seek {
                return Err(ErrorTrace::new(String::from("Not allowed to seek")));
            }
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
                SeekFrom::End(relative_offset) => {
                    match self.size.checked_add_signed(relative_offset) {
                        Some(offset) => offset,
                        None => {
                            return Err(ErrorTrace::new(format!(
                                "{}: Invalid offset value out of bounds",
                                crate::error_trace_function!(),
                            )));
                        }
                    }
                }
                SeekFrom::Start(offset) => offset,
            };
            Ok(self.current_offset)
        }
    }

    fn get_test_data() -> Vec<u8> {
        vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66]
    }

    fn get_test_data_stream() -> TestDataStream {
        TestDataStream::new(get_test_data())
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 3] = [0; 3];
        let read_count: usize = data_stream.read_at_position(&mut data, SeekFrom::Start(2))?;

        assert_eq!(read_count, 3);
        assert_eq!(&data, &[0x33, 0x44, 0x55]);

        Ok(())
    }

    #[test]
    fn test_read_at_position_beyond_size() -> Result<(), ErrorTrace> {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 4] = [0; 4];
        let read_count: usize = data_stream.read_at_position(&mut data, SeekFrom::Start(4))?;

        assert_eq!(read_count, 2);
        assert_eq!(&data, &[0x55, 0x66, 0, 0]);

        Ok(())
    }

    #[test]
    fn test_read_at_position_with_failing_seek() {
        let mut data_stream: TestDataStream = get_test_data_stream();

        data_stream.disable_seek();

        let mut data: [u8; 3] = [0; 3];
        let result: Result<usize, ErrorTrace> =
            data_stream.read_at_position(&mut data, SeekFrom::Start(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact() -> Result<(), ErrorTrace> {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 3] = [0; 3];
        data_stream.read_exact(&mut data)?;

        assert_eq!(&data, &[0x11, 0x22, 0x33]);

        Ok(())
    }

    #[test]
    fn test_read_exact_beyond_size() {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 8] = [0; 8];
        let result: Result<(), ErrorTrace> = data_stream.read_exact(&mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact_at_position() -> Result<(), ErrorTrace> {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 2] = [0; 2];
        let offset: u64 = data_stream.read_exact_at_position(&mut data, SeekFrom::Start(3))?;

        assert_eq!(offset, 3);
        assert_eq!(&data, &[0x44, 0x55]);

        Ok(())
    }

    #[test]
    fn test_read_exact_at_position_beyond_size() {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 4] = [0; 4];
        let result: Result<u64, ErrorTrace> =
            data_stream.read_exact_at_position(&mut data, SeekFrom::Start(3));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact_at_position_with_failing_seek() {
        let mut data_stream: TestDataStream = get_test_data_stream();

        data_stream.disable_seek();

        let mut data: [u8; 3] = [0; 3];
        let result: Result<u64, ErrorTrace> =
            data_stream.read_exact_at_position(&mut data, SeekFrom::Start(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_exact_with_empty_buffer() -> Result<(), ErrorTrace> {
        let mut data_stream: TestDataStream = get_test_data_stream();

        let mut data: [u8; 0] = [];
        data_stream.read_exact(&mut data)?;

        Ok(())
    }
}
