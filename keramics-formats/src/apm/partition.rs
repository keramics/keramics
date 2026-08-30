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
use keramics_types::ByteString;

use crate::range_stream::RangeStream;

use super::partition_map_entry::ApmPartitionMapEntry;

/// Apple Partition Map (APM) partition.
pub struct ApmPartition {
    /// The data stream.
    data_stream: DataStreamReference,

    /// The offset of the partition relative to start of the volume system.
    pub(super) offset: u64,

    /// The partition type identifier.
    type_identifier: ByteString,

    /// The name.
    name: ByteString,

    /// The status flags.
    status_flags: u32,

    /// The size.
    pub(super) size: u64,
}

impl ApmPartition {
    /// Creates a new partition.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        bytes_per_sector: u16,
        partition_entry: &ApmPartitionMapEntry,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            offset: (partition_entry.start_sector as u64) * (bytes_per_sector as u64),
            type_identifier: partition_entry.type_identifier.clone(),
            name: partition_entry.name.clone(),
            status_flags: partition_entry.status_flags,
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

    /// Retrieves the name.
    pub fn get_name(&self) -> &ByteString {
        &self.name
    }

    /// Retrieves the partition offset.
    pub fn get_partition_offset(&self) -> u64 {
        self.offset
    }

    /// Retrieves the partition size.
    pub fn get_partition_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the status flags.
    pub fn get_status_flags(&self) -> u32 {
        self.status_flags
    }

    /// Retrieves the type identifier.
    pub fn get_type_identifier(&self) -> &ByteString {
        &self.type_identifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{ErrorTrace, open_os_data_stream};

    use crate::tests::get_test_data_path;

    fn get_partition() -> Result<ApmPartition, ErrorTrace> {
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        let mut partition_entry: ApmPartitionMapEntry = ApmPartitionMapEntry::new();
        partition_entry.start_sector = 64;
        partition_entry.number_of_sectors = 8112;
        partition_entry.name = ByteString::from("identifier");
        partition_entry.type_identifier = ByteString::from("type_identifier");
        partition_entry.status_flags = 0x40000033;

        Ok(ApmPartition::new(&data_stream, 512, &partition_entry))
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        _ = partition.get_data_stream();

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        let name: &ByteString = partition.get_name();
        assert_eq!(name, &ByteString::from("identifier"));

        Ok(())
    }

    #[test]
    fn test_get_partition_offset() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        let partition_offset: u64 = partition.get_partition_offset();
        assert_eq!(partition_offset, 32768);

        Ok(())
    }

    #[test]
    fn test_get_partition_size() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        let partition_size: u64 = partition.get_partition_size();
        assert_eq!(partition_size, 4153344);

        Ok(())
    }

    #[test]
    fn test_get_status_flags() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        let status_flags: u32 = partition.get_status_flags();
        assert_eq!(status_flags, 0x40000033);

        Ok(())
    }

    #[test]
    fn test_get_type_identifier() -> Result<(), ErrorTrace> {
        let partition: ApmPartition = get_partition()?;

        let type_identifier: &ByteString = partition.get_type_identifier();
        assert_eq!(type_identifier, &ByteString::from("type_identifier"));

        Ok(())
    }
}
