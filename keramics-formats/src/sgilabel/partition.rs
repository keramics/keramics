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

use super::partition_entry::SgiDiskLabelPartitionEntry;

/// SGI disklabel (sgilabel) partition.
pub struct SgiDiskLabelPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// The index of the corresponding partition entry.
    entry_index: u8,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The size.
    pub(super) size: u64,

    /// The partition type.
    partition_type: u32,
}

impl SgiDiskLabelPartition {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        partition_entry: &SgiDiskLabelPartitionEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            entry_index: partition_entry.entry_index,
            offset: (partition_entry.start_sector_number as u64) * (bytes_per_sector as u64),
            size: (partition_entry.number_of_sectors as u64) * (bytes_per_sector as u64),
            partition_type: partition_entry.partition_type,
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

    /// Retrieves the partition (entry) index.
    pub fn get_partition_index(&self) -> u8 {
        self.entry_index
    }

    /// Retrieves the partition offset.
    pub fn get_partition_offset(&self) -> u64 {
        self.offset
    }

    /// Retrieves the partition size.
    pub fn get_partition_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the partition type.
    pub fn get_partition_type(&self) -> u32 {
        self.partition_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{ErrorTrace, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<SgiDiskLabelPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("sgilabel/sgilabel.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: SgiDiskLabelPartitionEntry = SgiDiskLabelPartitionEntry::new();
        partition_entry.entry_index = 0;
        partition_entry.number_of_sectors = 2049;
        partition_entry.start_sector_number = 5040;
        partition_entry.partition_type = 10;

        Ok(SgiDiskLabelPartition::new(
            &data_stream,
            512,
            &partition_entry,
        ))
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let partition: SgiDiskLabelPartition = get_partition()?;

        _ = partition.get_data_stream();

        Ok(())
    }

    #[test]
    fn test_get_partition_index() -> Result<(), ErrorTrace> {
        let partition: SgiDiskLabelPartition = get_partition()?;

        let partition_index: u8 = partition.get_partition_index();
        assert_eq!(partition_index, 0);

        Ok(())
    }

    #[test]
    fn test_get_partition_offset() -> Result<(), ErrorTrace> {
        let partition: SgiDiskLabelPartition = get_partition()?;

        let partition_offset: u64 = partition.get_partition_offset();
        assert_eq!(partition_offset, 2580480);

        Ok(())
    }

    #[test]
    fn test_get_partition_size() -> Result<(), ErrorTrace> {
        let partition: SgiDiskLabelPartition = get_partition()?;

        let partition_size: u64 = partition.get_partition_size();
        assert_eq!(partition_size, 1049088);

        Ok(())
    }

    #[test]
    fn test_get_partition_type() -> Result<(), ErrorTrace> {
        let partition: SgiDiskLabelPartition = get_partition()?;

        let partition_type: u32 = partition.get_partition_type();
        assert_eq!(partition_type, 10);

        Ok(())
    }
}
