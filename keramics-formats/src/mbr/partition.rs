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

use std::io::SeekFrom;

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

/// Master Boot Record (MBR) partition.
pub struct MbrPartition {
    /// The data stream.
    data_stream: Option<DataStreamReference>,

    /// The current offset.
    current_offset: u64,

    /// The index of the corresponding partition table entry.
    pub entry_index: usize,

    /// The offset of the partition relative to start of the volume system.
    pub offset: u64,

    /// The size of the partition.
    pub size: u64,

    /// The partition type.
    pub partition_type: u8,

    /// The flags.
    pub flags: u8,
}

impl MbrPartition {
    /// Creates a new partition.
    pub(super) fn new(
        entry_index: usize,
        offset: u64,
        size: u64,
        partition_type: u8,
        flags: u8,
    ) -> Self {
        Self {
            data_stream: None,
            current_offset: 0,
            entry_index: entry_index,
            offset: offset,
            size: size,
            partition_type: partition_type,
            flags: flags,
        }
    }

    /// Opens a partition.
    pub(super) fn open(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

impl DataStream for MbrPartition {
    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        Ok(self.size)
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        if self.current_offset >= self.size {
            return Ok(0);
        }
        let remaining_size: u64 = self.size - self.current_offset;
        let mut read_size: usize = buf.len();

        if (read_size as u64) > remaining_size {
            read_size = remaining_size as usize;
        }
        let read_count: usize = keramics_core::data_stream_read_at_position!(
            data_stream,
            &mut buf[0..read_size],
            SeekFrom::Start(self.offset + self.current_offset)
        );
        self.current_offset += read_count as u64;

        Ok(read_count)
    }

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        self.current_offset = match pos {
            SeekFrom::Current(relative_offset) => {
                let mut current_offset: i64 = self.current_offset as i64;
                current_offset += relative_offset;
                current_offset as u64
            }
            SeekFrom::End(relative_offset) => {
                let mut end_offset: i64 = self.size as i64;
                end_offset += relative_offset;
                end_offset as u64
            }
            SeekFrom::Start(offset) => offset,
        };
        Ok(self.current_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut partition = MbrPartition::new(0, 512, 66048, 0x83, 0x00);

        let path_buf: PathBuf = PathBuf::from("../test_data/mbr/mbr.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        partition.open(&data_stream)?;

        Ok(())
    }

    // TODO: add tests for read.
    // TODO: add tests for seek.
    // TODO: add tests for get_size.
}
