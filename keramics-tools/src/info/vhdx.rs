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

use std::collections::HashMap;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::vhdx::{VhdxDiskType, VhdxFile};

use crate::formatters::format_as_bytesize;

/// Information about a Virtual Hard Disk (VHDX) file.
pub struct VhdxInfo {}

impl VhdxInfo {
    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut vhdx_file: VhdxFile = VhdxFile::new();

        match vhdx_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHDX file");
                return Err(error);
            }
        };
        let disk_types = HashMap::<VhdxDiskType, &'static str>::from([
            (VhdxDiskType::Differential, "Differential"),
            (VhdxDiskType::Dynamic, "Dynamic"),
            (VhdxDiskType::Fixed, "Fixed"),
            (VhdxDiskType::Unknown, "Unknown"),
        ]);
        let disk_type_string: &str = disk_types.get(&vhdx_file.disk_type).unwrap();

        println!("Virtual Hard Disk (VHDX) information:");
        println!("    Format version\t\t\t: 2.{}", vhdx_file.format_version);
        println!("    Disk type\t\t\t\t: {}", disk_type_string);

        if vhdx_file.media_size < 1024 {
            println!("    Media size\t\t\t\t: {} bytes", vhdx_file.media_size);
        } else {
            let media_size_string: String = format_as_bytesize(vhdx_file.media_size, 1024);
            println!(
                "    Media size\t\t\t\t: {} ({} bytes)",
                media_size_string, vhdx_file.media_size
            );
        }
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            vhdx_file.bytes_per_sector
        );
        println!(
            "    Identifier\t\t\t\t: {}",
            vhdx_file.identifier.to_string()
        );

        match &vhdx_file.parent_identifier {
            Some(parent_identifier) => {
                println!(
                    "    Parent identifier\t\t\t: {}",
                    parent_identifier.to_string()
                );
            }
            None => {}
        }
        match &vhdx_file.parent_name {
            Some(parent_name) => {
                println!("    Parent name\t\t\t\t: {}", parent_name.to_string());
            }
            None => {}
        }
        println!("");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for print_file
}
