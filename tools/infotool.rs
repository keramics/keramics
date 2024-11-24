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
use std::io;
use std::io::SeekFrom;
use std::process::ExitCode;

use clap::Parser;

use keramics::formats::qcow::{QcowCompressionMethod, QcowEncryptionMethod, QcowFile};
use keramics::formats::sparseimage::SparseImageFile;
use keramics::formats::udif::{UdifCompressionMethod, UdifFile};
use keramics::formats::vhd::{VhdDiskType, VhdFile};
use keramics::formats::vhdx::{VhdxDiskType, VhdxFile};
use keramics::mediator::Mediator;
use keramics::sigscan::{BuildError, PatternType, ScanContext, Scanner, Signature};
use keramics::vfs::{
    VfsDataStreamReference, VfsFileSystemReference, VfsPath, VfsPathType, VfsResolver,
    VfsResolverReference,
};

mod formatters;

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

    // TODO: add functionality to detect sparse bundle.

    Ok(signature_scanner)
}

/// Prints information about a QCOW file.
fn print_qcow_file(vfs_file_system: VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut qcow_file: QcowFile = QcowFile::new();

    // TODO: change QcowFile open() function to match UdifFile
    match vfs_file_system.with_write_lock() {
        Ok(file_system) => match qcow_file.open(file_system.as_ref(), vfs_path) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open QCOW file with error: {}", error);
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            println!("{}", error);
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
    println!("    Format version\t: {}", qcow_file.format_version);
    println!(
        "    Media size\t\t: {} ({} bytes)",
        media_size_string, qcow_file.media_size
    );
    println!("    Compression method\t: {}", compression_method_string);
    println!("    Encryption method\t: {}", encryption_method_string);

    match &qcow_file.backing_file_name {
        Some(backing_file_name) => {
            println!("    Backing file name\t: {}", backing_file_name.to_string());
        }
        None => {}
    }
    // TODO: print feature flags.
    // TODO: print snapshot information.

    println!("");

    ExitCode::SUCCESS
}

/// Prints information about a sparse image file.
fn print_sparseimage_file(vfs_file_system: VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

    match sparseimage_file.open(vfs_file_system, vfs_path) {
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
        "    Media size\t\t: {} ({} bytes)",
        media_size_string, sparseimage_file.media_size
    );
    println!(
        "    Bytes per sector\t: {} bytes",
        sparseimage_file.bytes_per_sector
    );
    println!(
        "    Band size\t\t: {} ({} bytes)",
        band_size_string, sparseimage_file.block_size,
    );
    println!("");

    ExitCode::SUCCESS
}

/// Prints information about an UDIF file.
fn print_udif_file(vfs_file_system: VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut udif_file: UdifFile = UdifFile::new();

    match udif_file.open(vfs_file_system, vfs_path) {
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
        "    Media size\t\t: {} ({} bytes)",
        media_size_string, udif_file.media_size
    );
    println!(
        "    Bytes per sector\t: {} bytes",
        udif_file.bytes_per_sector
    );
    println!("    Compression method\t: {}", compression_method_string);

    println!("");

    ExitCode::SUCCESS
}

/// Prints information about a VHD file.
fn print_vhd_file(vfs_file_system: VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut vhd_file: VhdFile = VhdFile::new();

    // TODO: change VhdFile open() function to match UdifFile
    match vfs_file_system.with_write_lock() {
        Ok(file_system) => match vhd_file.open(file_system.as_ref(), vfs_path) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open VHD file with error: {}", error);
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            println!("{}", error);
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
    println!("    Format version\t: 1.0");
    println!("    Disk type\t\t: {}", disk_type_string);
    println!(
        "    Media size\t\t: {} ({} bytes)",
        media_size_string, vhd_file.media_size
    );
    println!(
        "    Bytes per sector\t: {} bytes",
        vhd_file.bytes_per_sector
    );
    println!("    Identifier\t\t: {}", vhd_file.identifier.to_string());

    match &vhd_file.parent_identifier {
        Some(parent_identifier) => {
            println!("    Parent identifier\t: {}", parent_identifier.to_string());
        }
        None => {}
    }
    match &vhd_file.parent_name {
        Some(parent_name) => {
            println!("    Parent name\t\t: {}", parent_name.to_string());
        }
        None => {}
    }
    println!("");

    ExitCode::SUCCESS
}

/// Prints information about a VHDX file.
fn print_vhdx_file(vfs_file_system: VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut vhdx_file: VhdxFile = VhdxFile::new();

    // TODO: change VhdxFile open() function to match UdifFile
    match vfs_file_system.with_write_lock() {
        Ok(file_system) => match vhdx_file.open(file_system.as_ref(), vfs_path) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to open VHDX file with error: {}", error);
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let disk_types = HashMap::<VhdxDiskType, &'static str>::from([
        (VhdxDiskType::Differential, "Differential"),
        (VhdxDiskType::Dynamic, "Dynamic"),
        (VhdxDiskType::Fixed, "Fixed"),
        (VhdxDiskType::Unknown, "Unknown"),
    ]);
    let disk_type_string: String = disk_types.get(&vhdx_file.disk_type).unwrap().to_string();
    let media_size_string: String = formatters::format_as_bytesize(vhdx_file.media_size, 1024);

    println!("Virtual Hard Disk (VHDX) information:");
    println!("    Format version\t: 2.{}", vhdx_file.format_version);
    println!("    Disk type\t\t: {}", disk_type_string);
    println!(
        "    Media size\t\t: {} ({} bytes)",
        media_size_string, vhdx_file.media_size
    );
    println!(
        "    Bytes per sector\t: {} bytes",
        vhdx_file.bytes_per_sector
    );
    println!("    Identifier\t\t: {}", vhdx_file.identifier.to_string());

    match &vhdx_file.parent_identifier {
        Some(parent_identifier) => {
            println!("    Parent identifier\t: {}", parent_identifier.to_string());
        }
        None => {}
    }
    match &vhdx_file.parent_name {
        Some(parent_name) => {
            println!("    Parent name\t\t: {}", parent_name.to_string());
        }
        None => {}
    }
    println!("");

    ExitCode::SUCCESS
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

    // TODO: provide information about supported format

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
            "qcow1" | "qcow2" | "qcow3" => return print_qcow_file(vfs_file_system, &vfs_path),
            "sparseimage1" => return print_sparseimage_file(vfs_file_system, &vfs_path),
            "udif1" => return print_udif_file(vfs_file_system, &vfs_path),
            "vhd1" => return print_vhd_file(vfs_file_system, &vfs_path),
            "vhdx1" => return print_vhdx_file(vfs_file_system, &vfs_path),
            _ => {
                println!("Unsupported format: {}", signature.identifier);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
