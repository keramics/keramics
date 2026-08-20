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

use std::sync::{Arc, RwLock};

use keramics_core::DataStreamReference;

use crate::range_stream::RangeStream;

use super::partition_entry::MbrPartitionEntry;

/// Master Boot Record (MBR) partition.
pub struct MbrPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// The index of the corresponding partition table entry.
    partition_index: usize,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The partition type.
    partition_type: u8,

    /// The flags.
    flags: u8,

    /// The size of the partition.
    pub(super) size: u64,
}

impl MbrPartition {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        partition_entry: &MbrPartitionEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            partition_index: partition_entry.index,
            offset: partition_entry.start_address_lba * (bytes_per_sector as u64),
            partition_type: partition_entry.partition_type,
            flags: partition_entry.flags,
            size: (partition_entry.number_of_sectors as u64) * (bytes_per_sector as u64),
        }
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        Arc::new(RwLock::new(RangeStream::new(
            &self.data_stream,
            self.offset,
            self.size,
        )))
    }

    /// Retrieves the flags.
    pub fn get_flags(&self) -> u8 {
        self.flags
    }

    /// Retrieves the partition (table entry) index.
    pub fn get_partition_index(&self) -> usize {
        self.partition_index
    }

    /// Retrieves the partition offset.
    pub fn get_partition_offset(&self) -> u64 {
        self.offset
    }

    /// Retrieves the partition size.
    pub fn get_partition_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the type identifier.
    pub fn get_partition_type(&self) -> u8 {
        self.partition_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{ErrorTrace, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<MbrPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: MbrPartitionEntry = MbrPartitionEntry::new();
        partition_entry.index = 0;
        partition_entry.flags = 0x00;
        partition_entry.partition_type = 0x83;
        partition_entry.start_address_lba = 1;
        partition_entry.number_of_sectors = 129;

        Ok(MbrPartition::new(&data_stream, 512, &partition_entry))
    }

    // TODO: add tests for get_data_stream
    // TODO: add tests for get_flags
    // TODO: add tests for get_partition_index
    // TODO: add tests for get_partition_offset

    #[test]
    fn test_get_partition_size() -> Result<(), ErrorTrace> {
        let partition: MbrPartition = get_partition()?;

        let partition_size: u64 = partition.get_partition_size();
        assert_eq!(partition_size, 66048);

        Ok(())
    }

    // TODO: add tests for get_partition_type
}
