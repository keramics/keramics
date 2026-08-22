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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_types::Ucs2String;

use crate::path_component::PathComponent;
use crate::traits::FileEntryIterator;

use super::block_allocation_table::ExFatBlockAllocationTable;
use super::block_reader::ExFatBlockReader;
use super::block_stream::ExFatBlockStream;
use super::constants::*;
use super::directory_entries::ExFatDirectoryEntries;
use super::directory_entry::ExFatDirectoryEntry;
use super::file_entries::ExFatFileEntriesIterator;

/// Extensible File Allocation Table (exFAT) file entry.
pub struct ExFatFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block allocation table.
    block_allocation_table: Arc<ExFatBlockAllocationTable>,

    /// The identifier.
    pub(super) identifier: u64,

    /// The directory entry.
    directory_entry: Option<ExFatDirectoryEntry>,

    /// The sub directory entries.
    sub_directory_entries: ExFatDirectoryEntries,
}

impl ExFatFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        block_allocation_table: &Arc<ExFatBlockAllocationTable>,
        identifier: u64,
        directory_entry: Option<ExFatDirectoryEntry>,
        sub_directory_entries: ExFatDirectoryEntries,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_allocation_table: block_allocation_table.clone(),
            identifier,
            directory_entry,
            sub_directory_entries,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.file_entry_record.access_time),
            None => None,
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.file_entry_record.creation_time),
            None => None,
        }
    }

    /// Retrieves the file attribute flags.
    pub fn get_file_attribute_flags(&self) -> u16 {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => directory_entry.file_entry_record.file_attribute_flags,
            None => 0,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> u64 {
        self.identifier
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.file_entry_record.modification_time),
            None => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(directory_entry.get_name()),
            None => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => directory_entry.valid_data_size,
            None => 0,
        }
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.is_directory() {
            return Ok(None);
        }
        match self.directory_entry.as_ref() {
            Some(directory_entry) => {
                let mut block_reader: ExFatBlockReader = ExFatBlockReader::new(
                    &self.data_stream,
                    self.block_allocation_table.cluster_block_size,
                    directory_entry.valid_data_size,
                );
                match block_reader.open(
                    &self.block_allocation_table,
                    directory_entry.data_start_cluster,
                    directory_entry.data_stream_no_fat_chain,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open block reader");
                        return Err(error);
                    }
                }
                Ok(Some(Arc::new(RwLock::new(ExFatBlockStream::new(
                    block_reader,
                )))))
            }
            None => Err(keramics_core::error_trace_new!("Missing directory entry")),
        }
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> ExFatFileEntriesIterator<'_> {
        ExFatFileEntriesIterator::new(self)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &PathComponent,
    ) -> Result<Option<Self>, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries.is_read() {
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
        match self
            .sub_directory_entries
            .get_entry_by_name(sub_file_entry_name)
        {
            Ok(Some(directory_entry)) => Ok(Some(Self::new(
                &self.data_stream,
                &self.block_allocation_table,
                directory_entry.identifier,
                Some(directory_entry.clone()),
                ExFatDirectoryEntries::new(
                    &self.sub_directory_entries.case_folding_mappings,
                    false,
                ),
            ))),
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve sub file entry");
                Err(error)
            }
        }
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        match &self.directory_entry {
            Some(directory_entry) => {
                directory_entry.file_entry_record.file_attribute_flags & 0x58
                    == EXFAT_FILE_ATTRIBUTE_FLAG_DIRECTORY
            }
            None => self.directory_entry.is_none(),
        }
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        self.directory_entry.is_none()
    }

    /// Reads the sub directory entries.
    fn read_sub_directory_entries(&mut self) -> Result<(), ErrorTrace> {
        let cluster_block_number: u32 = match &self.directory_entry {
            Some(directory_entry) => directory_entry.data_start_cluster as u32,
            None => {
                return Err(keramics_core::error_trace_new!("Missing directory entry"));
            }
        };
        match self.sub_directory_entries.read_at_cluster_block(
            &self.data_stream,
            &self.block_allocation_table,
            cluster_block_number,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read root directory from cluster block: {}",
                        cluster_block_number
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

impl FileEntryIterator for ExFatFileEntry {
    /// Retrieves the number of sub file entries.
    fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries.is_read() {
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
    fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<Self, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries.is_read() {
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
        match self
            .sub_directory_entries
            .get_entry_by_index(sub_file_entry_index)
        {
            Some(directory_entry) => Ok(Self::new(
                &self.data_stream,
                &self.block_allocation_table,
                directory_entry.identifier,
                Some(directory_entry.clone()),
                ExFatDirectoryEntries::new(
                    &self.sub_directory_entries.case_folding_mappings,
                    false,
                ),
            )),
            None => Err(keramics_core::error_trace_new!(format!(
                "Unable to retrieve sub file entry: {}",
                sub_file_entry_index
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::{FatTimeDate, FatTimeDate10Ms};

    use crate::exfat::file_system::ExFatFileSystem;
    use crate::path::Path;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<ExFatFileSystem, ErrorTrace> {
        let mut file_system: ExFatFileSystem = ExFatFileSystem::new();

        let path_string: String = get_test_data_path("exfat/exfat.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            exfat_file_entry.get_access_time(),
            Some(&DateTime::FatTimeDate(FatTimeDate {
                date: 0x5d15,
                time: 0x62cd,
                utc_offset: 0x80,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            exfat_file_entry.get_creation_time(),
            Some(&DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5d15,
                time: 0x62cd,
                fraction: 0x26,
                utc_offset: 0x80,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_file_attribute_flags() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let file_attribute_flags: u16 = exfat_file_entry.get_file_attribute_flags();
        assert_eq!(file_attribute_flags, 0x0020);

        Ok(())
    }

    #[test]
    fn test_get_identifier() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let identifier: u64 = exfat_file_entry.get_identifier();
        assert_eq!(identifier, 0x00201c00);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(
            exfat_file_entry.get_modification_time(),
            Some(&DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5d15,
                time: 0x62cd,
                fraction: 0x26,
                utc_offset: 0x80,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let name: Option<&Ucs2String> = exfat_file_entry.get_name();
        assert_eq!(name, Some(Ucs2String::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.get_size(), 9);

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = exfat_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let result: Option<DataStreamReference> = exfat_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let mut sub_file_entries_iterator: ExFatFileEntriesIterator =
            exfat_file_entry.sub_file_entries();

        let result: Option<Result<ExFatFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<ExFatFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(8).next();
        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests for get_sub_file_entry_by_name

    #[test]
    fn test_is_directory() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_directory(), true);

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_root_directory() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_root_directory(), true);

        let path: Path = Path::from("/testdir1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_root_directory(), false);

        let path: Path = Path::from("/testdir1/testfile1");
        let exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(exfat_file_entry.is_root_directory(), false);

        Ok(())
    }

    // TODO: add tests for read_sub_directory_entries

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize =
            exfat_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 3);

        let path: Path = Path::from("/testdir1/testfile1");
        let mut exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let number_of_sub_file_entries: usize =
            exfat_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let exfat_file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/testdir1");
        let mut exfat_file_entry: ExFatFileEntry =
            exfat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let sub_file_entry: ExFatFileEntry = exfat_file_entry.get_sub_file_entry_by_index(2)?;

        let name: Option<&Ucs2String> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(Ucs2String::from(
                "My long, very long file name, so very long"
            ))
            .as_ref()
        );
        Ok(())
    }
}
