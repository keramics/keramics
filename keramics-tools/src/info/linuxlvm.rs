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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::linuxlvm::{
    LinuxLvmDataFileDescriptor, LinuxLvmPhysicalVolume, LinuxLvmVolume, LinuxLvmVolumeSystem,
};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};

use crate::formatters::ByteSize;

/// Linux Logical Volume Manager (LVM) logical volume information.
struct LinuxLvmLogicalVolumeInfo<'a> {
    /// The logical volume index.
    index: usize,

    /// The logical volume.
    logical_volume: &'a LinuxLvmVolume,
}

impl<'a> LinuxLvmLogicalVolumeInfo<'a> {
    /// Creates new logical volume information.
    fn new(index: usize, logical_volume: &'a LinuxLvmVolume) -> Self {
        Self {
            index,
            logical_volume,
        }
    }
}

impl<'a> fmt::Display for LinuxLvmLogicalVolumeInfo<'a> {
    /// Formats logical volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Logical volume: {}", self.index + 1)?;

        writeln!(
            formatter,
            "        Name\t\t\t\t\t: {}",
            self.logical_volume.get_name()
        )?;
        writeln!(
            formatter,
            "        Identifier\t\t\t\t: {}",
            self.logical_volume.get_identifier()
        )?;
        let byte_size: ByteSize = ByteSize::new(self.logical_volume.get_volume_size(), 1024);
        writeln!(formatter, "        Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)
    }
}

/// Linux Logical Volume Manager (LVM) physical volume information.
struct LinuxLvmPhysicalVolumeInfo<'a> {
    /// The physical volume index.
    index: usize,

    /// The physical volume.
    physical_volume: &'a LinuxLvmPhysicalVolume,
}

impl<'a> LinuxLvmPhysicalVolumeInfo<'a> {
    /// Creates new physical volume information.
    fn new(index: usize, physical_volume: &'a LinuxLvmPhysicalVolume) -> Self {
        Self {
            index,
            physical_volume,
        }
    }
}

impl<'a> fmt::Display for LinuxLvmPhysicalVolumeInfo<'a> {
    /// Formats physical volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "    Physical volume: {}", self.index + 1)?;

        writeln!(
            formatter,
            "        Name\t\t\t\t\t: {}",
            self.physical_volume.get_name()
        )?;
        writeln!(
            formatter,
            "        Identifier\t\t\t\t: {}",
            self.physical_volume.get_identifier()
        )?;
        writeln!(formatter)
    }
}

/// Linux Logical Volume Manager (LVM) volume system information.
struct LinuxLvmVolumeSystemInfo<'a> {
    /// The volume system.
    volume_system: &'a LinuxLvmVolumeSystem,
}

impl<'a> LinuxLvmVolumeSystemInfo<'a> {
    /// Creates new volume system information.
    fn new(volume_system: &'a LinuxLvmVolumeSystem) -> Self {
        Self { volume_system }
    }
}

impl<'a> fmt::Display for LinuxLvmVolumeSystemInfo<'a> {
    /// Formats volume system information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Linux Logical Volume Manager (LVM) information:")?;

        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.volume_system.get_bytes_per_sector()
        )?;
        writeln!(formatter)?;

        writeln!(formatter, "    Volume group:")?;
        if let Some(name) = self.volume_system.get_name() {
            writeln!(formatter, "        Name\t\t\t\t\t: {}", name)?;
        }
        if let Some(identifier) = self.volume_system.get_identifier() {
            writeln!(formatter, "        Identifier\t\t\t\t: {}", identifier)?;
        }
        writeln!(
            formatter,
            "        Number of logical volumes\t\t: {}",
            self.volume_system.get_number_of_volumes()
        )?;
        writeln!(
            formatter,
            "        Number of physical volumes\t\t: {}",
            self.volume_system.get_number_of_physical_volumes()
        )?;
        writeln!(formatter)
    }
}

/// Information about a Linux Logical Volume Manager (LVM).
pub struct LinuxLvmInfo {}

impl LinuxLvmInfo {
    /// Opens a volume system.
    pub fn open_volume_system(path_buf: &PathBuf) -> Result<LinuxLvmVolumeSystem, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let file_name: PathComponent = match path_buf.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Unsupported file name"));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!("Missing file name"));
            }
        };
        let data_file_descriptors: [LinuxLvmDataFileDescriptor; 1] =
            [LinuxLvmDataFileDescriptor::new(file_name, 0)];

        let mut lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

        match lvm_volume_system.open(&file_resolver, &data_file_descriptors) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open Linux LVM volume system"
                );
                return Err(error);
            }
        }
        Ok(lvm_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let lvm_volume_system: LinuxLvmVolumeSystem = match Self::open_volume_system(path_buf) {
            Ok(lvm_volume_system) => lvm_volume_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                return Err(error);
            }
        };
        let volume_system_info: LinuxLvmVolumeSystemInfo =
            LinuxLvmVolumeSystemInfo::new(&lvm_volume_system);

        print!("{}", volume_system_info);

        for volume_index in 0..lvm_volume_system.get_number_of_physical_volumes() {
            let lvm_physical_volume: &LinuxLvmPhysicalVolume =
                match lvm_volume_system.get_physical_volume_by_index(volume_index) {
                    Some(physical_volume) => physical_volume,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing physical volume: {}",
                            volume_index
                        )));
                    }
                };
            let physical_volume_info: LinuxLvmPhysicalVolumeInfo =
                LinuxLvmPhysicalVolumeInfo::new(volume_index, lvm_physical_volume);

            print!("{}", physical_volume_info);
        }
        for (volume_index, result) in lvm_volume_system.volumes().enumerate() {
            let lvm_logical_volume: LinuxLvmVolume = match result {
                Ok(lvm_volume) => lvm_volume,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve volume: {}", volume_index)
                    );
                    return Err(error);
                }
            };
            let logical_volume_info: LinuxLvmLogicalVolumeInfo =
                LinuxLvmLogicalVolumeInfo::new(volume_index, &lvm_logical_volume);

            print!("{}", logical_volume_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::assert_lines_eq;

    #[test]
    fn test_logical_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/linuxlvm/lvm2.raw");
        let lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmInfo::open_volume_system(&path_buf)?;

        let lvm_logical_volume: LinuxLvmVolume = lvm_volume_system.get_volume_by_index(0)?;
        let test_struct: LinuxLvmLogicalVolumeInfo =
            LinuxLvmLogicalVolumeInfo::new(0, &lvm_logical_volume);

        let expected_string: &str = concat!(
            "    Logical volume: 1\n",
            "        Name\t\t\t\t\t: test_logical_volume1\n",
            "        Identifier\t\t\t\t: TDUzsI-6K36-Qipq-T835-ynjD-dCFB-Z36vG4\n",
            "        Size\t\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_physical_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/linuxlvm/lvm2.raw");
        let lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmInfo::open_volume_system(&path_buf)?;

        let lvm_physical_volume: &LinuxLvmPhysicalVolume =
            lvm_volume_system.get_physical_volume_by_index(0).unwrap();
        let test_struct: LinuxLvmPhysicalVolumeInfo =
            LinuxLvmPhysicalVolumeInfo::new(0, lvm_physical_volume);

        let expected_string: &str = concat!(
            "    Physical volume: 1\n",
            "        Name\t\t\t\t\t: pv0\n",
            "        Identifier\t\t\t\t: WBvRb2-vc3Y-T5Sl-rSxB-gmtd-tM9X-UJoDNT\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_volume_system_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/linuxlvm/lvm2.raw");
        let lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmInfo::open_volume_system(&path_buf)?;
        let test_struct: LinuxLvmVolumeSystemInfo =
            LinuxLvmVolumeSystemInfo::new(&lvm_volume_system);

        let expected_string: &str = concat!(
            "Linux Logical Volume Manager (LVM) information:\n",
            "    Bytes per sector\t\t\t\t: 512\n",
            "\n",
            "    Volume group:\n",
            "        Name\t\t\t\t\t: test_volume_group\n",
            "        Identifier\t\t\t\t: 22IVml-3dws-I85y-vDVy-wYV3-Umbr-pUkQSX\n",
            "        Number of logical volumes\t\t: 2\n",
            "        Number of physical volumes\t\t: 1\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
