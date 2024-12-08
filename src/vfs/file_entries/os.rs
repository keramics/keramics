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

use std::fs::{metadata, File, Metadata};
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use crate::types::SharedValue;
use crate::vfs::enums::VfsFileType;
use crate::vfs::path::VfsPath;
use crate::vfs::traits::VfsFileEntry;
use crate::vfs::types::{VfsDataStreamReference, VfsPathReference};

/// Operating system file entry.
pub struct OsVfsFileEntry {
    location: String,

    /// File type.
    file_type: VfsFileType,
}

impl OsVfsFileEntry {
    /// Creates a new file entry.
    pub fn new() -> Self {
        Self {
            location: String::new(),
            file_type: VfsFileType::NotSet,
        }
    }

    /// Initializes the file entry.
    #[cfg(unix)]
    pub(crate) fn initialize(&mut self, path: &VfsPath) -> io::Result<()> {
        let parent_path: Option<VfsPathReference> = path.get_parent();
        if parent_path.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Parent set in path",
            ));
        }
        let os_path: &Path = Path::new(path.location.as_str());

        let file_metadata: Metadata = metadata(os_path)?;

        let mode: u32 = file_metadata.mode();

        self.file_type = match mode & 0xf000 {
            0x1000 => VfsFileType::NamedPipe,
            0x2000 => VfsFileType::CharacterDevice,
            0x4000 => VfsFileType::Directory,
            0x6000 => VfsFileType::BlockDevice,
            0x8000 => VfsFileType::File,
            0xa000 => VfsFileType::SymbolicLink,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported file mode",
                ))
            }
        };
        self.location = path.location.clone();

        Ok(())
    }

    /// Initializes the file entry.
    #[cfg(windows)]
    pub(crate) fn initialize(&mut self, path: &VfsPath) -> io::Result<()> {
        // TODO: add Windows support.
        todo!();
    }
}

impl VfsFileEntry for OsVfsFileEntry {
    /// Retrieves the file type.
    fn get_vfs_file_type(&self) -> VfsFileType {
        self.file_type.clone()
    }

    /// Opens a data stream with the specified name.
    fn open_data_stream(&self, name: Option<&str>) -> io::Result<VfsDataStreamReference> {
        match name {
            None => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported data stream name",
                ))
            }
        };
        if self.file_type != VfsFileType::File {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported file type",
            ));
        }
        let os_path: &Path = Path::new(self.location.as_str());
        let file: File = File::open(os_path)?;

        Ok(SharedValue::new(Box::new(file)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsPathType;

    #[test]
    fn test_initialize() -> io::Result<()> {
        let mut vfs_file_entry: OsVfsFileEntry = OsVfsFileEntry::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        vfs_file_entry.initialize(&vfs_path)?;

        assert!(vfs_file_entry.file_type == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_data_stream() -> io::Result<()> {
        let mut vfs_file_entry: OsVfsFileEntry = OsVfsFileEntry::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        vfs_file_entry.initialize(&vfs_path)?;

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        let vfs_data_stream: VfsDataStreamReference = vfs_file_entry.open_data_stream(None)?;

        let mut test_data: Vec<u8> = vec![];
        let read_count: usize = match vfs_data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_to_end(&mut test_data)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert_eq!(read_count, 202);
        assert_eq!(test_data, expected_data.as_bytes());

        Ok(())
    }
}
