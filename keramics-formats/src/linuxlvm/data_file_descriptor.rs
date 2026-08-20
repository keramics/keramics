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

use crate::path_component::PathComponent;

/// Linux Logical Volume Manager (LVM) data file descriptor.
#[derive(Clone)]
pub struct LinuxLvmDataFileDescriptor {
    /// File name.
    pub(super) file_name: PathComponent,

    /// Start offset.
    pub(super) start_offset: u64,
}

impl LinuxLvmDataFileDescriptor {
    /// Creates a new data file descriptor.
    pub fn new(file_name: PathComponent, start_offset: u64) -> Self {
        Self {
            file_name,
            start_offset,
        }
    }
}
