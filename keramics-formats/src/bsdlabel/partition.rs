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

use super::disklabel_entry::BsdDiskLabelEntry;

/// BSD disklabel (bsdlabel) partition.
pub struct BsdDiskLabelPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// The index of the corresponding disklabel entry.
    disklabel_index: u16,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The size.
    pub(super) size: u64,
}

impl BsdDiskLabelPartition {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u32,
        disklabel_entry: &BsdDiskLabelEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            disklabel_index: disklabel_entry.entry_index,
            offset: (disklabel_entry.start_sector as u64) * (bytes_per_sector as u64),
            size: (disklabel_entry.number_of_sectors as u64) * (bytes_per_sector as u64),
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

    /// Retrieves the partition (disklabel entry) index.
    pub fn get_partition_index(&self) -> u16 {
        self.disklabel_index
    }

    /// Retrieves the partition offset.
    pub fn get_partition_offset(&self) -> u64 {
        self.offset
    }

    /// Retrieves the partition size.
    pub fn get_partition_size(&self) -> u64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{ErrorTrace, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<BsdDiskLabelPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("bsdlabel/bsdlabel.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut disklabel_entry: BsdDiskLabelEntry = BsdDiskLabelEntry::new();
        disklabel_entry.entry_index = 0;
        disklabel_entry.number_of_sectors = 8176;
        disklabel_entry.start_sector = 16;

        Ok(BsdDiskLabelPartition::new(
            &data_stream,
            512,
            &disklabel_entry,
        ))
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let partition: BsdDiskLabelPartition = get_partition()?;

        _ = partition.get_data_stream();

        Ok(())
    }

    #[test]
    fn test_get_partition_index() -> Result<(), ErrorTrace> {
        let partition: BsdDiskLabelPartition = get_partition()?;

        let partition_index: u16 = partition.get_partition_index();
        assert_eq!(partition_index, 0);

        Ok(())
    }

    #[test]
    fn test_get_partition_offset() -> Result<(), ErrorTrace> {
        let partition: BsdDiskLabelPartition = get_partition()?;

        let partition_offset: u64 = partition.get_partition_offset();
        assert_eq!(partition_offset, 8192);

        Ok(())
    }

    #[test]
    fn test_get_partition_size() -> Result<(), ErrorTrace> {
        let partition: BsdDiskLabelPartition = get_partition()?;

        let partition_size: u64 = partition.get_partition_size();
        assert_eq!(partition_size, 4186112);

        Ok(())
    }
}
