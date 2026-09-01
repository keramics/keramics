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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::sgilabel::{SgiDiskLabelPartition, SgiDiskLabelVolumeSystem};
use keramics_formats::{PartitionIterator, PathComponent};

use crate::enums::VfsFileType;

/// SGI disklabel (sgilabel) file entry.
pub enum SgiDiskLabelFileEntry {
    /// Partition file entry.
    Partition {
        /// File name index.
        name_index: usize,

        /// Partition.
        partition: SgiDiskLabelPartition,
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<SgiDiskLabelVolumeSystem>,
    },
}

impl SgiDiskLabelFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            SgiDiskLabelFileEntry::Partition { partition, .. } => {
                Ok(Some(partition.get_data_stream()))
            }
            SgiDiskLabelFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            SgiDiskLabelFileEntry::Partition { .. } => VfsFileType::File,
            SgiDiskLabelFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            SgiDiskLabelFileEntry::Partition { name_index, .. } => {
                PathComponent::from(format!("sgilabel{}", name_index + 1))
            }
            SgiDiskLabelFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the partition number.
    pub fn get_partition_number(&self) -> Option<usize> {
        match self {
            SgiDiskLabelFileEntry::Partition { partition, .. } => {
                let partition_index: u8 = partition.get_partition_index();

                Some((partition_index as usize) + 1)
            }
            SgiDiskLabelFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            SgiDiskLabelFileEntry::Partition { partition, .. } => partition.get_partition_size(),
            SgiDiskLabelFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            SgiDiskLabelFileEntry::Partition { .. } => 0,
            SgiDiskLabelFileEntry::Root { volume_system } => {
                volume_system.get_number_of_partitions()
            }
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<SgiDiskLabelFileEntry, ErrorTrace> {
        match self {
            SgiDiskLabelFileEntry::Partition { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            SgiDiskLabelFileEntry::Root { volume_system } => {
                match volume_system.get_partition_by_index(sub_file_entry_index) {
                    Ok(sgilabel_partition) => Ok(SgiDiskLabelFileEntry::Partition {
                        name_index: sub_file_entry_index,
                        partition: sgilabel_partition,
                    }),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve sgilabel partition: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        match self {
            SgiDiskLabelFileEntry::Partition { .. } => false,
            SgiDiskLabelFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<SgiDiskLabelVolumeSystem, ErrorTrace> {
        let mut volume_system: SgiDiskLabelVolumeSystem = SgiDiskLabelVolumeSystem::new();

        let path_string: String = get_test_data_path("sgilabel/sgilabel.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    fn get_partition_file_entry(
        sgilabel_volume_system: &Arc<SgiDiskLabelVolumeSystem>,
    ) -> Result<SgiDiskLabelFileEntry, ErrorTrace> {
        let sgilabel_partition: SgiDiskLabelPartition =
            sgilabel_volume_system.get_partition_by_index(0)?;

        Ok(SgiDiskLabelFileEntry::Partition {
            name_index: 0,
            partition: sgilabel_partition,
        })
    }

    fn get_root_file_entry(
        sgilabel_volume_system: &Arc<SgiDiskLabelVolumeSystem>,
    ) -> SgiDiskLabelFileEntry {
        SgiDiskLabelFileEntry::Root {
            volume_system: sgilabel_volume_system.clone(),
        }
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_none());

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let data_stream: Option<DataStreamReference> = file_entry.get_data_stream()?;
        assert!(data_stream.is_some());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("sgilabel1"));

        Ok(())
    }

    #[test]
    fn test_get_partition_number() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let partition_number: Option<usize> = file_entry.get_partition_number();
        assert_eq!(partition_number, None);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let partition_number: Option<usize> = file_entry.get_partition_number();
        assert_eq!(partition_number, Some(1));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 1049088);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 1);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);

        let sub_file_entry: SgiDiskLabelFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("sgilabel1"));

        let result: Result<SgiDiskLabelFileEntry, ErrorTrace> =
            file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let sgilabel_volume_system: Arc<SgiDiskLabelVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry: SgiDiskLabelFileEntry = get_root_file_entry(&sgilabel_volume_system);
        assert_eq!(file_entry.is_root_file_entry(), true);

        let file_entry: SgiDiskLabelFileEntry = get_partition_file_entry(&sgilabel_volume_system)?;
        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
