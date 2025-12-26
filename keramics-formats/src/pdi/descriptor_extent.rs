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

use super::descriptor_image::PdiDescriptorImage;

/// Parallels Disk Image (PDI) descriptor extent.
#[derive(Debug)]
pub(super) struct PdiDescriptorExtent {
    /// Start sector.
    pub start_sector: u64,

    /// End sector.
    pub end_sector: u64,

    /// Images.
    pub images: Vec<PdiDescriptorImage>,
}

impl PdiDescriptorExtent {
    /// Creates a new descriptor extent.
    pub fn new(start_sector: u64, end_sector: u64, images: Vec<PdiDescriptorImage>) -> Self {
        Self {
            start_sector,
            end_sector,
            images,
        }
    }
}
