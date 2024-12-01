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

use keramics::formats::udif::{UdifCompressionMethod, UdifFile};
use keramics::vfs::{VfsFileSystemReference, VfsPath};

/// Prints information about an UDIF file.
pub fn print_udif_file(
    parent_file_system: &VfsFileSystemReference,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut udif_file: UdifFile = UdifFile::new();

    match udif_file.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open UDIF file with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let compression_methods = HashMap::<UdifCompressionMethod, &'static str>::from([
        (UdifCompressionMethod::Adc, "ADC"),
        (UdifCompressionMethod::Bzip2, "bzip2"),
        (UdifCompressionMethod::Lzfse, "LZFSE/LZVN"),
        (UdifCompressionMethod::Lzma, "LZMA"),
        (UdifCompressionMethod::None, "Uncompressed"),
        (UdifCompressionMethod::Zlib, "zlib"),
    ]);

    let compression_method_string: String = compression_methods
        .get(&udif_file.compression_method)
        .unwrap()
        .to_string();
    let media_size_string: String = formatters::format_as_bytesize(udif_file.media_size, 1024);

    println!("Universal Disk Image Format (UDIF) information:");
    println!(
        "    Media size\t\t\t: {} ({} bytes)",
        media_size_string, udif_file.media_size
    );
    println!(
        "    Bytes per sector\t\t: {} bytes",
        udif_file.bytes_per_sector
    );
    println!("    Compression method\t\t: {}", compression_method_string);

    println!("");

    ExitCode::SUCCESS
}
