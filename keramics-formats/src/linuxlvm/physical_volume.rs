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

/// Linux Logical Volume Manager (LVM) physical volume.
pub struct LinuxLvmPhysicalVolume {
    /// Index.
    pub(super) index: usize,

    /// Name.
    pub(super) name: String,

    /// Identifier.
    pub(super) identifier: String,

    /// Device path.
    pub(super) device_path: String,

    /// Device size.
    pub(super) device_size: u64,

    /// Start extent.
    pub(super) start_extent: u32,

    /// Number of extents.
    pub(super) number_of_extents: u32,
}

impl LinuxLvmPhysicalVolume {
    /// Creates a new physical volume.
    pub fn new() -> Self {
        Self {
            index: 0,
            name: String::new(),
            identifier: String::new(),
            device_path: String::new(),
            device_size: 0,
            start_extent: 0,
            number_of_extents: 0,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &str {
        self.identifier.as_str()
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
