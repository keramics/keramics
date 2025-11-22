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

use std::ffi::OsStr;
use std::fs::{File, Metadata, read_link, symlink_metadata};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use keramics_core::{DataStreamReference, ErrorTrace};

#[cfg(unix)]
use keramics_datetime::{DateTime, PosixTime32, PosixTime64Ns};

#[cfg(windows)]
use keramics_datetime::{DateTime, Filetime};

use crate::enums::VfsFileType;

use super::directory_entries::OsDirectoryEntries;

/// Operating system file entry.
pub struct OsFileEntry {
    /// Path.
    path: PathBuf,

    /// File type.
    file_type: VfsFileType,

    /// Access time.
    access_time: Option<DateTime>,

    /// Change time.
    change_time: Option<DateTime>,

    /// Creation time.
    creation_time: Option<DateTime>,

    /// Device identifier.
    device_identifier: Option<u64>,

    /// File mode.
    file_mode: Option<u32>,

    /// Group identifier.
    group_identifier: Option<u32>,

    /// Inode number.
    inode_number: Option<u64>,

    /// Modification time.
    modification_time: Option<DateTime>,

    /// Number of links.
    number_of_links: Option<u64>,

    /// Owner identifier.
    owner_identifier: Option<u32>,

    /// Size in bytes.
    size: u64,

    /// Sub Sub directory entries.
    sub_directory_entries: OsDirectoryEntries,
}

impl OsFileEntry {
    /// Creates a new file entry.
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            file_type: VfsFileType::File,
            access_time: None,
            change_time: None,
            creation_time: None,
            device_identifier: None,
            file_mode: None,
            group_identifier: None,
            inode_number: None,
            modification_time: None,
            number_of_links: None,
            owner_identifier: None,
            size: 0,
            sub_directory_entries: OsDirectoryEntries::new(),
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

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        if self.file_type != VfsFileType::File {
            return Ok(None);
        }
        let file: File = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to open file",
                    error
                ));
            }
        };
        Ok(Some(Arc::new(RwLock::new(file))))
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        self.file_type.clone()
    }

    /// Retrieves the device identifier.
    pub fn get_device_identifier(&self) -> Option<u64> {
        self.device_identifier
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> Option<u32> {
        self.file_mode
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> Option<u32> {
        self.group_identifier
    }

    /// Retrieves the inode number.
    pub fn get_inode_number(&self) -> Option<u64> {
        self.inode_number
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        self.modification_time.as_ref()
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<&OsStr> {
        self.path.file_name()
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> Option<u64> {
        self.number_of_links
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> Option<u32> {
        self.owner_identifier
    }

    /// Determines the POSIX date and time value.
    #[cfg(unix)]
    fn get_posix_datetime_value(timestamp: i64, fraction: i64) -> DateTime {
        if fraction != 0 {
            DateTime::PosixTime64Ns(PosixTime64Ns::new(timestamp, fraction as u32))
        } else if timestamp != 0 {
            DateTime::PosixTime32(PosixTime32::new(timestamp as i32))
        } else {
            DateTime::NotSet
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&self) -> Option<PathBuf> {
        match read_link(&self.path) {
            Ok(link_target) => Some(link_target),
            Err(_) => None,
        }
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries.is_read() {
            match self.sub_directory_entries.read(&self.path) {
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
    ) -> Result<OsFileEntry, ErrorTrace> {
        if self.is_directory() && !self.sub_directory_entries.is_read() {
            match self.sub_directory_entries.read(&self.path) {
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
            Some(name) => {
                let mut os_file_entry: OsFileEntry = OsFileEntry::new();

                let mut path_buf: PathBuf = self.path.clone();
                path_buf.push(name);

                match os_file_entry.open(&path_buf) {
                    Ok(true) => {}
                    Ok(false) => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing sub file entry: {}",
                            sub_file_entry_index
                        )));
                    }
                    Err(error) => {
                        return Err(keramics_core::error_trace_new_with_error!(
                            format!("Unable to open sub file entry: {}", sub_file_entry_index),
                            error
                        ));
                    }
                }
                Ok(os_file_entry)
            }
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing directory entry: {}",
                sub_file_entry_index
            ))),
        }
    }

    /// Determines if the file entry is a directory.
    pub fn is_directory(&self) -> bool {
        self.file_type == VfsFileType::Directory
    }

    /// Opens the file entry.
    #[cfg(unix)]
    pub(crate) fn open(&mut self, path: &PathBuf) -> Result<bool, ErrorTrace> {
        // Note that symlink_metadata() is used to prevent traversing symbolic links.
        let file_metadata: Metadata = match symlink_metadata(path) {
            Ok(file_metadata) => file_metadata,
            Err(ref error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to retrieve file metadata",
                    error
                ));
            }
        };
        self.access_time = Some(Self::get_posix_datetime_value(
            file_metadata.atime(),
            file_metadata.atime_nsec(),
        ));
        self.change_time = Some(Self::get_posix_datetime_value(
            file_metadata.ctime(),
            file_metadata.ctime_nsec(),
        ));
        self.creation_time = match file_metadata.created() {
            Ok(system_time) => Some(DateTime::FakeTime(system_time)),
            Err(_) => None,
        };
        let mode: u32 = file_metadata.mode();

        self.file_type = match mode & 0xf000 {
            0x1000 => VfsFileType::NamedPipe,
            0x2000 => VfsFileType::CharacterDevice,
            0x4000 => VfsFileType::Directory,
            0x6000 => VfsFileType::BlockDevice,
            0x8000 => VfsFileType::File,
            0xa000 => VfsFileType::SymbolicLink,
            _ => {
                return Err(keramics_core::error_trace_new!("Unsupported file mode"));
            }
        };
        // Note that rdev() will return 0 if the file entry is not a device.
        self.device_identifier = match mode & 0xf000 {
            0x2000 | 0x6000 => Some(file_metadata.rdev()),
            _ => None,
        };
        self.file_mode = Some(mode);
        self.group_identifier = Some(file_metadata.gid());
        self.inode_number = Some(file_metadata.ino());
        self.modification_time = Some(Self::get_posix_datetime_value(
            file_metadata.mtime(),
            file_metadata.mtime_nsec(),
        ));
        self.number_of_links = Some(file_metadata.nlink());
        self.owner_identifier = Some(file_metadata.uid());
        self.size = file_metadata.len();

        self.path = path.clone();

        Ok(true)
    }

    /// Opens the file entry.
    #[cfg(windows)]
    pub(crate) fn open(&mut self, path: &PathBuf) -> Result<bool, ErrorTrace> {
        // Note that symlink_metadata() is used to prevent traversing symbolic links.
        let file_metadata: Metadata = match symlink_metadata(path) {
            Ok(file_metadata) => file_metadata,
            Err(ref error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to retrieve file metadata",
                    error
                ));
            }
        };
        self.access_time = Some(DateTime::Filetime(Filetime::new(
            file_metadata.last_access_time(),
        )));

        // TODO: add support for change_time

        self.creation_time = Some(DateTime::Filetime(Filetime::new(
            file_metadata.creation_time(),
        )));
        let file_attributes: u32 = file_metadata.file_attributes();

        self.file_type = match file_attributes & 0x000000f0 {
            0x00000010 => VfsFileType::Directory,
            0x00000040 => VfsFileType::Device,
            _ => VfsFileType::File,
        };
        self.modification_time = Some(DateTime::Filetime(Filetime::new(
            file_metadata.last_write_time(),
        )));
        self.size = file_metadata.len();

        self.path = path.clone();

        Ok(true)
    }

    /// Determines if the file entry is the root file entry.
    pub fn is_root_file_entry(&self) -> bool {
        self.path.has_root() && self.path.parent().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::get_test_data_path;

    fn get_os_file_entry(path_string: &str) -> Result<OsFileEntry, ErrorTrace> {
        let mut file_entry: OsFileEntry = OsFileEntry::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        if !file_entry.open(&path_buf)? {
            return Err(keramics_core::error_trace_new!("Missing file entry"));
        }
        Ok(file_entry)
    }

    #[test]
    fn test_get_access_time() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = file_entry.get_access_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = file_entry.get_change_time();
        if cfg!(windows) {
            assert_eq!(result, None);
        } else {
            // Note that the value can vary.
            assert!(result.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_creation_time() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = file_entry.get_creation_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let data_stream: DataStreamReference = match file_entry.get_data_stream()? {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut test_data: Vec<u8> = vec![0; 202];
        let read_count: usize = match data_stream.write() {
            Ok(mut data_stream) => data_stream.read(&mut test_data)?,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on data stream",
                    error
                ));
            }
        };
        assert_eq!(read_count, 202);

        let expected_data: String = [
            "A ceramic is any of the various hard, brittle, heat-resistant, and ",
            "corrosion-resistant materials made by shaping and then firing an inorganic, ",
            "nonmetallic material, such as clay, at a high temperature.\n",
        ]
        .join("");

        assert_eq!(test_data, expected_data.as_bytes());

        Ok(())
    }

    #[test]
    fn test_get_file_type() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let file_type: VfsFileType = file_entry.get_file_type();
        assert_eq!(file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let device_identifier: Option<u64> = file_entry.get_device_identifier();
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let file_mode: Option<u32> = file_entry.get_file_mode();
        if cfg!(windows) {
            assert_eq!(file_mode, None);
        } else {
            // Note that the value can vary.
            assert!(file_mode.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_group_identifier() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let group_identifier: Option<u32> = file_entry.get_group_identifier();
        if cfg!(windows) {
            assert_eq!(group_identifier, None);
        } else {
            // Note that the value can vary.
            assert!(group_identifier.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_inode_number() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let inode_number: Option<u64> = file_entry.get_inode_number();
        if cfg!(windows) {
            assert_eq!(inode_number, None);
        } else {
            // Note that the value can vary.
            assert!(inode_number.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_modification_time() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = file_entry.get_modification_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let name: Option<&OsStr> = file_entry.get_name();
        assert_eq!(name, Some(OsStr::new("file.txt")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_links: Option<u64> = file_entry.get_number_of_links();
        if cfg!(windows) {
            assert_eq!(number_of_links, None);
        } else {
            assert_eq!(number_of_links, Some(1));
        }
        Ok(())
    }

    #[test]
    fn test_get_owner_identifier() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let owner_identifier: Option<u32> = file_entry.get_owner_identifier();
        if cfg!(windows) {
            assert_eq!(owner_identifier, None);
        } else {
            // Note that the value can vary.
            assert!(owner_identifier.is_some());
        }
        Ok(())
    }

    // TODO: add tests for get_posix_datetime_value

    #[test]
    fn test_get_size() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let size: u64 = file_entry.get_size();
        assert_eq!(size, 202);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let link_target: Option<PathBuf> = file_entry.get_symbolic_link_target();
        assert_eq!(link_target, None);

        let file_entry: OsFileEntry = get_os_file_entry("directory/symbolic_link")?;

        let link_target: Option<PathBuf> = file_entry.get_symbolic_link_target();
        assert_eq!(link_target, Some(PathBuf::from("file.txt")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries() -> Result<(), ErrorTrace> {
        let mut file_entry: OsFileEntry = get_os_file_entry("directory")?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_sub_file_entries: usize = file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_get_sub_file_entry_by_index() -> Result<(), ErrorTrace> {
        let mut file_entry: OsFileEntry = get_os_file_entry("directory")?;

        // Note that the order of the directory entries can vary.
        let sub_file_entry: OsFileEntry = file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<&OsStr> = sub_file_entry.get_name();
        assert!(name.is_some());

        Ok(())
    }

    #[test]
    fn test_is_directory() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory")?;

        assert_eq!(file_entry.is_directory(), true);

        let file_entry: OsFileEntry = get_os_file_entry("directory/file.txt")?;

        assert_eq!(file_entry.is_directory(), false);

        Ok(())
    }

    #[test]
    fn test_open() -> Result<(), ErrorTrace> {
        let mut file_entry: OsFileEntry = OsFileEntry::new();

        let path_string: String = get_test_data_path("directory/file.txt");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let result: bool = file_entry.open(&path_buf)?;

        assert_eq!(result, true);
        assert_eq!(file_entry.file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_is_root_file_entry() -> Result<(), ErrorTrace> {
        let file_entry: OsFileEntry = get_os_file_entry("directory")?;

        assert_eq!(file_entry.is_root_file_entry(), false);

        Ok(())
    }
}
