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

use keramics_types::Uuid;

use super::enums::PdiDescriptorImageType;

/// Parallels Disk Image (PDI) descriptor image.
#[derive(Debug)]
pub(super) struct PdiDescriptorImage {
    /// File.
    pub file: String,

    /// Image type.
    pub image_type: PdiDescriptorImageType,

    /// Snapshot identifier.
    pub snapshot_identifier: Uuid,
}

impl PdiDescriptorImage {
    /// Creates a new descriptor image.
    pub fn new(
        file: String,
        image_type: PdiDescriptorImageType,
        snapshot_identifier: Uuid,
    ) -> Self {
        Self {
            file,
            image_type,
            snapshot_identifier,
        }
    }
}
