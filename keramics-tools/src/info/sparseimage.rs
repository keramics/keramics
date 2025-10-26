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

use std::process::ExitCode;

use keramics_core::DataStreamReference;
use keramics_formats::sparseimage::SparseImageFile;

use crate::formatters::format_as_bytesize;

/// Information about a Mac OS sparse image (.sparseimage) file.
pub struct SparseImageInfo {}

impl SparseImageInfo {
    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> ExitCode {
        let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

        match sparseimage_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open sparse image file with error: {}", error);
                return ExitCode::FAILURE;
            }
        };
        println!("Sparse image (.sparseimage) information:");

        if sparseimage_file.media_size < 1024 {
            println!(
                "    Media size\t\t\t\t: {} bytes",
                sparseimage_file.media_size
            );
        } else {
            let media_size_string: String = format_as_bytesize(sparseimage_file.media_size, 1024);
            println!(
                "    Media size\t\t\t\t: {} ({} bytes)",
                media_size_string, sparseimage_file.media_size
            );
        }
        println!(
            "    Bytes per sector\t\t\t: {} bytes",
            sparseimage_file.bytes_per_sector
        );
        if sparseimage_file.block_size < 1024 {
            println!(
                "    Band size\t\t\t\t: {} bytes",
                sparseimage_file.block_size,
            );
        } else {
            let band_size_string: String =
                format_as_bytesize(sparseimage_file.block_size as u64, 1024);
            println!(
                "    Band size\t\t\t\t: {} ({} bytes)",
                band_size_string, sparseimage_file.block_size,
            );
        }
        println!("");

        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for print_image
}
