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

use std::io;
use std::io::SeekFrom;
use std::process::ExitCode;

use clap::Parser;

use keramics::mediator::Mediator;
use keramics::sigscan::{BuildError, PatternType, ScanContext, Scanner, Signature};
use keramics::vfs::{
    VfsDataStreamReference, VfsFileSystemReference, VfsPath, VfsPathType, VfsResolver,
    VfsResolverReference,
};

mod formatters;
mod info;

#[derive(Parser)]
#[command(version, about = "Provides information about a supported file format", long_about = None)]
struct CommandLineArguments {
    #[arg(short, long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Path of the source file
    source: std::path::PathBuf,
}

/// Create a signature scanner.
fn create_signature_scanner() -> Result<Scanner, BuildError> {
    let mut signature_scanner: Scanner = Scanner::new();
    // APM signature.
    // Note that technically "PM" at offset 512 is the Apple Partion Map
    // signature but using the partition type is less error prone.
    signature_scanner.add_signature(Signature::new(
        "apm1",
        PatternType::BoundToStart,
        560,
        &[
            0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x70, 0x61, 0x72, 0x74, 0x69, 0x74, 0x69, 0x6f,
            0x6e, 0x5f, 0x6d, 0x61, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ],
    ));
    // GPT signature for 512 bytes per sector.
    signature_scanner.add_signature(Signature::new(
        "gpt1",
        PatternType::BoundToStart,
        512,
        &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
    ));
    // GPT signature for 1024 bytes per sector.
    signature_scanner.add_signature(Signature::new(
        "gpt1",
        PatternType::BoundToStart,
        1024,
        &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
    ));
    // GPT signature for 2048 bytes per sector.
    signature_scanner.add_signature(Signature::new(
        "gpt1",
        PatternType::BoundToStart,
        2048,
        &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
    ));
    // GPT signature for 4096 bytes per sector.
    signature_scanner.add_signature(Signature::new(
        "gpt1",
        PatternType::BoundToStart,
        4096,
        &[0x45, 0x46, 0x49, 0x20, 0x50, 0x41, 0x52, 0x54],
    ));
    // TODO: add MBR signature.

    // QCOW version 1 signature and version in header.
    signature_scanner.add_signature(Signature::new(
        "qcow1",
        PatternType::BoundToStart,
        0,
        &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x01],
    ));
    // QCOW version 2 signature and version in header.
    signature_scanner.add_signature(Signature::new(
        "qcow2",
        PatternType::BoundToStart,
        0,
        &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x02],
    ));
    // QCOW version 3 signature and version in header.
    signature_scanner.add_signature(Signature::new(
        "qcow3",
        PatternType::BoundToStart,
        0,
        &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03],
    ));
    // TODO: add sparse bundle signature.

    // Sparse disk image (.sparseimage) signature in header.
    signature_scanner.add_signature(Signature::new(
        "sparseimage1",
        PatternType::BoundToStart,
        0,
        &[0x73, 0x70, 0x72, 0x73],
    ));
    // Universal Disk Image Format (UDIF) (.dmg) signature in footer.
    signature_scanner.add_signature(Signature::new(
        "udif1",
        PatternType::BoundToEnd,
        512,
        &[0x6b, 0x6f, 0x6c, 0x79],
    ));
    // Virtual Hard Disk version 1 (VHD) signature in footer.
    signature_scanner.add_signature(Signature::new(
        "vhd1",
        PatternType::BoundToEnd,
        512,
        &[0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78],
    ));
    // Virtual Hard Disk version 2 (VHDX) signature in header.
    signature_scanner.add_signature(Signature::new(
        "vhdx1",
        PatternType::BoundToStart,
        0,
        &[0x76, 0x68, 0x64, 0x78, 0x66, 0x69, 0x6c, 0x65],
    ));
    signature_scanner.build()?;

    Ok(signature_scanner)
}

/// Scans a data stream for matching signatures.
fn scan_data_stream(
    scan_context: &mut ScanContext,
    vfs_data_stream: &VfsDataStreamReference,
) -> io::Result<()> {
    let (range_start_offset, range_end_offset): (u64, u64) = scan_context.get_header_range();
    let range_size: usize = (range_end_offset - range_start_offset) as usize;

    let mut data: Vec<u8> = vec![0; range_size];

    match vfs_data_stream.with_write_lock() {
        Ok(mut data_stream) => {
            data_stream.read_exact_at_position(&mut data, SeekFrom::Start(range_start_offset))?
        }
        Err(error) => return Err(keramics::error_to_io_error!(error)),
    };
    scan_context.scan_buffer(&data);

    let (range_start_offset, range_end_offset): (u64, u64) = scan_context.get_footer_range();
    let range_size: usize = (range_end_offset - range_start_offset) as usize;

    let mut data: Vec<u8> = vec![0; range_size];

    match vfs_data_stream.with_write_lock() {
        Ok(mut data_stream) => {
            data_stream.read_exact_at_position(&mut data, SeekFrom::Start(range_start_offset))?
        }
        Err(error) => return Err(keramics::error_to_io_error!(error)),
    };
    scan_context.data_offset = range_start_offset;
    scan_context.scan_buffer(&data);

    Ok(())
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    // TODO: add option to list supported formats

    let source: &str = match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    let signature_scanner: Scanner = match create_signature_scanner() {
        Ok(scanner) => scanner,
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let vfs_resolver: VfsResolverReference = VfsResolver::current();

    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
    let vfs_file_system: VfsFileSystemReference = match vfs_resolver.open_file_system(&vfs_path) {
        Ok(value) => value,
        Err(error) => {
            println!("Unable to open file system with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, source, None);

    let vfs_data_stream: VfsDataStreamReference = match vfs_file_system.with_write_lock() {
        Ok(file_system) => match file_system.open_data_stream(&vfs_path, None) {
            Ok(data_stream) => data_stream,
            Err(error) => {
                println!("Unable to open data stream with error: {}", error);
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let data_size: u64 = match vfs_data_stream.with_write_lock() {
        Ok(mut data_stream) => match data_stream.get_size() {
            Ok(size) => size,
            Err(error) => {
                println!(
                    "Unable to determine size of data stream with error: {}",
                    error
                );
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let mut scan_context: ScanContext = ScanContext::new(&signature_scanner, data_size);

    match scan_data_stream(&mut scan_context, &vfs_data_stream) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "Unable to scan data stream for signatures with error: {}",
                error
            );
            return ExitCode::FAILURE;
        }
    };
    if scan_context.results.len() > 1 {
        println!("Unsupported format found more than 1 signature");
        return ExitCode::FAILURE;
    }
    if let Some(signature) = scan_context.results.values().next() {
        match signature.identifier.as_str() {
            "apm1" => return info::print_apm_volume_system(&vfs_file_system, &vfs_path),
            "gpt1" | "gpt2" | "gpt3" | "gpt4" => {
                return info::print_gpt_volume_system(&vfs_file_system, &vfs_path)
            }
            "qcow1" | "qcow2" | "qcow3" => {
                return info::print_qcow_file(&vfs_file_system, &vfs_path)
            }
            "sparseimage1" => return info::print_sparseimage_file(&vfs_file_system, &vfs_path),
            "udif1" => return info::print_udif_file(&vfs_file_system, &vfs_path),
            "vhd1" => return info::print_vhd_file(&vfs_file_system, &vfs_path),
            "vhdx1" => return info::print_vhdx_file(&vfs_file_system, &vfs_path),
            _ => {
                println!("Unsupported format: {}", signature.identifier);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
