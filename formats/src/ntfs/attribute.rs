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

use types::Ucs2String;

use super::constants::*;
use super::file_name_attribute::NtfsFileNameAttribute;
use super::standard_information_attribute::NtfsStandardInformationAttribute;
use super::volume_information_attribute::NtfsVolumeInformationAttribute;

/// New Technologies File System (NTFS) attribute.
pub enum NtfsAttribute {
    FileName {
        file_name: NtfsFileNameAttribute,
    },
    StandardInformation {
        standard_information: NtfsStandardInformationAttribute,
    },
    Undefined {
        attribute_type: u32,
    },
    VolumeInformation {
        volume_information: NtfsVolumeInformationAttribute,
    },
    VolumeName {
        volume_name: Ucs2String,
    },
}

impl NtfsAttribute {
    /// Creates a new attribute.
    pub(super) fn new(attribute_type: u32) -> Self {
        match attribute_type {
            NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION => NtfsAttribute::StandardInformation {
                standard_information: NtfsStandardInformationAttribute::new(),
            },
            NTFS_ATTRIBUTE_TYPE_FILE_NAME => NtfsAttribute::FileName {
                file_name: NtfsFileNameAttribute::new(),
            },
            NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION => NtfsAttribute::VolumeInformation {
                volume_information: NtfsVolumeInformationAttribute::new(),
            },
            NTFS_ATTRIBUTE_TYPE_VOLUME_NAME => NtfsAttribute::VolumeName {
                volume_name: Ucs2String::new(),
            },
            _ => NtfsAttribute::Undefined {
                attribute_type: attribute_type,
            },
        }
    }

    /// Retrieves the attribute type.
    pub fn get_attribute_type(&self) -> u32 {
        match self {
            NtfsAttribute::FileName { .. } => NTFS_ATTRIBUTE_TYPE_FILE_NAME,
            NtfsAttribute::StandardInformation { .. } => NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION,
            NtfsAttribute::Undefined { attribute_type } => *attribute_type,
            NtfsAttribute::VolumeInformation { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION,
            NtfsAttribute::VolumeName { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_NAME,
        }
    }

    // TODO: add method to retrieve name
    // TODO: add methods to retrieve extents
    // TODO: add methods to read and seek data
}
