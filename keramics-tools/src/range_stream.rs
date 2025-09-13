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
use std::io;
use std::io::{Read, Seek};

use keramics_core::DataStream;

/// Data stream of a specific range within a file.
pub struct FileRangeDataStream {
    /// The file.
    file: Option<File>,

    /// The current offset.
    current_offset: u64,

    /// The offset of the range.
    range_offset: u64,

    /// The size of the range.
    range_size: u64,
}

impl FileRangeDataStream {
    /// Creates a new data stream.
    pub fn new(range_offset: u64) -> Self {
        Self {
            file: None,
            current_offset: 0,
            range_offset: range_offset,
            range_size: 0,
        }
    }

    /// Opens a data stream.
    pub fn open(&mut self, path: &str) -> io::Result<()> {
        let file: File = File::open(path)?;
        let metadata: Metadata = file.metadata()?;

        self.file = Some(file);
        self.range_size = metadata.len() - self.range_offset;

        Ok(())
    }
}

impl Read for FileRangeDataStream {
    /// Reads data.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut file: &File = match self.file.as_mut() {
            Some(file) => file,
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing file")),
        };
        if self.current_offset >= self.range_size {
            return Ok(0);
        }
        let remaining_size: u64 = self.range_size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        file.seek(io::SeekFrom::Start(self.range_offset + self.current_offset))?;

        let read_count: usize = file.read(&mut buf[0..read_size])?;

        self.current_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for FileRangeDataStream {
    /// Sets the current position of the data.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.current_offset = match pos {
            io::SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            io::SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.range_size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            io::SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

impl DataStream for FileRangeDataStream {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.range_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for open.
    // TODO: add tests for read.
    // TODO: add tests for seek.
    // TODO: add tests for get_size.
}
