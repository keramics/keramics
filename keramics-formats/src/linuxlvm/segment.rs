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

use super::stripe::LinuxLvmStripe;

/// Linux Logical Volume Manager (LVM) segment.
pub struct LinuxLvmSegment {
    /// Name.
    pub(super) name: String,

    /// Number of extents.
    pub(super) number_of_extents: u32,

    /// Start extent.
    pub(super) start_extent: u32,

    /// Segment type.
    pub(super) segment_type: String,

    /// Number of stripes.
    pub(super) number_of_stripes: u32,

    /// Stripes.
    pub(super) stripes: Vec<LinuxLvmStripe>,
}

impl LinuxLvmSegment {
    /// Creates a new segment.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            stripes: Vec::new(),
            number_of_extents: 0,
            segment_type: String::new(),
            start_extent: 0,
            number_of_stripes: 0,
        }
    }
}
