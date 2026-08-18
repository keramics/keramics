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

use super::logical_volume::LinuxLvmLogicalVolume;
use super::physical_volume::LinuxLvmPhysicalVolume;

/// Linux Logical Volume Manager (LVM) volume group.
pub struct LinuxLvmVolumeGroup {
    /// Name.
    pub(super) name: String,

    /// Identifier.
    pub(super) identifier: String,

    /// Extent size.
    pub(super) extent_size: u32,

    /// Sequence number.
    pub(super) sequence_number: u32,

    /// Number of metadata copies.
    pub(super) number_of_metadata_copies: u32,

    /// Logical volumes.
    pub(super) logical_volumes: Vec<LinuxLvmLogicalVolume>,

    /// Physical volumes.
    pub(super) physical_volumes: Vec<LinuxLvmPhysicalVolume>,
}

impl LinuxLvmVolumeGroup {
    /// Creates a new volume group.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            identifier: String::new(),
            extent_size: 0,
            sequence_number: 0,
            number_of_metadata_copies: 0,
            logical_volumes: Vec::new(),
            physical_volumes: Vec::new(),
        }
    }
}
