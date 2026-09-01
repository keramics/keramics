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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::traits::PartitionIterator;

use super::disklabel::BsdDiskLabel;
use super::partition::BsdDiskLabelPartition;
use super::partition_entry::BsdDiskLabelPartitionEntry;
use super::partitions::BsdDiskLabelPartitionsIterator;

/// BSD disklabel (bsdlabel) volume system.
pub struct BsdDiskLabelVolumeSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    bytes_per_sector: u32,

    /// Disklabel entries.
    disklabel_entries: Vec<BsdDiskLabelPartitionEntry>,
}

impl BsdDiskLabelVolumeSystem {
    /// Creates a volume system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            disklabel_entries: Vec::new(),
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u32 {
        self.bytes_per_sector
    }

    /// Retrieves a partitions iterator.
    pub fn partitions(&self) -> BsdDiskLabelPartitionsIterator<'_> {
        BsdDiskLabelPartitionsIterator::new(self, self.disklabel_entries.len())
    }

    /// Reads the volume system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut disklabel: BsdDiskLabel = BsdDiskLabel::new();

        match disklabel.read_at_position(data_stream, SeekFrom::Start(512)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read disklabel at offset: 512 (0x00000200)",
                );
                return Err(error);
            }
        }
        self.bytes_per_sector = disklabel.bytes_per_sector;
        self.disklabel_entries = disklabel.entries;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

impl PartitionIterator for BsdDiskLabelVolumeSystem {
    type PartitionItem = BsdDiskLabelPartition;

    /// Retrieves the number of partitions.
    fn get_number_of_partitions(&self) -> usize {
        self.disklabel_entries.len()
    }

    /// Retrieves a partition by index.
    fn get_partition_by_index(
        &self,
        partition_index: usize,
    ) -> Result<Self::PartitionItem, ErrorTrace> {
        match self.disklabel_entries.get(partition_index) {
            Some(partition_entry) => match self.data_stream.as_ref() {
                Some(data_stream) => Ok(BsdDiskLabelPartition::new(
                    &data_stream,
                    self.bytes_per_sector,
                    partition_entry,
                )),
                None => Err(keramics_core::error_trace_new!("Missing data stream")),
            },
            None => Err(keramics_core::error_trace_new!(format!(
                "No partition with index: {}",
                partition_index
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::{open_fake_data_stream_with_offset, open_os_data_stream};

    use crate::tests::get_test_data_path;

    const TEST_DISKLABEL: [u8; 512] = [
        0x57, 0x45, 0x56, 0x82, 0x00, 0x00, 0x00, 0x00, 0x61, 0x6d, 0x6e, 0x65, 0x73, 0x69, 0x61,
        0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x3f,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x82, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00,
        0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x0e, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x56,
        0x82, 0x67, 0x31, 0x08, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x1f,
        0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    fn get_volume_system() -> Result<BsdDiskLabelVolumeSystem, ErrorTrace> {
        let mut volume_system: BsdDiskLabelVolumeSystem = BsdDiskLabelVolumeSystem::new();

        let path_string: String = get_test_data_path("bsdlabel/bsdlabel.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let volume_system: BsdDiskLabelVolumeSystem = get_volume_system()?;

        let bytes_per_sector: u32 = volume_system.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_number_of_partitions() -> Result<(), ErrorTrace> {
        let volume_system: BsdDiskLabelVolumeSystem = get_volume_system()?;

        let number_of_partitions: usize = volume_system.get_number_of_partitions();
        assert_eq!(number_of_partitions, 1);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index() -> Result<(), ErrorTrace> {
        let volume_system: BsdDiskLabelVolumeSystem = get_volume_system()?;

        let partition: BsdDiskLabelPartition = volume_system.get_partition_by_index(0)?;

        assert_eq!(partition.offset, 8192);
        assert_eq!(partition.size, 4186112);

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index_with_unsupported_partition_index() -> Result<(), ErrorTrace> {
        let volume_system: BsdDiskLabelVolumeSystem = get_volume_system()?;

        let result: Result<BsdDiskLabelPartition, ErrorTrace> =
            volume_system.get_partition_by_index(1);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_partition_by_index_without_data_stream() {
        let mut volume_system: BsdDiskLabelVolumeSystem = BsdDiskLabelVolumeSystem::new();
        volume_system
            .disklabel_entries
            .push(BsdDiskLabelPartitionEntry::new());

        let result = volume_system.get_partition_by_index(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_partitions() -> Result<(), ErrorTrace> {
        let volume_system: BsdDiskLabelVolumeSystem = get_volume_system()?;

        let mut partitions_iterator: BsdDiskLabelPartitionsIterator = volume_system.partitions();

        let result: Option<Result<BsdDiskLabelPartition, ErrorTrace>> = partitions_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<BsdDiskLabelPartition, ErrorTrace>> =
            partitions_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut volume_system: BsdDiskLabelVolumeSystem = BsdDiskLabelVolumeSystem::new();

        let path_string: String = get_test_data_path("bsdlabel/bsdlabel.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        assert_eq!(volume_system.bytes_per_sector, 512);
        assert_eq!(volume_system.get_number_of_partitions(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_unsupported_size() {
        let mut volume_system: BsdDiskLabelVolumeSystem = BsdDiskLabelVolumeSystem::new();

        let data_stream: DataStreamReference =
            open_fake_data_stream_with_offset(&TEST_DISKLABEL[0..147], 512);

        let result: Result<(), ErrorTrace> = volume_system.read_data_stream(&data_stream);
        assert!(result.is_err());
    }
}
