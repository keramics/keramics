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
use keramics_formats::ntfs::NtfsDataFork;

use super::string::VfsString;

/// Virtual File System (VFS) data fork.
pub enum VfsDataFork<'a> {
    DataStream(DataStreamReference),
    Ntfs(NtfsDataFork<'a>),
}

impl<'a> VfsDataFork<'a> {
    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> Result<DataStreamReference, ErrorTrace> {
        match self {
            VfsDataFork::DataStream(data_stream) => Ok(data_stream.clone()),
            VfsDataFork::Ntfs(data_fork) => data_fork.get_data_stream(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<VfsString> {
        match self {
            VfsDataFork::DataStream(_) => None,
            VfsDataFork::Ntfs(data_fork) => match data_fork.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_formats::Path;
    use keramics_formats::ext::{ExtFileEntry, ExtFileSystem};
    use keramics_formats::ntfs::{NtfsFileEntry, NtfsFileSystem};

    use crate::tests::get_test_data_path;

    // Tests with ext.

    fn get_ext_file_system() -> Result<ExtFileSystem, ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ext/ext2.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_ext_file_entry(path_string: &str) -> Result<ExtFileEntry, ErrorTrace> {
        let ext_file_system: ExtFileSystem = get_ext_file_system()?;

        let path: Path = Path::from(path_string);
        match ext_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_data_stream_with_ext() -> Result<(), ErrorTrace> {
        let ext_file_entry: ExtFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let data_stream: DataStreamReference = ext_file_entry.get_data_stream()?.unwrap();
        let vfs_data_fork: VfsDataFork = VfsDataFork::DataStream(data_stream);

        vfs_data_fork.get_data_stream()?;

        Ok(())
    }

    #[test]
    fn test_get_name_with_ext() -> Result<(), ErrorTrace> {
        let ext_file_entry: ExtFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let data_stream: DataStreamReference = ext_file_entry.get_data_stream()?.unwrap();
        let vfs_data_fork: VfsDataFork = VfsDataFork::DataStream(data_stream);

        let name: Option<VfsString> = vfs_data_fork.get_name();
        assert_eq!(name, None);

        Ok(())
    }

    // Tests with NTFS.

    fn get_ntfs_file_system() -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_ntfs_file_entry(path_string: &str) -> Result<NtfsFileEntry, ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_ntfs_file_system()?;

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
        let ntfs_file_entry: NtfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(0)?;
        let vfs_data_fork: VfsDataFork = VfsDataFork::Ntfs(data_fork);

        vfs_data_fork.get_data_stream()?;

        // TODO: add test with ADS

        Ok(())
    }

    #[test]
    fn test_get_name_with_ntfs() -> Result<(), ErrorTrace> {
        let ntfs_file_entry: NtfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let data_fork: NtfsDataFork = ntfs_file_entry.get_data_fork_by_index(0)?;
        let vfs_data_fork: VfsDataFork = VfsDataFork::Ntfs(data_fork);

        let name: Option<VfsString> = vfs_data_fork.get_name();
        assert_eq!(name, None);

        // TODO: add test with ADS

        Ok(())
    }
}
