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

use std::fs::{metadata, File, Metadata};
use std::io;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use core::DataStreamReference;
use datetime::{DateTime, PosixTime32, PosixTime64Ns};

use crate::enums::VfsFileType;

/// Determines the POSIX date and time value.
fn get_posix_datetime_value(timestamp: i64, fraction: i64) -> DateTime {
    if fraction != 0 {
        DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction as u32))
    } else if timestamp != 0 {
        DateTime::PosixTime32(PosixTime32::new(timestamp as i32))
    } else {
        DateTime::NotSet
    }
}

/// Operating system file entry.
pub struct OsFileEntry {
    /// Path.
    path: String,

    /// File type.
    file_type: VfsFileType,

    /// Access time.
    access_time: Option<DateTime>,

    /// Change time.
    change_time: Option<DateTime>,

    /// Creation time.
    creation_time: Option<DateTime>,

    /// Modification time.
    modification_time: Option<DateTime>,
}

impl OsFileEntry {
    /// Creates a new file entry.
    pub fn new() -> Self {
        Self {
            path: String::new(),
            file_type: VfsFileType::NotSet,
            access_time: None,
            change_time: None,
            creation_time: None,
            modification_time: None,
        }
    }
    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        self.access_time.as_ref()
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        self.change_time.as_ref()
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        self.creation_time.as_ref()
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        self.file_type.clone()
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        self.modification_time.as_ref()
    }

    /// Initializes the file entry.
    #[cfg(unix)]
    pub(crate) fn initialize(&mut self, path: &str) -> io::Result<()> {
        let os_path: &Path = Path::new(path);

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
        self.modification_time = Some(get_posix_datetime_value(
            file_metadata.mtime(),
            file_metadata.mtime_nsec(),
        ));

        self.access_time = Some(get_posix_datetime_value(
            file_metadata.atime(),
            file_metadata.atime_nsec(),
        ));

        self.change_time = Some(get_posix_datetime_value(
            file_metadata.ctime(),
            file_metadata.ctime_nsec(),
        ));

        // Determine creation time.
        self.creation_time = match file_metadata.created() {
            Ok(system_time) => match system_time.duration_since(UNIX_EPOCH) {
                Ok(duration) => Some(DateTime::PosixTime64Ns(PosixTime64Ns::new(
                    duration.as_secs() as i64,
                    duration.subsec_nanos(),
                ))),
                Err(error) => return Err(core::error_to_io_error!(error)),
            },
            Err(_) => None,
        };
        self.path = path.to_string();

        Ok(())
    }

    /// Initializes the file entry.
    #[cfg(windows)]
    pub(crate) fn initialize(&mut self, path: &str) -> io::Result<()> {
        // TODO: add Windows support.
        todo!();
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<DataStreamReference>> {
        if self.file_type != VfsFileType::File {
            return Ok(None);
        }
        let os_path: &Path = Path::new(self.path.as_str());
        let file: File = File::open(os_path)?;

        Ok(Some(DataStreamReference::new(Box::new(file))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() -> io::Result<()> {
        let mut os_file_entry: OsFileEntry = OsFileEntry::new();

        os_file_entry.initialize("../test_data/file.txt")?;

        assert!(os_file_entry.file_type == VfsFileType::File);

        Ok(())
    }

    // TODO: add tests for OsFileEntry::get_access_time
    // TODO: add tests for OsFileEntry::get_change_time
    // TODO: add tests for OsFileEntry::get_creation_time
    // TODO: add tests for OsFileEntry::get_file_type
    // TODO: add tests for OsFileEntry::get_modification_time

    #[test]
    fn test_get_data_stream() -> io::Result<()> {
        let mut os_file_entry: OsFileEntry = OsFileEntry::new();

        os_file_entry.initialize("../test_data/file.txt")?;

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        let result: Option<DataStreamReference> = os_file_entry.get_data_stream()?;

        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Missing data stream"),
                ))
            }
        };
        let mut test_data: Vec<u8> = vec![];
        let read_count: usize = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_to_end(&mut test_data)?,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        assert_eq!(read_count, 202);
        assert_eq!(test_data, expected_data.as_bytes());

        Ok(())
    }
}
