/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::io::{Read, Seek};

use crate::vfs::VfsDataStream;

/// Extended File System inline data stream.
pub struct ExtInlineDataStream {
    /// Inline data buffer.
    data: [u8; 60],

    /// The current offset.
    current_offset: u64,

    /// The size.
    pub size: u64,
}

impl ExtInlineDataStream {
    /// Creates a new inline data stream.
    pub(super) fn new(size: u64) -> Self {
        Self {
            data: [0; 60],
            current_offset: 0,
            size: size,
        }
    }

    /// Opens an inline data stream.
    pub(super) fn open(&mut self, data: &[u8]) -> io::Result<()> {
        self.data.copy_from_slice(data);

        Ok(())
    }
}

impl Read for ExtInlineDataStream {
    /// Reads data.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let data_offset: usize = self.current_offset as usize;
        let data_end_offset: usize = data_offset + read_size;

        buf.copy_from_slice(&self.data[data_offset..data_end_offset]);

        self.current_offset += read_size as u64;

        Ok(read_size)
    }
}

impl Seek for ExtInlineDataStream {
    /// Sets the current position of the data.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.current_offset = match pos {
            io::SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            io::SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            io::SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

impl VfsDataStream for ExtInlineDataStream {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.size)
    }
}
