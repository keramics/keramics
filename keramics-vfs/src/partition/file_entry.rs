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
use keramics_formats::{PartitionIterator, PathComponent};

use crate::enums::VfsFileType;
use crate::traits::VfsPartition;

use super::identifier::VfsPartitionIdentifier;

/// Virtual File System (VFS) partition based file entry.
pub enum VfsPartitionFileEntry<P: VfsPartition, V: PartitionIterator<PartitionItem = P>> {
    /// Partition file entry.
    Partition {
        /// File name index.
        name_index: usize,

        /// Partition.
        partition: P,
    },

    /// Root file entry.
    Root {
        /// Volume system.
        volume_system: Arc<V>,
    },
}

impl<P: VfsPartition, V: PartitionIterator<PartitionItem = P>> VfsPartitionFileEntry<P, V> {
    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self {
            VfsPartitionFileEntry::Partition { partition, .. } => {
                Ok(Some(partition.get_data_stream()))
            }
            VfsPartitionFileEntry::Root { .. } => Ok(None),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VfsPartitionFileEntry::Partition { .. } => VfsFileType::File,
            VfsPartitionFileEntry::Root { .. } => VfsFileType::Directory,
        }
    }

    /// Retrieves the partition identifier.
    pub fn get_identifier(&self) -> Option<VfsPartitionIdentifier> {
        match self {
            VfsPartitionFileEntry::Partition { partition, .. } => partition.get_identifier(),
            VfsPartitionFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> PathComponent {
        match self {
            VfsPartitionFileEntry::Partition { name_index, .. } => {
                PathComponent::from(format!("{}{}", P::NAME_PREFIX, name_index + 1))
            }
            VfsPartitionFileEntry::Root { .. } => PathComponent::Root,
        }
    }

    /// Retrieves the partition number.
    pub fn get_partition_number(&self) -> Option<usize> {
        match self {
            VfsPartitionFileEntry::Partition { partition, .. } => {
                Some(partition.get_partition_number())
            }
            VfsPartitionFileEntry::Root { .. } => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VfsPartitionFileEntry::Partition { partition, .. } => partition.get_partition_size(),
            VfsPartitionFileEntry::Root { .. } => 0,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&self) -> usize {
        match self {
            VfsPartitionFileEntry::Partition { .. } => 0,
            VfsPartitionFileEntry::Root { volume_system } => {
                volume_system.get_number_of_partitions()
            }
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &self,
        sub_file_entry_index: usize,
    ) -> Result<VfsPartitionFileEntry<P, V>, ErrorTrace> {
        match self {
            VfsPartitionFileEntry::Partition { .. } => {
                Err(keramics_core::error_trace_new!("No sub file entries"))
            }
            VfsPartitionFileEntry::Root { volume_system } => {
                match volume_system.get_partition_by_index(sub_file_entry_index) {
                    Ok(partition) => Ok(VfsPartitionFileEntry::Partition {
                        name_index: sub_file_entry_index,
                        partition,
                    }),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve partition: {}", sub_file_entry_index)
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
            VfsPartitionFileEntry::Partition { .. } => false,
            VfsPartitionFileEntry::Root { .. } => true,
        }
    }
}
