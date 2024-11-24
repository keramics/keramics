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

use keramics::formats::apm::{ApmPartition, ApmVolumeSystem};
use keramics::formats::gpt::{GptPartition, GptVolumeSystem};
use keramics::formats::qcow::{QcowCompressionMethod, QcowEncryptionMethod, QcowFile};
use keramics::formats::sparseimage::SparseImageFile;
use keramics::formats::udif::{UdifCompressionMethod, UdifFile};
use keramics::formats::vhd::{VhdDiskType, VhdFile};
use keramics::formats::vhdx::{VhdxDiskType, VhdxFile};
use keramics::mediator::Mediator;
use keramics::sigscan::{BuildError, PatternType, ScanContext, Scanner, Signature};
use keramics::vfs::{
    VfsDataStreamReference, VfsFileSystem, VfsFileSystemReference, VfsPath, VfsPathType,
    VfsResolver, VfsResolverReference,
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

/// Prints information about an APM volume system.
fn print_apm_volume_system(
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
        if apm_partition.name.string.len() > 0 {
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

/// Prints information about a GPT volume system.
fn print_gpt_volume_system(
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
        "    Disk identifier\t\t: {}",
        gpt_volume_system.disk_identifier.to_string()
    );
    println!(
        "    Bytes per sector\t\t: {} bytes",
        gpt_volume_system.bytes_per_sector
    );
    let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();
    println!("    Number of partitions\t: {}", number_of_partitions);

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
            "    Identifier\t\t\t: {}",
            gpt_partition.identifier.to_string()
        );
        println!(
            "    Type identifier\t\t: {}",
            gpt_partition.type_identifier.to_string()
        );
        println!(
            "    Offset\t\t\t: {} (0x{:08x})",
            gpt_partition.offset, gpt_partition.offset
        );
        println!(
            "    Size\t\t\t: {} ({} bytes)",
            size_string, gpt_partition.size
        );
    }
    println!("");

    ExitCode::SUCCESS
}

/// Prints information about a QCOW file.
fn print_qcow_file(parent_file_system: &VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
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
    println!("    Format version\t\t: {}", qcow_file.format_version);
    println!(
        "    Media size\t\t\t: {} ({} bytes)",
        media_size_string, qcow_file.media_size
    );
    println!("    Compression method\t\t: {}", compression_method_string);
    println!("    Encryption method\t\t: {}", encryption_method_string);

    match &qcow_file.backing_file_name {
        Some(backing_file_name) => {
            println!(
                "    Backing file name\t\t: {}",
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

/// Prints information about a sparse image file.
fn print_sparseimage_file(
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

/// Prints information about an UDIF file.
fn print_udif_file(parent_file_system: &VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
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

/// Prints information about a VHD file.
fn print_vhd_file(parent_file_system: &VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut vhd_file: VhdFile = VhdFile::new();

    match vhd_file.open(parent_file_system, vfs_path) {
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
    println!("    Format version\t\t: 1.0");
    println!("    Disk type\t\t\t: {}", disk_type_string);
    println!(
        "    Media size\t\t\t: {} ({} bytes)",
        media_size_string, vhd_file.media_size
    );
    println!(
        "    Bytes per sector\t\t: {} bytes",
        vhd_file.bytes_per_sector
    );
    println!("    Identifier\t\t\t: {}", vhd_file.identifier.to_string());

    match &vhd_file.parent_identifier {
        Some(parent_identifier) => {
            println!(
                "    Parent identifier\t\t: {}",
                parent_identifier.to_string()
            );
        }
        None => {}
    }
    match &vhd_file.parent_name {
        Some(parent_name) => {
            println!("    Parent name\t\t\t: {}", parent_name.to_string());
        }
        None => {}
    }
    println!("");

    ExitCode::SUCCESS
}

/// Prints information about a VHDX file.
fn print_vhdx_file(parent_file_system: &VfsFileSystemReference, vfs_path: &VfsPath) -> ExitCode {
    let mut vhdx_file: VhdxFile = VhdxFile::new();

    match vhdx_file.open(parent_file_system, vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open VHDX file with error: {}", error);
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
    println!("    Format version\t\t: 2.{}", vhdx_file.format_version);
    println!("    Disk type\t\t\t: {}", disk_type_string);
    println!(
        "    Media size\t\t\t: {} ({} bytes)",
        media_size_string, vhdx_file.media_size
    );
    println!(
        "    Bytes per sector\t\t: {} bytes",
        vhdx_file.bytes_per_sector
    );
    println!("    Identifier\t\t\t: {}", vhdx_file.identifier.to_string());

    match &vhdx_file.parent_identifier {
        Some(parent_identifier) => {
            println!(
                "    Parent identifier\t\t: {}",
                parent_identifier.to_string()
            );
        }
        None => {}
    }
    match &vhdx_file.parent_name {
        Some(parent_name) => {
            println!("    Parent name\t\t\t: {}", parent_name.to_string());
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
            "apm1" => return print_apm_volume_system(&vfs_file_system, &vfs_path),
            "gpt1" | "gpt2" | "gpt3" | "gpt4" => {
                return print_gpt_volume_system(&vfs_file_system, &vfs_path)
            }
            "qcow1" | "qcow2" | "qcow3" => return print_qcow_file(&vfs_file_system, &vfs_path),
            "sparseimage1" => return print_sparseimage_file(&vfs_file_system, &vfs_path),
            "udif1" => return print_udif_file(&vfs_file_system, &vfs_path),
            "vhd1" => return print_vhd_file(&vfs_file_system, &vfs_path),
            "vhdx1" => return print_vhdx_file(&vfs_file_system, &vfs_path),
            _ => {
                println!("Unsupported format: {}", signature.identifier);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
