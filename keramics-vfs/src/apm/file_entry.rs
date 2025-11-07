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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};

use crate::enums::VfsFileType;

/// Apple Partition Map (APM) file entry.
pub enum ApmFileEntry {
    /// Partition file entry.
    Partition {
        /// Partition index.
        index: usize,

        /// Partition.
        partition: Arc<RwLock<ApmPartition>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<ApmVolumeSystem>,
    },
}

impl ApmFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            ApmFileEntry::Partition { partition, .. } => Ok(Some(partition.clone())),
            ApmFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            ApmFileEntry::Partition { .. } => VfsFileType::File,
            ApmFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            ApmFileEntry::Partition { index, .. } => Some(format!("apm{}", index + 1)),
            ApmFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            ApmFileEntry::Partition { size, .. } => *size,
            ApmFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> Result<usize, ErrorTrace> {
        match self {
            ApmFileEntry::Partition { .. } => Ok(0),
            ApmFileEntry::Root { volume_system } => Ok(volume_system.get_number_of_partitions()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<ApmFileEntry, ErrorTrace> {
        match self {
            ApmFileEntry::Partition { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            ApmFileEntry::Root { volume_system } => {
                match volume_system.get_partition_by_index(sub_file_entry_index) {
                    Ok(apm_partition) => {
                        let partition_size: u64 = apm_partition.size;

                        Ok(ApmFileEntry::Partition {
                            index: sub_file_entry_index,
                            partition: Arc::new(RwLock::new(apm_partition)),
                            size: partition_size,
                        })
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve APM partition: {}", sub_file_entry_index)
                        );
                        return Err(error);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<ApmVolumeSystem, ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("apm/apm.dmg").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = apm_partition.size;
        let file_entry = ApmFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(apm_partition)),
            size: partition_size,
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some(String::from("apm1")));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = apm_partition.size;
        let file_entry = ApmFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(apm_partition)),
            size: partition_size,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 4153344);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = apm_partition.size;
        let file_entry = ApmFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(apm_partition)),
            size: partition_size,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let sub_file_entry: ApmFileEntry = file_entry.get_sub_file_entry_by_index(0)?;
        assert_eq!(sub_file_entry.get_name(), Some(String::from("apm1")));

        let result: Result<ApmFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }
}
