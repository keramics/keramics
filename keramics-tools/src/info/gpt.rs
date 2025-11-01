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
use keramics_formats::gpt::{GptPartition, GptVolumeSystem};

use crate::formatters::format_as_bytesize;

/// Information about a GUID Partition Table (GPT).
pub struct GptInfo {}

impl GptInfo {
    /// Prints information about a volume system.
    pub fn print_volume_system(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut gpt_volume_system = GptVolumeSystem::new();

        match gpt_volume_system.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open GPT volume system");
                return Err(error);
            }
        };
        println!("GUID Partition Table (GPT) information:");
        println!(
            "    Disk identifier\t\t\t: {}",
            gpt_volume_system.disk_identifier.to_string()
        );
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            gpt_volume_system.bytes_per_sector
        );
        let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();
        println!("    Number of partitions\t\t: {}", number_of_partitions);

        for partition_index in 0..number_of_partitions {
            println!("");

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
            println!("Partition: {}", partition_index + 1);
            println!(
                "    Identifier\t\t\t\t: {}",
                gpt_partition.identifier.to_string()
            );
            println!(
                "    Type identifier\t\t\t: {}",
                gpt_partition.type_identifier.to_string()
            );
            println!(
                "    Offset\t\t\t\t: {} (0x{:08x})",
                gpt_partition.offset, gpt_partition.offset
            );
            if gpt_partition.size < 1024 {
                println!("    Size\t\t\t\t: {} bytes", gpt_partition.size);
            } else {
                let size_string: String = format_as_bytesize(gpt_partition.size, 1024);
                println!(
                    "    Size\t\t\t\t: {} ({} bytes)",
                    size_string, gpt_partition.size
                );
            }
        }
        println!("");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for print_volume_system
}
