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
use keramics_formats::gpt::{GptPartition, GptVolumeSystem};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

use super::constants::*;

/// GUID Partition Table (GPT) partition information.
struct GptPartitionInfo<'a> {
    /// The index of the corresponding partition table entry.
    partition_index: usize,

    /// Partition.
    partition: &'a GptPartition,
}

impl<'a> GptPartitionInfo<'a> {
    /// Creates new partition information.
    fn new(partition_index: usize, partition: &'a GptPartition) -> Self {
        Self {
            partition_index,
            partition,
        }
    }

    /// Retrieves the type identifier as a string.
    pub fn get_type_identifier_string(&self, type_identifier: &Uuid) -> Option<&str> {
        let lookup_key: String = type_identifier.to_string();
        GTP_TYPE_IDENTIFIERS
            .binary_search_by(|(key, _)| key.cmp(&lookup_key.as_str()))
            .map_or_else(|_| None, |index| Some(GTP_TYPE_IDENTIFIERS[index].1))
    }
}

impl<'a> fmt::Display for GptPartitionInfo<'a> {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.partition_index + 1)?;

        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.partition.get_identifier()
        )?;

        let type_identifier: &Uuid = self.partition.get_type_identifier();
        match self.get_type_identifier_string(&type_identifier) {
            Some(type_identifier_string) => {
                writeln!(
                    formatter,
                    "    Type\t\t\t\t\t: {} ({})",
                    type_identifier, type_identifier_string
                )?;
            }
            None => {
                writeln!(formatter, "    Type\t\t\t\t\t: {}", type_identifier)?;
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

        writeln!(formatter)
    }
}

/// Information about a GUID Partition Table (GPT).
pub struct GptInfo {}

impl GptInfo {
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
        }
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
            gpt_volume_system.get_disk_identifier()
        );
        println!(
            "    Bytes per sector\t\t\t\t: {}",
            gpt_volume_system.get_bytes_per_sector()
        );
        let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t\t: {}", number_of_partitions);

        println!();

        for (partition_index, result) in gpt_volume_system.partitions().enumerate() {
            let gpt_partition: GptPartition = match result {
                Ok(gpt_partition) => gpt_partition,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve partition: {}", partition_index)
                    );
                    return Err(error);
                }
            };
            let partition_info: GptPartitionInfo =
                GptPartitionInfo::new(partition_index, &gpt_partition);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/gpt/gpt.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let gpt_volume_system: GptVolumeSystem = GptInfo::open_volume_system(&data_stream)?;

        let gpt_partition: GptPartition = gpt_volume_system.get_partition_by_index(0)?;
        let test_struct: GptPartitionInfo = GptPartitionInfo::new(0, &gpt_partition);

        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Identifier\t\t\t\t\t: 0b119671-75ff-4e2a-a31a-0bc83f857fdd\n",
            "    Type\t\t\t\t\t: 0fc63daf-8483-4772-8e79-3d69d8477de4 (Linux filesystem data)\n",
            "    Offset\t\t\t\t\t: 1048576 (0x00100000)\n",
            "    Size\t\t\t\t\t: 1.0 MiB (1048576 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
