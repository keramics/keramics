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

use std::collections::HashSet;
use std::io;
use std::rc::Rc;

use core::DataStreamReference;
use datetime::DateTime;
use types::Ucs2String;

use super::attribute::NtfsAttribute;
use super::attribute_list::NtfsAttributeList;
use super::constants::*;
use super::data_fork::NtfsDataFork;
use super::directory_entries::NtfsDirectoryEntries;
use super::directory_entry::NtfsDirectoryEntry;
use super::directory_index::NtfsDirectoryIndex;
use super::file_name::NtfsFileName;
use super::master_file_table::NtfsMasterFileTable;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_attributes::NtfsMftAttributes;
use super::mft_entry::NtfsMftEntry;
use super::reparse_point::NtfsReparsePoint;
use super::standard_information::NtfsStandardInformation;
use super::volume_information::NtfsVolumeInformation;

/// New Technologies File System (NTFS) file entry.
pub struct NtfsFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Master File Table (MFT).
    mft: Rc<NtfsMasterFileTable>,

    /// The MFT entry number.
    pub mft_entry_number: u64,

    /// The MFT entry.
    mft_entry: NtfsMftEntry,

    /// The sequence number.
    pub sequence_number: u16,

    /// The name.
    name: Option<Ucs2String>,

    /// MFT attributes.
    mft_attributes: NtfsMftAttributes,

    /// The directory entry.
    directory_entry: Option<NtfsDirectoryEntry>,

    /// The directory index.
    directory_index: NtfsDirectoryIndex,

    /// Directory entries.
    directory_entries: NtfsDirectoryEntries,

    /// Value to indicate the file entry has directory entries.
    has_directory_entries: bool,

    /// Value to indicate the directory entries were read.
    read_directory_entries: bool,
}

impl NtfsFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        mft: &Rc<NtfsMasterFileTable>,
        case_folding_mappings: &Rc<Vec<u16>>,
        mft_entry_number: u64,
        mft_entry: NtfsMftEntry,
        name: Option<Ucs2String>,
        directory_entry: Option<NtfsDirectoryEntry>,
    ) -> Self {
        let sequence_number: u16 = mft_entry.sequence_number;
        let cluster_block_size: u32 = mft.cluster_block_size;

        Self {
            data_stream: data_stream.clone(),
            mft: mft.clone(),
            mft_entry_number: mft_entry_number,
            mft_entry: mft_entry,
            sequence_number: sequence_number,
            name: name,
            mft_attributes: NtfsMftAttributes::new(),
            directory_entry: directory_entry,
            directory_index: NtfsDirectoryIndex::new(cluster_block_size, case_folding_mappings),
            directory_entries: NtfsDirectoryEntries::new(),
            has_directory_entries: false,
            read_directory_entries: false,
        }
    }

    /// Retrieves the access time from the directory entry $FILE_NAME attribute.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.file_name.access_time),
            None => None,
        }
    }

    /// Retrieves the base record file reference.
    pub fn get_base_record_file_reference(&self) -> (u64, u16) {
        (
            self.mft_entry.base_record_file_reference & 0x0000ffffffffffff,
            (self.mft_entry.base_record_file_reference >> 48) as u16,
        )
    }

    /// Retrieves the change time from the directory entry $FILE_NAME attribute.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.file_name.entry_modification_time),
            None => None,
        }
    }

    /// Retrieves the creation time from the directory entry $FILE_NAME attribute.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.file_name.creation_time),
            None => None,
        }
    }

    /// Retrieves the file attribute flags.
    pub fn get_file_attribute_flags(&self) -> u32 {
        match &self.directory_entry {
            Some(directory_entry) => directory_entry.file_name.file_attribute_flags,
            None => 0,
        }
    }

    /// Retrieves the journal sequence number.
    pub fn get_journal_sequence_number(&self) -> u64 {
        self.mft_entry.journal_sequence_number
    }

    /// Retrieves the modification time from the directory entry $FILE_NAME attribute.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.file_name.modification_time),
            None => None,
        }
    }

    /// Retrieves the name from the directory entry $FILE_NAME attribute.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.name.as_ref()
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match &self.directory_entry {
            Some(directory_entry) => directory_entry.file_name.data_size,
            None => 0,
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> io::Result<Option<&Ucs2String>> {
        match &self.mft_attributes.reparse_point {
            Some(NtfsReparsePoint::SymbolicLink { .. }) => {
                // TODO: implement
                todo!();
            }
            _ => Ok(None),
        }
    }

    /// Retrieves the number of attributes.
    pub fn get_number_of_attributes(&self) -> io::Result<usize> {
        self.mft_attributes.get_number_of_attributes()
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute_by_index(&self, attribute_index: usize) -> io::Result<NtfsAttribute> {
        let mft_attribute: &NtfsMftAttribute = self
            .mft_attributes
            .get_attribute_by_index(attribute_index)?;

        let attribute: NtfsAttribute = match mft_attribute.attribute_type {
            NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION => {
                let standard_information: NtfsStandardInformation =
                    NtfsStandardInformation::from_attribute(mft_attribute)?;
                NtfsAttribute::StandardInformation {
                    standard_information: standard_information,
                }
            }
            NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST => {
                let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();
                attribute_list.read_attribute(
                    mft_attribute,
                    &self.data_stream,
                    self.mft.cluster_block_size,
                )?;

                NtfsAttribute::AttributeList {
                    attribute_list: attribute_list,
                }
            }
            NTFS_ATTRIBUTE_TYPE_FILE_NAME => {
                let file_name: NtfsFileName = NtfsFileName::from_attribute(mft_attribute)?;
                NtfsAttribute::FileName {
                    file_name: file_name,
                }
            }
            NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION => {
                let volume_information: NtfsVolumeInformation =
                    NtfsVolumeInformation::from_attribute(mft_attribute)?;
                NtfsAttribute::VolumeInformation {
                    volume_information: volume_information,
                }
            }
            NTFS_ATTRIBUTE_TYPE_VOLUME_NAME => {
                if !mft_attribute.is_resident() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported non-resident $VOLUME_NAME attribute.",
                    ));
                }
                let volume_name: Ucs2String =
                    Ucs2String::from_le_bytes(&mft_attribute.resident_data);
                NtfsAttribute::VolumeName {
                    volume_name: volume_name,
                }
            }
            NTFS_ATTRIBUTE_TYPE_REPARSE_POINT => {
                let reparse_point: NtfsReparsePoint =
                    NtfsReparsePoint::from_attribute(mft_attribute)?;
                NtfsAttribute::ReparsePoint {
                    reparse_point: reparse_point,
                }
            }
            _ => NtfsAttribute::Undefined {
                attribute_type: mft_attribute.attribute_type,
            },
        };
        Ok(attribute)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
        self.mft_attributes.get_data_stream_by_name(
            &None,
            &self.data_stream,
            self.mft.cluster_block_size,
        )
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: &Option<Ucs2String>,
    ) -> io::Result<Option<DataStreamReference>> {
        self.mft_attributes.get_data_stream_by_name(
            name,
            &self.data_stream,
            self.mft.cluster_block_size,
        )
    }

    /// Retrieves the number of data forks.
    pub fn get_number_of_data_forks(&self) -> io::Result<usize> {
        Ok(self.mft_attributes.get_number_of_data_attributes())
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(&self, data_fork_index: usize) -> io::Result<NtfsDataFork> {
        match self
            .mft_attributes
            .get_data_attribute_by_index(data_fork_index)
        {
            Some(data_attribute) => Ok(NtfsDataFork::new(
                &self.data_stream,
                self.mft.cluster_block_size,
                &self.mft_attributes,
                data_attribute,
            )),
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Missing data attribute: {}", data_fork_index),
            )),
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        if !self.has_directory_entries {
            return Ok(0);
        }
        if !self.read_directory_entries {
            self.read_directory_entries()?;
        }
        Ok(self.directory_entries.get_number_of_entries())
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<NtfsFileEntry> {
        if !self.read_directory_entries {
            self.read_directory_entries()?;
        }
        let directory_entry: &NtfsDirectoryEntry = self
            .directory_entries
            .get_entry_by_index(sub_file_entry_index)?;

        let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;
        let mft_entry: NtfsMftEntry = self.mft.get_entry(&self.data_stream, mft_entry_number)?;

        let name: &Ucs2String = directory_entry.get_name();

        let mut file_entry: NtfsFileEntry = NtfsFileEntry::new(
            &self.data_stream,
            &self.mft,
            &self.directory_index.case_folding_mappings,
            mft_entry_number,
            mft_entry,
            Some(name.clone()),
            Some(directory_entry.clone()),
        );
        file_entry.read_attributes()?;

        Ok(file_entry)
    }

    /// Reads the attributes.
    pub(super) fn read_attributes(&mut self) -> io::Result<()> {
        self.mft_entry.read_attributes(&mut self.mft_attributes)?;

        match self.mft_attributes.attribute_list {
            Some(attribute_index) => {
                let mft_attribute: &NtfsMftAttribute = self
                    .mft_attributes
                    .get_attribute_by_index(attribute_index)?;

                let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();
                attribute_list.read_attribute(
                    &mft_attribute,
                    &self.data_stream,
                    self.mft.cluster_block_size,
                )?;
                let mut attribute_list_mft_entries: HashSet<u64> = HashSet::new();
                for entry in attribute_list.entries.iter() {
                    let mft_entry_number: u64 = entry.file_reference & 0x0000ffffffffffff;
                    if mft_entry_number != self.mft_entry_number {
                        attribute_list_mft_entries.insert(mft_entry_number);
                    }
                }
                for mft_entry_number in attribute_list_mft_entries.iter() {
                    let mft_entry: NtfsMftEntry =
                        self.mft.get_entry(&self.data_stream, *mft_entry_number)?;
                    mft_entry.read_attributes(&mut self.mft_attributes)?;
                }
            }
            None => {}
        };
        let i30_index_name: Option<Ucs2String> = Some(Ucs2String::from_string("$I30"));
        self.has_directory_entries = self.mft_attributes.has_attribute_group(&i30_index_name);

        Ok(())
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &Ucs2String,
    ) -> io::Result<Option<NtfsFileEntry>> {
        if !self.has_directory_entries {
            return Ok(None);
        }
        if !self.directory_index.is_initialized {
            self.directory_index.initialize(&self.mft_attributes)?;
        }
        match self
            .directory_index
            .get_directory_entry_by_name(&self.data_stream, sub_file_entry_name)?
        {
            Some(directory_entry) => {
                let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;
                let mft_entry: NtfsMftEntry =
                    self.mft.get_entry(&self.data_stream, mft_entry_number)?;

                let name: &Ucs2String = directory_entry.get_name();

                let mut file_entry: NtfsFileEntry = NtfsFileEntry::new(
                    &self.data_stream,
                    &self.mft,
                    &self.directory_index.case_folding_mappings,
                    mft_entry_number,
                    mft_entry,
                    Some(name.clone()),
                    Some(directory_entry),
                );
                file_entry.read_attributes()?;

                Ok(Some(file_entry))
            }
            None => Ok(None),
        }
    }

    /// Determines if the file entry has directory entries.
    pub fn has_directory_entries(&self) -> bool {
        self.has_directory_entries
    }

    /// Determines if the file entry is allocated (used).
    pub fn is_allocated(&self) -> bool {
        self.mft_entry.is_allocated
    }

    /// Determines if the file entry is marked as bad.
    pub fn is_bad(&self) -> bool {
        self.mft_entry.is_bad
    }

    /// Determines if the file entry is empty.
    pub fn is_empty(&self) -> bool {
        self.mft_entry.is_empty
    }

    /// Determines if the file entry is a symbolic link.
    pub fn is_symbolic_link(&self) -> bool {
        match &self.mft_attributes.reparse_point {
            Some(NtfsReparsePoint::SymbolicLink { .. }) => true,
            _ => false,
        }
    }

    /// Reads the directory entries.
    fn read_directory_entries(&mut self) -> io::Result<()> {
        if !self.has_directory_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Missing directory entries"),
            ));
        }
        if !self.directory_index.is_initialized {
            self.directory_index.initialize(&self.mft_attributes)?;
        }
        self.directory_index
            .get_directory_entries(&self.data_stream, &mut self.directory_entries)?;
        self.read_directory_entries = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for get_access_time
    // TODO: add tests for get_base_record_file_reference
    // TODO: add tests for get_change_time
    // TODO: add tests for get_creation_time
    // TODO: add tests for get_file_attribute_flags
    // TODO: add tests for get_journal_sequence_number
    // TODO: add tests for get_modification_time
    // TODO: add tests for get_name
    // TODO: add tests for get_size
    // TODO: add tests for get_number_of_attributes
    // TODO: add tests for get_attribute_by_index
    // TODO: add tests for get_data_stream
    // TODO: add tests for get_data_stream_by_name
    // TODO: add tests for get_number_of_data_forks
    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
    // TODO: add tests for get_sub_file_entry_by_name
    // TODO: add tests for has_directory_entries
    // TODO: add tests for is_allocated
    // TODO: add tests for is_bad
    // TODO: add tests for is_empty
    // TODO: add tests for is_symbolic_link
    // TODO: add tests for read_directory_entries
}
