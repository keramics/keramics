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

use keramics_core::{DataStreamReference, ErrorTrace};
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
        data_stream: &'a DataStreamReference,
        cluster_block_size: u32,
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
            NtfsAttribute::Generic { mft_attribute, .. } => mft_attribute.attribute_type,
            NtfsAttribute::ReparsePoint { .. } => NTFS_ATTRIBUTE_TYPE_REPARSE_POINT,
            NtfsAttribute::StandardInformation { .. } => NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION,
            NtfsAttribute::VolumeInformation { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION,
            NtfsAttribute::VolumeName { .. } => NTFS_ATTRIBUTE_TYPE_VOLUME_NAME,
        }
    }

    /// Retrieves the attribute name (if present).
    pub fn get_name(&self) -> Option<&Ucs2String> {
        match self {
            NtfsAttribute::Generic { mft_attribute, .. } => mft_attribute.name.as_ref(),
            _ => None,
        }
    }

    /// Retrieves the allocated data size.
    pub fn get_allocated_data_size(&self) -> u64 {
        match self {
            NtfsAttribute::Generic { mft_attribute, .. } => mft_attribute.allocated_data_size,
            _ => 0,
        }
    }

    /// Retrieves the data size.
    pub fn get_data_size(&self) -> u64 {
        match self {
            NtfsAttribute::Generic { mft_attribute, .. } => mft_attribute.data_size,
            _ => 0,
        }
    }

    /// Retrieves the valid data size.
    pub fn get_valid_data_size(&self) -> u64 {
        match self {
            NtfsAttribute::Generic { mft_attribute, .. } => mft_attribute.valid_data_size,
            _ => 0,
        }
    }

    /// Retrieves the data stream for this attribute.
    pub fn get_data_stream(&self) -> Result<DataStreamReference, ErrorTrace> {
        match self {
            NtfsAttribute::Generic {
                data_stream,
                cluster_block_size,
                mft_attribute,
            } => mft_attribute.get_data_stream(data_stream, *cluster_block_size),
            _ => Err(keramics_core::error_trace_new!(
                "Unsupported attribute type without data stream"
            )),
        }
    }

    /// Retrieves the data stream for this attribute with a specific valid data size.
    pub fn get_data_stream_with_valid_data_size(
        &self,
        valid_data_size: u64,
    ) -> Result<DataStreamReference, ErrorTrace> {
        match self {
            NtfsAttribute::Generic {
                data_stream,
                cluster_block_size,
                mft_attribute,
            } => mft_attribute.get_data_stream_with_valid_data_size(
                data_stream,
                *cluster_block_size,
                valid_data_size,
            ),
            _ => Err(keramics_core::error_trace_new!(
                "Unsupported attribute type without data stream"
            )),
        }
    }

    // TODO: add methods to retrieve extents
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    #[test]
    fn test_get_attribute_type() {
        let attribute: NtfsAttribute = NtfsAttribute::AttributeList {
            attribute_list: NtfsAttributeList::new(),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST
        );

        let attribute: NtfsAttribute = NtfsAttribute::FileName {
            file_name: NtfsFileName::new(),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_FILE_NAME
        );

        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        mft_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_DATA;
        let attribute: NtfsAttribute = NtfsAttribute::Generic {
            data_stream: &data_stream,
            cluster_block_size: 4096,
            mft_attribute: &mft_attribute,
        };
        assert_eq!(attribute.get_attribute_type(), NTFS_ATTRIBUTE_TYPE_DATA);

        let attribute: NtfsAttribute = NtfsAttribute::ReparsePoint {
            reparse_point: NtfsReparsePoint::new(0),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_REPARSE_POINT
        );

        let attribute: NtfsAttribute = NtfsAttribute::StandardInformation {
            standard_information: NtfsStandardInformation::new(),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION
        );

        let attribute: NtfsAttribute = NtfsAttribute::VolumeInformation {
            volume_information: NtfsVolumeInformation::new(),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION
        );

        let attribute: NtfsAttribute = NtfsAttribute::VolumeName {
            volume_name: Ucs2String::from("test"),
        };
        assert_eq!(
            attribute.get_attribute_type(),
            NTFS_ATTRIBUTE_TYPE_VOLUME_NAME
        );
    }

    #[test]
    fn test_get_name() {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        mft_attribute.name = Some(Ucs2String::from("test"));
        let attribute: NtfsAttribute = NtfsAttribute::Generic {
            data_stream: &data_stream,
            cluster_block_size: 4096,
            mft_attribute: &mft_attribute,
        };
        let expected_name: Ucs2String = Ucs2String::from("test");
        assert_eq!(attribute.get_name(), Some(&expected_name));

        let mut mft_attribute_unnamed: NtfsMftAttribute = NtfsMftAttribute::new();
        mft_attribute_unnamed.name = None;
        let attribute: NtfsAttribute = NtfsAttribute::Generic {
            data_stream: &data_stream,
            cluster_block_size: 4096,
            mft_attribute: &mft_attribute_unnamed,
        };
        assert_eq!(attribute.get_name(), None);

        let attribute: NtfsAttribute = NtfsAttribute::StandardInformation {
            standard_information: NtfsStandardInformation::new(),
        };
        assert_eq!(attribute.get_name(), None);
    }

    #[test]
    fn test_get_data_size_and_valid_data_size() {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        mft_attribute.allocated_data_size = 12288;
        mft_attribute.data_size = 11358;
        mft_attribute.valid_data_size = 8192;
        let attribute: NtfsAttribute = NtfsAttribute::Generic {
            data_stream: &data_stream,
            cluster_block_size: 4096,
            mft_attribute: &mft_attribute,
        };
        assert_eq!(attribute.get_allocated_data_size(), 12288);
        assert_eq!(attribute.get_data_size(), 11358);
        assert_eq!(attribute.get_valid_data_size(), 8192);

        let attribute_non_generic: NtfsAttribute = NtfsAttribute::StandardInformation {
            standard_information: NtfsStandardInformation::new(),
        };
        assert_eq!(attribute_non_generic.get_allocated_data_size(), 0);
        assert_eq!(attribute_non_generic.get_data_size(), 0);
        assert_eq!(attribute_non_generic.get_valid_data_size(), 0);
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let data_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        mft_attribute.non_resident_flag = 0;
        mft_attribute.resident_data = vec![1, 2, 3, 4];
        mft_attribute.data_size = 4;
        let attribute: NtfsAttribute = NtfsAttribute::Generic {
            data_stream: &data_stream,
            cluster_block_size: 4096,
            mft_attribute: &mft_attribute,
        };
        let stream: DataStreamReference = attribute.get_data_stream()?;
        let size: u64 = match stream.write() {
            Ok(mut stream_guard) => stream_guard.get_size()?,
            Err(_) => return Err(keramics_core::error_trace_new!("Failed to acquire lock")),
        };
        assert_eq!(size, 4);

        let stream_custom: DataStreamReference =
            attribute.get_data_stream_with_valid_data_size(4)?;
        let size_custom: u64 = match stream_custom.write() {
            Ok(mut stream_guard) => stream_guard.get_size()?,
            Err(_) => return Err(keramics_core::error_trace_new!("Failed to acquire lock")),
        };
        assert_eq!(size_custom, 4);

        let attribute_non_generic: NtfsAttribute = NtfsAttribute::StandardInformation {
            standard_information: NtfsStandardInformation::new(),
        };
        let result: Result<DataStreamReference, ErrorTrace> =
            attribute_non_generic.get_data_stream();
        assert!(result.is_err());

        let result_custom: Result<DataStreamReference, ErrorTrace> =
            attribute_non_generic.get_data_stream_with_valid_data_size(0);
        assert!(result_custom.is_err());

        Ok(())
    }
}
