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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::apm::{ApmPartition, ApmVolumeSystem};

use crate::formatters::format_as_bytesize;

/// Information about an Apple Partition Map (APM).
pub struct ApmInfo {}

impl ApmInfo {
    /// Prints the partition status flags.
    fn print_partition_status_flags(flags: u32) {
        if flags & 0x00000001 != 0 {
            println!("        0x00000001: Is valid");
        }
        if flags & 0x00000002 != 0 {
            println!("        0x00000002: Is allocated");
        }
        if flags & 0x00000004 != 0 {
            println!("        0x00000004: Is in use");
        }
        if flags & 0x00000008 != 0 {
            println!("        0x00000008: Contains boot information");
        }
        if flags & 0x00000010 != 0 {
            println!("        0x00000010: Is readable");
        }
        if flags & 0x00000020 != 0 {
            println!("        0x00000020: Is writeable");
        }
        if flags & 0x00000040 != 0 {
            println!("        0x00000040: Boot code is position independent");
        }

        if flags & 0x00000100 != 0 {
            println!("        0x00000100: Contains a chain-compatible driver");
        }
        if flags & 0x00000200 != 0 {
            println!("        0x00000200: Contains a real driver");
        }
        if flags & 0x00000400 != 0 {
            println!("        0x00000400: Contains a chain driver");
        }

        if flags & 0x40000000 != 0 {
            println!("        0x40000000: Automatic mount at startup");
        }
        if flags & 0x80000000 != 0 {
            println!("        0x80000000: Is startup partition");
        }
    }

    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut apm_volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

        match apm_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open APM volume system");
                return Err(error);
            }
        };
        println!("Apple Partition Map (APM) information:");
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            apm_volume_system.bytes_per_sector
        );
        let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t: {}", number_of_partitions);

        for partition_index in 0..number_of_partitions {
            println!("");

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
            println!("Partition: {}", partition_index + 1);
            println!(
                "    Type identifier\t\t\t: {}",
                apm_partition.type_identifier.to_string()
            );
            if !apm_partition.name.is_empty() {
                println!("    Name\t\t\t\t: {}", apm_partition.name.to_string());
            }
            println!(
                "    Offset\t\t\t\t: {} (0x{:08x})",
                apm_partition.offset, apm_partition.offset
            );
            if apm_partition.size < 1024 {
                println!("    Size\t\t\t\t: {}", apm_partition.size);
            } else {
                let size_string: String = format_as_bytesize(apm_partition.size, 1024);
                println!(
                    "    Size\t\t\t\t: {} ({} bytes)",
                    size_string, apm_partition.size
                );
            }
            println!(
                "    Status flags\t\t\t: 0x{:08x}",
                apm_partition.status_flags
            );
            Self::print_partition_status_flags(apm_partition.status_flags);
        }
        println!("");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for print_partition_status_flags
    // TODO: add tests for print_volume_system
}
