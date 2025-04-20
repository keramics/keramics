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

use std::collections::BTreeMap;
use std::io;
use std::rc::Rc;

use crate::datetime::DateTime;
use crate::types::{SharedValue, Ucs2String};
use crate::vfs::{FakeDataStream, VfsDataStreamReference};

use super::attribute::NtfsAttribute;
use super::block_stream::NtfsBlockStream;
use super::constants::*;
use super::data_fork::NtfsDataFork;
use super::directory_entry::NtfsDirectoryEntry;
use super::directory_index::NtfsDirectoryIndex;
use super::master_file_table::NtfsMasterFileTable;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_entry::NtfsMftEntry;

/// New Technologies File System (NTFS) file entry.
pub struct NtfsFileEntry {
    /// The data stream.
    data_stream: VfsDataStreamReference,

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

    /// The directory entry.
    directory_entry: Option<NtfsDirectoryEntry>,

    /// Directory entries.
    directory_entries: BTreeMap<Ucs2String, NtfsDirectoryEntry>,

    /// Value to indicate the directory entries were read.
    read_directory_entries: bool,
}

impl NtfsFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &VfsDataStreamReference,
        mft: &Rc<NtfsMasterFileTable>,
        mft_entry_number: u64,
        mft_entry: NtfsMftEntry,
        name: Option<Ucs2String>,
        directory_entry: Option<NtfsDirectoryEntry>,
    ) -> Self {
        let sequence_number: u16 = mft_entry.sequence_number;

        Self {
            data_stream: data_stream.clone(),
            mft: mft.clone(),
            mft_entry_number: mft_entry_number,
            mft_entry: mft_entry,
            sequence_number: sequence_number,
            name: name,
            directory_entry: directory_entry,
            directory_entries: BTreeMap::new(),
            read_directory_entries: false,
        }
    }

    /// Retrieves the access time from the directory entry $FILE_NAME attribute.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.access_time),
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
            Some(directory_entry) => Some(&directory_entry.entry_modification_time),
            None => None,
        }
    }

    /// Retrieves the creation time from the directory entry $FILE_NAME attribute.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match &self.directory_entry {
            Some(directory_entry) => Some(&directory_entry.creation_time),
            None => None,
        }
    }

    /// Retrieves the file attribute flags.
    pub fn get_file_attribute_flags(&self) -> u32 {
        match &self.directory_entry {
            Some(directory_entry) => directory_entry.file_attribute_flags,
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
            Some(directory_entry) => Some(&directory_entry.modification_time),
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
            Some(directory_entry) => directory_entry.data_size,
            None => 0,
        }
    }
    /// Retrieves the number of attributes.
    pub fn get_number_of_attributes(&self) -> io::Result<usize> {
        Ok(self.mft_entry.attributes.len())
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute_by_index(&self, attribute_index: usize) -> io::Result<NtfsAttribute> {
        let mft_attribute: &NtfsMftAttribute = match self.mft_entry.attributes.get(attribute_index)
        {
            Some(mft_attribute) => mft_attribute,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Missing attribute: {}", attribute_index),
                ));
            }
        };
        let mut attribute: NtfsAttribute = NtfsAttribute::new(mft_attribute.attribute_type);

        match attribute {
            NtfsAttribute::FileName { ref mut file_name } => {
                if !mft_attribute.is_resident() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported non-resident $FILE_NAME attribute.",
                    ));
                }
                file_name.read_data(&mft_attribute.resident_data)?;
            }
            NtfsAttribute::StandardInformation {
                ref mut standard_information,
            } => {
                if !mft_attribute.is_resident() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported non-resident $STANDARD_INFORMATION attribute.",
                    ));
                }
                standard_information.read_data(&mft_attribute.resident_data)?;
            }
            NtfsAttribute::Undefined { .. } => {}
            NtfsAttribute::VolumeInformation {
                ref mut volume_information,
            } => {
                if !mft_attribute.is_resident() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported non-resident $VOLUME_INFORMATION attribute.",
                    ));
                }
                volume_information.read_data(&mft_attribute.resident_data)?;
            }
            NtfsAttribute::VolumeName {
                ref mut volume_name,
            } => {
                if !mft_attribute.is_resident() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported non-resident $VOLUME_NAME attribute.",
                    ));
                }
                Ucs2String::read_elements_le(
                    &mut volume_name.elements,
                    &mft_attribute.resident_data,
                );
            }
        };
        Ok(attribute)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<VfsDataStreamReference>> {
        let data_stream: Option<VfsDataStreamReference> = match self
            .mft_entry
            .get_attribute(&None, NTFS_ATTRIBUTE_TYPE_DATA)
        {
            Some(data_attribute) => {
                if data_attribute.is_resident() {
                    let inline_stream: FakeDataStream = FakeDataStream::new(
                        &data_attribute.resident_data,
                        data_attribute.data_size,
                    );

                    Some(SharedValue::new(Box::new(inline_stream)))
                } else {
                    let mut block_stream: NtfsBlockStream =
                        NtfsBlockStream::new(self.mft.cluster_block_size);
                    block_stream.open(&self.data_stream, data_attribute)?;

                    Some(SharedValue::new(Box::new(block_stream)))
                }
            }
            None => None,
        };
        Ok(data_stream)
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: &Option<Ucs2String>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        let data_stream: Option<VfsDataStreamReference> =
            match self.mft_entry.get_attribute(name, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(data_attribute) => {
                    if data_attribute.is_resident() {
                        let inline_stream: FakeDataStream = FakeDataStream::new(
                            &data_attribute.resident_data,
                            data_attribute.data_size,
                        );

                        Some(SharedValue::new(Box::new(inline_stream)))
                    } else {
                        let mut block_stream: NtfsBlockStream =
                            NtfsBlockStream::new(self.mft.cluster_block_size);
                        block_stream.open(&self.data_stream, data_attribute)?;

                        Some(SharedValue::new(Box::new(block_stream)))
                    }
                }
                None => None,
            };
        Ok(data_stream)
    }

    /// Retrieves the number of data forks.
    pub fn get_number_of_data_forks(&self) -> io::Result<usize> {
        Ok(self.mft_entry.get_number_of_data_attributes())
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(&self, data_fork_index: usize) -> io::Result<NtfsDataFork> {
        match self.mft_entry.get_data_attribute_by_index(data_fork_index) {
            Some(data_attribute) => Ok(NtfsDataFork::new(
                &self.data_stream,
                self.mft.cluster_block_size,
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
        if !self.read_directory_entries {
            self.read_directory_entries()?;
        }
        Ok(self.directory_entries.len())
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<NtfsFileEntry> {
        if !self.read_directory_entries {
            self.read_directory_entries()?;
        }
        let (name, directory_entry): (&Ucs2String, &NtfsDirectoryEntry) =
            match self.directory_entries.iter().nth(sub_file_entry_index) {
                Some(key_and_value) => key_and_value,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Missing directory entry: {}", sub_file_entry_index),
                    ));
                }
            };
        let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;
        let mut mft_entry: NtfsMftEntry =
            self.mft.get_entry(&self.data_stream, mft_entry_number)?;
        mft_entry.read_attributes()?;

        let file_entry: NtfsFileEntry = NtfsFileEntry::new(
            &self.data_stream,
            &self.mft,
            mft_entry_number,
            mft_entry,
            Some(name.clone()),
            Some(directory_entry.clone()),
        );
        Ok(file_entry)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &Ucs2String,
    ) -> io::Result<Option<NtfsFileEntry>> {
        if !self.read_directory_entries {
            self.read_directory_entries()?;
        }
        // TODO: case insensitive search
        let (name, directory_entry): (&Ucs2String, &NtfsDirectoryEntry) =
            match self.directory_entries.get_key_value(sub_file_entry_name) {
                Some(key_and_value) => key_and_value,
                None => return Ok(None),
            };

        let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;
        let mut mft_entry: NtfsMftEntry =
            self.mft.get_entry(&self.data_stream, mft_entry_number)?;
        mft_entry.read_attributes()?;

        let file_entry: NtfsFileEntry = NtfsFileEntry::new(
            &self.data_stream,
            &self.mft,
            mft_entry_number,
            mft_entry,
            Some(name.clone()),
            Some(directory_entry.clone()),
        );
        Ok(Some(file_entry))
    }

    /// Determines if the file entry has directory entries.
    pub fn has_directory_entries(&self) -> bool {
        let i30_index_name: Option<Ucs2String> = Some(Ucs2String::from_string("$I30"));
        self.mft_entry.has_attribute_group(&i30_index_name)
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

    /// Reads the directory entries.
    fn read_directory_entries(&mut self) -> io::Result<()> {
        let mut directory_index: NtfsDirectoryIndex =
            NtfsDirectoryIndex::new(self.mft.cluster_block_size);

        // TODO: fill directory_entries
        directory_index.read_mft_entry(
            &self.mft_entry,
            &self.data_stream,
            &mut self.directory_entries,
        )?;
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
    // TODO: add tests for read_directory_entries
}
