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

use crate::enums::VfsType;
use crate::location::VfsLocation;

/// Virtual File System (VFS) scan node.
#[derive(Debug)]
pub struct VfsScanNode {
    /// Location.
    pub location: VfsLocation,

    /// Sub nodes.
    pub sub_nodes: Vec<VfsScanNode>,
}

impl VfsScanNode {
    /// Creates a new scan node.
    pub(super) fn new(location: VfsLocation) -> Self {
        Self {
            location: location,
            sub_nodes: Vec::new(),
        }
    }

    /// Retrieves the type.
    pub fn get_type(&self) -> &VfsType {
        self.location.get_type()
    }

    /// Determines if the scan node is empty.
    pub fn is_empty(&self) -> bool {
        self.sub_nodes.is_empty()
    }

    /// Determines if the scan node contains a file system format.
    pub fn is_file_system(&self) -> bool {
        // Note that below a catch all match is not used to ensure a compiler
        // error is raised when a new VFS type is added.
        match self.location.get_type() {
            VfsType::Apm
            | VfsType::Ewf
            | VfsType::Fake
            | VfsType::Gpt
            | VfsType::Mbr
            | VfsType::Os
            | VfsType::Qcow
            | VfsType::SparseImage
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx => false,
            VfsType::Ext | VfsType::Fat | VfsType::Ntfs => true,
        }
    }
}
