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

use keramics_types::Ucs2String;

use super::attribute_list::NtfsAttributeList;
use super::file_name::NtfsFileName;
use super::mft_attribute::NtfsMftAttribute;
use super::reparse_point::NtfsReparsePoint;
use super::standard_information::NtfsStandardInformation;
use super::volume_information::NtfsVolumeInformation;

/// New Technologies File System (NTFS) attribute value.
pub enum NtfsAttributeValue {
    AttributeList(NtfsAttributeList),
    FileName(NtfsFileName),
    ReparsePoint(NtfsReparsePoint),
    StandardInformation(NtfsStandardInformation),
    VolumeInformation(NtfsVolumeInformation),
    VolumeName(Ucs2String),
}

/// New Technologies File System (NTFS) attribute.
pub struct NtfsAttribute<'a> {
    /// MFT attribute.
    mft_attribute: &'a NtfsMftAttribute,

    /// Value.
    pub(super) value: Option<NtfsAttributeValue>,
}

impl<'a> NtfsAttribute<'a> {
    /// Creates a new attribute.
    pub fn new(mft_attribute: &'a NtfsMftAttribute) -> Self {
        Self {
            mft_attribute,
            value: None,
        }
    }

    /// Retrieves the allocated data size.
    pub fn get_allocated_data_size(&self) -> Option<u64> {
        self.mft_attribute.get_allocated_data_size()
    }

    /// Retrieves the attribute type.
    pub fn get_attribute_type(&self) -> u32 {
        self.mft_attribute.attribute_type
    }

    /// Retrieves the data flags.
    pub fn get_data_flags(&self) -> u16 {
        self.mft_attribute.get_data_flags()
    }

    /// Retrieves the data size.
    pub fn get_data_size(&self) -> u64 {
        self.mft_attribute.get_data_size()
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.mft_attribute.get_name()
    }

    /// Retrieves the valid data size.
    pub fn get_valid_data_size(&self) -> Option<u64> {
        self.mft_attribute.get_valid_data_size()
    }

    /// Retrieves the value.
    pub fn get_value(&self) -> Option<&NtfsAttributeValue> {
        self.value.as_ref()
    }

    // TODO: add methods to retrieve extents
    // TODO: add methods to read and seek data
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::ErrorTrace;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x90, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x04, 0x18, 0x00, 0x00, 0x00,
            0x11, 0x00, 0x38, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x24, 0x00, 0x53, 0x00,
            0x44, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00,
            0x28, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_get_allocated_data_size() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let allocated_data_size: Option<u64> = test_struct.get_allocated_data_size();
        assert_eq!(allocated_data_size, None);

        Ok(())
    }

    #[test]
    fn test_get_attribute_type() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let attribute_type: u32 = test_struct.get_attribute_type();
        assert_eq!(attribute_type, 0x00000090);

        Ok(())
    }

    #[test]
    fn test_get_data_flags() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let data_flags: u16 = test_struct.get_data_flags();
        assert_eq!(data_flags, 0x0000);

        Ok(())
    }

    #[test]
    fn test_get_data_size() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let data_size: u64 = test_struct.get_data_size();
        assert_eq!(data_size, 56);

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let name: Option<&Ucs2String> = test_struct.get_name();
        assert_eq!(name, Some(Ucs2String::from("$SDH")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_valid_data_size() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut mft_attribute = NtfsMftAttribute::new();
        mft_attribute.read_data(&test_data)?;

        let test_struct = NtfsAttribute::new(&mft_attribute);

        let valid_data_size: Option<u64> = test_struct.get_valid_data_size();
        assert_eq!(valid_data_size, None);

        Ok(())
    }

    // TODO: add tests for get_value
}
