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
use keramics_formats::apfs::{ApfsContainer, ApfsFileEntry, ApfsFileSystem, ApfsVolume};
use keramics_types::{ByteString, Uuid};

use crate::enums::DisplayPathType;
use crate::formatters::ByteSize;

use super::constants::*;
use super::posix::PosixFileModeInfo;

/// Apple File System (APFS) container feature flags information.
struct ApfsContainerFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsContainerFeatureFlagsInfo {
    /// Creates new container feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsContainerFeatureFlagsInfo {
    /// Formats container feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: Supports defragmentation (NX_FEATURE_DEFRAG)"
            )?;
        }
        if self.flags & 0x0000000000000002 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000002: Uses low-capacity Fusion Drive mode (NX_FEATURE_LCFD)"
            )?;
        }
        Ok(())
    }
}

/// Apple File System (APFS) container incompatibility feature flags information.
struct ApfsContainerIncompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsContainerIncompatibilityFeatureFlagsInfo {
    /// Creates new container incompatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsContainerIncompatibilityFeatureFlagsInfo {
    /// Formats container incompatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: (NX_INCOMPAT_VERSION1)"
            )?;
        }
        if self.flags & 0x0000000000000002 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000002: (NX_INCOMPAT_VERSION2)"
            )?;
        }

        if self.flags & 0x0000000000000100 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000100: (NX_INCOMPAT_FUSION)"
            )?;
        }
        Ok(())
    }
}

/// Apple File System (APFS) container information.
struct ApfsContainerInfo {
    /// Identifier.
    pub identifier: Uuid,

    /// Block size.
    pub block_size: u32,

    /// Features flags.
    pub feature_flags: u64,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u64,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u64,

    /// Number of volumes.
    pub number_of_volumes: usize,
}

impl ApfsContainerInfo {
    /// Creates new container information.
    fn new() -> Self {
        Self {
            identifier: Uuid::new(),
            block_size: 0,
            number_of_volumes: 0,
            feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
        }
    }
}

impl fmt::Display for ApfsContainerInfo {
    /// Formats container information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Apple File System (APFS) information:")?;

        println!("    Features\t\t\t\t\t: 0x{:016x}", self.feature_flags);
        let flags_info: ApfsContainerFeatureFlagsInfo =
            ApfsContainerFeatureFlagsInfo::new(self.feature_flags);
        println!("{}", flags_info);

        println!(
            "    Read-only compatible features\t\t: 0x{:016x}",
            self.read_only_compatible_feature_flags
        );
        let flags_info: ApfsContainerReadOnlyCompatibilityFeatureFlagsInfo =
            ApfsContainerReadOnlyCompatibilityFeatureFlagsInfo::new(
                self.read_only_compatible_feature_flags,
            );
        println!("{}", flags_info);

        println!(
            "    Incompatible features\t\t\t: 0x{:016x}",
            self.incompatible_feature_flags
        );
        let flags_info: ApfsContainerIncompatibilityFeatureFlagsInfo =
            ApfsContainerIncompatibilityFeatureFlagsInfo::new(self.incompatible_feature_flags);
        println!("{}", flags_info);

        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;

        let byte_size: ByteSize = ByteSize::new(self.block_size as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Number of volumes\t\t\t\t: {}",
            self.number_of_volumes
        )?;
        writeln!(formatter)
    }
}

/// Apple File System (APFS) container read-only compatibility feature flags information.
struct ApfsContainerReadOnlyCompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsContainerReadOnlyCompatibilityFeatureFlagsInfo {
    /// Creates new container read-only compatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsContainerReadOnlyCompatibilityFeatureFlagsInfo {
    /// Formats container read-only compatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        _ = formatter;
        _ = self.flags;

        Ok(())
    }
}

/// Apple File System (APFS) file entry information.
struct ApfsFileEntryInfo {
    /// The identifier.
    pub identifier: u64,

    /// The name.
    pub name: Option<ByteString>,

    /// The size.
    pub size: u64,

    /// Creation date and time.
    pub creation_time: DateTime,

    /// Modifiation date and time.
    pub modification_time: DateTime,

    /// Access date and time.
    pub access_time: DateTime,

    /// Change date and time.
    pub change_time: DateTime,

    /// Number of links.
    pub number_of_links: u32,

    /// Owner identifier.
    pub owner_identifier: u32,

    /// Group identifier.
    pub group_identifier: u32,

    /// File mode.
    pub file_mode: u16,
}

impl ApfsFileEntryInfo {
    /// Creates new file entry information.
    fn new() -> Self {
        Self {
            identifier: 0,
            name: None,
            size: 0,
            creation_time: DateTime::NotSet,
            modification_time: DateTime::NotSet,
            access_time: DateTime::NotSet,
            change_time: DateTime::NotSet,
            number_of_links: 0,
            owner_identifier: 0,
            group_identifier: 0,
            file_mode: 0,
        }
    }

    /// Retrieves the string representation of a date and time value.
    fn get_date_time_string(date_time: &DateTime) -> String {
        match date_time {
            DateTime::ApfsTime(apfs_time) => apfs_time.to_iso8601_string(),
            DateTime::NotSet => String::from(NOT_SET_VALUE),
            _ => return String::from("Unsupported date time"),
        }
    }
}

impl fmt::Display for ApfsFileEntryInfo {
    /// Formats file entry information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;

        // TODO: print parent identifier
        // TODO: print link identifier

        if let Some(name) = &self.name {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name)?;
        };
        let byte_size: ByteSize = ByteSize::new(self.size, 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

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
        // TODO: convert to formatter.
        let date_time_string: String = Self::get_date_time_string(&self.access_time);

        writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_string)?;

        // TODO: convert to formatter.
        let date_time_string: String = Self::get_date_time_string(&self.change_time);

        writeln!(formatter, "    Change time\t\t\t\t\t: {}", date_time_string)?;

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
        let file_mode_info: PosixFileModeInfo = PosixFileModeInfo::new(self.file_mode);

        writeln!(formatter, "    File mode\t\t\t\t\t: {}", file_mode_info)?;

        // TODO: print extended attributes

        writeln!(formatter)
    }
}

/// Apple File System (APFS) volume feature flags information.
struct ApfsVolumeFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsVolumeFeatureFlagsInfo {
    /// Creates new volume feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsVolumeFeatureFlagsInfo {
    /// Formats volume feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: (APFS_FEATURE_DEFRAG_PRERELEASE)"
            )?;
        }
        if self.flags & 0x0000000000000002 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000002: (APFS_FEATURE_HARDLINK_MAP_RECORDS)"
            )?;
        }
        if self.flags & 0x0000000000000004 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000004: (APFS_FEATURE_DEFRAG)"
            )?;
        }
        if self.flags & 0x0000000000000008 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000008: (APFS_FEATURE_STRICTATIME)"
            )?;
        }
        if self.flags & 0x0000000000000010 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000010: (APFS_FEATURE_VOLGRP_SYSTEM_INO_SPACE)"
            )?;
        }
        Ok(())
    }
}

/// Apple File System (APFS) volume incompatibility feature flags information.
struct ApfsVolumeIncompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsVolumeIncompatibilityFeatureFlagsInfo {
    /// Creates new volume incompatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsVolumeIncompatibilityFeatureFlagsInfo {
    /// Formats volume incompatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: (APFS_INCOMPAT_CASE_INSENSITIVE)"
            )?;
        }
        if self.flags & 0x0000000000000002 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000002: (APFS_INCOMPAT_DATALESS_SNAPS)"
            )?;
        }
        if self.flags & 0x0000000000000004 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000004: (APFS_INCOMPAT_ENC_ROLLED)"
            )?;
        }
        if self.flags & 0x0000000000000008 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000008: (APFS_INCOMPAT_NORMALIZATION_INSENSITIVE)"
            )?;
        }
        if self.flags & 0x0000000000000010 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000010: (APFS_INCOMPAT_INCOMPLETE_RESTORE)"
            )?;
        }
        if self.flags & 0x0000000000000020 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000020: (APFS_INCOMPAT_SEALED_VOLUME)"
            )?;
        }
        if self.flags & 0x0000000000000040 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000040: (APFS_INCOMPAT_RESERVED_40)"
            )?;
        }
        Ok(())
    }
}

/// Apple File System (APFS) volume information.
struct ApfsVolumeInfo {
    /// The volume index.
    pub index: usize,

    /// Identifier.
    pub identifier: Uuid,

    /// Volume label.
    pub volume_label: Option<ByteString>,

    /// Features flags.
    pub feature_flags: u64,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u64,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u64,
}

impl ApfsVolumeInfo {
    /// Creates new volume information.
    fn new() -> Self {
        Self {
            index: 0,
            identifier: Uuid::new(),
            volume_label: None,
            feature_flags: 0,
            read_only_compatible_feature_flags: 0,
            incompatible_feature_flags: 0,
        }
    }
}

impl fmt::Display for ApfsVolumeInfo {
    /// Formats volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Volume: {}", self.index + 1)?;

        println!("    Features\t\t\t\t\t: 0x{:016x}", self.feature_flags);
        let flags_info: ApfsVolumeFeatureFlagsInfo =
            ApfsVolumeFeatureFlagsInfo::new(self.feature_flags);
        println!("{}", flags_info);

        println!(
            "    Read-only compatible features\t\t: 0x{:016x}",
            self.read_only_compatible_feature_flags
        );
        let flags_info: ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo =
            ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo::new(
                self.read_only_compatible_feature_flags,
            );
        println!("{}", flags_info);

        println!(
            "    Incompatible features\t\t\t: 0x{:016x}",
            self.incompatible_feature_flags
        );
        let flags_info: ApfsVolumeIncompatibilityFeatureFlagsInfo =
            ApfsVolumeIncompatibilityFeatureFlagsInfo::new(self.incompatible_feature_flags);
        println!("{}", flags_info);

        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;

        let volume_label: String = match &self.volume_label {
            Some(volume_label) => volume_label.to_string(),
            None => String::new(),
        };
        writeln!(formatter, "    Volume label\t\t\t\t: {}", volume_label)?;

        writeln!(formatter)
    }
}

/// Apple File System (APFS) volume read-only compatibility feature flags information.
struct ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo {
    /// Creates new volume read-only compatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo {
    /// Formats volume read-only compatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        _ = formatter;
        _ = self.flags;

        Ok(())
    }
}

/// Information about an Apple File System (APFS).
pub struct ApfsInfo {}

impl ApfsInfo {
    /// Retrieves the container information.
    fn get_container_information(apfs_container: &ApfsContainer) -> ApfsContainerInfo {
        let mut container_information: ApfsContainerInfo = ApfsContainerInfo::new();

        container_information.identifier = apfs_container.get_identifier().clone();
        container_information.block_size = apfs_container.get_block_size();
        container_information.feature_flags = apfs_container.get_feature_flags();
        container_information.read_only_compatible_feature_flags =
            apfs_container.get_read_only_compatible_feature_flags();
        container_information.incompatible_feature_flags =
            apfs_container.get_incompatible_feature_flags();
        container_information.number_of_volumes = apfs_container.get_number_of_volumes();

        container_information
    }

    /// Retrieves the file entry information.
    fn get_file_entry_information(file_entry: &ApfsFileEntry) -> ApfsFileEntryInfo {
        let mut file_entry_information: ApfsFileEntryInfo = ApfsFileEntryInfo::new();

        file_entry_information.identifier = file_entry.get_identifier();
        file_entry_information.name = file_entry.get_name().cloned();
        file_entry_information.size = file_entry.get_size();
        file_entry_information.creation_time = file_entry.get_creation_time().clone();
        file_entry_information.modification_time = file_entry.get_modification_time().clone();
        file_entry_information.access_time = file_entry.get_access_time().clone();
        file_entry_information.change_time = file_entry.get_change_time().clone();
        file_entry_information.number_of_links = file_entry.get_number_of_links();
        file_entry_information.owner_identifier = file_entry.get_owner_identifier();
        file_entry_information.group_identifier = file_entry.get_group_identifier();
        file_entry_information.file_mode = file_entry.get_file_mode();

        file_entry_information
    }

    /// Retrieves the volume information.
    fn get_volume_information(volume_index: usize, apfs_volume: &ApfsVolume) -> ApfsVolumeInfo {
        let mut volume_information: ApfsVolumeInfo = ApfsVolumeInfo::new();

        volume_information.index = volume_index;
        volume_information.identifier = apfs_volume.get_identifier().clone();
        volume_information.volume_label = apfs_volume.get_volume_label().cloned();
        volume_information.feature_flags = apfs_volume.get_feature_flags();
        volume_information.read_only_compatible_feature_flags =
            apfs_volume.get_read_only_compatible_feature_flags();
        volume_information.incompatible_feature_flags =
            apfs_volume.get_incompatible_feature_flags();

        volume_information
    }

    /// Opens a container.
    pub fn open_container(data_stream: &DataStreamReference) -> Result<ApfsContainer, ErrorTrace> {
        let mut apfs_container: ApfsContainer = ApfsContainer::new();

        match apfs_container.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open APFS container");
                return Err(error);
            }
        }
        Ok(apfs_container)
    }

    /// Prints information about a container.
    pub fn print_container(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let apfs_container: ApfsContainer = match Self::open_container(data_stream) {
            Ok(container) => container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        let container_info: ApfsContainerInfo = Self::get_container_information(&apfs_container);

        print!("{}", container_info);

        for (volume_index, result) in apfs_container.volumes().enumerate() {
            let apfs_volume: ApfsVolume = match result {
                Ok(volume) => volume,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve volume: {}", volume_index)
                    );
                    return Err(error);
                }
            };
            let volume_info: ApfsVolumeInfo =
                Self::get_volume_information(volume_index, &apfs_volume);

            print!("{}", volume_info);
        }
        Ok(())
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut ApfsFileEntry) -> Result<(), ErrorTrace> {
        let file_entry_information: ApfsFileEntryInfo =
            Self::get_file_entry_information(file_entry);

        print!("{}", file_entry_information);

        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_identifier(
        data_stream: &DataStreamReference,
        volume_number: usize,
        apfs_entry_identifier: u64,
    ) -> Result<(), ErrorTrace> {
        let apfs_container: ApfsContainer = match Self::open_container(data_stream) {
            Ok(container) => container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        let volume_index: usize = if volume_number > 0 {
            volume_number - 1
        } else {
            let number_of_volumes: usize = apfs_container.get_number_of_volumes();

            if number_of_volumes == 0 {
                return Err(keramics_core::error_trace_new!(
                    "No volumes found in container"
                ));
            } else if number_of_volumes > 1 {
                return Err(keramics_core::error_trace_new!(
                    "Container has more than one volume"
                ));
            }
            0
        };
        let apfs_volume: ApfsVolume = match apfs_container.get_volume_by_index(volume_index) {
            Ok(volume) => volume,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open volume: {}", volume_index)
                );
                return Err(error);
            }
        };
        let apfs_file_system: ApfsFileSystem = match apfs_volume.get_file_system() {
            Ok(file_system) => file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve file system in volume: {}", volume_index)
                );
                return Err(error);
            }
        };
        let mut file_entry: ApfsFileEntry =
            match apfs_file_system.get_file_entry_by_identifier(apfs_entry_identifier) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!("Missing file entry"));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve file entry: {}", apfs_entry_identifier)
                    );
                    return Err(error);
                }
            };
        println!("Apple File System (APFS) file entry information:");

        match Self::print_file_entry(&mut file_entry) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to print file entry: {}", apfs_entry_identifier)
                );
                return Err(error);
            }
        }
        Ok(())
    }

    /// Prints information about a specific file entry.
    pub fn print_file_entry_by_path(
        data_stream: &DataStreamReference,
        volume_number: usize,
        path: &Path,
    ) -> Result<(), ErrorTrace> {
        let apfs_container: ApfsContainer = match Self::open_container(data_stream) {
            Ok(container) => container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        // TODO: add support to determine volume from path.
        let volume_index: usize = if volume_number > 0 {
            volume_number - 1
        } else {
            let number_of_volumes: usize = apfs_container.get_number_of_volumes();

            if number_of_volumes == 0 {
                return Err(keramics_core::error_trace_new!(
                    "No volumes found in container"
                ));
            } else if number_of_volumes > 1 {
                return Err(keramics_core::error_trace_new!(
                    "Container has more than one volume"
                ));
            }
            0
        };
        let apfs_volume: ApfsVolume = match apfs_container.get_volume_by_index(volume_index) {
            Ok(volume) => volume,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to open volume: {}", volume_index)
                );
                return Err(error);
            }
        };
        let apfs_file_system: ApfsFileSystem = match apfs_volume.get_file_system() {
            Ok(file_system) => file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve file system in volume: {}", volume_index)
                );
                return Err(error);
            }
        };
        let mut file_entry: ApfsFileEntry = match apfs_file_system.get_file_entry_by_path(path) {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => return Err(keramics_core::error_trace_new!("Missing file entry")),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        println!("Apple File System (APFS) file entry information:");

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

    /// Prints the file system hierarchy.
    pub fn print_hierarchy(
        data_stream: &DataStreamReference,
        volume_number: usize,
        volume_path_type: &DisplayPathType,
        path: Option<&String>,
    ) -> Result<(), ErrorTrace> {
        let apfs_container: ApfsContainer = match Self::open_container(data_stream) {
            Ok(container) => container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        // TODO: handle volume index/identifier in combination with path
        if path.is_some() {
            todo!();
        }
        println!("Apple File System (APFS) hierarchy:");

        for (volume_index, result) in apfs_container.volumes().enumerate() {
            if volume_number != 0 && volume_number != volume_index + 1 {
                continue;
            }
            let apfs_volume: ApfsVolume = match result {
                Ok(volume) => volume,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve volume: {}", volume_index + 1)
                    );
                    return Err(error);
                }
            };
            let volume_path_component: String = match volume_path_type {
                DisplayPathType::Identifier => format!("apfs{{{}}}", apfs_volume.get_identifier()),
                DisplayPathType::Index => format!("apfs{}", volume_index + 1),
            };
            let apfs_file_system: ApfsFileSystem = match apfs_volume.get_file_system() {
                Ok(file_system) => file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve file system of volume: {}",
                            volume_index + 1
                        )
                    );
                    return Err(error);
                }
            };
            let mut file_entry: ApfsFileEntry = match apfs_file_system.get_root_directory() {
                Ok(result) => match result {
                    Some(file_entry) => file_entry,
                    None => {
                        if volume_number != 0 {
                            println!("No root directory found");
                            return Ok(());
                        }
                        continue;
                    }
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve root directory"
                    );
                    return Err(error);
                }
            };
            let mut path_components: Vec<String> = vec![volume_path_component];

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
        }
        Ok(())
    }

    /// Prints the file entry hierarchy.
    fn print_hierarchy_file_entry(
        file_entry: &mut ApfsFileEntry,
        path_components: &mut Vec<String>,
    ) -> Result<(), ErrorTrace> {
        let path: String = if file_entry.is_root_directory() {
            format!("/{}/", path_components.join("/"))
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
            let mut sub_file_entry: ApfsFileEntry = match result {
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
    use keramics_datetime::ApfsTime;

    #[test]
    fn test_get_container_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;
        let test_struct: ApfsContainerInfo = ApfsInfo::get_container_information(&apfs_container);

        assert_eq!(
            test_struct.identifier.to_string(),
            "34d0674d-da87-4991-a3de-27eb13011c3e"
        );
        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.feature_flags, 0x00000000);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.incompatible_feature_flags, 0x00000002);
        assert_eq!(test_struct.number_of_volumes, 1);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;
        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let apfs_file_system: ApfsFileSystem = apfs_volume.get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: ApfsFileEntryInfo = ApfsInfo::get_file_entry_information(&apfs_file_entry);

        assert_eq!(test_struct.identifier, 18);
        assert_eq!(test_struct.name, Some(ByteString::from("testfile1")));
        assert_eq!(test_struct.size, 9);
        assert_eq!(
            test_struct.creation_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254516511
            })
        );
        assert_eq!(
            test_struct.modification_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254251713
            })
        );
        assert_eq!(
            test_struct.access_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765254251713
            })
        );
        assert_eq!(
            test_struct.change_time,
            DateTime::ApfsTime(ApfsTime {
                timestamp: 1785841765262832871
            })
        );
        assert_eq!(test_struct.number_of_links, 2);
        assert_eq!(test_struct.owner_identifier, 99);
        assert_eq!(test_struct.group_identifier, 99);
        assert_eq!(test_struct.file_mode, 0o100644);

        Ok(())
    }

    #[test]
    fn test_get_volume_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;
        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let test_struct: ApfsVolumeInfo = ApfsInfo::get_volume_information(0, &apfs_volume);

        assert_eq!(test_struct.index, 0);
        assert_eq!(
            test_struct.identifier.to_string(),
            "33d13da9-f1c8-4d2a-b9c7-71ab9dbe5fe2"
        );
        assert_eq!(
            test_struct.volume_label,
            Some(ByteString::from("apfs_test"))
        );
        assert_eq!(test_struct.feature_flags, 0x00000002);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.incompatible_feature_flags, 0x00000001);

        Ok(())
    }

    // TODO: add tests for open_container
    // TODO: add tests for print_container
}
