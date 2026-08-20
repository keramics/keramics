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

use super::extent::LinuxLvmExtent;
use super::segment::LinuxLvmSegment;

/// Linux Logical Volume Manager (LVM) logical volume.
pub struct LinuxLvmLogicalVolume {
    /// Name.
    pub(super) name: String,

    /// Identifier.
    pub(super) identifier: String,

    /// Number of segments.
    pub(super) number_of_segments: u32,

    /// Segments.
    pub(super) segments: Vec<LinuxLvmSegment>,

    /// Size.
    pub(super) size: u64,

    /// Extents.
    pub(super) extents: Vec<LinuxLvmExtent>,
}

impl LinuxLvmLogicalVolume {
    /// Creates a new logical volume.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            identifier: String::new(),
            number_of_segments: 0,
            segments: Vec::new(),
            size: 0,
            extents: Vec::new(),
        }
    }
}
