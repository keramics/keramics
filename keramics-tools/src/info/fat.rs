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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::Path;
use keramics_formats::fat::{FatFileEntry, FatFileSystem, FatFormat, FatString};

/// File Allocation Table (FAT) file attribute flags information.
struct FatFileAttributeFlagsInfo {
    /// Flags.
    flags: u8,
}

impl FatFileAttributeFlagsInfo {
    /// Creates new file attribute flags information.
    fn new(flags: u8) -> Self {
        Self { flags }
    }
}

impl fmt::Display for FatFileAttributeFlagsInfo {
    /// Formats partition file attribute flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x01 != 0 {
            writeln!(
                formatter,
                "        0x0001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)"
            )?;
        }
        if self.flags & 0x02 != 0 {
            writeln!(
                formatter,
                "        0x0002: Is hidden (FILE_ATTRIBUTE_HIDDEN)"
            )?;
        }
        if self.flags & 0x04 != 0 {
            writeln!(
                formatter,
                "        0x0004: Is system (FILE_ATTRIBUTE_SYSTEM)"
            )?;
        }

        if self.flags & 0x10 != 0 {
            writeln!(
                formatter,
                "        0x0010: Is directory (FILE_ATTRIBUTE_DIRECTORY)"
            )?;
        }
        if self.flags & 0x20 != 0 {
            writeln!(
                formatter,
                "        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)"
            )?;
        }
        if self.flags & 0x40 != 0 {
            writeln!(
                formatter,
                "        0x0040: Is device (FILE_ATTRIBUTE_DEVICE)"
            )?;
        }
        if self.flags & 0x80 != 0 {
            writeln!(
                formatter,
                "        0x0080: Is normal (FILE_ATTRIBUTE_NORMAL)"
            )?;
        }
        Ok(())
    }
}

/// File Allocation Table (FAT) file entry information.
struct FatFileEntryInfo {
    /// The identifier.
    pub identifier: u32,

    /// The name.
    pub name: Option<FatString>,

    /// The size.
    pub size: u64,

    /// Creation date and time.
    pub creation_time: Option<DateTime>,

    /// Access date and time.
    pub access_time: Option<DateTime>,

    /// Modifiation date and time.
    pub modification_time: Option<DateTime>,

    /// File attribute flags.
    pub file_attribute_flags: u8,
}

impl FatFileEntryInfo {
    /// Creates new file entry information.
    fn new() -> Self {
        Self {
            identifier: 0,
            name: None,
            size: 0,
            modification_time: None,
            access_time: None,
            creation_time: None,
            file_attribute_flags: 0,
        }
    }

    /// Retrieves the string representation of a date and time value.
    fn get_date_time_string(date_time: &DateTime) -> String {
        match date_time {
            DateTime::FatDate(fat_date) => fat_date.to_iso8601_string(),
            DateTime::FatTimeDate(fat_date_time) => fat_date_time.to_iso8601_string(),
            DateTime::FatTimeDate10Ms(fat_date_time_10ms) => fat_date_time_10ms.to_iso8601_string(),
            DateTime::NotSet => String::from("Not set (0)"),
            _ => return String::from("Unsupported date time"),
        }
    }
}

impl fmt::Display for FatFileEntryInfo {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: 0x{:08x}",
            self.identifier
        )?;

        if let Some(name) = &self.name {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        writeln!(formatter, "    Size\t\t\t\t\t: {}", self.size)?;

        if let Some(date_time) = &self.modification_time {
            // TODO: convert to formatter.
            let date_time_string: String = Self::get_date_time_string(date_time);

            writeln!(
                formatter,
                "    Modification time\t\t\t\t: {}",
                date_time_string
            )?;
        }
        if let Some(date_time) = &self.access_time {
            // TODO: convert to formatter.
            let date_time_string: String = Self::get_date_time_string(date_time);

            writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_string)?;
        }
        if let Some(date_time) = &self.creation_time {
            // TODO: convert to formatter.
            let date_time_string: String = Self::get_date_time_string(date_time);

            writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_string)?;
        }
        writeln!(
            formatter,
            "    File attribute flags\t\t\t: 0x{:02x}",
            self.file_attribute_flags
        )?;
        let flags_info: FatFileAttributeFlagsInfo =
            FatFileAttributeFlagsInfo::new(self.file_attribute_flags);

        flags_info.fmt(formatter)?;

        writeln!(formatter)
    }
}

/// Information about a File Allocation Table (FAT).
pub struct FatInfo {}

impl FatInfo {
    /// Retrieves the file entry information.
    fn get_file_entry_information(fat_file_entry: &FatFileEntry) -> FatFileEntryInfo {
        let mut file_entry_information: FatFileEntryInfo = FatFileEntryInfo::new();

        file_entry_information.identifier = fat_file_entry.identifier;
        file_entry_information.name = fat_file_entry.get_name();
        file_entry_information.size = fat_file_entry.get_size();
        file_entry_information.modification_time = fat_file_entry.get_modification_time().cloned();
        file_entry_information.access_time = fat_file_entry.get_access_time().cloned();
        file_entry_information.creation_time = fat_file_entry.get_creation_time().cloned();
        file_entry_information.file_attribute_flags = fat_file_entry.get_file_attribute_flags();

        file_entry_information
    }

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
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
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

        let file_entry_information: FatFileEntryInfo =
            Self::get_file_entry_information(&file_entry);

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
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        let fat_file_entry: FatFileEntry = match fat_file_system.get_file_entry_by_path(path) {
            Ok(Some(fat_file_entry)) => fat_file_entry,
            Ok(None) => return Err(keramics_core::error_trace_new!("Missing file entry")),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        println!("File Allocation Table (FAT) file entry information:");

        println!("    Path\t\t\t\t\t: {}", path);

        let file_entry_information: FatFileEntryInfo =
            Self::get_file_entry_information(&fat_file_entry);

        print!("{}", file_entry_information);

        Ok(())
    }

    /// Prints information about the file system.
    pub fn print_file_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        println!("File Allocation Table (FAT) information:");

        let format_version: u8 = match &fat_file_system.format {
            FatFormat::Fat12 => 12,
            FatFormat::Fat16 => 16,
            FatFormat::Fat32 => 32,
        };
        println!("    Format version\t\t\t\t: FAT-{}", format_version);

        let volume_label: String = match fat_file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        println!("    Volume label\t\t\t\t: {}", volume_label);

        println!("");

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let fat_file_system: FatFileSystem = match Self::open_file_system(data_stream) {
            Ok(fat_file_system) => fat_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        println!("File Allocation Table (FAT) hierarchy:");

        let mut file_entry: FatFileEntry = match fat_file_system.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        let mut path_components: Vec<String> = Vec::new();

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
                Ok(fat_file_entry) => fat_file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve sub file entry: {}",
                            sub_file_entry_index
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
                            "Unable to print hierarchy of sub file entry: {}",
                            sub_file_entry_index
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
    use keramics_types::Ucs2String;

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/fat/fat12.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let fat_file_system: FatFileSystem = FatInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry = fat_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: FatFileEntryInfo = FatInfo::get_file_entry_information(&fat_file_entry);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "    Identifier\t\t\t\t\t: 0x00006260\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9\n",
            "    Modification time\t\t\t\t: 2025-10-19T18:44:30\n",
            "    Access time\t\t\t\t\t: 2025-10-19\n",
            "    Creation time\t\t\t\t: 2025-10-19T18:44:31.25\n",
            "    File attribute flags\t\t\t: 0x20\n",
            "        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/fat/fat12.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let fat_file_system: FatFileSystem = FatInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let fat_file_entry: FatFileEntry = fat_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: FatFileEntryInfo = FatInfo::get_file_entry_information(&fat_file_entry);

        assert_eq!(test_struct.identifier, 0x00006260);
        assert_eq!(
            test_struct.name,
            Some(FatString::Ucs2String(Ucs2String::from("testfile1")))
        );
        assert_eq!(test_struct.size, 9);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::FatTimeDate(FatTimeDate {
                date: 0x5b53,
                time: 0x958f
            }))
        );
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::FatDate(FatDate { date: 0x5b53 }))
        );
        assert_eq!(
            test_struct.creation_time,
            Some(DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5b53,
                time: 0x958f,
                fraction: 0x7d,
            }))
        );
        assert_eq!(test_struct.file_attribute_flags, 0x20);

        Ok(())
    }

    // TODO: add tests for get_date_time_string
    // TODO: add tests for open_file_system
    // TODO: add tests for print_file_entry_by_identifier
    // TODO: add tests for print_file_entry_by_path
    // TODO: add tests for print_file_system
    // TODO: add tests for print_hierarchy
    // TODO: add tests for print_hierarchy_file_entry
}
