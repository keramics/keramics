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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::Path;
use keramics_formats::fat::{FatFileEntry, FatFileSystem, FatFormat};

use crate::formatters::ByteSize;

use super::constants::*;
use super::windows::WindowsFileAttributeFlagsInfo;

/// File Allocation Table (FAT) date and time information.
struct FatDateTimeInfo<'a> {
    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> FatDateTimeInfo<'a> {
    /// Creates new date and time information.
    fn new(date_time: &'a DateTime) -> Self {
        Self { date_time }
    }
}

impl<'a> fmt::Display for FatDateTimeInfo<'a> {
    /// Formats date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::FatDate(fat_date) => write!(formatter, "{}", fat_date.to_iso8601_string()),
            DateTime::FatTimeDate(fat_date_time) => {
                write!(formatter, "{}", fat_date_time.to_iso8601_string())
            }
            DateTime::FatTimeDate10Ms(fat_date_time_10ms) => {
                write!(formatter, "{}", fat_date_time_10ms.to_iso8601_string())
            }
            DateTime::NotSet => write!(formatter, "{}", NOT_SET_VALUE),
            _ => write!(formatter, "Unsupported date time"),
        }
    }
}

/// File Allocation Table (FAT) file entry information.
struct FatFileEntryInfo<'a> {
    /// File entry.
    file_entry: &'a FatFileEntry,
}

impl<'a> FatFileEntryInfo<'a> {
    /// Creates new file entry information.
    fn new(file_entry: &'a FatFileEntry) -> Self {
        Self { file_entry }
    }
}

impl<'a> fmt::Display for FatFileEntryInfo<'a> {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: 0x{:08x}",
            self.file_entry.get_identifier()
        )?;

        if let Some(name) = self.file_entry.get_name() {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        let byte_size: ByteSize = ByteSize::new(self.file_entry.get_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        if let Some(date_time) = self.file_entry.get_creation_time() {
            let date_time_info: FatDateTimeInfo = FatDateTimeInfo::new(date_time);
            writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_info)?;
        }
        if let Some(date_time) = self.file_entry.get_modification_time() {
            let date_time_info: FatDateTimeInfo = FatDateTimeInfo::new(date_time);
            writeln!(
                formatter,
                "    Modification time\t\t\t\t: {}",
                date_time_info
            )?;
        }
        if let Some(date_time) = self.file_entry.get_access_time() {
            let date_time_info: FatDateTimeInfo = FatDateTimeInfo::new(date_time);
            writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_info)?;
        }
        let flags: u8 = self.file_entry.get_file_attribute_flags();

        writeln!(formatter, "    File attribute flags\t\t\t: 0x{:02x}", flags)?;
        let flags_info: WindowsFileAttributeFlagsInfo =
            WindowsFileAttributeFlagsInfo::new(flags as u16);

        flags_info.fmt(formatter)?;

        writeln!(formatter)
    }
}

/// File Allocation Table (FAT) file system information.
struct FatFileSystemInfo<'a> {
    /// File system.
    file_system: &'a FatFileSystem,
}

impl<'a> FatFileSystemInfo<'a> {
    /// Creates new file system information.
    fn new(file_system: &'a FatFileSystem) -> Self {
        Self { file_system }
    }
}

impl<'a> fmt::Display for FatFileSystemInfo<'a> {
    /// Formats file system information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "File Allocation Table (FAT) information:")?;

        let format_version: u8 = match self.file_system.get_format() {
            FatFormat::Fat12 => 12,
            FatFormat::Fat16 => 16,
            FatFormat::Fat32 => 32,
        };
        writeln!(
            formatter,
            "    Format version\t\t\t\t: FAT-{}",
            format_version
        )?;
        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.file_system.get_bytes_per_sector()
        )?;
        let volume_label: String = match self.file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        writeln!(formatter, "    Volume label\t\t\t\t: {}", volume_label)?;

        writeln!(formatter)
    }
}

/// Information about a File Allocation Table (FAT).
pub struct FatInfo {}

impl FatInfo {
    /// Opens a file system.
    pub fn open_file_system(
        data_stream: &DataStreamReference,
    ) -> Result<FatFileSystem, ErrorTrace> {
        let mut fat_file_system: FatFileSystem = FatFileSystem::new();

        match fat_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open FAT file system");
                return Err(error);
            }
        }
        Ok(fat_file_system)
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        fat_entry_identifier: u64,
    ) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        if fat_entry_identifier > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported identifier: {} value out of bounds",
                fat_entry_identifier
            )));
        }
        let file_entry: FatFileEntry =
            match fat_file_system.get_file_entry_by_identifier(fat_entry_identifier as u32) {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve file entry: 0x{:08x}",
                            fat_entry_identifier
                        )
                    );
                    return Err(error);
                }
            };
        println!("File Allocation Table (FAT) file entry information:");

        let file_entry_information: FatFileEntryInfo = FatFileEntryInfo::new(&file_entry);

        print!("{}", file_entry_information);

        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_path(
        data_stream: &DataStreamReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        let file_entry: FatFileEntry = match fat_file_system.get_file_entry_by_path(path) {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => return Err(keramics_core::error_trace_new!("Missing file entry")),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        println!("File Allocation Table (FAT) file entry information:");

        println!("    Path\t\t\t\t\t: {}", path);

        let file_entry_information: FatFileEntryInfo = FatFileEntryInfo::new(&file_entry);

        print!("{}", file_entry_information);

        Ok(())
    }

    /// Prints information about the file system.
    pub fn print_file_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        let file_system_information: FatFileSystemInfo = FatFileSystemInfo::new(&fat_file_system);

        print!("{}", file_system_information);

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(
        data_stream: &DataStreamReference,
        path: Option<&String>,
    ) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        println!("File Allocation Table (FAT) hierarchy:");

        let mut file_entry: FatFileEntry = match path {
            Some(path) => match fat_file_system.get_file_entry_by_path(&Path::from(path)) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing file entry for path: {}",
                        path
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve file entry for path: {}", path)
                    );
                    return Err(error);
                }
            },
            None => match fat_file_system.get_root_directory() {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve root directory"
                    );
                    return Err(error);
                }
            },
        };
        let mut path_components: Vec<String> = match path {
            Some(path) => path
                .split('/')
                .skip(2)
                .map(|component| component.to_string())
                .collect::<Vec<String>>(),
            None => Vec::new(),
        };
        match Self::print_hierarchy_file_entry(&mut file_entry, &mut path_components) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to print file entry hierarchy"
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Prints the file entry hierarchy.
    fn print_hierarchy_file_entry(
        file_entry: &mut FatFileEntry,
        path_components: &mut Vec<String>,
    ) -> Result<(), ErrorTrace> {
        let path: String = if file_entry.is_root_directory() {
            String::from("/")
        } else {
            let name_string: String = match file_entry.get_name() {
                Some(name) => name.to_string(),
                None => String::new(),
            };
            path_components.push(name_string);
            format!("/{}", path_components.join("/"))
        };
        println!("{}", path);

        for (sub_file_entry_index, result) in file_entry.sub_file_entries().enumerate() {
            let mut sub_file_entry: FatFileEntry = match result {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve sub file entry: {} of path: {}",
                            sub_file_entry_index, path
                        )
                    );
                    return Err(error);
                }
            };
            match Self::print_hierarchy_file_entry(&mut sub_file_entry, path_components) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to print hierarchy of sub file entry: {} of path: {}",
                            sub_file_entry_index, path
                        )
                    );
                    return Err(error);
                }
            }
        }
        if !file_entry.is_root_directory() {
            path_components.pop();
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

    use crate::assert_lines_eq;

    #[test]
    fn test_date_time_information_fmt() {
        let date_time: DateTime = DateTime::FatDate(FatDate::new(0x3d0c));
        let test_struct: FatDateTimeInfo = FatDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12");

        let date_time: DateTime = DateTime::FatTimeDate(FatTimeDate::new(0x3d0c, 0xa8d0));
        let test_struct: FatDateTimeInfo = FatDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T21:06:32");

        let date_time: DateTime =
            DateTime::FatTimeDate10Ms(FatTimeDate10Ms::new(0x3d0c, 0xa8d0, 0x7d));
        let test_struct: FatDateTimeInfo = FatDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T21:06:33.25");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: FatDateTimeInfo = FatDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, NOT_SET_VALUE);
    }

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/fat/fat12.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let fat_file_system: FatFileSystem = FatInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry = fat_file_system.get_file_entry_by_path(&path)?.unwrap();

        let test_struct: FatFileEntryInfo = FatFileEntryInfo::new(&fat_file_entry);

        let expected_string: &str = concat!(
            "    Identifier\t\t\t\t\t: 0x00006260\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9 bytes\n",
            "    Creation time\t\t\t\t: 2025-10-19T18:44:31.25\n",
            "    Modification time\t\t\t\t: 2025-10-19T18:44:30\n",
            "    Access time\t\t\t\t\t: 2025-10-19\n",
            "    File attribute flags\t\t\t: 0x20\n",
            "        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_file_system_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/fat/fat12.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let fat_file_system: FatFileSystem = FatInfo::open_file_system(&data_stream)?;

        let test_struct: FatFileSystemInfo = FatFileSystemInfo::new(&fat_file_system);

        let expected_string: &str = concat!(
            "File Allocation Table (FAT) information:\n",
            "    Format version\t\t\t\t: FAT-12\n",
            "    Bytes per sector\t\t\t\t: 512\n",
            "    Volume label\t\t\t\t: FAT12_TEST\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_file_system
    // TODO: add tests for print_file_entry_by_identifier
    // TODO: add tests for print_file_entry_by_path
    // TODO: add tests for print_file_system
    // TODO: add tests for print_hierarchy
    // TODO: add tests for print_hierarchy_file_entry
}
