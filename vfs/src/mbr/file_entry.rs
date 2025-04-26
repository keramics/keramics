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

use core::DataStreamReference;
use formats::mbr::{MbrPartition, MbrVolumeSystem};

use crate::enums::VfsFileType;

/// Master Boot Record (MBR) file entry.
pub enum MbrFileEntry {
    /// Partition file entry.
    Partition {
        /// Partition index.
        index: usize,

        /// Partition.
        partition: Arc<RwLock<MbrPartition>>,
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<MbrVolumeSystem>,
    },
}

impl MbrFileEntry {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
        match self {
            MbrFileEntry::Partition { partition, .. } => Ok(Some(partition.clone())),
            MbrFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            MbrFileEntry::Partition { .. } => VfsFileType::File,
            MbrFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<String> {
        match self {
            MbrFileEntry::Partition { index, .. } => Some(format!("mbr{}", index + 1)),
            MbrFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        let number_of_sub_file_entries: usize = match self {
            MbrFileEntry::Partition { .. } => 0,
            MbrFileEntry::Root { volume_system } => volume_system.get_number_of_partitions(),
        };
        Ok(number_of_sub_file_entries)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<MbrFileEntry> {
        match self {
            MbrFileEntry::Partition { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing partitions",
            )),
            MbrFileEntry::Root { volume_system } => {
                let mbr_partition: MbrPartition =
                    volume_system.get_partition_by_index(sub_file_entry_index)?;

                Ok(MbrFileEntry::Partition {
                    index: sub_file_entry_index,
                    partition: Arc::new(RwLock::new(mbr_partition)),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::open_os_data_stream;

    fn get_volume_system() -> io::Result<MbrVolumeSystem> {
        let mut volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/mbr/mbr.raw")?;
        volume_system.read_data_stream(&data_stream)?;

        Ok(volume_system)
    }

    // TODO: add tests for get_data_stream

    #[test]
    fn test_get_file_type() -> io::Result<()> {
        let mbr_volume_system: Arc<MbrVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = MbrFileEntry::Root {
            volume_system: mbr_volume_system.clone(),
        };

        let file_type: VfsFileType = file_entry.get_file_type();
        assert!(file_type == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_name() -> io::Result<()> {
        let mbr_volume_system: Arc<MbrVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = MbrFileEntry::Root {
            volume_system: mbr_volume_system.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let mbr_partition: MbrPartition = mbr_volume_system.get_partition_by_index(0)?;

        let file_entry = MbrFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(mbr_partition)),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("mbr1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
