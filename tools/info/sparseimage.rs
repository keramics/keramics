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

use keramics::formats::sparseimage::SparseImageFile;
use keramics::vfs::{VfsFileSystemReference, VfsPath};

/// Prints information about a sparse image file.
pub fn print_sparseimage_file(
    parent_file_system: &VfsFileSystemReference,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

    match sparseimage_file.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open sparse image file with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let media_size_string: String =
        formatters::format_as_bytesize(sparseimage_file.media_size, 1024);
    let band_size_string: String =
        formatters::format_as_bytesize(sparseimage_file.block_size as u64, 1024);

    println!("Sparse image (.sparseimage) information:");
    println!(
        "    Media size\t\t\t: {} ({} bytes)",
        media_size_string, sparseimage_file.media_size
    );
    println!(
        "    Bytes per sector\t\t: {} bytes",
        sparseimage_file.bytes_per_sector
    );
    println!(
        "    Band size\t\t\t: {} ({} bytes)",
        band_size_string, sparseimage_file.block_size,
    );
    println!("");

    ExitCode::SUCCESS
}
