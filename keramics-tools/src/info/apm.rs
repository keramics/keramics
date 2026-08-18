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
use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};
use keramics_types::ByteString;

use crate::formatters::ByteSize;

/// Apple Partition Map (APM) partition information.
struct ApmPartitionInfo<'a> {
    /// The partition index.
    index: usize,

    /// The partition.
    partition: &'a ApmPartition,
}

impl<'a> ApmPartitionInfo<'a> {
    /// Creates new partition information.
    fn new(index: usize, partition: &'a ApmPartition) -> Self {
        Self { index, partition }
    }
}

impl<'a> fmt::Display for ApmPartitionInfo<'a> {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.index + 1)?;

        writeln!(
            formatter,
            "    Type identifier\t\t\t\t: {}",
            self.partition.get_type_identifier()
        )?;
        let name: &ByteString = self.partition.get_name();

        if !name.is_empty() {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", name,)?;
        }
        let partition_offset: u64 = self.partition.get_partition_offset();

        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            partition_offset, partition_offset,
        )?;
        let byte_size: ByteSize = ByteSize::new(self.partition.get_partition_size(), 1024);

        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        let status_flags: u32 = self.partition.get_status_flags();

        writeln!(
            formatter,
            "    Status flags\t\t\t\t: 0x{:08x}",
            status_flags,
        )?;
        let flags_info: ApmPartitionStatusFlagsInfo =
            ApmPartitionStatusFlagsInfo::new(status_flags);

        flags_info.fmt(formatter)?;

        writeln!(formatter)
    }
}

/// Apple Partition Map (APM) partition status flags information.
struct ApmPartitionStatusFlagsInfo {
    /// Flags.
    flags: u32,
}

impl ApmPartitionStatusFlagsInfo {
    /// Creates new partition status flags information.
    fn new(flags: u32) -> Self {
        Self { flags }
    }
}

impl fmt::Display for ApmPartitionStatusFlagsInfo {
    /// Formats partition status flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x00000001 != 0 {
            writeln!(formatter, "        0x00000001: Is valid")?;
        }
        if self.flags & 0x00000002 != 0 {
            writeln!(formatter, "        0x00000002: Is allocated")?;
        }
        if self.flags & 0x00000004 != 0 {
            writeln!(formatter, "        0x00000004: Is in use")?;
        }
        if self.flags & 0x00000008 != 0 {
            writeln!(formatter, "        0x00000008: Contains boot information")?;
        }
        if self.flags & 0x00000010 != 0 {
            writeln!(formatter, "        0x00000010: Is readable")?;
        }
        if self.flags & 0x00000020 != 0 {
            writeln!(formatter, "        0x00000020: Is writeable")?;
        }
        if self.flags & 0x00000040 != 0 {
            writeln!(
                formatter,
                "        0x00000040: Boot code is position independent"
            )?;
        }

        if self.flags & 0x00000100 != 0 {
            writeln!(
                formatter,
                "        0x00000100: Contains a chain-compatible driver"
            )?;
        }
        if self.flags & 0x00000200 != 0 {
            writeln!(formatter, "        0x00000200: Contains a real driver")?;
        }
        if self.flags & 0x00000400 != 0 {
            writeln!(formatter, "        0x00000400: Contains a chain driver")?;
        }

        if self.flags & 0x40000000 != 0 {
            writeln!(formatter, "        0x40000000: Automatic mount at startup")?;
        }
        if self.flags & 0x80000000 != 0 {
            writeln!(formatter, "        0x80000000: Is startup partition")?;
        }
        Ok(())
    }
}

/// Apple Partition Map (APM) volume system information.
struct ApmVolumeSystemInfo {
    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Number of partitions.
    pub number_of_partitions: usize,
}

impl ApmVolumeSystemInfo {
    /// Creates new volume system information.
    fn new() -> Self {
        Self {
            bytes_per_sector: 0,
            number_of_partitions: 0,
        }
    }
}

impl fmt::Display for ApmVolumeSystemInfo {
    /// Formats volume system information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Apple Partition Map (APM) information:")?;

        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.bytes_per_sector
        )?;
        writeln!(
            formatter,
            "    Number of partitions\t\t\t: {}",
            self.number_of_partitions
        )?;
        writeln!(formatter)
    }
}

/// Information about an Apple Partition Map (APM).
pub struct ApmInfo {}

impl ApmInfo {
    /// Retrieves the volume system information.
    fn get_volume_system_information(apm_volume_system: &ApmVolumeSystem) -> ApmVolumeSystemInfo {
        let mut volume_system_information: ApmVolumeSystemInfo = ApmVolumeSystemInfo::new();

        volume_system_information.bytes_per_sector = apm_volume_system.get_bytes_per_sector();
        volume_system_information.number_of_partitions =
            apm_volume_system.get_number_of_partitions();

        volume_system_information
    }

    /// Opens a volume system.
    pub fn open_volume_system(
        data_stream: &DataStreamReference,
    ) -> Result<ApmVolumeSystem, ErrorTrace> {
        let mut apm_volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        match apm_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open APM volume system");
                return Err(error);
            }
        }
        Ok(apm_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let apm_volume_system: ApmVolumeSystem = match Self::open_volume_system(data_stream) {
            Ok(apm_volume_system) => apm_volume_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                return Err(error);
            }
        };
        let volume_system_info: ApmVolumeSystemInfo =
            Self::get_volume_system_information(&apm_volume_system);

        print!("{}", volume_system_info);

        for (partition_index, result) in apm_volume_system.partitions().enumerate() {
            let apm_partition: ApmPartition = match result {
                Ok(apm_partition) => apm_partition,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve partition: {}", partition_index)
                    );
                    return Err(error);
                }
            };
            let partition_info: ApmPartitionInfo =
                ApmPartitionInfo::new(partition_index, &apm_partition);

            print!("{}", partition_info);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::assert_lines_eq;

    #[test]
    fn test_partition_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apm/apm.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apm_volume_system: ApmVolumeSystem = ApmInfo::open_volume_system(&data_stream)?;

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let test_struct: ApmPartitionInfo = ApmPartitionInfo::new(0, &apm_partition);

        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Type identifier\t\t\t\t: Apple_HFS\n",
            "    Name\t\t\t\t\t: disk image\n",
            "    Offset\t\t\t\t\t: 32768 (0x00008000)\n",
            "    Size\t\t\t\t\t: 4.0 MiB (4153344 bytes)\n",
            "    Status flags\t\t\t\t: 0x40000033\n",
            "        0x00000001: Is valid\n",
            "        0x00000002: Is allocated\n",
            "        0x00000010: Is readable\n",
            "        0x00000020: Is writeable\n",
            "        0x40000000: Automatic mount at startup\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_partition_status_flags_fmt() -> Result<(), ErrorTrace> {
        let test_struct: ApmPartitionStatusFlagsInfo = ApmPartitionStatusFlagsInfo::new(0xc000077f);

        let expected_string: &str = concat!(
            "        0x00000001: Is valid\n",
            "        0x00000002: Is allocated\n",
            "        0x00000004: Is in use\n",
            "        0x00000008: Contains boot information\n",
            "        0x00000010: Is readable\n",
            "        0x00000020: Is writeable\n",
            "        0x00000040: Boot code is position independent\n",
            "        0x00000100: Contains a chain-compatible driver\n",
            "        0x00000200: Contains a real driver\n",
            "        0x00000400: Contains a chain driver\n",
            "        0x40000000: Automatic mount at startup\n",
            "        0x80000000: Is startup partition\n",
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_get_volume_system_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apm/apm.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apm_volume_system: ApmVolumeSystem = ApmInfo::open_volume_system(&data_stream)?;
        let test_struct: ApmVolumeSystemInfo =
            ApmInfo::get_volume_system_information(&apm_volume_system);

        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.number_of_partitions, 2);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
