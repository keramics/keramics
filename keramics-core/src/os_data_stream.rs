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
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        let metadata: Metadata = match self.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                return Err(ErrorTrace::new(format!(
                    "{}: Unable to retrieve file metadata with error: {}",
                    crate::error_trace_function!(),
                    error.to_string(),
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
                error.to_string(),
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
                error.to_string(),
            ))),
        }
    }
}

/// Opens a file.
macro_rules! open_file {
    ( $path:expr ) => {
        match File::open($path) {
            Ok(file) => file,
            Err(error) => {
                return Err(ErrorTrace::new(format!(
                    "{}: Unable to open file with error: {}",
                    crate::error_trace_function!(),
                    error.to_string(),
                )));
            }
        }
    };
}

/// Opens a new operating system data stream.
pub fn open_os_data_stream(path: &PathBuf) -> Result<DataStreamReference, ErrorTrace> {
    let file: File = open_file!(path);

    Ok(Arc::new(RwLock::new(file)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let mut file: File = open_file!("../test_data/file.txt");

        let size: u64 = file.get_size()?;
        assert_eq!(size, 202);

        Ok(())
    }

    #[test]
    fn test_open_os_data_stream() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/file.txt");
        let _ = open_os_data_stream(&path_buf)?;

        Ok(())
    }
}
