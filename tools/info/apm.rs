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

use keramics::formats::apm::{ApmPartition, ApmVolumeSystem};
use keramics::vfs::{VfsFileSystem, VfsFileSystemReference, VfsPath};

/// Prints information about an APM volume system.
pub fn print_apm_volume_system(
    parent_file_system: &VfsFileSystemReference,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut apm_volume_system = ApmVolumeSystem::new();

    match apm_volume_system.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open APM volume system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    println!("Apple Partition Map (APM) information:");
    println!(
        "    Bytes per sector\t\t: {} bytes",
        apm_volume_system.bytes_per_sector
    );
    let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();
    println!("    Number of partitions\t: {}", number_of_partitions);

    for partition_index in 0..number_of_partitions {
        println!("");

        let apm_partition: ApmPartition =
            match apm_volume_system.get_partition_by_index(partition_index) {
                Ok(partition) => partition,
                Err(error) => {
                    println!(
                        "Unable to retrieve APM partition: {} with error: {}",
                        partition_index, error
                    );
                    return ExitCode::FAILURE;
                }
            };
        let size_string: String = formatters::format_as_bytesize(apm_partition.size, 1024);

        println!("Partition: {}", partition_index + 1);
        println!(
            "    Type identifier\t\t: {}",
            apm_partition.type_identifier.to_string()
        );
        if apm_partition.name.elements.len() > 0 {
            println!("    Name\t\t\t: {}", apm_partition.name.to_string());
        }
        println!(
            "    Offset\t\t\t: {} (0x{:08x})",
            apm_partition.offset, apm_partition.offset
        );
        println!(
            "    Size\t\t\t: {} ({} bytes)",
            size_string, apm_partition.size
        );
        println!("    Status flags\t\t: 0x{:08x}", apm_partition.status_flags);
        if apm_partition.status_flags & 0x00000001 != 0 {
            println!("        Is valid");
        }
        if apm_partition.status_flags & 0x00000002 != 0 {
            println!("        Is allocated");
        }
        if apm_partition.status_flags & 0x00000004 != 0 {
            println!("        Is in use");
        }
        if apm_partition.status_flags & 0x00000008 != 0 {
            println!("        Contains boot information");
        }
        if apm_partition.status_flags & 0x00000010 != 0 {
            println!("        Is readable");
        }
        if apm_partition.status_flags & 0x00000020 != 0 {
            println!("        Is writeable");
        }
        if apm_partition.status_flags & 0x00000040 != 0 {
            println!("        Boot code is position independent");
        }

        if apm_partition.status_flags & 0x00000100 != 0 {
            println!("        Contains a chain-compatible driver");
        }
        if apm_partition.status_flags & 0x00000200 != 0 {
            println!("        Contains a real driver");
        }
        if apm_partition.status_flags & 0x00000400 != 0 {
            println!("        Contains a chain driver");
        }

        if apm_partition.status_flags & 0x40000000 != 0 {
            println!("        Automatic mount at startup");
        }
        if apm_partition.status_flags & 0x80000000 != 0 {
            println!("        Is startup partition");
        }
    }
    println!("");

    ExitCode::SUCCESS
}
