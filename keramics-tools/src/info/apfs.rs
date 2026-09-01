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
use keramics_formats::apfs::{
    ApfsContainer, ApfsExtendedAttribute, ApfsFileEntry, ApfsFileSystem, ApfsVolume,
};
use keramics_formats::{ExtendedAttributeIterator, Path};
use keramics_types::ByteString;

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
struct ApfsContainerInfo<'a> {
    /// Container.
    container: &'a ApfsContainer,
}

impl<'a> ApfsContainerInfo<'a> {
    /// Creates new container information.
    fn new(container: &'a ApfsContainer) -> Self {
        Self { container }
    }
}

impl<'a> fmt::Display for ApfsContainerInfo<'a> {
    /// Formats container information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Apple File System (APFS) information:")?;

        let flags: u64 = self.container.get_feature_flags();
        writeln!(formatter, "    Features\t\t\t\t\t: 0x{:016x}", flags)?;
        let flags_info: ApfsContainerFeatureFlagsInfo = ApfsContainerFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let flags: u64 = self.container.get_read_only_compatible_feature_flags();
        writeln!(
            formatter,
            "    Read-only compatible features\t\t: 0x{:016x}",
            flags
        )?;
        // Note that currently there are no known read-only compatible features.
        writeln!(formatter)?;

        let flags: u64 = self.container.get_incompatible_feature_flags();
        writeln!(
            formatter,
            "    Incompatible features\t\t\t: 0x{:016x}",
            flags
        )?;
        let flags_info: ApfsContainerIncompatibilityFeatureFlagsInfo =
            ApfsContainerIncompatibilityFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.container.get_identifier()
        )?;
        let byte_size: ByteSize = ByteSize::new(self.container.get_block_size() as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Number of volumes\t\t\t\t: {}",
            self.container.get_number_of_volumes()
        )?;
        writeln!(formatter)
    }
}

/// Apple File System (APFS) file entry information.
struct ApfsFileEntryInfo<'a> {
    /// File entry.
    file_entry: &'a ApfsFileEntry,
}

impl<'a> ApfsFileEntryInfo<'a> {
    /// Creates new file entry information.
    fn new(file_entry: &'a ApfsFileEntry) -> Self {
        Self { file_entry }
    }
}

impl<'a> fmt::Display for ApfsFileEntryInfo<'a> {
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

        let date_time_info: ApfsTimeInfo = ApfsTimeInfo::new(self.file_entry.get_creation_time());

        writeln!(formatter, "    Creation time\t\t\t\t: {}", date_time_info)?;

        let date_time_info: ApfsTimeInfo =
            ApfsTimeInfo::new(self.file_entry.get_modification_time());

        writeln!(
            formatter,
            "    Modification time\t\t\t\t: {}",
            date_time_info
        )?;
        let date_time_info: ApfsTimeInfo = ApfsTimeInfo::new(self.file_entry.get_access_time());

        writeln!(formatter, "    Access time\t\t\t\t\t: {}", date_time_info)?;

        let date_time_info: ApfsTimeInfo = ApfsTimeInfo::new(self.file_entry.get_change_time());

        writeln!(formatter, "    Change time\t\t\t\t\t: {}", date_time_info)?;

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

        writeln!(formatter)
    }
}

/// Apple File System (APFS) date and time information.
struct ApfsTimeInfo<'a> {
    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> ApfsTimeInfo<'a> {
    /// Creates new date and time information.
    fn new(date_time: &'a DateTime) -> Self {
        Self { date_time }
    }
}

impl<'a> fmt::Display for ApfsTimeInfo<'a> {
    /// Formats date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::NotSet => write!(formatter, "{}", NOT_SET_VALUE),
            DateTime::ApfsTime(apfs_time) => {
                write!(formatter, "{}+00:00", apfs_time.to_iso8601_string())
            }
            _ => write!(formatter, "Unsupported date time"),
        }
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
struct ApfsVolumeInfo<'a> {
    /// Volume index.
    index: usize,

    /// Volue.
    volume: &'a ApfsVolume,
}

impl<'a> ApfsVolumeInfo<'a> {
    /// Creates new volume information.
    fn new(index: usize, volume: &'a ApfsVolume) -> Self {
        Self { index, volume }
    }
}

impl<'a> fmt::Display for ApfsVolumeInfo<'a> {
    /// Formats volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Volume: {}", self.index + 1)?;

        let flags: u64 = self.volume.get_feature_flags();
        writeln!(formatter, "    Features\t\t\t\t\t: 0x{:016x}", flags)?;
        let flags_info: ApfsVolumeFeatureFlagsInfo = ApfsVolumeFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let flags: u64 = self.volume.get_read_only_compatible_feature_flags();
        writeln!(
            formatter,
            "    Read-only compatible features\t\t: 0x{:016x}",
            flags
        )?;
        let flags_info: ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo =
            ApfsVolumeReadOnlyCompatibilityFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let flags: u64 = self.volume.get_incompatible_feature_flags();
        writeln!(
            formatter,
            "    Incompatible features\t\t\t: 0x{:016x}",
            flags
        )?;
        let flags_info: ApfsVolumeIncompatibilityFeatureFlagsInfo =
            ApfsVolumeIncompatibilityFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.volume.get_identifier()
        )?;
        let volume_label: String = match self.volume.get_volume_label() {
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
        let container_info: ApfsContainerInfo = ApfsContainerInfo::new(&apfs_container);

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
            let volume_info: ApfsVolumeInfo = ApfsVolumeInfo::new(volume_index, &apfs_volume);

            print!("{}", volume_info);
        }
        Ok(())
    }

    /// Prints information about a file entry.
    fn print_file_entry(file_entry: &mut ApfsFileEntry) -> Result<(), ErrorTrace> {
        let file_entry_information: ApfsFileEntryInfo = ApfsFileEntryInfo::new(&file_entry);

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
                let apfs_extended_attribute: ApfsExtendedAttribute = match result {
                    Ok(extended_attribute) => extended_attribute,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve extended attribute: {}", attribute_index)
                        );
                        return Err(error);
                    }
                };
                let attribute_name: &ByteString = apfs_extended_attribute.get_name();

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

    use crate::assert_lines_eq;

    #[test]
    fn test_container_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;

        let test_struct: ApfsContainerInfo = ApfsContainerInfo::new(&apfs_container);

        let expected_string: &str = concat!(
            "Apple File System (APFS) information:\n",
            "    Features\t\t\t\t\t: 0x0000000000000000\n",
            "\n",
            "    Read-only compatible features\t\t: 0x0000000000000000\n",
            "\n",
            "    Incompatible features\t\t\t: 0x0000000000000002\n",
            "        0x0000000000000002: (NX_INCOMPAT_VERSION2)\n",
            "\n",
            "    Identifier\t\t\t\t\t: 34d0674d-da87-4991-a3de-27eb13011c3e\n",
            "    Block size\t\t\t\t\t: 4.0 KiB (4096 bytes)\n",
            "    Number of volumes\t\t\t\t: 1\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_date_time_information_fmt() {
        let date_time: DateTime = DateTime::ApfsTime(ApfsTime::new(1281643591987654321));
        let test_struct: ApfsTimeInfo = ApfsTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T20:06:31.987654321+00:00");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: ApfsTimeInfo = ApfsTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, NOT_SET_VALUE);
    }

    #[test]
    fn test_file_entry_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;
        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;
        let apfs_file_system: ApfsFileSystem = apfs_volume.get_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let apfs_file_entry: ApfsFileEntry =
            apfs_file_system.get_file_entry_by_path(&path)?.unwrap();
        let test_struct: ApfsFileEntryInfo = ApfsFileEntryInfo::new(&apfs_file_entry);

        let expected_string: &str = concat!(
            "    Identifier\t\t\t\t\t: 18\n",
            "    Name\t\t\t\t\t: testfile1\n",
            "    Size\t\t\t\t\t: 9 bytes\n",
            "    Creation time\t\t\t\t: 2026-08-04T11:09:25.254516511+00:00\n",
            "    Modification time\t\t\t\t: 2026-08-04T11:09:25.254251713+00:00\n",
            "    Access time\t\t\t\t\t: 2026-08-04T11:09:25.254251713+00:00\n",
            "    Change time\t\t\t\t\t: 2026-08-04T11:09:25.262832871+00:00\n",
            "    Number of links\t\t\t\t: 2\n",
            "    Owner identifier\t\t\t\t: 99\n",
            "    Group identifier\t\t\t\t: 99\n",
            "    File mode\t\t\t\t\t: -rw-r--r-- (0o100644)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apfs/apfs.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apfs_container: ApfsContainer = ApfsInfo::open_container(&data_stream)?;
        let apfs_volume: ApfsVolume = apfs_container.get_volume_by_index(0)?;

        let test_struct: ApfsVolumeInfo = ApfsVolumeInfo::new(0, &apfs_volume);

        let expected_string: &str = concat!(
            "Volume: 1\n",
            "    Features\t\t\t\t\t: 0x0000000000000002\n",
            "        0x0000000000000002: (APFS_FEATURE_HARDLINK_MAP_RECORDS)\n",
            "\n",
            "    Read-only compatible features\t\t: 0x0000000000000000\n",
            "\n",
            "    Incompatible features\t\t\t: 0x0000000000000001\n",
            "        0x0000000000000001: (APFS_INCOMPAT_CASE_INSENSITIVE)\n",
            "\n",
            "    Identifier\t\t\t\t\t: 33d13da9-f1c8-4d2a-b9c7-71ab9dbe5fe2\n",
            "    Volume label\t\t\t\t: apfs_test\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_container
    // TODO: add tests for print_container
}
