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
use keramics_formats::apfs::{ApfsContainer, ApfsVolume};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

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
        writeln!(formatter)?;
        writeln!(formatter, "Container:")?;

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

/// Apple File System (APFS) volume information.
struct ApfsVolumeInfo {
    /// The volume index.
    pub index: usize,

    /// Identifier.
    pub identifier: Uuid,

    /// Features flags.
    pub feature_flags: u64,

    /// Read-only compatible feature flags.
    pub read_only_compatible_feature_flags: u64,

    /// Incompatible feature flags.
    pub incompatible_feature_flags: u64,
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

impl ApfsVolumeInfo {
    /// Creates new volume information.
    fn new() -> Self {
        Self {
            index: 0,
            identifier: Uuid::new(),
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

    /// Retrieves the volume information.
    fn get_volume_information(volume_index: usize, apfs_volume: &ApfsVolume) -> ApfsVolumeInfo {
        let mut volume_information: ApfsVolumeInfo = ApfsVolumeInfo::new();

        volume_information.index = volume_index;
        volume_information.identifier = apfs_volume.get_identifier().clone();
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
            Ok(apfs_container) => apfs_container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        let container_info: ApfsContainerInfo = Self::get_container_information(&apfs_container);

        print!("{}", container_info);

        for (volume_index, result) in apfs_container.volumes().enumerate() {
            let apfs_volume: ApfsVolume = match result {
                Ok(apfs_volume) => apfs_volume,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

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
        assert_eq!(test_struct.feature_flags, 0x00000002);
        assert_eq!(test_struct.read_only_compatible_feature_flags, 0x00000000);
        assert_eq!(test_struct.incompatible_feature_flags, 0x00000001);

        Ok(())
    }

    // TODO: add tests for open_container
    // TODO: add tests for print_container
}
