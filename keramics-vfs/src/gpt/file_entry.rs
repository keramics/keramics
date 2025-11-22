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
use keramics_formats::PathComponent;
use keramics_formats::gpt::{GptPartition, GptVolumeSystem};
use keramics_types::Uuid;

use crate::enums::VfsFileType;

/// GUID Partition Table (GPT) file entry.
pub enum GptFileEntry {
    /// Partition file entry.
    Partition {
        /// Partition index.
        index: usize,

        /// Partition.
        partition: Arc<RwLock<GptPartition>>,

        /// Size.
        size: u64,
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<GptVolumeSystem>,
    },
}

impl GptFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            GptFileEntry::Partition { partition, .. } => Ok(Some(partition.clone())),
            GptFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            GptFileEntry::Partition { .. } => VfsFileType::File,
            GptFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> Option<Uuid> {
        match self {
            GptFileEntry::Partition { partition, .. } => match partition.read() {
                Ok(gpt_partition) => Some(gpt_partition.identifier.clone()),
                Err(_) => None,
            },
            GptFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            GptFileEntry::Partition { index, .. } => {
                PathComponent::from(format!("gpt{}", index + 1))
            }
            GptFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            GptFileEntry::Partition { size, .. } => *size,
            GptFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            GptFileEntry::Partition { .. } => 0,
            GptFileEntry::Root { volume_system } => volume_system.get_number_of_partitions(),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<GptFileEntry, ErrorTrace> {
        match self {
            GptFileEntry::Partition { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            GptFileEntry::Root { volume_system } => {
                match volume_system.get_partition_by_index(sub_file_entry_index) {
                    Ok(gpt_partition) => {
                        let partition_size: u64 = gpt_partition.size;

                        Ok(GptFileEntry::Partition {
                            index: sub_file_entry_index,
                            partition: Arc::new(RwLock::new(gpt_partition)),
                            size: partition_size,
                        })
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve GPT partition: {}", sub_file_entry_index)
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
            GptFileEntry::Partition { .. } => false,
            GptFileEntry::Root { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_volume_system() -> Result<GptVolumeSystem, ErrorTrace> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::Root);

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = gpt_partition.size;
        let file_entry = GptFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(gpt_partition)),
            size: partition_size,
        };

        let name: PathComponent = file_entry.get_name();
        assert_eq!(name, PathComponent::from("gpt1"));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 0);

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = gpt_partition.size;
        let file_entry = GptFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(gpt_partition)),
            size: partition_size,
        };

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 1048576);

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 2);

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = gpt_partition.size;
        let file_entry = GptFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(gpt_partition)),
            size: partition_size,
        };

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries();
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let sub_file_entry: GptFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: PathComponent = sub_file_entry.get_name();
        assert_eq!(name, PathComponent::from("gpt1"));

        let result: Result<GptFileEntry, ErrorTrace> = file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        assert_eq!(file_entry.is_root_file_entry(), true);

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let partition_size: u64 = gpt_partition.size;
        let file_entry = GptFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(gpt_partition)),
            size: partition_size,
        };

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
