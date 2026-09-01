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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::Path;

use super::partition::VfsPartitionIdentifier;
use super::types::VfsFileSystemReference;

/// Virtual File System (VFS) partition trait for VfsPartitionFileEntry.
pub trait VfsPartition {
    const NAME_PREFIX: &'static str;

    /// Retrieves the default data stream.
    fn get_data_stream(&self) -> DataStreamReference;

    /// Retrieves the partition identifier.
    fn get_identifier(&self) -> Option<VfsPartitionIdentifier>;

    /// Retrieves the partition number.
    fn get_partition_number(&self) -> usize;

    /// Retrieves the partition size.
    fn get_partition_size(&self) -> u64;
}

/// Virtual File System (VFS) partition system trait for VfsPartitionFileSystem.
pub trait VfsPartitionSystem {
    const PATH_PREFIX: &'static str;

    /// Creates a new partition (volume) system.
    fn new() -> Self
    where
        Self: Sized;

    /// Opens the partition system from VFS.
    fn open_from_vfs(
        &mut self,
        file_system: &VfsFileSystemReference,
        path: &Path,
    ) -> Result<(), ErrorTrace>;
}
