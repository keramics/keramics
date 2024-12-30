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

use crate::types::{SharedValue, Uuid};
use crate::vfs::VfsDataStream;

use super::file::VhdxFile;

/// Virtual Hard Disk version 2 (VHDX) storage media image layer.
pub struct VhdxLayer {
    /// The file.
    file: SharedValue<VhdxFile>,

    /// The current offset.
    current_offset: u64,

    /// Identifier.
    pub identifier: Uuid,

    /// The size of the layer.
    pub size: u64,
}

impl VhdxLayer {
    /// Creates a new layer.
    pub(super) fn new() -> Self {
        Self {
            file: SharedValue::none(),
            current_offset: 0,
            identifier: Uuid::new(),
            size: 0,
        }
    }

    /// Opens a layer.
    pub(super) fn open(&mut self, file: &SharedValue<VhdxFile>) -> io::Result<()> {
        match file.with_read_lock() {
            Ok(file) => {
                self.identifier = file.identifier.clone();
                self.size = file.media_size;
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.file = file.clone();

        Ok(())
    }
}

impl Read for VhdxLayer {
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
        let read_count: usize = match self.file.with_write_lock() {
            Ok(mut file) => {
                file.seek(io::SeekFrom::Start(self.current_offset))?;

                file.read(&mut buf[0..read_size])?
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for VhdxLayer {
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

impl VfsDataStream for VhdxLayer {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.size)
    }
}
