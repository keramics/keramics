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
use keramics_formats::mbr::{MbrPartition, MbrVolumeSystem};

use crate::formatters::ByteSize;

use super::constants::*;

/// Master Boot Record (MBR) partition information.
struct MbrPartitionInfo<'a> {
    /// The partition.
    partition: &'a MbrPartition,
}

impl<'a> MbrPartitionInfo<'a> {
    /// Creates new partition information.
    fn new(partition: &'a MbrPartition) -> Self {
        Self { partition }
    }

    /// Retrieves the partition type as a string.
    pub fn get_partition_type_string(&self, partition_type: &u8) -> Option<&str> {
        MBR_PARTITION_TYPES
            .binary_search_by(|(key, _)| key.cmp(partition_type))
            .map_or_else(|_| None, |index| Some(MBR_PARTITION_TYPES[index].1))
    }
}

impl<'a> fmt::Display for MbrPartitionInfo<'a> {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Partition: {}",
            self.partition.get_partition_index() + 1
        )?;

        let partition_type: u8 = self.partition.get_partition_type();
        match self.get_partition_type_string(&partition_type) {
            Some(partition_type_string) => {
                writeln!(
                    formatter,
                    "    Type\t\t\t\t\t: 0x{:02x} ({})",
                    partition_type, partition_type_string
                )?;
            }
            None => {
                writeln!(formatter, "    Type\t\t\t\t\t: 0x{:02x}", partition_type)?;
            }
        };
        let partition_offset: u64 = self.partition.get_partition_offset();
        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            partition_offset, partition_offset
        )?;
        let byte_size: ByteSize = ByteSize::new(self.partition.get_partition_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Flags\t\t\t\t\t: 0x{:02x}",
            self.partition.get_flags()
        )?;
        writeln!(formatter)
    }
}

/// Information about a Master Boot Record (MBR).
pub struct MbrInfo {}

impl MbrInfo {
    /// Opens a volume system.
    pub fn open_volume_system(
        data_stream: &DataStreamReference,
    ) -> Result<MbrVolumeSystem, ErrorTrace> {
        let mut mbr_volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

        match mbr_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open MBR volume system");
                return Err(error);
            }
        }
        Ok(mbr_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut mbr_volume_system: MbrVolumeSystem = match Self::open_volume_system(data_stream) {
            Ok(mbr_volume_system) => mbr_volume_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                return Err(error);
            }
        };
        if mbr_volume_system.get_bytes_per_sector() == 0 {
            // TODO: allow to specify bytes per sector as an argument
            match mbr_volume_system.set_bytes_per_sector(512) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to set bytes per sector to 512"
                    );
                    return Err(error);
                }
            }
        }
        println!("Master Boot Record (MBR) information:");

        println!(
            "    Disk identity\t\t\t\t: 0x{:x}",
            mbr_volume_system.get_disk_identity()
        );
        println!(
            "    Bytes per sector\t\t\t\t: {}",
            mbr_volume_system.get_bytes_per_sector()
        );
        let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t\t: {}", number_of_partitions);

        println!();

        for (partition_index, result) in mbr_volume_system.partitions().enumerate() {
            let mbr_partition: MbrPartition = match result {
                Ok(mbr_partition) => mbr_partition,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve partition: {}", partition_index)
                    );
                    return Err(error);
                }
            };
            let partition_info: MbrPartitionInfo = MbrPartitionInfo::new(&mbr_partition);

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
    use keramics_formats::PartitionIterator;

    use crate::assert_lines_eq;

    #[test]
    fn test_partition_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/mbr/mbr.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mbr_volume_system: MbrVolumeSystem = MbrInfo::open_volume_system(&data_stream)?;

        let mbr_partition: MbrPartition = mbr_volume_system.get_partition_by_index(0)?;
        let test_struct: MbrPartitionInfo = MbrPartitionInfo::new(&mbr_partition);

        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Type\t\t\t\t\t: 0x83 (Linux)\n",
            "    Offset\t\t\t\t\t: 512 (0x00000200)\n",
            "    Size\t\t\t\t\t: 1.0 MiB (1049088 bytes)\n",
            "    Flags\t\t\t\t\t: 0x00\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
