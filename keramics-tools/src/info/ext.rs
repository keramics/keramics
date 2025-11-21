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
use keramics_encodings::CharacterEncoding;
use keramics_formats::Path;
use keramics_formats::ext::constants::*;
use keramics_formats::ext::{ExtExtendedAttribute, ExtFileEntry, ExtFileSystem};
use keramics_types::ByteString;

/// Extended File System (ext) compatible feature flags information.
struct ExtCompatibleFeatureFlagsInfo {
    /// Flags.
    flags: u32,
}

impl ExtCompatibleFeatureFlagsInfo {
    /// Creates new compatible feature flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ExtCompatibleFeatureFlagsInfo {
    /// Formats partition compatible feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(
                formatter,
                "        0x00000001: Pre-allocate directory blocks (EXT2_COMPAT_PREALLOC)"
            )?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(
                formatter,
                "        0x00000002: Has AFS server inodes (EXT2_FEATURE_COMPAT_IMAGIC_INODES)"
            )?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(
                formatter,
                "        0x00000004: Has journal (EXT3_FEATURE_COMPAT_HAS_JOURNAL)"
            )?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(
                formatter,
                "        0x00000008: Has extended attributes (EXT2_FEATURE_COMPAT_EXT_ATTR)"
            )?;
        }
        if self.flags & 0x00000010 != 0 {
            writeln!(
                formatter,
                "        0x00000010: Is resizable (EXT2_FEATURE_COMPAT_RESIZE_INO)"
            )?;
        }
        if self.flags & 0x00000020 != 0 {
            writeln!(
                formatter,
                "        0x00000020: Has indexed directories (EXT2_FEATURE_COMPAT_DIR_INDEX)"
            )?;
        }

        if self.flags & 0x00000200 != 0 {
            writeln!(
                formatter,
                "        0x00000200: Has sparse superblock version 2 (EXT4_FEATURE_COMPAT_SPARSE_SUPER2)"
            )?;
        }
        if self.flags & 0x00000400 != 0 {
            writeln!(
                formatter,
                "        0x00000400: (EXT4_FEATURE_COMPAT_FAST_COMMIT)"
            )?;
        }
        if self.flags & 0x00000800 != 0 {
            writeln!(
                formatter,
                "        0x00000800: (EXT4_FEATURE_COMPAT_STABLE_INODES)"
            )?;
        }
        if self.flags & 0x00001000 != 0 {
            writeln!(
                formatter,
                "        0x00001000: Has orphan file (EXT4_FEATURE_COMPAT_ORPHAN_FILE)"
            )?;
        }
        Ok(())
    }
}

/// Extended File System (ext) date and time information.
struct ExtDateTimeInfo<'a> {
    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> ExtDateTimeInfo<'a> {
    /// Creates new date and time information.
    fn new(date_time: &'a DateTime) -> Self {
        Self { date_time }
    }
}

impl<'a> fmt::Display for ExtDateTimeInfo<'a> {
    /// Formats partition date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::NotSet => write!(formatter, "Not set (0)"),
            DateTime::PosixTime32(posix_time32) => {
                write!(formatter, "{}", posix_time32.to_iso8601_string())
            }
            DateTime::PosixTime64Ns(posix_time64ns) => {
                write!(formatter, "{}", posix_time64ns.to_iso8601_string())
            }
            _ => return write!(formatter, "Unsupported date time"),
        }
    }
}

/// Extended File System (ext) compatible file entry information.
struct ExtFileEntryInfo {
    /// The inode number.
    pub inode_number: u32,

    /// The name.
    pub name: Option<ByteString>,

    /// The size.
    pub size: u64,

    /// Modification date and time.
    pub modification_time: Option<DateTime>,

    /// Access date and time.
    pub access_time: Option<DateTime>,

    /// Change date and time.
    pub change_time: Option<DateTime>,

    /// Creation date and time.
    pub creation_time: Option<DateTime>,

    /// Deletion date and time.
    pub deletion_time: DateTime,

    /// Number of links.
    pub number_of_links: u16,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// File mode.
    pub file_mode: u16,

    /// Device identifier.
    pub device_identifier: Option<u16>,

    /// Symbolic link target.
    pub symbolic_link_target: Option<ByteString>,
}

impl ExtFileEntryInfo {
    /// Creates new file entry information.
    fn new() -> Self {
        Self {
            inode_number: 0,
            name: None,
            size: 0,
            modification_time: None,
            access_time: None,
            change_time: None,
            creation_time: None,
            deletion_time: DateTime::NotSet,
            number_of_links: 0,
            owner_identifier: 0,
            group_identifier: 0,
            file_mode: 0,
            device_identifier: None,
            symbolic_link_target: None,
        }
    }
}

impl fmt::Display for ExtFileEntryInfo {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Inode number\t\t\t\t: {}", self.inode_number)?;

        if let Some(name) = &self.name {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        writeln!(formatter, "    Size\t\t\t\t\t: {}", self.size)?;

        if let Some(date_time) = &self.modification_time {
            let date_time_info: ExtDateTimeInfo = ExtDateTimeInfo::new(date_time);

            writeln!(
                formatter,
                "    Modification time\t\t\t\t: {}",
                date_time_info
            )?;
        }
        if let Some(date_time) = &self.access_time {
            let date_time_info: ExtDateTimeInfo = ExtDateTimeInfo::new(date_time);

            writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_info)?;
        }
        if let Some(date_time) = &self.change_time {
            let date_time_info: ExtDateTimeInfo = ExtDateTimeInfo::new(date_time);

            writeln!(
                formatter,
                "    Inode change time\t\t\t\t: {}",
                date_time_info
            )?;
        }
        if let Some(date_time) = &self.creation_time {
            let date_time_info: ExtDateTimeInfo = ExtDateTimeInfo::new(date_time);

            writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_info)?;
        }
        let date_time_info: ExtDateTimeInfo = ExtDateTimeInfo::new(&self.deletion_time);

        writeln!(formatter, "    Deletion time\t\t\t\t: {}", date_time_info)?;

        writeln!(
            formatter,
            "    Number of links\t\t\t\t: {}",
            self.number_of_links
        )?;
        writeln!(
            formatter,
            "    Owner identifier\t\t\t\t: {}",
            self.owner_identifier
        )?;
        writeln!(
            formatter,
            "    Group identifier\t\t\t\t: {}",
            self.group_identifier
        )?;
        let file_mode_info: ExtFileModeInfo = ExtFileModeInfo::new(self.file_mode);

        writeln!(formatter, "    File mode\t\t\t\t\t: {}", file_mode_info)?;
        if let Some(device_identifier) = &self.device_identifier {
            writeln!(
                formatter,
                "    Device number\t\t\t\t: {},{}",
                device_identifier >> 8,
                device_identifier & 0x00ff
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

/// Extended File System (ext) file mode information.
struct ExtFileModeInfo {
    /// Flags.
    file_mode: u16,
}

impl ExtFileModeInfo {
    /// Creates new file mode information.
    fn new(file_mode: u16) -> Self {
        Self { file_mode }
    }

    /// Retrieves a file mode string representation.
    fn get_file_mode_string(file_mode: u16) -> String {
        let mut string_parts: Vec<&str> = vec!["-"; 10];

        if file_mode & 0x0001 != 0 {
            string_parts[9] = "x";
        }
        if file_mode & 0x0002 != 0 {
            string_parts[8] = "w";
        }
        if file_mode & 0x0004 != 0 {
            string_parts[7] = "r";
        }
        if file_mode & 0x0008 != 0 {
            string_parts[6] = "x";
        }
        if file_mode & 0x0010 != 0 {
            string_parts[5] = "w";
        }
        if file_mode & 0x0020 != 0 {
            string_parts[4] = "r";
        }
        if file_mode & 0x0040 != 0 {
            string_parts[3] = "x";
        }
        if file_mode & 0x0080 != 0 {
            string_parts[2] = "w";
        }
        if file_mode & 0x0100 != 0 {
            string_parts[1] = "r";
        }
        string_parts[0] = match file_mode & 0xf000 {
            EXT_FILE_MODE_TYPE_FIFO => "p",
            EXT_FILE_MODE_TYPE_CHARACTER_DEVICE => "c",
            EXT_FILE_MODE_TYPE_DIRECTORY => "d",
            EXT_FILE_MODE_TYPE_BLOCK_DEVICE => "b",
            EXT_FILE_MODE_TYPE_SYMBOLIC_LINK => "l",
            EXT_FILE_MODE_TYPE_SOCKET => "s",
            _ => "-",
        };
        string_parts.join("")
    }
}

impl fmt::Display for ExtFileModeInfo {
    /// Formats partition file mode information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: String = Self::get_file_mode_string(self.file_mode);

        write!(formatter, "{} (0o{:0o})", string, self.file_mode)
    }
}

/// Extended File System (ext) incompatible feature flags information.
struct ExtIncompatibleFeatureFlagsInfo {
    /// Flags.
    flags: u32,
}

impl ExtIncompatibleFeatureFlagsInfo {
    /// Creates new incompatible feature flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ExtIncompatibleFeatureFlagsInfo {
    /// Formats partition incompatible feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(
                formatter,
                "        0x00000001: Has compression (EXT2_FEATURE_INCOMPAT_COMPRESSION)"
            )?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(
                formatter,
                "        0x00000002: Directory entry has file type (EXT2_FEATURE_INCOMPAT_FILETYPE)"
            )?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(
                formatter,
                "        0x00000004: Needs recovery (EXT3_FEATURE_INCOMPAT_RECOVER)"
            )?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(
                formatter,
                "        0x00000008: Journal device (EXT3_FEATURE_INCOMPAT_JOURNAL_DEV)"
            )?;
        }
        if self.flags & 0x00000010 != 0 {
            writeln!(
                formatter,
                "        0x00000010: Has meta block groups (EXT2_FEATURE_INCOMPAT_META_BG)"
            )?;
        }

        if self.flags & 0x00000040 != 0 {
            writeln!(
                formatter,
                "        0x00000040: Has extents (EXT4_FEATURE_INCOMPAT_EXTENTS)"
            )?;
        }
        if self.flags & 0x00000080 != 0 {
            writeln!(
                formatter,
                "        0x00000080: Has 64-bit support (EXT4_FEATURE_INCOMPAT_64BIT)"
            )?;
        }
        if self.flags & 0x00000100 != 0 {
            writeln!(formatter, "        0x00000100: (EXT4_FEATURE_INCOMPAT_MMP)")?;
        }
        if self.flags & 0x00000200 != 0 {
            writeln!(
                formatter,
                "        0x00000200: Has flexible block groups (EXT4_FEATURE_INCOMPAT_FLEX_BG)"
            )?;
        }
        if self.flags & 0x00000400 != 0 {
            writeln!(
                formatter,
                "        0x00000400: (EXT4_FEATURE_INCOMPAT_EA_INODE)"
            )?;
        }

        if self.flags & 0x00001000 != 0 {
            writeln!(
                formatter,
                "        0x00001000: (EXT4_FEATURE_INCOMPAT_DIRDATA)"
            )?;
        }
        if self.flags & 0x00002000 != 0 {
            writeln!(
                formatter,
                "        0x00002000: Has metadata checksum seed (EXT4_FEATURE_INCOMPAT_CSUM_SEED)"
            )?;
        }
        if self.flags & 0x00004000 != 0 {
            writeln!(
                formatter,
                "        0x00004000: (EXT4_FEATURE_INCOMPAT_LARGEDIR)"
            )?;
        }
        if self.flags & 0x00008000 != 0 {
            writeln!(
                formatter,
                "        0x00008000: (EXT4_FEATURE_INCOMPAT_INLINE_DATA)"
            )?;
        }
        if self.flags & 0x00010000 != 0 {
            writeln!(
                formatter,
                "        0x00010000: (EXT4_FEATURE_INCOMPAT_ENCRYPT)"
            )?;
        }
        if self.flags & 0x00020000 != 0 {
            writeln!(
                formatter,
                "        0x00020000: (EXT4_FEATURE_INCOMPAT_CASEFOLD)"
            )?;
        }
        Ok(())
    }
}

/// Extended File System (ext) read-only compatible feature flags information.
struct ExtReadOnlyCompatibleFeatureFlagsInfo {
    /// Flags.
    flags: u32,
}

impl ExtReadOnlyCompatibleFeatureFlagsInfo {
    /// Creates new read-only compatible feature flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ExtReadOnlyCompatibleFeatureFlagsInfo {
    /// Formats partition read-only compatible feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(
                formatter,
                "        0x00000001: Has sparse superblocks and group descriptor tables (EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER)"
            )?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(
                formatter,
                "        0x00000002: Contains large files (EXT2_FEATURE_RO_COMPAT_LARGE_FILE)"
            )?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(
                formatter,
                "        0x00000004: Has directory B-tree (EXT2_FEATURE_RO_COMPAT_BTREE_DIR)"
            )?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(
                formatter,
                "        0x00000008: (EXT4_FEATURE_RO_COMPAT_HUGE_FILE)"
            )?;
        }
        if self.flags & 0x00000010 != 0 {
            writeln!(
                formatter,
                "        0x00000010: (EXT4_FEATURE_RO_COMPAT_GDT_CSUM)"
            )?;
        }
        if self.flags & 0x00000020 != 0 {
            writeln!(
                formatter,
                "        0x00000020: (EXT4_FEATURE_RO_COMPAT_DIR_NLINK)"
            )?;
        }
        if self.flags & 0x00000040 != 0 {
            writeln!(
                formatter,
                "        0x00000040: Has large inodes (EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE)"
            )?;
        }
        if self.flags & 0x00000080 != 0 {
            writeln!(
                formatter,
                "        0x00000080: (EXT4_FEATURE_RO_COMPAT_HAS_SNAPSHOT)"
            )?;
        }
        if self.flags & 0x00000100 != 0 {
            writeln!(
                formatter,
                "        0x00000100: (EXT4_FEATURE_RO_COMPAT_QUOTA)"
            )?;
        }
        if self.flags & 0x00000200 != 0 {
            writeln!(
                formatter,
                "        0x00000200: (EXT4_FEATURE_RO_COMPAT_BIGALLOC)"
            )?;
        }
        if self.flags & 0x00000400 != 0 {
            writeln!(
                formatter,
                "        0x00000400: Has metadata checksums (EXT4_FEATURE_RO_COMPAT_METADATA_CSUM)"
            )?;
        }
        if self.flags & 0x00000800 != 0 {
            writeln!(
                formatter,
                "        0x00000800: (EXT4_FEATURE_RO_COMPAT_REPLICA)"
            )?;
        }
        if self.flags & 0x00001000 != 0 {
            writeln!(
                formatter,
                "        0x00001000: (EXT4_FEATURE_RO_COMPAT_READONLY)"
            )?;
        }
        if self.flags & 0x00002000 != 0 {
            writeln!(
                formatter,
                "        0x00002000: (EXT4_FEATURE_RO_COMPAT_PROJECT)"
            )?;
        }
        if self.flags & 0x00004000 != 0 {
            writeln!(
                formatter,
                "        0x00004000: (EXT4_FEATURE_RO_COMPAT_SHARED_BLOCKS)"
            )?;
        }
        if self.flags & 0x00008000 != 0 {
            writeln!(
                formatter,
                "        0x00008000: (EXT4_FEATURE_RO_COMPAT_VERITY)"
            )?;
        }
        if self.flags & 0x00010000 != 0 {
            writeln!(
                formatter,
                "        0x00010000: Orphan file may be non-empty (EXT4_FEATURE_RO_COMPAT_ORPHAN_PRESENT)"
            )?;
        }
        Ok(())
    }
}

/// Information about an Extended File System (ext).
pub struct ExtInfo {}

impl ExtInfo {
    /// Retrieves the file entry information.
    fn get_file_entry_information(
        ext_file_entry: &mut ExtFileEntry,
    ) -> Result<ExtFileEntryInfo, ErrorTrace> {
        let mut file_entry_information: ExtFileEntryInfo = ExtFileEntryInfo::new();

        file_entry_information.inode_number = ext_file_entry.get_inode_number();
        file_entry_information.name = ext_file_entry.get_name().cloned();
        file_entry_information.size = ext_file_entry.get_size();
        file_entry_information.modification_time = ext_file_entry.get_modification_time().cloned();
        file_entry_information.access_time = ext_file_entry.get_access_time().cloned();
        file_entry_information.change_time = ext_file_entry.get_change_time().cloned();
        file_entry_information.creation_time = ext_file_entry.get_creation_time().cloned();
        file_entry_information.deletion_time = ext_file_entry.get_deletion_time().clone();
        file_entry_information.number_of_links = ext_file_entry.get_number_of_links();
        file_entry_information.owner_identifier = ext_file_entry.get_owner_identifier();
        file_entry_information.group_identifier = ext_file_entry.get_group_identifier();
        file_entry_information.file_mode = ext_file_entry.get_file_mode();

        match ext_file_entry.get_device_identifier() {
            Ok(result) => file_entry_information.device_identifier = result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve device identifier"
                );
                return Err(error);
            }
        }
        match ext_file_entry.get_symbolic_link_target() {
            Ok(result) => file_entry_information.symbolic_link_target = result.cloned(),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve symbolic link target"
                );
                return Err(error);
            }
        }
        Ok(file_entry_information)
    }

    /// Opens a file system.
    pub fn open_file_system(
        data_stream: &DataStreamReference,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<ExtFileSystem, ErrorTrace> {
        let mut ext_file_system: ExtFileSystem = ExtFileSystem::new();

        match character_encoding {
            Some(encoding) => match ext_file_system.set_character_encoding(encoding) {
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
        match ext_file_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open ext file system");
                return Err(error);
            }
        }
        Ok(ext_file_system)
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut ExtFileEntry) -> Result<(), ErrorTrace> {
        let file_entry_information: ExtFileEntryInfo =
            match Self::get_file_entry_information(file_entry) {
                Ok(file_entry_information) => file_entry_information,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve file entry information"
                    );
                    return Err(error);
                }
            };
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
                let ext_extended_attribute: ExtExtendedAttribute = match result {
                    Ok(ext_extended_attribute) => ext_extended_attribute,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve extended attribute: {}", attribute_index)
                        );
                        return Err(error);
                    }
                };
                let attribute_name: &ByteString = ext_extended_attribute.get_name();

                println!(
                    "        Attribute {}\t\t\t\t: {}",
                    attribute_index + 1,
                    attribute_name
                );
            }
            println!("");
        }
        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        ext_entry_identifier: u64,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<(), ErrorTrace> {
        let ext_file_system: ExtFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(ext_file_system) => ext_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                    return Err(error);
                }
            };
        if ext_entry_identifier > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported identifier: {} value out of bounds",
                ext_entry_identifier
            )));
        }
        let mut file_entry: ExtFileEntry =
            match ext_file_system.get_file_entry_by_identifier(ext_entry_identifier as u32) {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve file entry: {}", ext_entry_identifier)
                    );
                    return Err(error);
                }
            };
        println!("Extended File System (ext) file entry information:");

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to print file entry: {}", ext_entry_identifier)
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
        let ext_file_system: ExtFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(ext_file_system) => ext_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                    return Err(error);
                }
            };
        let mut file_entry: Option<ExtFileEntry> =
            match ext_file_system.get_file_entry_by_path(path) {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                    return Err(error);
                }
            };
        if file_entry.is_none() {
            return Err(keramics_core::error_trace_new!("Missing file entry"));
        }
        println!("Extended File System (ext) file entry information:");

        println!("    Path\t\t\t\t\t: {}", path);

        match Self::print_file_entry(file_entry.as_mut().unwrap()) {
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
        let ext_file_system: ExtFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(ext_file_system) => ext_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                    return Err(error);
                }
            };
        println!("Extended File System (ext) information:");

        let format_version: u8 = ext_file_system.get_format_version();
        println!("    Format version\t\t\t\t: ext{}", format_version);

        let volume_label: String = match ext_file_system.get_volume_label() {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        println!("    Volume label\t\t\t\t: {}", volume_label);

        let feature_flags: u32 = ext_file_system.get_compatible_feature_flags();
        println!("    Compatible features\t\t\t\t: 0x{:08x}", feature_flags);

        let flags_info: ExtCompatibleFeatureFlagsInfo =
            ExtCompatibleFeatureFlagsInfo::new(feature_flags);
        println!("{}", flags_info);

        let feature_flags: u32 = ext_file_system.get_incompatible_feature_flags();
        println!("    Incompatible features\t\t\t: 0x{:08x}", feature_flags);

        let flags_info: ExtIncompatibleFeatureFlagsInfo =
            ExtIncompatibleFeatureFlagsInfo::new(feature_flags);
        println!("{}", flags_info);

        let feature_flags: u32 = ext_file_system.get_read_only_compatible_feature_flags();
        println!(
            "    Read-only compatible features\t\t: 0x{:08x}",
            feature_flags
        );

        let flags_info: ExtReadOnlyCompatibleFeatureFlagsInfo =
            ExtReadOnlyCompatibleFeatureFlagsInfo::new(feature_flags);
        println!("{}", flags_info);

        println!(
            "    Number of inodes\t\t\t\t: {}",
            ext_file_system.number_of_inodes
        );
        println!(
            "    Last mount path\t\t\t\t: {}",
            ext_file_system.last_mount_path
        );
        let date_time_string: String = match ext_file_system.last_mount_time {
            DateTime::NotSet => String::from("Not set (0)"),
            DateTime::PosixTime32(posix_time32) => posix_time32.to_iso8601_string(),
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported last mount time"
                ));
            }
        };
        println!("    Last mount time\t\t\t\t: {}", date_time_string);
        let date_time_string: String = match ext_file_system.last_written_time {
            DateTime::NotSet => String::from("Not set (0)"),
            DateTime::PosixTime32(posix_time32) => posix_time32.to_iso8601_string(),
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported last written time"
                ));
            }
        };
        println!("    Last written time\t\t\t\t: {}", date_time_string);
        println!("");

        Ok(())
    }

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(
        data_stream: &DataStreamReference,
        character_encoding: Option<&CharacterEncoding>,
    ) -> Result<(), ErrorTrace> {
        let ext_file_system: ExtFileSystem =
            match Self::open_file_system(data_stream, character_encoding) {
                Ok(ext_file_system) => ext_file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file file system");
                    return Err(error);
                }
            };
        println!("Extended File System (ext) hierarchy:");

        let mut file_entry: ExtFileEntry = match ext_file_system.get_root_directory() {
            Ok(result) => match result {
                Some(file_entry) => file_entry,
                None => {
                    println!("No root directory found");
                    return Ok(());
                }
            },
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
        file_entry: &mut ExtFileEntry,
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
            let mut sub_file_entry: ExtFileEntry = match result {
                Ok(ext_file_entry) => ext_file_entry,
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
    use keramics_datetime::{PosixTime32, PosixTime64Ns};

    #[test]
    fn test_compatible_feature_status_flags_information_fmt() -> Result<(), ErrorTrace> {
        let test_struct: ExtCompatibleFeatureFlagsInfo =
            ExtCompatibleFeatureFlagsInfo::new(0x00000038);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "        0x00000008: Has extended attributes (EXT2_FEATURE_COMPAT_EXT_ATTR)\n",
            "        0x00000010: Is resizable (EXT2_FEATURE_COMPAT_RESIZE_INO)\n",
            "        0x00000020: Has indexed directories (EXT2_FEATURE_COMPAT_DIR_INDEX)\n",
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_date_time_information_fmt() {
        let date_time: DateTime = DateTime::PosixTime32(PosixTime32::new(1281643591));
        let test_struct: ExtDateTimeInfo = ExtDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T20:06:31");

        let date_time: DateTime =
            DateTime::PosixTime64Ns(PosixTime64Ns::new(1281643591, 987654321));
        let test_struct: ExtDateTimeInfo = ExtDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T20:06:31.987654321");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: ExtDateTimeInfo = ExtDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "Not set (0)");
    }

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/ext/ext2.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let ext_file_system: ExtFileSystem =
            ExtInfo::open_file_system(&data_stream, Some(&CharacterEncoding::Utf8))?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: ExtFileEntryInfo =
            ExtInfo::get_file_entry_information(&mut ext_file_entry)?;

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "    Inode number\t\t\t\t: 14\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9\n",
            "    Modification time\t\t\t\t: 2025-01-04T07:58:01\n",
            "    Access time\t\t\t\t\t: 2025-01-04T07:58:02\n",
            "    Inode change time\t\t\t\t: 2025-01-04T07:58:01\n",
            "    Deletion time\t\t\t\t: Not set (0)\n",
            "    Number of links\t\t\t\t: 2\n",
            "    Owner identifier\t\t\t\t: 1000\n",
            "    Group identifier\t\t\t\t: 1000\n",
            "    File mode\t\t\t\t\t: -rw-r--r-- (0o100644)\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_string() {
        let string: String = ExtFileModeInfo::get_file_mode_string(0x1000);
        assert_eq!(string, "p---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0x2000);
        assert_eq!(string, "c---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0x4000);
        assert_eq!(string, "d---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0x6000);
        assert_eq!(string, "b---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0xa000);
        assert_eq!(string, "l---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0xc000);
        assert_eq!(string, "s---------");

        let string: String = ExtFileModeInfo::get_file_mode_string(0x81ff);
        assert_eq!(string, "-rwxrwxrwx");
    }

    #[test]
    fn test_file_mode_information_fmt() {
        let test_struct: ExtFileModeInfo = ExtFileModeInfo::new(0x81a4);
        let string: String = test_struct.to_string();
        assert_eq!(string, "-rw-r--r-- (0o100644)");
    }

    #[test]
    fn test_incompatible_feature_status_flags_information_fmt() -> Result<(), ErrorTrace> {
        let test_struct: ExtIncompatibleFeatureFlagsInfo =
            ExtIncompatibleFeatureFlagsInfo::new(0x00000002);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "        0x00000002: Directory entry has file type (EXT2_FEATURE_INCOMPAT_FILETYPE)\n",
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_read_only_compatible_feature_status_flags_information_fmt() -> Result<(), ErrorTrace> {
        let test_struct: ExtReadOnlyCompatibleFeatureFlagsInfo =
            ExtReadOnlyCompatibleFeatureFlagsInfo::new(0x00000003);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "        0x00000001: Has sparse superblocks and group descriptor tables (EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER)\n",
            "        0x00000002: Contains large files (EXT2_FEATURE_RO_COMPAT_LARGE_FILE)\n",
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/ext/ext2.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let ext_file_system: ExtFileSystem =
            ExtInfo::open_file_system(&data_stream, Some(&CharacterEncoding::Utf8))?;

        let path: Path = Path::from("/testdir1/testfile1");
        let mut ext_file_entry: ExtFileEntry =
            ext_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: ExtFileEntryInfo =
            ExtInfo::get_file_entry_information(&mut ext_file_entry)?;

        assert_eq!(test_struct.inode_number, 14);
        assert_eq!(test_struct.name, Some(ByteString::from("testfile1")));
        assert_eq!(test_struct.size, 9);
        assert_eq!(
            test_struct.modification_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977481)))
        );
        assert_eq!(
            test_struct.access_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977482)))
        );
        assert_eq!(
            test_struct.change_time,
            Some(DateTime::PosixTime32(PosixTime32::new(1735977481)))
        );
        assert_eq!(test_struct.creation_time, None);
        assert_eq!(test_struct.deletion_time, DateTime::NotSet);
        assert_eq!(test_struct.number_of_links, 2);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.device_identifier, None);
        assert_eq!(test_struct.symbolic_link_target, None);

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
