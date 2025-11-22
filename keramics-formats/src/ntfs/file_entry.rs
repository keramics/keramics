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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_types::Ucs2String;

use crate::path_component::PathComponent;

use super::attribute::NtfsAttribute;
use super::attribute_list::NtfsAttributeList;
use super::constants::*;
use super::data_fork::NtfsDataFork;
use super::directory_entries::NtfsDirectoryEntries;
use super::directory_entry::NtfsDirectoryEntry;
use super::directory_index::NtfsDirectoryIndex;
use super::file_entries::NtfsFileEntriesIterator;
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
    mft: Arc<NtfsMasterFileTable>,

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

    /// Sub directory entries.
    sub_directory_entries: NtfsDirectoryEntries,

    /// Value to indicate the file entry has sub directory entries.
    has_sub_directory_entries: bool,

    /// Value to indicate the sub directory entries were read.
    read_sub_directory_entries: bool,
}

impl NtfsFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        mft: &Arc<NtfsMasterFileTable>,
        case_folding_mappings: &Arc<HashMap<u16, u16>>,
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
            mft_entry_number,
            mft_entry,
            sequence_number,
            name,
            mft_attributes: NtfsMftAttributes::new(),
            directory_entry,
            directory_index: NtfsDirectoryIndex::new(cluster_block_size, case_folding_mappings),
            sub_directory_entries: NtfsDirectoryEntries::new(),
            has_sub_directory_entries: false,
            read_sub_directory_entries: false,
        }
    }

    /// Retrieves the access time from the $STANDARD_INFORMATION attribute.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match &self.mft_attributes.standard_information {
            Some(standard_information) => Some(&standard_information.access_time),
            _ => None,
        }
    }

    /// Retrieves the base record file reference.
    pub fn get_base_record_file_reference(&self) -> u64 {
        self.mft_entry.base_record_file_reference
    }

    /// Retrieves the change time from the $STANDARD_INFORMATION attribute.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match &self.mft_attributes.standard_information {
            Some(standard_information) => Some(&standard_information.entry_modification_time),
            _ => None,
        }
    }

    /// Retrieves the creation time from the $STANDARD_INFORMATION attribute.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match &self.mft_attributes.standard_information {
            Some(standard_information) => Some(&standard_information.creation_time),
            _ => None,
        }
    }

    /// Retrieves the file attribute flags from the $STANDARD_INFORMATION attribute.
    pub fn get_file_attribute_flags(&self) -> u32 {
        match &self.mft_attributes.standard_information {
            Some(standard_information) => standard_information.file_attribute_flags,
            _ => 0,
        }
    }

    /// Retrieves the file reference.
    pub fn get_file_reference(&self) -> u64 {
        match &self.directory_entry {
            Some(directory_entry) => directory_entry.file_reference,
            None => self.mft_entry_number | ((self.sequence_number as u64) << 48),
        }
    }

    /// Retrieves the journal sequence number.
    pub fn get_journal_sequence_number(&self) -> u64 {
        self.mft_entry.journal_sequence_number
    }

    /// Retrieves the modification time from the $STANDARD_INFORMATION attribute.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match &self.mft_attributes.standard_information {
            Some(standard_information) => Some(&standard_information.modification_time),
            _ => None,
        }
    }

    /// Retrieves the name from the directory entry $FILE_NAME.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.name.as_ref()
    }

    /// Retrieves the parent file reference.
    pub fn get_parent_file_reference(&self) -> Option<u64> {
        match &self.directory_entry {
            Some(directory_entry) => Some(directory_entry.file_name.parent_file_reference),
            None => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self
            .mft_attributes
            .get_attribute_by_name_and_type(&None, NTFS_ATTRIBUTE_TYPE_DATA)
        {
            Some(data_attribute) => data_attribute.data_size,
            None => 0,
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&self) -> Result<Option<&Ucs2String>, ErrorTrace> {
        let result: Option<&Ucs2String> = match &self.mft_attributes.reparse_point {
            Some(NtfsReparsePoint::SymbolicLink { reparse_data }) => {
                Some(&reparse_data.substitute_name)
            }
            _ => None,
        };
        Ok(result)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.mft_entry.base_record_file_reference != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT entry with base record file reference: {}-{}",
                self.mft_entry.base_record_file_reference & 0x0000ffffffffffff,
                self.mft_entry.base_record_file_reference >> 48,
            )));
        }
        match self.mft_attributes.get_data_stream_by_name(
            &None,
            &self.data_stream,
            self.mft.cluster_block_size,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve nameless data stream"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: Option<&PathComponent>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.mft_entry.base_record_file_reference != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT entry with base record file reference: {}-{}",
                self.mft_entry.base_record_file_reference & 0x0000ffffffffffff,
                self.mft_entry.base_record_file_reference >> 48,
            )));
        }
        let attribute_name: Option<Ucs2String> = match name {
            Some(path_component) => match path_component.to_ucs2_string() {
                Ok(ucs2_string) => Some(ucs2_string),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UCS-2 string"
                    );
                    return Err(error);
                }
            },
            None => None,
        };
        match self.mft_attributes.get_data_stream_by_name(
            &attribute_name,
            &self.data_stream,
            self.mft.cluster_block_size,
        ) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                Err(error)
            }
        }
    }

    /// Retrieves the number of data forks.
    pub fn get_number_of_data_forks(&self) -> Result<usize, ErrorTrace> {
        Ok(self.mft_attributes.get_number_of_data_attributes())
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(
        &self,
        data_fork_index: usize,
    ) -> Result<NtfsDataFork, ErrorTrace> {
        let data_attribute: &NtfsMftAttribute = match self
            .mft_attributes
            .get_data_attribute_by_index(data_fork_index)
        {
            Some(data_attribute) => data_attribute,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data attribute: {}",
                    data_fork_index
                )));
            }
        };
        let data_stream: DataStreamReference = match self.mft_attributes.get_data_stream_by_name(
            &data_attribute.name,
            &self.data_stream,
            self.mft.cluster_block_size,
        ) {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream for data attribute: {}",
                    data_fork_index
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                return Err(error);
            }
        };
        Ok(NtfsDataFork::new(
            data_attribute.name.as_ref(),
            data_stream,
            self.mft_entry.base_record_file_reference,
        ))
    }

    // TODO: add get data fork by name.

    /// Retrieves the number of attributes.
    pub fn get_number_of_attributes(&self) -> usize {
        self.mft_attributes.get_number_of_attributes()
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute_by_index(
        &self,
        attribute_index: usize,
    ) -> Result<NtfsAttribute<'_>, ErrorTrace> {
        let mft_attribute: &NtfsMftAttribute =
            match self.mft_attributes.get_attribute_by_index(attribute_index) {
                Ok(attribute) => attribute,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve attribute: {}", attribute_index)
                    );
                    return Err(error);
                }
            };
        let attribute: NtfsAttribute = match mft_attribute.attribute_type {
            NTFS_ATTRIBUTE_TYPE_STANDARD_INFORMATION => {
                let standard_information: NtfsStandardInformation =
                    match NtfsStandardInformation::from_attribute(mft_attribute) {
                        Ok(standard_information) => standard_information,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to create standard information from attribute"
                            );
                            return Err(error);
                        }
                    };
                NtfsAttribute::StandardInformation {
                    standard_information,
                }
            }
            NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST => {
                let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();

                match attribute_list.read_attribute(
                    mft_attribute,
                    &self.data_stream,
                    self.mft.cluster_block_size,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute list"
                        );
                        return Err(error);
                    }
                }
                NtfsAttribute::AttributeList { attribute_list }
            }
            NTFS_ATTRIBUTE_TYPE_FILE_NAME => {
                let file_name: NtfsFileName = match NtfsFileName::from_attribute(mft_attribute) {
                    Ok(file_name) => file_name,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to create file name from attribute"
                        );
                        return Err(error);
                    }
                };
                NtfsAttribute::FileName { file_name }
            }
            NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION => {
                let volume_information: NtfsVolumeInformation =
                    match NtfsVolumeInformation::from_attribute(mft_attribute) {
                        Ok(volume_information) => volume_information,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to create volume information from attribute"
                            );
                            return Err(error);
                        }
                    };
                NtfsAttribute::VolumeInformation { volume_information }
            }
            NTFS_ATTRIBUTE_TYPE_VOLUME_NAME => {
                if !mft_attribute.is_resident() {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported non-resident $VOLUME_NAME attribute"
                    ));
                }
                let volume_name: Ucs2String =
                    Ucs2String::from_le_bytes(&mft_attribute.resident_data);
                NtfsAttribute::VolumeName { volume_name }
            }
            NTFS_ATTRIBUTE_TYPE_REPARSE_POINT => {
                let reparse_point: NtfsReparsePoint =
                    match NtfsReparsePoint::from_attribute(mft_attribute) {
                        Ok(reparse_point) => reparse_point,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to create reparse point from attribute"
                            );
                            return Err(error);
                        }
                    };
                NtfsAttribute::ReparsePoint { reparse_point }
            }
            _ => NtfsAttribute::Generic { mft_attribute },
        };
        Ok(attribute)
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        if !self.has_sub_directory_entries {
            return Ok(0);
        }
        if !self.read_sub_directory_entries {
            match self.read_sub_directory_entries() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read sub directory entries"
                    );
                    return Err(error);
                }
            }
        }
        Ok(self.sub_directory_entries.get_number_of_entries())
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<NtfsFileEntry, ErrorTrace> {
        if !self.read_sub_directory_entries {
            match self.read_sub_directory_entries() {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read sub directory entries"
                    );
                    return Err(error);
                }
            }
        }
        let directory_entry: &NtfsDirectoryEntry = match self
            .sub_directory_entries
            .get_entry_by_index(sub_file_entry_index)
        {
            Ok(mft_entry) => mft_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve directory entry");
                return Err(error);
            }
        };
        let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;

        let mft_entry: NtfsMftEntry = match self.mft.get_entry(&self.data_stream, mft_entry_number)
        {
            Ok(mft_entry) => mft_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve MFT entry");
                return Err(error);
            }
        };
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
        match file_entry.read_attributes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        Ok(file_entry)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &PathComponent,
    ) -> Result<Option<NtfsFileEntry>, ErrorTrace> {
        if !self.has_sub_directory_entries {
            return Ok(None);
        }
        if !self.directory_index.is_initialized {
            match self.directory_index.initialize(&self.mft_attributes) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to initialize directory index"
                    );
                    return Err(error);
                }
            }
        }
        match self
            .directory_index
            .get_directory_entry_by_name(&self.data_stream, sub_file_entry_name)
        {
            Ok(Some(directory_entry)) => {
                let mft_entry_number: u64 = directory_entry.file_reference & 0x0000ffffffffffff;
                let mft_entry: NtfsMftEntry =
                    match self.mft.get_entry(&self.data_stream, mft_entry_number) {
                        Ok(mft_entry) => mft_entry,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve MFT entry"
                            );
                            return Err(error);
                        }
                    };
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
                match file_entry.read_attributes() {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                        return Err(error);
                    }
                }
                Ok(Some(file_entry))
            }
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve directory entry");
                return Err(error);
            }
        }
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> NtfsFileEntriesIterator<'_> {
        NtfsFileEntriesIterator::new(self)
    }

    /// Reads the attributes.
    pub(super) fn read_attributes(&mut self) -> Result<(), ErrorTrace> {
        match self.mft_entry.read_attributes(&mut self.mft_attributes) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        match self.mft_attributes.attribute_list {
            Some(attribute_index) => {
                let mft_attribute: &NtfsMftAttribute =
                    match self.mft_attributes.get_attribute_by_index(attribute_index) {
                        Ok(attribute) => attribute,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to retrieve attribute: {}", attribute_index)
                            );
                            return Err(error);
                        }
                    };
                let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();

                match attribute_list.read_attribute(
                    &mft_attribute,
                    &self.data_stream,
                    self.mft.cluster_block_size,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute list"
                        );
                        return Err(error);
                    }
                }
                let mut mft_entries_set: HashSet<u64> = HashSet::new();

                for entry in attribute_list.entries.iter() {
                    let mft_entry_number: u64 = entry.file_reference & 0x0000ffffffffffff;
                    if mft_entry_number != self.mft_entry_number {
                        mft_entries_set.insert(mft_entry_number);
                    }
                }
                let mut mft_entries: Vec<u64> = mft_entries_set.drain().collect::<Vec<u64>>();

                mft_entries.sort();

                for mft_entry_number in mft_entries.iter() {
                    let mft_entry: NtfsMftEntry =
                        match self.mft.get_entry(&self.data_stream, *mft_entry_number) {
                            Ok(mft_entry) => mft_entry,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve MFT entry"
                                );
                                return Err(error);
                            }
                        };
                    match mft_entry.read_attributes(&mut self.mft_attributes) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read attributes"
                            );
                            return Err(error);
                        }
                    }
                }
            }
            None => {}
        };
        let i30_index_name: Option<Ucs2String> = Some(Ucs2String::from("$I30"));

        self.has_sub_directory_entries = self.mft_attributes.has_attribute_group(&i30_index_name);

        Ok(())
    }

    /// Determines if the file entry has sub directory entries.
    #[deprecated(since = "0.0.1", note = "please use `is_directory` instead")]
    pub fn has_directory_entries(&self) -> bool {
        self.is_directory()
    }

    /// Determines if the file entry is allocated (used).
    pub fn is_allocated(&self) -> bool {
        self.mft_entry.is_allocated
    }

    /// Determines if the file entry is marked as bad.
    pub fn is_bad(&self) -> bool {
        self.mft_entry.is_bad
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        self.has_sub_directory_entries
    }

    /// Determines if the file entry is empty.
    pub fn is_empty(&self) -> bool {
        self.mft_entry.is_empty
    }

    /// Determines if the file entry is a junction.
    pub fn is_junction(&self) -> bool {
        match &self.mft_attributes.reparse_point {
            Some(NtfsReparsePoint::Junction { .. }) => true,
            _ => false,
        }
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        self.mft_entry_number == NTFS_ROOT_DIRECTORY_IDENTIFIER
    }

    /// Determines if the file entry is a symbolic link.
    pub fn is_symbolic_link(&self) -> bool {
        match &self.mft_attributes.reparse_point {
            Some(NtfsReparsePoint::SymbolicLink { .. }) => true,
            _ => false,
        }
    }

    /// Reads the sub directory entries.
    fn read_sub_directory_entries(&mut self) -> Result<(), ErrorTrace> {
        if !self.has_sub_directory_entries {
            return Err(keramics_core::error_trace_new!(
                "Missing sub directory entries"
            ));
        }
        if !self.directory_index.is_initialized {
            match self.directory_index.initialize(&self.mft_attributes) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to initialize directory index"
                    );
                    return Err(error);
                }
            }
        }
        match self
            .directory_index
            .get_directory_entries(&self.data_stream, &mut self.sub_directory_entries)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve sub directory entries"
                );
                return Err(error);
            }
        }
        self.read_sub_directory_entries = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::Filetime;

    use crate::ntfs::file_system::NtfsFileSystem;
    use crate::path::Path;

    use crate::tests::get_test_data_path;

    fn get_file_system(path_string: &str) -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            ntfs_file_entry.get_access_time(),
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba6892474
            }))
        );
        Ok(())
    }

    // TODO: add tests for get_base_record_file_reference

    #[test]
    fn test_get_change_time() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            ntfs_file_entry.get_change_time(),
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba689275d
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            ntfs_file_entry.get_creation_time(),
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba6892474
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_file_attribute_flags() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let file_attribute_flags: u32 = ntfs_file_entry.get_file_attribute_flags();
        assert_eq!(file_attribute_flags, 0x00000020);

        Ok(())
    }

    #[test]
    fn test_get_journal_sequence_number() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let journal_sequence_number: u64 = ntfs_file_entry.get_journal_sequence_number();
        assert_eq!(journal_sequence_number, 0);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            ntfs_file_entry.get_modification_time(),
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba689275d
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<&Ucs2String> = ntfs_file_entry.get_name();
        assert_eq!(name, Some(Ucs2String::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.get_size(), 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let symbolic_link_target: Option<&Ucs2String> =
            ntfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(symbolic_link_target, None);

        // TODO: test with symbolic link file entry

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = ntfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = ntfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_test_get_data_stream_by_name() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            ntfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        let path: Path = Path::from("/$UpCase");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            ntfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from(Ucs2String::from("$Info")));
        let result: Option<DataStreamReference> =
            ntfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from(Ucs2String::from("bogus")));
        let result: Option<DataStreamReference> =
            ntfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_data_forks: usize = ntfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let path: Path = Path::from("/$UpCase");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_data_forks: usize = ntfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 2);

        Ok(())
    }

    #[test]
    fn test_get_data_fork_by_index() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/$UpCase");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(0)?;

        let name: Option<&Ucs2String> = data_fork.get_name();
        assert_eq!(name, None);

        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(1)?;

        let name: Option<&Ucs2String> = data_fork.get_name();
        assert_eq!(name, Some(Ucs2String::from("$Info")).as_ref());

        let result: Result<NtfsDataFork, ErrorTrace> = ntfs_file_entry.get_data_fork_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_number_of_attributes() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_attributes: usize = ntfs_file_entry.get_number_of_attributes();
        assert_eq!(number_of_attributes, 4);

        Ok(())
    }

    // TODO: add test_get_attribute_by_index

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = ntfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 3);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize = ntfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: NtfsFileEntry = ntfs_file_entry.get_sub_file_entry_by_index(2)?;
        assert_eq!(
            sub_file_entry.get_name(),
            Some(Ucs2String::from("TestFile2")).as_ref()
        );
        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_name() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: PathComponent = PathComponent::Ucs2String(Ucs2String::from("TestFile2"));
        let result: Option<NtfsFileEntry> = ntfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_some());

        let name: PathComponent = PathComponent::Ucs2String(Ucs2String::from("bogus"));
        let result: Option<NtfsFileEntry> = ntfs_file_entry.get_sub_file_entry_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1");
        let mut ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: NtfsFileEntriesIterator =
            ntfs_file_entry.sub_file_entries();

        let result: Option<Result<NtfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<NtfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(2).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_is_allocated() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_allocated(), true);

        Ok(())
    }

    #[test]
    fn test_is_bad() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_bad(), false);

        // TODO: test with bad file entry

        Ok(())
    }

    #[test]
    fn test_is_directory() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_empty() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_empty(), false);

        // TODO: test with empty file entry

        Ok(())
    }

    #[test]
    fn test_is_junction() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_junction(), false);

        // TODO: test with junction file entry

        Ok(())
    }

    #[test]
    fn test_is_root_directory() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_root_directory(), true);

        let path: Path = Path::from("/testdir1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_root_directory(), false);

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_root_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_symbolic_link() -> Result<(), ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system("ntfs/ntfs.raw")?;

        let path: Path = Path::from("/testdir1/testfile1");
        let ntfs_file_entry: NtfsFileEntry =
            ntfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(ntfs_file_entry.is_symbolic_link(), false);

        // TODO: test with symbolic link file entry

        Ok(())
    }

    // TODO: add tests for read_sub_directory_entries
}
