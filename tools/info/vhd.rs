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

use std::collections::HashMap;

use std::process::ExitCode;

use crate::formatters;

use keramics::formats::vhd::{VhdDiskType, VhdFile};
use keramics::vfs::{VfsFileSystemReference, VfsPath};

/// Prints information about a VHD file.
pub fn print_vhd_file(file_system: &VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut vhd_file: VhdFile = VhdFile::new();

    match vhd_file.open(file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open VHD file with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let disk_types = HashMap::<VhdDiskType, &'static str>::from([
        (VhdDiskType::Differential, "Differential"),
        (VhdDiskType::Dynamic, "Dynamic"),
        (VhdDiskType::Fixed, "Fixed"),
        (VhdDiskType::Unknown, "Unknown"),
    ]);
    let disk_type_string: String = disk_types.get(&vhd_file.disk_type).unwrap().to_string();
    let media_size_string: String = formatters::format_as_bytesize(vhd_file.media_size, 1024);

    println!("Virtual Hard Disk (VHD) information:");
    println!("    Format version\t\t\t: 1.0");
    println!("    Disk type\t\t\t\t: {}", disk_type_string);
    println!(
        "    Media size\t\t\t\t: {} ({} bytes)",
        media_size_string, vhd_file.media_size
    );
    println!(
        "    Bytes per sector\t\t\t: {} bytes",
        vhd_file.bytes_per_sector
    );
    println!(
        "    Identifier\t\t\t\t: {}",
        vhd_file.identifier.to_string()
    );

    match &vhd_file.parent_identifier {
        Some(parent_identifier) => {
            println!(
                "    Parent identifier\t\t\t: {}",
                parent_identifier.to_string()
            );
        }
        None => {}
    }
    match &vhd_file.parent_name {
        Some(parent_name) => {
            println!("    Parent name\t\t\t\t: {}", parent_name.to_string());
        }
        None => {}
    }
    println!("");

    ExitCode::SUCCESS
}
