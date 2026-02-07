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

use keramics_datetime::DateTime;

use super::catalog_file_entry_record::HfsCatalogFileEntryRecord;
use super::fork_descriptor::HfsForkDescriptor;
use super::string::HfsString;

/// Hierarchical File System (HFS) directory entry.
#[derive(Clone)]
pub struct HfsDirectoryEntry {
    /// Name.
    pub name: Option<HfsString>,

    /// File entry record.
    record: HfsCatalogFileEntryRecord,
}

impl HfsDirectoryEntry {
    /// Creates a new directory entry.
    pub fn new(record: HfsCatalogFileEntryRecord) -> Self {
        Self { name: None, record }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.access_time.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.access_time.as_ref()
            }
        }
    }

    /// Retrieves the backup time.
    pub fn get_backup_time(&self) -> &DateTime {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                &catalog_file_record.backup_time
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                &catalog_folder_record.backup_time
            }
        }
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.change_time.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.change_time.as_ref()
            }
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> &DateTime {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                &catalog_file_record.creation_time
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                &catalog_folder_record.creation_time
            }
        }
    }

    /// Retrieves the data fork descriptor.
    pub fn get_data_fork_descriptor(&self) -> Option<&HfsForkDescriptor> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                Some(&catalog_file_record.data_fork_descriptor)
            }
            HfsCatalogFileEntryRecord::Folder(_) => None,
        }
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> Option<&u16> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.file_mode.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.file_mode.as_ref()
            }
        }
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> Option<&u32> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.group_identifier.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.group_identifier.as_ref()
            }
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> u32 {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => catalog_file_record.identifier,
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.identifier
            }
        }
    }

    /// Retrieves the link reference.
    pub fn get_link_reference(&self) -> Option<&u32> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.link_reference.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(_) => None,
        }
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> &DateTime {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                &catalog_file_record.modification_time
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                &catalog_folder_record.modification_time
            }
        }
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> u32 {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                match &catalog_file_record.special_permissions {
                    Some(number_of_links) => *number_of_links,
                    None => 1,
                }
            }
            HfsCatalogFileEntryRecord::Folder(_) => 1,
        }
    }

    /// Retrieves the resource fork descriptor.
    pub fn get_resource_fork_descriptor(&self) -> Option<&HfsForkDescriptor> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                Some(&catalog_file_record.resource_fork_descriptor)
            }
            HfsCatalogFileEntryRecord::Folder(_) => None,
        }
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> Option<&u32> {
        match &self.record {
            HfsCatalogFileEntryRecord::File(catalog_file_record) => {
                catalog_file_record.owner_identifier.as_ref()
            }
            HfsCatalogFileEntryRecord::Folder(catalog_folder_record) => {
                catalog_folder_record.owner_identifier.as_ref()
            }
        }
    }
}
