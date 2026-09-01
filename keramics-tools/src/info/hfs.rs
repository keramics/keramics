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
use keramics_formats::hfs::{
    HfsExtendedAttribute, HfsFileEntry, HfsFileSystem, HfsFormat, HfsString,
};
use keramics_formats::{ExtendedAttributeIterator, Path};

use crate::formatters::ByteSize;

use super::constants::*;
use super::posix::PosixFileModeInfo;

/// Hierarchical File System (HFS) date and time information.
struct HfsTimeInfo<'a> {
    /// Format.
    format: HfsFormat,

    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> HfsTimeInfo<'a> {
    /// Creates new date and time information.
    fn new(format: &HfsFormat, date_time: &'a DateTime) -> Self {
        Self {
            format: format.clone(),
            date_time,
        }
    }
}

impl<'a> fmt::Display for HfsTimeInfo<'a> {
    /// Formats date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::NotSet => write!(formatter, "{}", NOT_SET_VALUE),
            DateTime::HfsTime(hfs_time) => {
                let iso8601_string: String = hfs_time.to_iso8601_string();

                if self.format == HfsFormat::Hfs {
                    write!(formatter, "{}", iso8601_string)
                } else {
                    write!(formatter, "{}+00:00", iso8601_string)
                }
            }
            _ => write!(formatter, "Unsupported date time"),
        }
    }
}

/// Hierarchical File System (HFS) file entry information.
struct HfsFileEntryInfo<'a> {
    /// Format.
    format: HfsFormat,

    /// File entry.
    file_entry: &'a HfsFileEntry,
}

impl<'a> HfsFileEntryInfo<'a> {
    /// Creates new file entry information.
    fn new(format: &HfsFormat, file_entry: &'a HfsFileEntry) -> Self {
        Self {
            format: format.clone(),
            file_entry,
        }
    }
}

impl<'a> fmt::Display for HfsFileEntryInfo<'a> {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.file_entry.get_identifier()
        )?;

        // TODO: print parent identifier
        // TODO: print link identifier

        if let Some(name) = self.file_entry.get_name() {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        let byte_size: ByteSize = ByteSize::new(self.file_entry.get_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        let date_time_info: HfsTimeInfo =
            HfsTimeInfo::new(&self.format, self.file_entry.get_creation_time());

        writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_info)?;

        let date_time_info: HfsTimeInfo =
            HfsTimeInfo::new(&self.format, self.file_entry.get_modification_time());

        writeln!(
            formatter,
            "    Modification time\t\t\t\t: {}",
            date_time_info
        )?;
        if let Some(date_time) = self.file_entry.get_access_time() {
            let date_time_info: HfsTimeInfo = HfsTimeInfo::new(&self.format, date_time);

            writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_info)?;
        }
        if let Some(date_time) = self.file_entry.get_change_time() {
            let date_time_info: HfsTimeInfo = HfsTimeInfo::new(&self.format, date_time);

            writeln!(formatter, "    Change time\t\t\t\t\t: {}", date_time_info)?;
        }
        let date_time_info: HfsTimeInfo =
            HfsTimeInfo::new(&self.format, self.file_entry.get_backup_time());

        writeln!(formatter, "    Backup time\t\t\t\t\t: {}", date_time_info)?;

        // TODO: print added time

        writeln!(
            formatter,
            "    Number of links\t\t\t\t: {}",
            self.file_entry.get_number_of_links()
        )?;
        if let Some(owner_identifier) = self.file_entry.get_owner_identifier() {
            writeln!(
                formatter,
                "    Owner identifier\t\t\t\t: {}",
                owner_identifier
            )?;
        }
        if let Some(group_identifier) = self.file_entry.get_group_identifier() {
            writeln!(
                formatter,
                "    Group identifier\t\t\t\t: {}",
                group_identifier
            )?;
        }
        if let Some(file_mode) = self.file_entry.get_file_mode() {
            let file_mode_info: PosixFileModeInfo = PosixFileModeInfo::new(*file_mode);

            writeln!(formatter, "    File mode\t\t\t\t\t: {}", file_mode_info)?;
        }
        writeln!(formatter)
    }
}

/// Hierarchical File System (HFS) file system information.
struct HfsFileSystemInfo<'a> {
    /// File system.
    file_system: &'a HfsFileSystem,
}

impl<'a> HfsFileSystemInfo<'a> {
    /// Creates new file system information.
    fn new(file_system: &'a HfsFileSystem) -> Self {
        Self { file_system }
    }
}

impl<'a> fmt::Display for HfsFileSystemInfo<'a> {
    /// Formats file system information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Hierarchical File System (HFS) information:")?;

        let format_string: &str = match self.file_system.get_format() {
            HfsFormat::Hfs => "HFS",
            HfsFormat::HfsPlus => "HFS+",
            HfsFormat::HfsX => "HFSX",
        };
        writeln!(formatter, "    Format\t\t\t\t\t: {}", format_string)?;

        let volume_label: String = match self.file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        writeln!(formatter, "    Volume label\t\t\t\t: {}", volume_label)?;

        let byte_size: ByteSize = ByteSize::new(self.file_system.block_size as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        if let Some(embedded_volume_extent) = self.file_system.get_embedded_volume_extent() {
            writeln!(formatter)?;

            writeln!(formatter, "Embedded volume:")?;

            let offset: u64 =
                (embedded_volume_extent.block_number as u64) * (self.file_system.block_size as u64);
            writeln!(
                formatter,
                "    Offset\t\t\t\t\t: {} (0x{:08x})",
                offset, offset
            )?;

            let size: u64 = (embedded_volume_extent.number_of_blocks as u64)
                * (self.file_system.block_size as u64);
            let byte_size: ByteSize = ByteSize::new(size, 1024);
            writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;
        }
        writeln!(formatter)
    }
}

/// Information about a Hierarchical File System (HFS).
pub struct HfsInfo {}

impl HfsInfo {
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
    fn print_file_entry(
        format: &HfsFormat,
        file_entry: &mut HfsFileEntry,
    ) -> Result<(), ErrorTrace> {
        let file_entry_information: HfsFileEntryInfo = HfsFileEntryInfo::new(format, &file_entry);

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
                    Ok(extended_attribute) => extended_attribute,
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
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
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

        let format: &HfsFormat = hfs_file_system.get_format();
        match Self::print_file_entry(format, &mut file_entry) {
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
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
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

        let format: &HfsFormat = hfs_file_system.get_format();
        match Self::print_file_entry(format, &mut file_entry) {
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
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        let file_system_information: HfsFileSystemInfo = HfsFileSystemInfo::new(&hfs_file_system);

        print!("{}", file_system_information);

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(
        data_stream: &DataStreamReference,
        path: Option<&String>,
    ) -> Result<(), ErrorTrace> {
        let hfs_file_system: HfsFileSystem = match Self::open_file_system(data_stream) {
            Ok(hfs_file_system) => hfs_file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        println!("Hierarchical File System (HFS) hierarchy:");

        let mut file_entry: HfsFileEntry = match path {
            Some(path) => match hfs_file_system.get_file_entry_by_path(&Path::from(path)) {
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
            None => match hfs_file_system.get_root_directory() {
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
    use keramics_datetime::HfsTime;

    use crate::assert_lines_eq;

    #[test]
    fn test_date_time_information_fmt() {
        let date_time: DateTime = DateTime::HfsTime(HfsTime::new(3458215528));
        let test_struct: HfsTimeInfo = HfsTimeInfo::new(&HfsFormat::Hfs, &date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2013-08-01T15:25:28");

        let date_time: DateTime = DateTime::HfsTime(HfsTime::new(3458215528));
        let test_struct: HfsTimeInfo = HfsTimeInfo::new(&HfsFormat::HfsPlus, &date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2013-08-01T15:25:28+00:00");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: HfsTimeInfo = HfsTimeInfo::new(&HfsFormat::HfsPlus, &date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, NOT_SET_VALUE);
    }

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/hfs/hfsplus.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let hfs_file_system: HfsFileSystem = HfsInfo::open_file_system(&data_stream)?;

        let path: Path = Path::from("/testdir1/testfile1");
        let hfs_file_entry: HfsFileEntry = hfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let format: &HfsFormat = hfs_file_system.get_format();
        let test_struct: HfsFileEntryInfo = HfsFileEntryInfo::new(format, &hfs_file_entry);

        let expected_string: &str = concat!(
            "    Identifier\t\t\t\t\t: 20\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9 bytes\n",
            "    Creation time\t\t\t\t: 2026-08-04T11:09:04+00:00\n",
            "    Modification time\t\t\t\t: 2026-08-04T11:09:04+00:00\n",
            "    Access time\t\t\t\t\t: 2026-08-04T11:09:04+00:00\n",
            "    Change time\t\t\t\t\t: 2026-08-04T11:09:05+00:00\n",
            "    Backup time\t\t\t\t\t: Not set (0)\n",
            "    Number of links\t\t\t\t: 1\n",
            "    Owner identifier\t\t\t\t: 501\n",
            "    Group identifier\t\t\t\t: 20\n",
            "    File mode\t\t\t\t\t: -rw-r--r-- (0o100644)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_file_system_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/hfs/hfsplus.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let hfs_file_system: HfsFileSystem = HfsInfo::open_file_system(&data_stream)?;

        let test_struct: HfsFileSystemInfo = HfsFileSystemInfo::new(&hfs_file_system);

        let expected_string: &str = concat!(
            "Hierarchical File System (HFS) information:\n",
            "    Format\t\t\t\t\t: HFS+\n",
            "    Volume label\t\t\t\t: hfsplus_test\n",
            "    Block size\t\t\t\t\t: 4.0 KiB (4096 bytes)\n",
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
