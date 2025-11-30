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

use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use super::data_stream::{DataStream, DataStreamReference};
use super::errors::ErrorTrace;

impl DataStream for File {
    /// Retrieves the current position.
    fn get_offset(&mut self) -> Result<u64, ErrorTrace> {
        match Seek::stream_position(self) {
            Ok(offset) => Ok(offset),
            Err(error) => Err(ErrorTrace::new(format!(
                "{}: Unable to retrieve current position with error: {}",
                crate::error_trace_function!(),
                error,
            ))),
        }
    }

    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        let metadata: Metadata = match self.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                return Err(ErrorTrace::new(format!(
                    "{}: Unable to retrieve file metadata with error: {}",
                    crate::error_trace_function!(),
                    error,
                )));
            }
        };
        Ok(metadata.len())
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        match Read::read(self, buf) {
            Ok(read_count) => Ok(read_count),
            Err(error) => Err(ErrorTrace::new(format!(
                "{}: Unable to read data with error: {}",
                crate::error_trace_function!(),
                error,
            ))),
        }
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        match Seek::seek(self, pos) {
            Ok(offset) => Ok(offset),
            Err(error) => Err(ErrorTrace::new(format!(
                "{}: Unable to seek position with error: {}",
                crate::error_trace_function!(),
                error,
            ))),
        }
    }
}

/// Opens a new operating system data stream.
pub fn open_os_data_stream(path_buf: &PathBuf) -> Result<DataStreamReference, ErrorTrace> {
    let file: File = match File::open(path_buf) {
        Ok(file) => file,
        Err(error) => {
            return Err(ErrorTrace::new(format!(
                "{}: Unable to open file with error: {}",
                crate::error_trace_function!(),
                error,
            )));
        }
    };
    Ok(Arc::new(RwLock::new(file)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::get_test_data_path;

    fn get_data_stream() -> Result<File, ErrorTrace> {
        let path_string: String = get_test_data_path("directory/file.txt");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file: File = match File::open(&path_buf) {
            Ok(file) => file,
            Err(error) => {
                return Err(ErrorTrace::new(format!(
                    "{}: Unable to open file with error: {}",
                    crate::error_trace_function!(),
                    error,
                )));
            }
        };
        Ok(file)
    }

    #[test]
    fn test_get_offset() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        DataStream::seek(&mut file, SeekFrom::Start(101))?;

        let offset: u64 = file.get_offset()?;
        assert_eq!(offset, 101);

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let size: u64 = file.get_size()?;
        assert_eq!(size, 202);

        Ok(())
    }

    #[test]
    fn test_open_os_data_stream() -> Result<(), ErrorTrace> {
        let path_string: String = get_test_data_path("directory/file.txt");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let _ = open_os_data_stream(&path_buf)?;

        Ok(())
    }

    #[test]
    fn test_seek() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let offset: u64 = DataStream::seek(&mut file, SeekFrom::Start(101))?;
        assert_eq!(offset, 101);

        Ok(())
    }

    #[test]
    fn test_seek_from_end() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let offset: u64 = DataStream::seek(&mut file, SeekFrom::End(-101))?;
        assert_eq!(offset, 202 - 101);

        Ok(())
    }

    #[test]
    fn test_seek_from_current() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let offset = DataStream::seek(&mut file, SeekFrom::Start(101))?;
        assert_eq!(offset, 101);

        let offset: u64 = DataStream::seek(&mut file, SeekFrom::Current(-50))?;
        assert_eq!(offset, 51);

        Ok(())
    }

    #[test]
    fn test_seek_before_zero() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let result: Result<u64, ErrorTrace> = DataStream::seek(&mut file, SeekFrom::Current(-512));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_seek_beyond_size() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        let offset: u64 = DataStream::seek(&mut file, SeekFrom::End(101))?;
        assert_eq!(offset, 202 + 101);

        Ok(())
    }

    #[test]
    fn test_seek_and_read() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;
        DataStream::seek(&mut file, SeekFrom::Start(128))?;

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = DataStream::read(&mut file, &mut data)?;
        assert_eq!(read_size, 64);

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        assert_eq!(data, expected_data.as_bytes()[128..192]);

        Ok(())
    }

    #[test]
    fn test_seek_and_read_beyond_size() -> Result<(), ErrorTrace> {
        let mut file: File = get_data_stream()?;

        DataStream::seek(&mut file, SeekFrom::End(512))?;

        let mut data: Vec<u8> = vec![0; 64];
        let read_size: usize = DataStream::read(&mut file, &mut data)?;
        assert_eq!(read_size, 0);

        Ok(())
    }
}
