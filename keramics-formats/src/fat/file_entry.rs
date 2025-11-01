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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;

use super::block_allocation_table::FatBlockAllocationTable;
use super::block_stream::FatBlockStream;
use super::constants::*;
use super::directory_entries::FatDirectoryEntries;
use super::directory_entry::FatDirectoryEntry;
use super::string::FatString;

/// File Allocation Table (FAT) file entry.
pub struct FatFileEntry {
    /// The data stream.
    data_stream: DataStreamReference,

    /// Block allocation table.
    block_allocation_table: Arc<FatBlockAllocationTable>,

    /// The identifier.
    pub identifier: u32,

    /// The directory entry.
    directory_entry: Option<FatDirectoryEntry>,

    /// The sub directory entries.
    sub_directory_entries: FatDirectoryEntries,
}

impl FatFileEntry {
    /// Creates a new file entry.
    pub(super) fn new(
        data_stream: &DataStreamReference,
        block_allocation_table: &Arc<FatBlockAllocationTable>,
        identifier: u32,
        directory_entry: Option<FatDirectoryEntry>,
        sub_directory_entries: FatDirectoryEntries,
    ) -> Self {
        Self {
            data_stream: data_stream.clone(),
            block_allocation_table: block_allocation_table.clone(),
            identifier: identifier,
            directory_entry: directory_entry,
            sub_directory_entries: sub_directory_entries,
        }
    }

    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.short_name.access_time),
            None => None,
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.short_name.creation_time),
            None => None,
        }
    }

    /// Retrieves the file attribute flags.
    pub fn get_file_attribute_flags(&self) -> u8 {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => directory_entry.short_name.file_attribute_flags,
            None => 0,
        }
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => Some(&directory_entry.short_name.modification_time),
            None => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<FatString> {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => match directory_entry.long_name.as_ref() {
                Some(long_name) => Some(FatString::Ucs2String(long_name.clone())),
                None => Some(FatString::ByteString(
                    directory_entry.short_name.name.clone(),
                )),
            },
            None => None,
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self.directory_entry.as_ref() {
            Some(directory_entry) => directory_entry.short_name.data_size as u64,
            None => 0,
        }
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.is_directory() {
            return Ok(None);
        }
        let (data_start_cluster, data_size): (u16, u32) = match self.directory_entry.as_ref() {
            Some(directory_entry) => (
                directory_entry.short_name.data_start_cluster,
                directory_entry.short_name.data_size,
            ),
            None => (0, 0),
        };
        let mut block_stream: FatBlockStream =
            FatBlockStream::new(self.block_allocation_table.cluster_block_size, data_size);

        match block_stream.open(
            &self.data_stream,
            &self.block_allocation_table,
            data_start_cluster as u32,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open block stream");
                return Err(error);
            }
        }
        Ok(Some(Arc::new(RwLock::new(block_stream))))
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
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
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<FatFileEntry, ErrorTrace> {
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
            Some(directory_entry) => Ok(FatFileEntry::new(
                &self.data_stream,
                &self.block_allocation_table,
                directory_entry.identifier,
                Some(directory_entry.clone()),
                FatDirectoryEntries::new(&self.sub_directory_entries.case_folding_mappings),
            )),
            None => Err(keramics_core::error_trace_new!(format!(
                "Unable to retrieve sub file entry: {}",
                sub_file_entry_index
            ))),
        }
    }

    // TODO: add get_sub_file_entries iterator

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_name(
        &mut self,
        sub_file_entry_name: &FatString,
    ) -> Result<Option<FatFileEntry>, ErrorTrace> {
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
        let result = match self
            .sub_directory_entries
            .get_entry_by_name(sub_file_entry_name)
        {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve sub file entry");
                return Err(error);
            }
        };
        match result {
            Some(directory_entry) => Ok(Some(FatFileEntry::new(
                &self.data_stream,
                &self.block_allocation_table,
                directory_entry.identifier,
                Some(directory_entry.clone()),
                FatDirectoryEntries::new(&self.sub_directory_entries.case_folding_mappings),
            ))),
            None => Ok(None),
        }
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        match &self.directory_entry {
            Some(directory_entry) => {
                directory_entry.short_name.file_attribute_flags & 0x58
                    == FAT_FILE_ATTRIBUTE_FLAG_DIRECTORY
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
            Some(directory_entry) => directory_entry.short_name.data_start_cluster as u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::{FatDate, FatTimeDate, FatTimeDate10Ms};

    use crate::fat::file_system::FatFileSystem;
    use crate::fat::path::FatPath;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<FatFileSystem, ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("fat/fat12.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(
            fat_file_entry.get_access_time(),
            Some(&DateTime::FatDate(FatDate { date: 0x5b53 }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(
            fat_file_entry.get_creation_time(),
            Some(&DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5b53,
                time: 0x958f,
                fraction: 0x7d,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_file_attribute_flags() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let file_attribute_flags: u8 = fat_file_entry.get_file_attribute_flags();
        assert_eq!(file_attribute_flags, 0x20);

        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(
            fat_file_entry.get_modification_time(),
            Some(&DateTime::FatTimeDate(FatTimeDate {
                date: 0x5b53,
                time: 0x958f
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let name: Option<FatString> = fat_file_entry.get_name();
        assert_eq!(name, Some(FatString::from("testfile1")));

        Ok(())
    }

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.get_size(), 9);

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let result: Option<DataStreamReference> = fat_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let result: Option<DataStreamReference> = fat_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1");
        let mut fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let number_of_sub_file_entries: usize = fat_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 3);

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let mut fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let number_of_sub_file_entries: usize = fat_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/testdir1");
        let mut fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        let sub_file_entry: FatFileEntry = fat_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<FatString> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(FatString::from(
                "My long, very long file name, so very long"
            ))
        );

        Ok(())
    }

    // TODO: add tests for get_sub_file_entry_by_name

    #[test]
    fn test_is_directory() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_directory(), true);

        let fat_path: FatPath = FatPath::from("/testdir1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_directory(), true);

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_is_root_directory() -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_root_directory(), true);

        let fat_path: FatPath = FatPath::from("/testdir1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_root_directory(), false);

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry =
            fat_file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(fat_file_entry.is_root_directory(), false);

        Ok(())
    }

    // TODO: add tests for read_sub_directory_entries
}
