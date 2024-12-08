/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::process::ExitCode;

use crate::formatters;

use keramics::formats::gpt::{GptPartition, GptVolumeSystem};
use keramics::vfs::{VfsFileSystem, VfsFileSystemReference, VfsPath};

/// Prints information about a GPT volume system.
pub fn print_gpt_volume_system(
    parent_file_system: &VfsFileSystemReference,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut gpt_volume_system = GptVolumeSystem::new();

    match gpt_volume_system.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open GPT volume system with error: {}", error);
            return ExitCode::FAILURE;
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
                Err(error) => {
                    println!(
                        "Unable to retrieve GPT partition: {} with error: {}",
                        partition_index, error
                    );
                    return ExitCode::FAILURE;
                }
            };
        let size_string: String = formatters::format_as_bytesize(gpt_partition.size, 1024);

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
        println!(
            "    Size\t\t\t\t: {} ({} bytes)",
            size_string, gpt_partition.size
        );
    }
    println!("");

    ExitCode::SUCCESS
}
