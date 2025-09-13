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

use keramics_types::Ucs2String;

use super::attribute_list::NtfsAttributeList;
use super::constants::*;
use super::file_name::NtfsFileName;
use super::mft_attribute::NtfsMftAttribute;
use super::reparse_point::NtfsReparsePoint;
use super::standard_information::NtfsStandardInformation;
use super::volume_information::NtfsVolumeInformation;

/// New Technologies File System (NTFS) attribute.
pub enum NtfsAttribute<'a> {
    AttributeList {
        attribute_list: NtfsAttributeList,
    },
    FileName {
        file_name: NtfsFileName,
    },
    Generic {
        mft_attribute: &'a NtfsMftAttribute,
    },
    ReparsePoint {
        reparse_point: NtfsReparsePoint,
    },
    StandardInformation {
        standard_information: NtfsStandardInformation,
    },
    VolumeInformation {
        volume_information: NtfsVolumeInformation,
    },
    VolumeName {
        volume_name: Ucs2String,
    },
}

impl<'a> NtfsAttribute<'a> {
    /// Retrieves the attribute type.
    pub fn get_attribute_type(&self) -> u32 {
        match self {
            NtfsAttribute::AttributeList { .. } => NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST,
            NtfsAttribute::FileName { .. } => NTFS_ATTRIBUTE_TYPE_FILE_NAME,
            NtfsAttribute::Generic { mft_attribute } => mft_attribute.attribute_type,
            NtfsAttribute::ReparsePoint { .. } => NTFS_ATTRIBUTE_TYPE_REPARSE_POINT,
            NtfsAttribute::StandardInformation { .. } => NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION,
            NtfsAttribute::VolumeInformation { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION,
            NtfsAttribute::VolumeName { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_NAME,
        }
    }

    // TODO: add method to retrieve name
    // TODO: add methods to retrieve extents
    // TODO: add methods to read and seek data
}
