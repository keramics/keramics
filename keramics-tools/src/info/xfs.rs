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
use keramics_encodings::CharacterEncoding;
use keramics_formats::Path;
use keramics_formats::xfs::{XfsExtendedAttribute, XfsFileEntry, XfsFileSystem};
use keramics_types::ByteString;

use crate::formatters::ByteSize;

use super::constants::*;
use super::posix::PosixFileModeInfo;

/// X File System (XFS) date and time information.
struct XfsDateTimeInfo<'a> {
    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> XfsDateTimeInfo<'a> {
    /// Creates new date and time information.
    fn new(date_time: &'a DateTime) -> Self {
        Self { date_time }
    }
}

impl<'a> fmt::Display for XfsDateTimeInfo<'a> {
    /// Formats date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::NotSet => write!(formatter, "{}", NOT_SET_VALUE),
            DateTime::PosixTime32(posix_time32) => {
                write!(formatter, "{}+00:00", posix_time32.to_iso8601_string())
            }
            DateTime::PosixTime64Ns(posix_time64ns) => {
                write!(formatter, "{}+00:00", posix_time64ns.to_iso8601_string())
            }
            _ => write!(formatter, "Unsupported date time"),
        }
    }
}

/// X File System (XFS) file entry information.
struct XfsFileEntryInfo<'a> {
    /// File entry.
    file_entry: &'a XfsFileEntry,

    /// Symbolic link target.
    pub symbolic_link_target: Option<ByteString>,
}

impl<'a> XfsFileEntryInfo<'a> {
    /// Creates new file entry information.
    fn new(file_entry: &'a XfsFileEntry) -> Self {
        Self {
            file_entry,
            symbolic_link_target: None,
        }
    }
}

impl<'a> fmt::Display for XfsFileEntryInfo<'a> {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "    Inode number\t\t\t\t: {}",
            self.file_entry.get_inode_number()
        )?;

        if let Some(name) = self.file_entry.get_name() {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        let byte_size: ByteSize = ByteSize::new(self.file_entry.get_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        if let Some(date_time) = self.file_entry.get_creation_time() {
            let date_time_info: XfsDateTimeInfo = XfsDateTimeInfo::new(date_time);

            writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_info)?;
        }
        let date_time_info: XfsDateTimeInfo =
            XfsDateTimeInfo::new(self.file_entry.get_modification_time());

        writeln!(
            formatter,
            "    Modification time\t\t\t\t: {}",
            date_time_info
        )?;
        let date_time_info: XfsDateTimeInfo =
            XfsDateTimeInfo::new(self.file_entry.get_access_time());

        writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_info)?;

        let date_time_info: XfsDateTimeInfo =
            XfsDateTimeInfo::new(self.file_entry.get_change_time());

        writeln!(
            formatter,
            "    Inode change time\t\t\t\t: {}",
            date_time_info
        )?;
        writeln!(
            formatter,
            "    Number of links\t\t\t\t: {}",
            self.file_entry.get_number_of_links()
        )?;
        writeln!(
            formatter,
            "    Owner identifier\t\t\t\t: {}",
            self.file_entry.get_owner_identifier()
        )?;
        writeln!(
            formatter,
            "    Group identifier\t\t\t\t: {}",
            self.file_entry.get_group_identifier()
        )?;
        let file_mode_info: PosixFileModeInfo =
            PosixFileModeInfo::new(self.file_entry.get_file_mode());

        writeln!(formatter, "    File mode\t\t\t\t\t: {}", file_mode_info)?;

        if let Some(device_identifier) = self.file_entry.get_device_identifier() {
            writeln!(
                formatter,
                "    Device number\t\t\t\t: {},{}",
                *device_identifier >> 18,
                *device_identifier & 0x0003ffff
            )?;
        }
        if let Some(symbolic_link_target) = &self.symbolic_link_target {
            writeln!(
                formatter,
                "    Symbolic link target\t\t\t: {}",
                symbolic_link_target
            )?;
        }
        writeln!(formatter)
    }
}

/// X File System (XFS) file system information.
struct XfsFileSystemInfo<'a> {
    /// File system.
    file_system: &'a XfsFileSystem,
}

impl<'a> XfsFileSystemInfo<'a> {
    /// Creates new file system information.
    fn new(file_system: &'a XfsFileSystem) -> Self {
        Self { file_system }
    }
}

impl<'a> fmt::Display for XfsFileSystemInfo<'a> {
    /// Formats file sytem information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "X File System (XFS) information:")?;

        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}",
            self.file_system.get_format_version()
        )?;
        // TODO: print feature flags
        // TODO: print secondary feature flags

        let flags: u32 = self.file_system.get_compatible_feature_flags();

        writeln!(
            formatter,
            "    Compatible features\t\t\t\t: 0x{:08x}",
            flags
        )?;
        writeln!(formatter)?;

        let flags: u32 = self.file_system.get_read_only_compatible_feature_flags();

        writeln!(
            formatter,
            "    Read-only compatible features\t\t: 0x{:08x}",
            flags
        )?;
        let flags_info: XfsReadOnlyCompatibleFeatureFlagsInfo =
            XfsReadOnlyCompatibleFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let flags: u32 = self.file_system.get_incompatible_feature_flags();

        writeln!(
            formatter,
            "    Incompatible features\t\t\t: 0x{:08x}",
            flags
        )?;
        let flags_info: XfsIncompatibleFeatureFlagsInfo =
            XfsIncompatibleFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let volume_label: String = match self.file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        writeln!(formatter, "    Volume label\t\t\t\t: {}", volume_label)?;

        let byte_size: ByteSize = ByteSize::new(self.file_system.get_block_size() as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        let byte_size: ByteSize = ByteSize::new(self.file_system.get_inode_size() as u64, 1024);
        writeln!(formatter, "    Inode size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)
    }
}

/// X File System (XFS) incompatible feature flags information.
struct XfsIncompatibleFeatureFlagsInfo {
    /// Flags.
    flags: u32,
}

impl XfsIncompatibleFeatureFlagsInfo {
    /// Creates new incompatible feature flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for XfsIncompatibleFeatureFlagsInfo {
    /// Formats incompatible feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(
                formatter,
                "        0x00000001: (XFS_SB_FEAT_INCOMPAT_FTYPE)"
            )?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(
                formatter,
                "        0x00000002: (XFS_SB_FEAT_INCOMPAT_SPINODES)"
            )?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(
                formatter,
                "        0x00000004: (XFS_SB_FEAT_INCOMPAT_META_UUID)"
            )?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(
                formatter,
                "        0x00000008: (XFS_SB_FEAT_INCOMPAT_BIGTIME)"
            )?;
        }
        if self.flags & 0x00000010 != 0 {
            writeln!(
                formatter,
                "        0x00000010: (XFS_SB_FEAT_INCOMPAT_NEEDSREPAIR)"
            )?;
        }
        if self.flags & 0x00000020 != 0 {
            writeln!(
                formatter,
                "        0x00000020: (XFS_SB_FEAT_INCOMPAT_NREXT64)"
            )?;
        }
        if self.flags & 0x00000040 != 0 {
            writeln!(
                formatter,
                "        0x00000040: (XFS_SB_FEAT_INCOMPAT_EXCHRANGE)"
            )?;
        }
        if self.flags & 0x00000080 != 0 {
            writeln!(
                formatter,
                "        0x00000080: (XFS_SB_FEAT_INCOMPAT_PARENT)"
            )?;
        }
        if self.flags & 0x00000100 != 0 {
            writeln!(
                formatter,
                "        0x00000100: (XFS_SB_FEAT_INCOMPAT_METADIR)"
            )?;
        }
        if self.flags & 0x00000200 != 0 {
            writeln!(
                formatter,
                "        0x00000200: (XFS_SB_FEAT_INCOMPAT_ZONED)"
            )?;
        }
        if self.flags & 0x00000400 != 0 {
            writeln!(
                formatter,
                "        0x00000400: (XFS_SB_FEAT_INCOMPAT_ZONE_GAPS)"
            )?;
        }
        Ok(())
    }
}

/// X File System (XFS) read-only compatible feature flags information.
struct XfsReadOnlyCompatibleFeatureFlagsInfo {
    /// Flags.
    flags: u32,
}

impl XfsReadOnlyCompatibleFeatureFlagsInfo {
    /// Creates new read-only compatible feature flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for XfsReadOnlyCompatibleFeatureFlagsInfo {
    /// Formats read-only compatible feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(
                formatter,
                "        0x00000001: (XFS_SB_FEAT_RO_COMPAT_FINOBT)"
            )?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(
                formatter,
                "        0x00000002: (XFS_SB_FEAT_RO_COMPAT_RMAPBT)"
            )?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(
                formatter,
                "        0x00000004: (XFS_SB_FEAT_RO_COMPAT_REFLINK)"
            )?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(
                formatter,
                "        0x00000008: (XFS_SB_FEAT_RO_COMPAT_INOBTCNT)"
            )?;
        }
        Ok(())
    }
}

/// Information about an X File System (XFS).
pub struct XfsInfo {}

impl XfsInfo {
    /// Opens a file system.
    pub fn open_file_system(
        data_stream: &DataStreamReference,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<XfsFileSystem, ErrorTrace> {
        let mut xfs_file_system: XfsFileSystem = XfsFileSystem::new();

        match character_encoding {
            Some(encoding) => match xfs_file_system.set_character_encoding(encoding) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to set character encoding"
                    );
                    return Err(error);
                }
            },
            None => {}
        }
        match xfs_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open XFS file system");
                return Err(error);
            }
        }
        Ok(xfs_file_system)
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut XfsFileEntry) -> Result<(), ErrorTrace> {
        let symbolic_link_target: Option<ByteString> = match file_entry.get_symbolic_link_target() {
            Ok(link_target) => link_target.cloned(),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve symbolic link target"
                );
                return Err(error);
            }
        };
        let mut file_entry_information: XfsFileEntryInfo = XfsFileEntryInfo::new(&file_entry);
        file_entry_information.symbolic_link_target = symbolic_link_target;

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
            println!("    Xfsended attributes:");

            for (attribute_index, result) in file_entry.extended_attributes().enumerate() {
                let xfs_extended_attribute: XfsExtendedAttribute = match result {
                    Ok(xfs_extended_attribute) => xfs_extended_attribute,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve extended attribute: {}", attribute_index)
                        );
                        return Err(error);
                    }
                };
                let attribute_name: &ByteString = xfs_extended_attribute.get_name();

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
        xfs_entry_identifier: u64,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(xfs_file_system) => xfs_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };
        if xfs_entry_identifier > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported identifier: {} value out of bounds",
                xfs_entry_identifier
            )));
        }
        let mut file_entry: XfsFileEntry =
            match xfs_file_system.get_file_entry_by_identifier(xfs_entry_identifier) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!("Missing file entry"));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve file entry: {}", xfs_entry_identifier)
                    );
                    return Err(error);
                }
            };
        println!("X File System (XFS) file entry information:");

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to print file entry: {}", xfs_entry_identifier)
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
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(xfs_file_system) => xfs_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };
        let mut file_entry: XfsFileEntry = match xfs_file_system.get_file_entry_by_path(path) {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!("Missing file entry"));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        println!("X File System (XFS) file entry information:");

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
    pub fn print_file_system(
        data_stream: &DataStreamReference,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(xfs_file_system) => xfs_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };
        let file_system_information: XfsFileSystemInfo = XfsFileSystemInfo::new(&xfs_file_system);

        print!("{}", file_system_information);

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(
        data_stream: &DataStreamReference,
        character_encoding: Option<&CharacterEncoding>,
        path: Option<&String>,
    ) -> Result<(), ErrorTrace> {
        let xfs_file_system: XfsFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(xfs_file_system) => xfs_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };
        println!("X File System (XFS) hierarchy:");

        let mut file_entry: XfsFileEntry = match path {
            Some(path) => match xfs_file_system.get_file_entry_by_path(&Path::from(path)) {
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
            None => match xfs_file_system.get_root_directory() {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => {
                    println!("No root directory found");
                    return Ok(());
                }
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
        file_entry: &mut XfsFileEntry,
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
            let mut sub_file_entry: XfsFileEntry = match result {
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
    use keramics_datetime::{PosixTime32, PosixTime64Ns};

    use crate::assert_lines_eq;

    #[test]
    fn test_date_time_information_fmt() {
        let date_time: DateTime = DateTime::PosixTime32(PosixTime32::new(1281643591));
        let test_struct: XfsDateTimeInfo = XfsDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T20:06:31+00:00");

        let date_time: DateTime =
            DateTime::PosixTime64Ns(PosixTime64Ns::new(1281643591, 987654321));
        let test_struct: XfsDateTimeInfo = XfsDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T20:06:31.987654321+00:00");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: XfsDateTimeInfo = XfsDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, NOT_SET_VALUE);
    }

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/xfs/xfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let xfs_file_system: XfsFileSystem =
            XfsInfo::open_file_system(&data_stream, Some(&CharacterEncoding::Utf8))?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut xfs_file_entry: XfsFileEntry =
            xfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let symbolic_link_target: Option<ByteString> =
            xfs_file_entry.get_symbolic_link_target()?.cloned();
        let mut test_struct: XfsFileEntryInfo = XfsFileEntryInfo::new(&xfs_file_entry);
        test_struct.symbolic_link_target = symbolic_link_target;

        let expected_string: &str = concat!(
            "    Inode number\t\t\t\t: 16133\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9 bytes\n",
            "    Creation time\t\t\t\t: 2026-08-27T13:34:08.180859596+00:00\n",
            "    Modification time\t\t\t\t: 2026-08-27T13:34:08.181438932+00:00\n",
            "    Access time\t\t\t\t\t: 2026-08-27T13:34:08.180859596+00:00\n",
            "    Inode change time\t\t\t\t: 2026-08-27T13:34:08.182386739+00:00\n",
            "    Number of links\t\t\t\t: 2\n",
            "    Owner identifier\t\t\t\t: 1000\n",
            "    Group identifier\t\t\t\t: 1000\n",
            "    File mode\t\t\t\t\t: -rw-r--r-- (0o100644)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_file_system_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/xfs/xfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let xfs_file_system: XfsFileSystem =
            XfsInfo::open_file_system(&data_stream, Some(&CharacterEncoding::Utf8))?;

        let test_struct: XfsFileSystemInfo = XfsFileSystemInfo::new(&xfs_file_system);

        let expected_string: &str = concat!(
            "X File System (XFS) information:\n",
            "    Format version\t\t\t\t: 5\n",
            "    Compatible features\t\t\t\t: 0x00000000\n",
            "\n",
            "    Read-only compatible features\t\t: 0x0000000f\n",
            "        0x00000001: (XFS_SB_FEAT_RO_COMPAT_FINOBT)\n",
            "        0x00000002: (XFS_SB_FEAT_RO_COMPAT_RMAPBT)\n",
            "        0x00000004: (XFS_SB_FEAT_RO_COMPAT_REFLINK)\n",
            "        0x00000008: (XFS_SB_FEAT_RO_COMPAT_INOBTCNT)\n",
            "\n",
            "    Incompatible features\t\t\t: 0x000000e3\n",
            "        0x00000001: (XFS_SB_FEAT_INCOMPAT_FTYPE)\n",
            "        0x00000002: (XFS_SB_FEAT_INCOMPAT_SPINODES)\n",
            "        0x00000020: (XFS_SB_FEAT_INCOMPAT_NREXT64)\n",
            "        0x00000040: (XFS_SB_FEAT_INCOMPAT_EXCHRANGE)\n",
            "        0x00000080: (XFS_SB_FEAT_INCOMPAT_PARENT)\n",
            "\n",
            "    Volume label\t\t\t\t: xfs_test\n",
            "    Block size\t\t\t\t\t: 4.0 KiB (4096 bytes)\n",
            "    Inode size\t\t\t\t\t: 512 bytes\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_incompatible_feature_status_flags_information_fmt() -> Result<(), ErrorTrace> {
        let test_struct: XfsIncompatibleFeatureFlagsInfo =
            XfsIncompatibleFeatureFlagsInfo::new(0x000000e3);

        let expected_string: &str = concat!(
            "        0x00000001: (XFS_SB_FEAT_INCOMPAT_FTYPE)\n",
            "        0x00000002: (XFS_SB_FEAT_INCOMPAT_SPINODES)\n",
            "        0x00000020: (XFS_SB_FEAT_INCOMPAT_NREXT64)\n",
            "        0x00000040: (XFS_SB_FEAT_INCOMPAT_EXCHRANGE)\n",
            "        0x00000080: (XFS_SB_FEAT_INCOMPAT_PARENT)\n",
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_read_only_compatible_feature_status_flags_information_fmt() -> Result<(), ErrorTrace> {
        let test_struct: XfsReadOnlyCompatibleFeatureFlagsInfo =
            XfsReadOnlyCompatibleFeatureFlagsInfo::new(0x0000000f);

        let expected_string: &str = concat!(
            "        0x00000001: (XFS_SB_FEAT_RO_COMPAT_FINOBT)\n",
            "        0x00000002: (XFS_SB_FEAT_RO_COMPAT_RMAPBT)\n",
            "        0x00000004: (XFS_SB_FEAT_RO_COMPAT_REFLINK)\n",
            "        0x00000008: (XFS_SB_FEAT_RO_COMPAT_INOBTCNT)\n",
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_file_system
    // TODO: add tests for print_file_entry
    // TODO: add tests for print_file_entry_by_identifier
    // TODO: add tests for print_file_entry_by_path
    // TODO: add tests for print_file_system
    // TODO: add tests for print_hierarchy
    // TODO: add tests for print_hierarchy_file_entry
}
