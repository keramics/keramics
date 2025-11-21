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
use keramics_formats::gpt::{GptPartition, GptVolumeSystem};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

/// GUID Partition Table (GPT) parition information.
struct GptPartitionInfo {
    /// The partition index.
    pub index: usize,

    /// The partition identifier.
    pub identifier: Uuid,

    /// The partition type identifier.
    pub type_identifier: Uuid,

    /// The offset of the partition relative to start of the volume system.
    pub offset: u64,

    /// The size of the partition.
    pub size: u64,
}

impl GptPartitionInfo {
    /// Creates new partition information.
    fn new() -> Self {
        Self {
            index: 0,
            identifier: Uuid::new(),
            type_identifier: Uuid::new(),
            offset: 0,
            size: 0,
        }
    }
}

impl fmt::Display for GptPartitionInfo {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.index + 1)?;

        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;
        writeln!(
            formatter,
            "    Type identifier\t\t\t\t: {}",
            self.type_identifier
        )?;
        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            self.offset, self.offset
        )?;
        let byte_size: ByteSize = ByteSize::new(self.size, 1024);

        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)
    }
}

/// Information about a GUID Partition Table (GPT).
pub struct GptInfo {}

impl GptInfo {
    /// Retrieves the partition information.
    fn get_partition_information(
        partition_index: usize,
        gpt_partition: &GptPartition,
    ) -> GptPartitionInfo {
        let mut partition_information: GptPartitionInfo = GptPartitionInfo::new();

        partition_information.index = partition_index;
        partition_information.identifier = gpt_partition.identifier.clone();
        partition_information.type_identifier = gpt_partition.type_identifier.clone();
        partition_information.offset = gpt_partition.offset;
        partition_information.size = gpt_partition.size;

        partition_information
    }

    /// Opens a volume system.
    pub fn open_volume_system(
        data_stream: &DataStreamReference,
    ) -> Result<GptVolumeSystem, ErrorTrace> {
        let mut gpt_volume_system: GptVolumeSystem = GptVolumeSystem::new();

        match gpt_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open GPT volume system");
                return Err(error);
            }
        };
        Ok(gpt_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let gpt_volume_system: GptVolumeSystem = match Self::open_volume_system(data_stream) {
            Ok(gpt_volume_system) => gpt_volume_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                return Err(error);
            }
        };
        println!("GUID Partition Table (GPT) information:");

        println!(
            "    Disk identifier\t\t\t\t: {}",
            gpt_volume_system.disk_identifier
        );
        println!(
            "    Bytes per sector\t\t\t\t: {} bytes",
            gpt_volume_system.bytes_per_sector
        );
        let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t\t: {}", number_of_partitions);

        println!("");

        for partition_index in 0..number_of_partitions {
            let gpt_partition: GptPartition =
                match gpt_volume_system.get_partition_by_index(partition_index) {
                    Ok(partition) => partition,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve GPT partition: {}", partition_index)
                        );
                        return Err(error);
                    }
                };
            let partition_info: GptPartitionInfo =
                Self::get_partition_information(partition_index, &gpt_partition);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/gpt/gpt.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let gpt_volume_system: GptVolumeSystem = GptInfo::open_volume_system(&data_stream)?;

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let test_struct: GptPartitionInfo = GptInfo::get_partition_information(0, &gpt_partition);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Identifier\t\t\t\t\t: 0b119671-75ff-4e2a-a31a-0bc83f857fdd\n",
            "    Type identifier\t\t\t\t: 0fc63daf-8483-4772-8e79-3d69d8477de4\n",
            "    Offset\t\t\t\t\t: 1048576 (0x00100000)\n",
            "    Size\t\t\t\t\t: 1.0 MiB (1048576 bytes)\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_partition_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/gpt/gpt.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let gpt_volume_system: GptVolumeSystem = GptInfo::open_volume_system(&data_stream)?;

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let test_struct: GptPartitionInfo = GptInfo::get_partition_information(0, &gpt_partition);

        assert_eq!(test_struct.index, 0);
        assert_eq!(
            test_struct.identifier,
            Uuid::from_string("0b119671-75ff-4e2a-a31a-0bc83f857fdd")?
        );
        assert_eq!(
            test_struct.type_identifier,
            Uuid::from_string("0fc63daf-8483-4772-8e79-3d69d8477de4")?
        );
        assert_eq!(test_struct.offset, 1048576);
        assert_eq!(test_struct.size, 1048576);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
