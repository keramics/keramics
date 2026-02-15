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
use keramics_formats::hfs::{
    HfsExtendedAttribute, HfsFileEntry, HfsFileSystem, HfsFormat, HfsString,
};

use super::posix::PosixFileModeInfo;

/// Hierarchical File System (HFS) file entry information.
struct HfsFileEntryInfo {
    /// The identifier.
    pub identifier: u32,

    /// The name.
    pub name: Option<HfsString>,

    /// The size.
    pub size: u64,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Modifiation date and time.
    pub modification_time: DateTime,

    /// Access date and time.
    pub access_time: Option<DateTime>,

    /// Change date and time.
    pub change_time: Option<DateTime>,

    /// Backup date and time.
    pub backup_time: DateTime,

    /// Number of links.
    pub number_of_links: u32,

    /// Owner identifier.
    pub owner_identifier: Option<u32>,

    /// Group identifier.
    pub group_identifier: Option<u32>,

    /// File mode.
    pub file_mode: Option<u16>,
}

impl HfsFileEntryInfo {
    /// Creates new file entry information.
    fn new() -> Self {
        Self {
            identifier: 0,
            name: None,
            size: 0,
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            access_time: None,
            change_time: None,
            backup_time: DateTime::NotSet,
            number_of_links: 0,
            owner_identifier: None,
            group_identifier: None,
            file_mode: None,
        }
    }

    /// Retrieves the string representation of a date and time value.
    fn get_date_time_string(date_time: &DateTime) -> String {
        match date_time {
            DateTime::HfsTime(hfs_time) => hfs_time.to_iso8601_string(),
            DateTime::NotSet => String::from("Not set (0)"),
            _ => return String::from("Unsupported date time"),
        }
    }
}

impl fmt::Display for HfsFileEntryInfo {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;

        // TODO: print parent identifier
        // TODO: print link identifier

        if let Some(name) = &self.name {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        writeln!(formatter, "    Size\t\t\t\t\t: {}", self.size)?;

        // TODO: convert to formatter.
        let date_time_string: String = Self::get_date_time_string(&self.creation_time);

        writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_string)?;

        // TODO: convert to formatter.
        let date_time_string: String = Self::get_date_time_string(&self.modification_time);

        writeln!(
            formatter,
            "    Modification time\t\t\t\t: {}",
            date_time_string
        )?;
        if let Some(date_time) = &self.access_time {
            // TODO: convert to formatter.
            let date_time_string: String = Self::get_date_time_string(date_time);

            writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_string)?;
        }
        if let Some(date_time) = &self.change_time {
            // TODO: convert to formatter.
            let date_time_string: String = Self::get_date_time_string(date_time);

            writeln!(formatter, "    Change time\t\t\t\t\t: {}", date_time_string)?;
        }
        // TODO: convert to formatter.
        let date_time_string: String = Self::get_date_time_string(&self.backup_time);

        writeln!(formatter, "    Backup time\t\t\t\t\t: {}", date_time_string)?;
        // TODO: print added time

        writeln!(
            formatter,
            "    Number of links\t\t\t\t: {}",
            self.number_of_links
        )?;
        if let Some(owner_identifier) = self.owner_identifier {
            writeln!(
                formatter,
                "    Owner identifier\t\t\t\t: {}",
                owner_identifier
            )?;
        }
        if let Some(group_identifier) = self.group_identifier {
            writeln!(
                formatter,
                "    Group identifier\t\t\t\t: {}",
                group_identifier
            )?;
        }
        if let Some(file_mode) = self.file_mode {
            let file_mode_info: PosixFileModeInfo = PosixFileModeInfo::new(file_mode);

            writeln!(formatter, "    File mode\t\t\t\t\t: {}", file_mode_info)?;
        }
        // TODO: print extended attributes

        writeln!(formatter)
    }
}

/// Information about a Hierarchical File System (HFS).
pub struct HfsInfo {}

impl HfsInfo {
    /// Retrieves the file entry information.
    fn get_file_entry_information(file_entry: &HfsFileEntry) -> HfsFileEntryInfo {
        let mut file_entry_information: HfsFileEntryInfo = HfsFileEntryInfo::new();

        file_entry_information.identifier = file_entry.get_identifier();
        file_entry_information.name = file_entry.get_name().cloned();
        file_entry_information.size = file_entry.get_size();
        file_entry_information.creation_time = file_entry.get_creation_time().clone();
        file_entry_information.modification_time = file_entry.get_modification_time().clone();
        file_entry_information.access_time = file_entry.get_access_time().cloned();
        file_entry_information.change_time = file_entry.get_change_time().cloned();
        file_entry_information.backup_time = file_entry.get_backup_time().clone();
        file_entry_information.number_of_links = file_entry.get_number_of_links();
        file_entry_information.owner_identifier = file_entry.get_owner_identifier().cloned();
        file_entry_information.group_identifier = file_entry.get_group_identifier().cloned();
        file_entry_information.file_mode = file_entry.get_file_mode().cloned();

        file_entry_information
    }

    /// Opens a file system.
    pub fn open_file_system(
        data_stream: &DataStreamReference,
    ) -> Result<HfsFileSystem, ErrorTrace> {
        let mut hfs_file_system: HfsFileSystem = HfsFileSystem::new();

        match hfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open HFS file system");
                return Err(error);
            }
        }
        Ok(hfs_file_system)
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut HfsFileEntry) -> Result<(), ErrorTrace> {
        let file_entry_information: HfsFileEntryInfo = Self::get_file_entry_information(file_entry);

        print!("{}", file_entry_information);

        let number_of_attributes: usize = match file_entry.get_number_of_extended_attributes() {
            Ok(number_of_attributes) => number_of_attributes,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of extended attributes"
                );
                return Err(error);
            }
        };
        if number_of_attributes > 0 {
            println!("    Extended attributes:");

            for (attribute_index, result) in file_entry.extended_attributes().enumerate() {
                let hfs_extended_attribute: HfsExtendedAttribute = match result {
                    Ok(hfs_extended_attribute) => hfs_extended_attribute,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve extended attribute: {}", attribute_index)
                        );
                        return Err(error);
                    }
                };
                let attribute_name: &HfsString = hfs_extended_attribute.get_name();

                println!(
                    "        Attribute {}\t\t\t\t: {}",
                    attribute_index + 1,
                    attribute_name
                );
            }
            println!();
        }
        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        hfs_entry_identifier: u64,
    ) -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = match Self::open_file_system(data_stream) {
            Ok(hfs_file_system) => hfs_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        if hfs_entry_identifier > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported identifier: {} value out of bounds",
                hfs_entry_identifier
            )));
        }
        let mut file_entry: HfsFileEntry =
            match hfs_file_system.get_file_entry_by_identifier(hfs_entry_identifier as u32) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!("Missing file entry"));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve file entry: 0x{:08x}",
                            hfs_entry_identifier
                        )
                    );
                    return Err(error);
                }
            };
        println!("Hierarchical File System (HFS) file entry information:");

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to print file entry: {}", hfs_entry_identifier)
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_path(
        data_stream: &DataStreamReference,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = match Self::open_file_system(data_stream) {
            Ok(hfs_file_system) => hfs_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        let mut file_entry: HfsFileEntry = match hfs_file_system.get_file_entry_by_path(path) {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => return Err(keramics_core::error_trace_new!("Missing file entry")),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        println!("Hierarchical File System (HFS) file entry information:");

        println!("    Path\t\t\t\t\t: {}", path);

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to print file entry");
                return Err(error);
            }
        }
        Ok(())
    }

    /// Prints information about the file system.
    pub fn print_file_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = match Self::open_file_system(data_stream) {
            Ok(hfs_file_system) => hfs_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        println!("Hierarchical File System (HFS) information:");

        let format_version: &str = match hfs_file_system.get_format() {
            HfsFormat::Hfs => "HFS",
            HfsFormat::HfsPlus => "HFS+",
            HfsFormat::HfsX => "HFSX",
        };
        println!("    Format version\t\t\t\t: {}", format_version);

        let volume_label: String = match hfs_file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        println!("    Volume label\t\t\t\t: {}", volume_label);

        println!(
            "    Block size\t\t\t\t\t: {} bytes",
            hfs_file_system.block_size
        );
        println!();

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = match Self::open_file_system(data_stream) {
            Ok(hfs_file_system) => hfs_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                return Err(error);
            }
        };
        println!("Hierarchical File System (HFS) hierarchy:");

        let mut file_entry: HfsFileEntry = match hfs_file_system.get_root_directory() {
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
        file_entry: &mut HfsFileEntry,
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
            let mut sub_file_entry: HfsFileEntry = match result {
                Ok(file_entry) => file_entry,
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
    use keramics_datetime::HfsTime;
    use keramics_types::Utf16String;

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/hfs/hfsplus.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let hfs_file_system: HfsFileSystem = HfsInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: HfsFileEntryInfo = HfsInfo::get_file_entry_information(&hfs_file_entry);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "    Identifier\t\t\t\t\t: 21\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9\n",
            "    Creation time\t\t\t\t: 2024-11-17T15:13:40\n",
            "    Modification time\t\t\t\t: 2024-11-17T15:13:40\n",
            "    Access time\t\t\t\t\t: 2024-11-17T15:13:57\n",
            "    Change time\t\t\t\t\t: 2024-11-17T15:14:02\n",
            "    Backup time\t\t\t\t\t: Not set (0)\n",
            "    Number of links\t\t\t\t: 1\n",
            "    Owner identifier\t\t\t\t: 501\n",
            "    Group identifier\t\t\t\t: 20\n",
            "    File mode\t\t\t\t\t: -rw-r--r-- (0o100644)\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/hfs/hfsplus.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let hfs_file_system: HfsFileSystem = HfsInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: HfsFileEntryInfo = HfsInfo::get_file_entry_information(&hfs_file_entry);

        assert_eq!(test_struct.identifier, 21);
        assert_eq!(
            test_struct.name,
            Some(HfsString::Utf16String(Utf16String::from("testfile1")))
        );
        assert_eq!(test_struct.size, 9);
        assert_eq!(
            test_struct.creation_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::HfsTime(HfsTime {
                timestamp: 3814701220
            })
        );
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701237
            }))
        );
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::HfsTime(HfsTime {
                timestamp: 3814701242
            }))
        );
        assert_eq!(test_struct.backup_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.owner_identifier, Some(501));
        assert_eq!(test_struct.group_identifier, Some(20));
        assert_eq!(test_struct.file_mode, Some(0o100644));

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
