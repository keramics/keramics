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
    Ext(DataStreamReference),
    Ntfs(NtfsDataFork<'a>),
}

impl<'a> VfsDataFork<'a> {
    /// Retrieves the data stream.
    pub fn get_data_stream(&self) -> Result<DataStreamReference, ErrorTrace> {
        match self {
            VfsDataFork::Ext(data_stream) => Ok(data_stream.clone()),
            VfsDataFork::Ntfs(data_fork) => data_fork.get_data_stream(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<VfsString> {
        match self {
            VfsDataFork::Ext(_) => None,
            VfsDataFork::Ntfs(data_fork) => match data_fork.get_name() {
                Some(name) => Some(VfsString::Ucs2(name.clone())),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsType;
    use crate::file_entry::VfsFileEntry;
    use crate::file_system::VfsFileSystem;
    use crate::location::{VfsLocation, new_os_vfs_location};
    use crate::path::VfsPath;
    use crate::types::VfsFileSystemReference;

    use crate::tests::get_test_data_path;

    fn get_parent_file_system() -> VfsFileSystemReference {
        VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os))
    }

    fn get_ext_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ext);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("ext/ext2.raw").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

    // TODO: add tests
}
