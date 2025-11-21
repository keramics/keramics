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
use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};
use keramics_types::ByteString;

use crate::formatters::ByteSize;

/// Apple Partition Map (APM) parition information.
struct ApmPartitionInfo {
    /// The partition index.
    pub index: usize,

    /// The partition type identifier.
    pub type_identifier: ByteString,

    /// The name.
    pub name: ByteString,

    /// The offset of the partition relative to start of the volume system.
    pub offset: u64,

    /// The size of the partition.
    pub size: u64,

    /// The status flags.
    pub status_flags: u32,
}

impl ApmPartitionInfo {
    /// Creates new partition information.
    fn new() -> Self {
        Self {
            index: 0,
            type_identifier: ByteString::new(),
            name: ByteString::new(),
            offset: 0,
            size: 0,
            status_flags: 0,
        }
    }
}

impl fmt::Display for ApmPartitionInfo {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.index + 1)?;

        writeln!(
            formatter,
            "    Type identifier\t\t\t\t: {}",
            self.type_identifier
        )?;
        if !self.name.is_empty() {
            writeln!(formatter, "    Name\t\t\t\t\t: {}", self.name)?;
        }
        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            self.offset, self.offset
        )?;
        let byte_size: ByteSize = ByteSize::new(self.size, 1024);

        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        let flags_info: ApmPartitionStatusFlagsInfo =
            ApmPartitionStatusFlagsInfo::new(self.status_flags);

        writeln!(
            formatter,
            "    Status flags\t\t\t\t: 0x{:08x}",
            self.status_flags
        )?;
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

/// Information about an Apple Partition Map (APM).
pub struct ApmInfo {}

impl ApmInfo {
    /// Retrieves the partition information.
    fn get_partition_information(
        partition_index: usize,
        apm_partition: &ApmPartition,
    ) -> ApmPartitionInfo {
        let mut partition_information: ApmPartitionInfo = ApmPartitionInfo::new();

        partition_information.index = partition_index;
        partition_information.type_identifier = apm_partition.type_identifier.clone();
        partition_information.name = apm_partition.name.clone();
        partition_information.offset = apm_partition.offset;
        partition_information.size = apm_partition.size;
        partition_information.status_flags = apm_partition.status_flags;

        partition_information
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
        };
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
        println!("Apple Partition Map (APM) information:");

        println!(
            "    Bytes per sector\t\t\t\t: {} bytes",
            apm_volume_system.bytes_per_sector
        );
        let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t\t: {}", number_of_partitions);

        println!("");

        for partition_index in 0..number_of_partitions {
            let apm_partition: ApmPartition =
                match apm_volume_system.get_partition_by_index(partition_index) {
                    Ok(partition) => partition,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve APM partition: {}", partition_index)
                        );
                        return Err(error);
                    }
                };
            let partition_info: ApmPartitionInfo =
                Self::get_partition_information(partition_index, &apm_partition);

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

    #[test]
    fn test_partition_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apm/apm.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apm_volume_system: ApmVolumeSystem = ApmInfo::open_volume_system(&data_stream)?;

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let test_struct: ApmPartitionInfo = ApmInfo::get_partition_information(0, &apm_partition);

        let string: String = test_struct.to_string();
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
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_partition_status_flags_fmt() -> Result<(), ErrorTrace> {
        let test_struct: ApmPartitionStatusFlagsInfo = ApmPartitionStatusFlagsInfo::new(0x40000033);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "        0x00000001: Is valid\n",
            "        0x00000002: Is allocated\n",
            "        0x00000010: Is readable\n",
            "        0x00000020: Is writeable\n",
            "        0x40000000: Automatic mount at startup\n",
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_partition_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/apm/apm.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let apm_volume_system: ApmVolumeSystem = ApmInfo::open_volume_system(&data_stream)?;

        let apm_partition: ApmPartition = apm_volume_system.get_partition_by_index(0)?;
        let test_struct: ApmPartitionInfo = ApmInfo::get_partition_information(0, &apm_partition);

        assert_eq!(test_struct.index, 0);
        assert_eq!(test_struct.type_identifier, ByteString::from("Apple_HFS"));
        assert_eq!(test_struct.name, ByteString::from("disk image"));
        assert_eq!(test_struct.offset, 32768);
        assert_eq!(test_struct.size, 4153344);
        assert_eq!(test_struct.status_flags, 0x40000033);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
