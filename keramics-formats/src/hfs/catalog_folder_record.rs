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

use super::catalog_folder_record_extended::HfsExtendedCatalogFolderRecord;
use super::catalog_folder_record_standard::HfsStandardCatalogFolderRecord;
use super::enums::HfsFormat;

/// Hierarchical File System (HFS) catalog folder record.
#[derive(Clone)]
pub struct HfsCatalogFolderRecord {
    /// Record type.
    pub record_type: u16,

    /// Flags.
    pub flags: u16,

    /// Identifier.
    pub identifier: u32,

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

    /// Added date and time.
    pub added_time: Option<DateTime>,
}

impl HfsCatalogFolderRecord {
    /// Creates a new catalog folder record.
    pub fn new() -> Self {
        Self {
            record_type: 0,
            flags: 0,
            identifier: 0,
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            backup_time: DateTime::NotSet,
            change_time: None,
            access_time: None,
            owner_identifier: None,
            group_identifier: None,
            file_mode: None,
            added_time: None,
        }
    }

    /// Reads the catalog folder record for debugging.
    pub fn debug_read_data(format: &HfsFormat, data: &[u8]) -> String {
        match format {
            HfsFormat::Hfs => HfsStandardCatalogFolderRecord::debug_read_data(data),
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogFolderRecord::debug_read_data(data)
            }
        }
    }

    /// Reads the catalog folder record from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardCatalogFolderRecord::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedCatalogFolderRecord::read_data(self, data)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::HfsTime;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![
            0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0xe5, 0x79, 0x60, 0xda,
            0xe5, 0x79, 0x60, 0xda, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x02, 0xe3, 0x5f,
            0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xba, 0xe3, 0x5f, 0xb8, 0xbb, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00,
            0x41, 0xed, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x7e,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsCatalogFolderRecord::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0100);
        assert_eq!(test_struct.flags, 0x0000);
        assert_eq!(test_struct.identifier, 2);
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

        let mut test_struct = HfsCatalogFolderRecord::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.record_type, 0x0001);
        assert_eq!(test_struct.flags, 0x0000);
        assert_eq!(test_struct.identifier, 2);
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701242,
            })
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701243,
            }))
        );
        assert_eq!(test_struct.access_time, Some(DateTime::NotSet));
        assert_eq!(test_struct.owner_identifier, Some(501));
        assert_eq!(test_struct.group_identifier, Some(20));
        assert_eq!(test_struct.file_mode, Some(0o40755));
        assert_eq!(test_struct.added_time, None);

        Ok(())
    }
}
