/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::io::{Read, Seek, SeekFrom};

use super::enums::VfsFileType;
use super::path::VfsPath;
use super::types::{VfsDataStreamReference, VfsFileEntryReference, VfsFileSystemReference};

/// Virtual File System (VFS) data stream trait.
pub trait VfsDataStream: Read + Seek {
    // TODO: add get_extents()

    /// Retrieves the size of the data stream.
    fn get_size(&mut self) -> io::Result<u64>;

    #[inline(always)]
    fn read_at_position(&mut self, data: &mut [u8], position: SeekFrom) -> io::Result<usize> {
        self.seek(position)?;
        self.read(data)
    }

    #[inline(always)]
    fn read_exact_at_position(&mut self, data: &mut [u8], position: SeekFrom) -> io::Result<u64> {
        let offset: u64 = self.seek(position)?;
        self.read_exact(data)?;
        Ok(offset)
    }
}

/// Virtual File System (VFS) file entry trait.
pub trait VfsFileEntry {
    // TODO: add get_attributes()

    /// Retrieves the file type.
    fn get_vfs_file_type(&self) -> VfsFileType;

    /// Opens a data stream with the specified name.
    fn open_data_stream(&self, name: Option<&str>) -> io::Result<VfsDataStreamReference>;
}

/// Virtual File System (VFS) file system trait.
pub trait VfsFileSystem {
    /// Determines if the file entry with the specified path exists.
    fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool>;

    /// Retrieves the directory name of the specified location.
    fn get_directory_name<'a>(&self, location: &'a str) -> &'a str {
        let directory_name: &str = match location.rsplit_once("/") {
            Some(path_components) => path_components.0,
            None => "",
        };
        if directory_name == "" {
            "/"
        } else {
            directory_name
        }
    }

    /// Opens a file system.
    fn open(
        &mut self,
        parent_file_system: &VfsFileSystemReference,
        path: &VfsPath,
    ) -> io::Result<()>;

    /// Opens a data stream with the specified path and name.
    #[inline(always)]
    fn open_data_stream(
        &self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<VfsDataStreamReference> {
        let file_entry: VfsFileEntryReference = match self.open_file_entry(path)? {
            Some(file_entry) => file_entry,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing file entry",
                ));
            }
        };
        file_entry.open_data_stream(name)
    }

    /// Opens a file entry with the specified path.
    fn open_file_entry(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntryReference>>;
}
