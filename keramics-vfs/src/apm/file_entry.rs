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

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        match self {
            ApmFileEntry::Partition { .. } => Ok(0),
            ApmFileEntry::Root { volume_system } => Ok(volume_system.get_number_of_partitions()),
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<ApmFileEntry, ErrorTrace> {
        match self {
            ApmFileEntry::Partition { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            ApmFileEntry::Root { volume_system } => {
                match volume_system.get_partition_by_index(sub_file_entry_index) {
                    Ok(apm_partition) => Ok(ApmFileEntry::Partition {
                        index: sub_file_entry_index,
                        partition: Arc::new(RwLock::new(apm_partition)),
                    }),
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

    use keramics_core::open_os_data_stream;

    fn get_volume_system() -> Result<ApmVolumeSystem, ErrorTrace> {
        let mut volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        let data_stream: DataStreamReference = match open_os_data_stream("../test_data/apm/apm.dmg")
        {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
                return Err(error);
            }
        };
        match volume_system.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read APM volume system from data stream"
                );
                return Err(error);
            }
        }
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
    fn test_name() -> Result<(), ErrorTrace> {
        let apm_volume_system: Arc<ApmVolumeSystem> = Arc::new(get_volume_system()?);

        let file_entry = ApmFileEntry::Root {
            volume_system: apm_volume_system.clone(),
        };

        let name: Option<String> = file_entry.get_name();
        assert!(name.is_none());

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let file_entry = ApmFileEntry::Partition {
            index: 0,
            partition: Arc::new(RwLock::new(apm_partition)),
        };

        let name: Option<String> = file_entry.get_name();
        assert_eq!(name, Some("apm1".to_string()));

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
}
