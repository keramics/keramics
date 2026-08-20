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
use keramics_types::Uuid;

use crate::range_stream::RangeStream;

use super::partition_entry::GptPartitionEntry;

/// GUID Partition Table (GPT) partition.
pub struct GptPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// The index of the corresponding partition table entry.
    partition_index: usize,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The partition type identifier.
    type_identifier: Uuid,

    /// The partition identifier.
    identifier: Uuid,

    /// The size of the partition.
    pub(super) size: u64,
}

impl GptPartition {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        partition_entry: &GptPartitionEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            partition_index: partition_entry.index,
            offset: (partition_entry.start_block_number as u64) * (bytes_per_sector as u64),
            type_identifier: partition_entry.type_identifier.clone(),
            identifier: partition_entry.identifier.clone(),
            size: partition_entry.get_number_of_blocks() * (bytes_per_sector as u64),
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

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
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
    pub fn get_type_identifier(&self) -> &Uuid {
        &self.type_identifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{ErrorTrace, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<GptPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: GptPartitionEntry = GptPartitionEntry::new(0);
        partition_entry.type_identifier =
            Uuid::from_string("ebd0a0a2-b9e5-4433-87c0-68b6b72699c7")?;
        partition_entry.identifier = Uuid::from_string("0b119671-75ff-4e2a-a31a-0bc83f857fdd")?;
        partition_entry.start_block_number = 2048;
        partition_entry.end_block_number = 2175;

        Ok(GptPartition::new(&data_stream, 512, &partition_entry))
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let partition: GptPartition = get_partition()?;

        let identifier: &Uuid = partition.get_identifier();
        assert_eq!(
            identifier.to_string(),
            "0b119671-75ff-4e2a-a31a-0bc83f857fdd"
        );
        Ok(())
    }

    #[test]
    fn test_get_partition_index() -> Result<(), ErrorTrace> {
        let partition: GptPartition = get_partition()?;

        let partition_index: usize = partition.get_partition_index();
        assert_eq!(partition_index, 0);

        Ok(())
    }

    #[test]
    fn test_get_partition_size() -> Result<(), ErrorTrace> {
        let partition: GptPartition = get_partition()?;

        let partition_size: u64 = partition.get_partition_size();
        assert_eq!(partition_size, 65536);

        Ok(())
    }

    #[test]
    fn test_get_type_identifier() -> Result<(), ErrorTrace> {
        let partition: GptPartition = get_partition()?;

        let type_identifier: &Uuid = partition.get_type_identifier();
        assert_eq!(
            type_identifier.to_string(),
            "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7"
        );
        Ok(())
    }
}
