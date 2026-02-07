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

use keramics_core::ErrorTrace;
use keramics_datetime::DateTime;

use super::catalog_file_record_extended::HfsExtendedCatalogFileRecord;
use super::catalog_file_record_standard::HfsStandardCatalogFileRecord;
use super::enums::HfsFormat;
use super::fork_descriptor::HfsForkDescriptor;

/// Hierarchical File System (HFS) catalog file record.
#[derive(Clone)]
pub struct HfsCatalogFileRecord {
    /// Record type.
    pub record_type: u16,

    /// Flags.
    pub flags: u16,

    /// Identifier.
    pub identifier: u32,

    /// Data fork descriptor.
    pub data_fork_descriptor: HfsForkDescriptor,

    /// Resource fork descriptor.
    pub resource_fork_descriptor: HfsForkDescriptor,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Modification date and time.
    pub modification_time: DateTime,

    /// Backup date and time.
    pub backup_time: DateTime,

    /// Change date and time.
    pub change_time: Option<DateTime>,

    /// Access date and time.
    pub access_time: Option<DateTime>,

    /// Owner identifier.
    pub owner_identifier: Option<u32>,

    /// Group identifier.
    pub group_identifier: Option<u32>,

    /// File mode.
    pub file_mode: Option<u16>,

    /// Special permissions.
    pub special_permissions: Option<u32>,

    /// Link reference.
    pub link_reference: Option<u32>,

    /// Added date and time.
    pub added_time: Option<DateTime>,
}

impl HfsCatalogFileRecord {
    /// Creates a new catalog file record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            flags: 0,
            identifier: 0,
            data_fork_descriptor: HfsForkDescriptor::new(),
            resource_fork_descriptor: HfsForkDescriptor::new(),
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            backup_time: DateTime::NotSet,
            change_time: None,
            access_time: None,
            owner_identifier: None,
            group_identifier: None,
            file_mode: None,
            special_permissions: None,
            link_reference: None,
            added_time: None,
        }
    }

    /// Reads the catalog file record for debugging.
    pub fn debug_read_data(format: &HfsFormat, data: &[u8]) -> String {
        match format {
            HfsFormat::Hfs => HfsStandardCatalogFileRecord::debug_read_data(data),
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogFileRecord::debug_read_data(data)
            }
        }
    }

    /// Reads the catalog file record from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardCatalogFileRecord::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogFileRecord::read_data(self, data)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::{HfsTime, PosixTime32};

    use crate::hfs::extent_descriptor::HfsExtentDescriptor;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![
            0x02, 0x00, 0x82, 0x00, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x09, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xe5, 0x79, 0x60, 0xda, 0xe5, 0x79, 0x60, 0xda, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x7e, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x09, 0x74,
            0x65, 0x73, 0x74, 0x66, 0x69, 0x6c, 0x65, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x09, 0x54, 0x65, 0x73,
            0x74, 0x46, 0x69, 0x6c, 0x65, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xb2, 0x01, 0x7c,
        ];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![
            0x00, 0x02, 0x00, 0xa2, 0x00, 0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x15, 0xe3, 0x5f,
            0xb8, 0xa4, 0xe3, 0x5f, 0xb8, 0xa4, 0xe3, 0x5f, 0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xb5,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x81, 0xa4, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x3a,
            0x08, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0xcf, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogFileRecord::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0200);
        assert_eq!(test_struct.flags, 0x0082);
        assert_eq!(test_struct.identifier, 18);
        assert_eq!(
            test_struct.data_fork_descriptor,
            HfsForkDescriptor {
                size: 9,
                number_of_blocks: 0,
                extents: vec![HfsExtentDescriptor {
                    block_number: 126,
                    number_of_blocks: 1
                }],
            }
        );
        assert_eq!(
            test_struct.resource_fork_descriptor,
            HfsForkDescriptor {
                size: 0,
                number_of_blocks: 0,
                extents: vec![],
            }
        );
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3849937114,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3849937114,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);

        Ok(())
    }

    #[test]
    fn test_read_data_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsCatalogFileRecord::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0002);
        assert_eq!(test_struct.flags, 0x00a2);
        assert_eq!(test_struct.identifier, 21);
        assert_eq!(
            test_struct.data_fork_descriptor,
            HfsForkDescriptor {
                size: 9,
                number_of_blocks: 1,
                extents: vec![HfsExtentDescriptor {
                    block_number: 463,
                    number_of_blocks: 1
                },],
            }
        );
        assert_eq!(
            test_struct.resource_fork_descriptor,
            HfsForkDescriptor {
                size: 0,
                number_of_blocks: 0,
                extents: vec![],
            }
        );
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            }))
        );
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701237,
            }))
        );
        assert_eq!(test_struct.owner_identifier, Some(501));
        assert_eq!(test_struct.group_identifier, Some(20));
        assert_eq!(test_struct.file_mode, Some(0o100644));
        assert_eq!(test_struct.special_permissions, Some(2));
        assert_eq!(test_struct.link_reference, None);

        assert_eq!(
            test_struct.added_time,
            Some(DateTime::PosixTime32(PosixTime32 {
                timestamp: 1731856442,
            }))
        );
        Ok(())
    }
}
