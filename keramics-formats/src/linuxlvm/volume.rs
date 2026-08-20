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

use std::sync::{Arc, RwLock};

use keramics_core::DataStreamReference;

use crate::block_stream::BlockStream;
use crate::file_resolver::FileResolverReference;

use super::block_reader::LinuxLvmBlockReader;
use super::data_file_descriptor::LinuxLvmDataFileDescriptor;
use super::extent::LinuxLvmExtent;
use super::logical_volume::LinuxLvmLogicalVolume;

/// Linux Logical Volume Manager (LVM) volume.
pub struct LinuxLvmVolume {
    /// File resolver.
    file_resolver: FileResolverReference,

    /// Data file descriptors.
    data_file_descriptors: Vec<LinuxLvmDataFileDescriptor>,

    /// Index.
    index: usize,

    /// Identifier.
    identifier: String,

    /// Name.
    name: String,

    /// Extents.
    extents: Vec<LinuxLvmExtent>,

    /// The size.
    size: u64,
}

impl LinuxLvmVolume {
    /// Creates a new volume.
    pub(super) fn new(
        file_resolver: &FileResolverReference,
        data_file_descriptors: &[LinuxLvmDataFileDescriptor],
        index: usize,
        logical_volume: &LinuxLvmLogicalVolume,
    ) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            data_file_descriptors: data_file_descriptors.to_vec(),
            index,
            identifier: logical_volume.identifier.clone(),
            name: logical_volume.name.clone(),
            extents: logical_volume.extents.clone(),
            size: logical_volume.size,
        }
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(&self) -> DataStreamReference {
        let block_reader: LinuxLvmBlockReader = LinuxLvmBlockReader::new(
            &self.file_resolver,
            &self.data_file_descriptors,
            &self.extents,
            self.size,
        );
        Arc::new(RwLock::new(BlockStream::<LinuxLvmBlockReader>::new(
            block_reader,
        )))
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &str {
        self.identifier.as_str()
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Retrieves the volume index.
    pub fn get_volume_index(&self) -> usize {
        self.index
    }

    /// Retrieves the volume size.
    pub fn get_volume_size(&self) -> u64 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::ErrorTrace;

    use crate::os_file_resolver::open_os_file_resolver;
    use crate::path_component::PathComponent;
    use crate::tests::get_test_data_path;

    fn get_volume() -> Result<LinuxLvmVolume, ErrorTrace> {
        let path_string: String = get_test_data_path("linuxlvm/lvm2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let file_resolver: FileResolverReference = open_os_file_resolver(&path_buf)?;

        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(
                PathComponent::from("lvm2.raw"),
                0,
            )];

        let mut logical_volume: LinuxLvmLogicalVolume = LinuxLvmLogicalVolume::new();
        logical_volume.size = 4194304;

        Ok(LinuxLvmVolume::new(
            &file_resolver,
            &data_file_descriptors,
            512,
            &logical_volume,
        ))
    }

    // TODO: add tests for get_data_stream
    // TODO: add tests for get_identifier
    // TODO: add tests for get_name
    // TODO: add tests for get_volume_index

    #[test]
    fn test_get_volume_size() -> Result<(), ErrorTrace> {
        let volume: LinuxLvmVolume = get_volume()?;

        let volume_size: u64 = volume.get_volume_size();
        assert_eq!(volume_size, 4194304);

        Ok(())
    }
}
