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
    pub(super) fn new(file_resolver: &FileResolverReference) -> Self {
        Self {
            file_resolver: file_resolver.clone(),
            data_file_descriptors: Vec::new(),
            index: 0,
            identifier: String::new(),
            name: String::new(),
            extents: Vec::new(),
            size: 0,
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

    /// Opens a volume.
    pub(super) fn open(
        &mut self,
        data_file_descriptors: &Vec<LinuxLvmDataFileDescriptor>,
        index: usize,
        logical_volume: &LinuxLvmLogicalVolume,
    ) {
        self.index = index;
        self.data_file_descriptors = data_file_descriptors.to_vec();
        self.identifier = logical_volume.identifier.clone();
        self.name = logical_volume.name.clone();
        self.extents = logical_volume.extents.clone();
        self.size = logical_volume.size;
    }
}
