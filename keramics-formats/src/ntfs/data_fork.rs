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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::Ucs2String;

/// New Technologies File System (NTFS) data fork.
pub struct NtfsDataFork {
    /// The name.
    name: Option<Ucs2String>,

    /// The data stream.
    data_stream: DataStreamReference,

    /// Base record file reference.
    base_record_file_reference: u64,
}

impl NtfsDataFork {
    /// Creates a new data fork.
    pub fn new(
        name: Option<&Ucs2String>,
        data_stream: DataStreamReference,
        base_record_file_reference: u64,
    ) -> Self {
        Self {
            name: name.cloned(),
            data_stream,
            base_record_file_reference,
        }
    }

    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> Result<&DataStreamReference, ErrorTrace> {
        if self.base_record_file_reference != 0 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT entry with base record file reference: {}-{}",
                self.base_record_file_reference & 0x0000ffffffffffff,
                self.base_record_file_reference >> 48,
            )));
        }
        Ok(&self.data_stream)
    }

    /// Retrieves the name from the directory entry $DATA attribute.
    pub fn get_name(&self) -> Option<&Ucs2String> {
        self.name.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::ntfs::file_entry::NtfsFileEntry;
    use crate::ntfs::file_system::NtfsFileSystem;
    use crate::path::Path;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_file_entry(path_string: &str) -> Result<NtfsFileEntry, ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_file_system()?;

        let path: Path = Path::from(path_string);
        match ntfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_data_stream_with_ntfs() -> Result<(), ErrorTrace> {
        // TODO: change to not retrieve data fork from file entry.
        let ntfs_file_entry: NtfsFileEntry = get_file_entry("/testdir1/testfile1")?;
        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(0)?;

        data_fork.get_data_stream()?;

        // TODO: add test with ADS

        Ok(())
    }

    #[test]
    fn test_get_name_with_ntfs() -> Result<(), ErrorTrace> {
        // TODO: change to not retrieve data fork from file entry.
        let ntfs_file_entry: NtfsFileEntry = get_file_entry("/testdir1/testfile1")?;
        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(0)?;

        let name: Option<&Ucs2String> = data_fork.get_name();
        assert_eq!(name, None);

        // TODO: add test with ADS

        Ok(())
    }
}
