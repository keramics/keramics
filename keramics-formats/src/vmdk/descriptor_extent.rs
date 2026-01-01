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

use keramics_types::ByteString;

use super::enums::{VmdkDescriptorExtentAccessMode, VmdkDescriptorExtentType};

/// VMware Virtual Disk (VMDK) descriptor extent.
#[derive(Debug)]
pub struct VmdkDescriptorExtent {
    /// Start sector.
    pub start_sector: u64,

    /// Number of sectors.
    pub number_of_sectors: u64,

    /// File name.
    pub file_name: Option<ByteString>,

    /// Extent type.
    pub extent_type: VmdkDescriptorExtentType,

    /// Access mode.
    pub access_mode: VmdkDescriptorExtentAccessMode,
}

impl VmdkDescriptorExtent {
    /// Creates a new descriptor extent.
    pub fn new(
        start_sector: u64,
        number_of_sectors: u64,
        file_name: Option<ByteString>,
        extent_type: VmdkDescriptorExtentType,
        access_mode: VmdkDescriptorExtentAccessMode,
    ) -> Self {
        Self {
            start_sector,
            number_of_sectors,
            file_name,
            extent_type,
            access_mode,
        }
    }
}
