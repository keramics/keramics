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

use keramics::formats::qcow::{QcowCompressionMethod, QcowEncryptionMethod, QcowFile};
use keramics::vfs::{VfsFileSystemReference, VfsPath};

/// Prints information about a QCOW file.
pub fn print_qcow_file(
    parent_file_system: &VfsFileSystemReference,
    vfs_path: &VfsPath,
) -> ExitCode {
    let mut qcow_file: QcowFile = QcowFile::new();

    match qcow_file.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open QCOW file with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let compression_methods = HashMap::<QcowCompressionMethod, &'static str>::from([
        (QcowCompressionMethod::Unknown, "Unknown"),
        (QcowCompressionMethod::Zlib, "zlib"),
    ]);
    let encryption_methods = HashMap::<QcowEncryptionMethod, &'static str>::from([
        (QcowEncryptionMethod::AesCbc128, "AES-CBC 128-bit"),
        (QcowEncryptionMethod::Luks, "Linux Unified Key Setup (LUKS)"),
        (QcowEncryptionMethod::None, "None"),
        (QcowEncryptionMethod::Unknown, "Unknown"),
    ]);

    let compression_method_string: String = compression_methods
        .get(&qcow_file.compression_method)
        .unwrap()
        .to_string();
    let encryption_method_string: String = encryption_methods
        .get(&qcow_file.encryption_method)
        .unwrap()
        .to_string();
    let media_size_string: String = formatters::format_as_bytesize(qcow_file.media_size, 1024);

    println!("QEMU Copy-On-Write (QCOW) information:");
    println!("    Format version\t\t\t: {}", qcow_file.format_version);
    println!(
        "    Media size\t\t\t\t: {} ({} bytes)",
        media_size_string, qcow_file.media_size
    );
    println!(
        "    Compression method\t\t\t: {}",
        compression_method_string
    );
    println!("    Encryption method\t\t\t: {}", encryption_method_string);

    match &qcow_file.backing_file_name {
        Some(backing_file_name) => {
            println!(
                "    Backing file name\t\t\t: {}",
                backing_file_name.to_string()
            );
        }
        None => {}
    }
    // TODO: print feature flags.
    // TODO: print snapshot information.

    println!("");

    ExitCode::SUCCESS
}
