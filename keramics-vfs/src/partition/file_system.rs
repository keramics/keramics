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

use keramics_core::ErrorTrace;
use keramics_formats::{PartitionIterator, Path};

use crate::location::VfsLocation;
use crate::path::VfsPath;
use crate::traits::{VfsPartition, VfsPartitionSystem};
use crate::types::VfsFileSystemReference;

use super::file_entry::VfsPartitionFileEntry;

/// Virtual File System (VFS) partition based file system.
pub struct VfsPartitionFileSystem<
    P: VfsPartition,
    V: VfsPartitionSystem + PartitionIterator<PartitionItem = P>,
> {
    /// Partition system.
    partition_system: Arc<V>,

    /// Number of partitions.
    pub(crate) number_of_partitions: usize,
}

impl<P: VfsPartition, V: VfsPartitionSystem + PartitionIterator<PartitionItem = P>>
    VfsPartitionFileSystem<P, V>
{
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            partition_system: Arc::new(VfsPartitionSystem::new()),
            number_of_partitions: 0,
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &Path) -> bool {
        if path.is_relative() {
            return false;
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return false;
                }
                let partition_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, P::NAME_PREFIX) {
                        Some(partition_index) => partition_index,
                        None => return false,
                    };
                if partition_index == 0 || partition_index > self.number_of_partitions {
                    false
                } else {
                    true
                }
            }
            None => {
                if path.is_empty() {
                    false
                } else {
                    true
                }
            }
        }
    }

    /// Retrieves the file entry with the specific location.
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<VfsPartitionFileEntry<P, V>>, ErrorTrace> {
        if path.is_relative() {
            return Ok(None);
        }
        match path.get_component_by_index(1) {
            Some(path_component) => {
                if path.get_number_of_components() > 2 {
                    return Ok(None);
                }
                let mut partition_index: usize =
                    match VfsPath::get_numeric_suffix(path_component, P::NAME_PREFIX) {
                        Some(partition_index) => partition_index,
                        None => return Ok(None),
                    };
                if partition_index == 0 || partition_index > self.number_of_partitions {
                    return Ok(None);
                }
                partition_index -= 1;

                let partition: P = match self
                    .partition_system
                    .get_partition_by_index(partition_index)
                {
                    Ok(partition) => partition,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve partition: {}", partition_index)
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsPartitionFileEntry::Partition {
                    name_index: partition_index,
                    partition,
                }))
            }
            None => {
                if path.is_empty() {
                    return Ok(None);
                }
                Ok(Some(self.get_root_file_entry()))
            }
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> VfsPartitionFileEntry<P, V> {
        VfsPartitionFileEntry::Root {
            volume_system: self.partition_system.clone(),
        }
    }

    /// Opens the file system.
    pub fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let path: &Path = vfs_location.get_path();

        match Arc::get_mut(&mut self.partition_system) {
            Some(partition_system) => {
                match partition_system.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open partition system"
                        );
                        return Err(error);
                    }
                }
                self.number_of_partitions = partition_system.get_number_of_partitions();
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to partition system"
                ));
            }
        }
        Ok(())
    }
}
