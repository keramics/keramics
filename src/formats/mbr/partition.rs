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

use crate::types::SharedValue;
use crate::vfs::{VfsDataStream, VfsDataStreamReference};

/// Master Boot Record (MBR) partition.
pub struct MbrPartition {
    /// The data stream.
    data_stream: VfsDataStreamReference,

    /// The current offset.
    current_offset: u64,

    /// The index of the corresponding partition table entry.
    pub entry_index: usize,

    /// The offset of the partition relative to start of the volume system.
    pub offset: u64,

    /// The size of the partition.
    pub size: u64,

    /// The flags.
    pub flags: u8,
}

impl MbrPartition {
    /// Creates a new partition.
    pub(super) fn new(entry_index: usize, offset: u64, size: u64, flags: u8) -> Self {
        Self {
            data_stream: SharedValue::none(),
            current_offset: 0,
            entry_index: entry_index,
            offset: offset,
            size: size,
            flags: flags,
        }
    }

    /// Opens a partition.
    pub(super) fn open(&mut self, data_stream: &VfsDataStreamReference) -> io::Result<()> {
        self.data_stream = data_stream.clone();

        Ok(())
    }
}

impl Read for MbrPartition {
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
        let read_count: usize = match self.data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_at_position(
                &mut buf[0..read_size],
                io::SeekFrom::Start(self.offset + self.current_offset),
            )?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        self.current_offset += read_count as u64;

        Ok(read_count)
    }
}

impl Seek for MbrPartition {
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

impl VfsDataStream for MbrPartition {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64> {
        Ok(self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsFileSystemReference, VfsPath, VfsPathType};

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let parent_file_system_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let parent_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&parent_file_system_path)?;

        let mut partition = MbrPartition::new(0, 512, 66048, 0);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let data_stream: VfsDataStreamReference = match parent_file_system.with_write_lock() {
            Ok(file_system) => file_system.open_data_stream(&vfs_path, None)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        partition.open(&data_stream)?;

        Ok(())
    }
}
