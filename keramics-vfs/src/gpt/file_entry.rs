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

use std::io;
use std::sync::{Arc, RwLock};

use keramics_core::DataStreamReference;
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
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<GptVolumeSystem>,
    },
}

impl GptFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
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
    pub fn get_name(&self) -> Option<String> {
        match self {
            GptFileEntry::Partition { index, .. } => Some(format!("gpt{}", index + 1)),
            GptFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        match self {
            GptFileEntry::Partition { .. } => Ok(0),
            GptFileEntry::Root { volume_system } => Ok(volume_system.get_number_of_partitions()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<GptFileEntry> {
        match self {
            GptFileEntry::Partition { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No sub file entries",
            )),
            GptFileEntry::Root { volume_system } => {
                let gpt_partition: GptPartition =
                    volume_system.get_partition_by_index(sub_file_entry_index)?;

                Ok(GptFileEntry::Partition {
                    index: sub_file_entry_index,
                    partition: Arc::new(RwLock::new(gpt_partition)),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_volume_system() -> io::Result<GptVolumeSystem> {
        let mut volume_system: GptVolumeSystem = GptVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/gpt/gpt.raw")?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> io::Result<()> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> io::Result<()> {
        let gpt_volume_system: Arc<GptVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = GptFileEntry::Root {
            volume_system: gpt_volume_system.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;

        let file_entry = GptFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(gpt_partition)),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("gpt1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
