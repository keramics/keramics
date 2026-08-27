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
use keramics_formats::bsdlabel::{BsdDiskLabelPartition, BsdDiskLabelVolumeSystem};

use crate::formatters::ByteSize;

/// BSD disklabel (bsdlabel) partition information.
struct BsdDiskLabelPartitionInfo<'a> {
    /// The partition index.
    index: usize,

    /// The partition.
    partition: &'a BsdDiskLabelPartition,
}

impl<'a> BsdDiskLabelPartitionInfo<'a> {
    /// Creates new partition information.
    fn new(index: usize, partition: &'a BsdDiskLabelPartition) -> Self {
        Self { index, partition }
    }
}

impl<'a> fmt::Display for BsdDiskLabelPartitionInfo<'a> {
    /// Formats partition information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Partition: {}", self.index + 1)?;

        let partition_index: u16 = self.partition.get_partition_index();
        writeln!(
            formatter,
            "    Label\t\t\t\t\t: {}",
            (b'a' + (partition_index as u8)) as char
        )?;
        let partition_offset: u64 = self.partition.get_partition_offset();
        writeln!(
            formatter,
            "    Offset\t\t\t\t\t: {} (0x{:08x})",
            partition_offset, partition_offset,
        )?;
        let byte_size: ByteSize = ByteSize::new(self.partition.get_partition_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)
    }
}

/// BSD disklabel (bsdlabel) volume system information.
struct BsdDiskLabelVolumeSystemInfo<'a> {
    /// Volume system.
    volume_system: &'a BsdDiskLabelVolumeSystem,
}

impl<'a> BsdDiskLabelVolumeSystemInfo<'a> {
    /// Creates new volume system information.
    fn new(volume_system: &'a BsdDiskLabelVolumeSystem) -> Self {
        Self { volume_system }
    }
}

impl<'a> fmt::Display for BsdDiskLabelVolumeSystemInfo<'a> {
    /// Formats volume system information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "BSD disklabel (bsdlabel) information:")?;

        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.volume_system.get_bytes_per_sector(),
        )?;
        writeln!(
            formatter,
            "    Number of partitions\t\t\t: {}",
            self.volume_system.get_number_of_partitions()
        )?;
        writeln!(formatter)
    }
}

/// Information about a BSD disklabel (bsdlabel).
pub struct BsdDiskLabelInfo {}

impl BsdDiskLabelInfo {
    /// Opens a volume system.
    pub fn open_volume_system(
        data_stream: &DataStreamReference,
    ) -> Result<BsdDiskLabelVolumeSystem, ErrorTrace> {
        let mut bsdlabel_volume_system: BsdDiskLabelVolumeSystem = BsdDiskLabelVolumeSystem::new();

        match bsdlabel_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open bsdlabel volume system"
                );
                return Err(error);
            }
        }
        Ok(bsdlabel_volume_system)
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let bsdlabel_volume_system: BsdDiskLabelVolumeSystem =
            match Self::open_volume_system(data_stream) {
                Ok(bsdlabel_volume_system) => bsdlabel_volume_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open volume system");
                    return Err(error);
                }
            };
        let volume_system_info: BsdDiskLabelVolumeSystemInfo =
            BsdDiskLabelVolumeSystemInfo::new(&bsdlabel_volume_system);

        print!("{}", volume_system_info);

        for (partition_index, result) in bsdlabel_volume_system.partitions().enumerate() {
            let bsdlabel_partition: BsdDiskLabelPartition = match result {
                Ok(bsdlabel_partition) => bsdlabel_partition,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve partition: {}", partition_index)
                    );
                    return Err(error);
                }
            };
            let partition_info: BsdDiskLabelPartitionInfo =
                BsdDiskLabelPartitionInfo::new(partition_index, &bsdlabel_partition);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/bsdlabel/bsdlabel.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let bsdlabel_volume_system: BsdDiskLabelVolumeSystem =
            BsdDiskLabelInfo::open_volume_system(&data_stream)?;

        let bsdlabel_partition: BsdDiskLabelPartition =
            bsdlabel_volume_system.get_partition_by_index(0)?;
        let test_struct: BsdDiskLabelPartitionInfo =
            BsdDiskLabelPartitionInfo::new(0, &bsdlabel_partition);

        let expected_string: &str = concat!(
            "Partition: 1\n",
            "    Label\t\t\t\t\t: a\n",
            "    Offset\t\t\t\t\t: 8192 (0x00002000)\n",
            "    Size\t\t\t\t\t: 4.0 MiB (4186112 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_volume_system_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/bsdlabel/bsdlabel.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let bsdlabel_volume_system: BsdDiskLabelVolumeSystem =
            BsdDiskLabelInfo::open_volume_system(&data_stream)?;

        let test_struct: BsdDiskLabelVolumeSystemInfo =
            BsdDiskLabelVolumeSystemInfo::new(&bsdlabel_volume_system);

        let expected_string: &str = concat!(
            "BSD disklabel (bsdlabel) information:\n",
            "    Bytes per sector\t\t\t\t: 512\n",
            "    Number of partitions\t\t\t: 1\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_volume_system
    // TODO: add tests for print_volume_system
}
