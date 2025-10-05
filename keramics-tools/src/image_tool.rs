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

use std::collections::HashSet;
use std::fmt::Write;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use keramics_core::formatters::format_as_string;
use keramics_core::mediator::Mediator;
use keramics_core::{
    DataStream, DataStreamReference, FileResolverReference, open_os_data_stream,
    open_os_file_resolver,
};
use keramics_formats::ewf::EwfImage;
use keramics_formats::qcow::{QcowImage, QcowImageLayer};
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::udif::UdifFile;
use keramics_formats::vhd::{VhdImage, VhdImageLayer};
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};
use keramics_formats::{FormatIdentifier, FormatScanner};
use keramics_hashes::{DigestHashContext, Md5Context};
use keramics_vfs::{
    VfsFileEntry, VfsLocation, VfsPath, VfsResolver, VfsResolverReference, VfsScanContext,
    VfsScanNode, VfsScanner, VfsType, new_os_vfs_location,
};

#[derive(Parser)]
#[command(version, about = "Analyzes the contents of a storage media image", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Path of the source file
    source: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Calculate digest hashes of a storage media image
    Hash,

    /// Show the hierarchy of the volumes, partitions and file systems
    Hierarchy,
}

/// Scans a data stream for storage media image format signatures.
fn scan_for_storage_image_formats(
    data_stream: &DataStreamReference,
) -> io::Result<Option<FormatIdentifier>> {
    let mut format_scanner: FormatScanner = FormatScanner::new();
    format_scanner.add_ewf_signatures();
    format_scanner.add_qcow_signatures();
    // TODO: support for sparse bundle.
    format_scanner.add_sparseimage_signatures();
    format_scanner.add_udif_signatures();
    format_scanner.add_vhd_signatures();
    format_scanner.add_vhdx_signatures();

    match format_scanner.build() {
        Ok(_) => {}
        Err(error) => return Err(keramics_core::error_to_io_error!(error)),
    };
    let mut scan_results: HashSet<FormatIdentifier> =
        format_scanner.scan_data_stream(data_stream)?;

    let result: Option<FormatIdentifier> = scan_results.drain().next();

    Ok(result)
}

/// Prints information about a scan node.
fn print_scan_node(scan_node: &VfsScanNode, depth: usize) -> io::Result<()> {
    let indentation: String = vec![" "; depth * 4].join("");
    let vfs_resolver: VfsResolverReference = VfsResolver::current();
    let suffix: String = match vfs_resolver.get_file_entry_by_path(&scan_node.location)? {
        Some(file_entry) => match file_entry {
            VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry.get_identifier() {
                Some(identifier) => format!(" (identifier: {})", identifier.to_string()),
                _ => String::new(),
            },
            _ => String::new(),
        },
        None => String::new(),
    };
    let vfs_path: &VfsPath = scan_node.location.get_path();
    let vfs_type: &VfsType = scan_node.get_type();

    println!(
        "{}{}: path: {}{}",
        indentation,
        vfs_type.as_str(),
        vfs_path.to_string(),
        suffix,
    );
    for sub_scan_node in scan_node.sub_nodes.iter() {
        print_scan_node(sub_scan_node, depth + 1)?;
    }
    Ok(())
}

/// Storage media image.
enum StorageMediaImage {
    Ewf(EwfImage),
    Qcow(QcowImage),
    SparseImage(SparseImageFile),
    Udif(UdifFile),
    Vhd(VhdImage),
    Vhdx(VhdxImage),
}

impl StorageMediaImage {
    /// Opens a storage media image.
    fn get_base_path_and_file_name<'a>(path: &'a PathBuf) -> io::Result<(&'a str, &'a str)> {
        let base_path: &str = match path.parent() {
            Some(parent_path) => match parent_path.to_str() {
                Some(path_string) => path_string,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Unsupported source - invalid parent directory",
                    ));
                }
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported source - missing parent directory",
                ));
            }
        };
        let file_name: &str = match path.file_name() {
            Some(file_name_path) => match file_name_path.to_str() {
                Some(path_string) => path_string,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Unsupported source - invalid file name",
                    ));
                }
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported source - missing file name",
                ));
            }
        };
        Ok((base_path, file_name))
    }

    /// Retrieves the stored MD5 hash.
    fn get_md5_hash(&self) -> Option<&[u8]> {
        match self {
            StorageMediaImage::Ewf(image) => Some(&image.md5_hash),
            _ => None,
        }
    }

    /// Retrieves the media size.
    fn get_media_size(&self) -> u64 {
        match self {
            StorageMediaImage::Ewf(image) => image.media_size,
            // TODO: add Qcow layer support.
            StorageMediaImage::SparseImage(file) => file.media_size,
            StorageMediaImage::Udif(file) => file.media_size,
            // TODO: add Vhd layer support.
            // TODO: add Vhdx layer support.
            _ => todo!(),
        }
    }

    /// Retrieves the stored SHA1 hash.
    fn get_sha1_hash(&self) -> Option<&[u8]> {
        match self {
            StorageMediaImage::Ewf(image) => Some(&image.sha1_hash),
            _ => None,
        }
    }

    /// Opens a storage media image.
    fn open(&mut self, path: &PathBuf) -> io::Result<()> {
        match self {
            StorageMediaImage::Ewf(image) => {
                let (base_path, file_name) = StorageMediaImage::get_base_path_and_file_name(path)?;
                let file_resolver: FileResolverReference = open_os_file_resolver(base_path)?;

                image.open(&file_resolver, file_name)?;
            }
            StorageMediaImage::Qcow(image) => {
                let (base_path, file_name) = StorageMediaImage::get_base_path_and_file_name(path)?;
                let file_resolver: FileResolverReference = open_os_file_resolver(base_path)?;

                image.open(&file_resolver, file_name)?;
            }
            StorageMediaImage::SparseImage(file) => {
                let path_string: &str = match path.to_str() {
                    Some(path_string) => path_string,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Unsupported path",
                        ));
                    }
                };
                let data_stream: DataStreamReference = open_os_data_stream(path_string)?;
                file.read_data_stream(&data_stream)?;
            }
            StorageMediaImage::Udif(file) => {
                let path_string: &str = match path.to_str() {
                    Some(path_string) => path_string,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Unsupported path",
                        ));
                    }
                };
                let data_stream: DataStreamReference = open_os_data_stream(path_string)?;
                file.read_data_stream(&data_stream)?;
            }
            StorageMediaImage::Vhd(image) => {
                let (base_path, file_name) = StorageMediaImage::get_base_path_and_file_name(path)?;
                let file_resolver: FileResolverReference = open_os_file_resolver(base_path)?;

                image.open(&file_resolver, file_name)?;
            }
            StorageMediaImage::Vhdx(image) => {
                let (base_path, file_name) = StorageMediaImage::get_base_path_and_file_name(path)?;
                let file_resolver: FileResolverReference = open_os_file_resolver(base_path)?;

                image.open(&file_resolver, file_name)?;
            }
        }
        Ok(())
    }
}

impl Read for StorageMediaImage {
    /// Reads media data.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            StorageMediaImage::Ewf(image) => image.read(buf),
            // TODO: add Qcow layer support.
            StorageMediaImage::SparseImage(file) => file.read(buf),
            StorageMediaImage::Udif(file) => file.read(buf),
            // TODO: add Vhd layer support.
            // TODO: add Vhdx layer support.
            _ => todo!(),
        }
    }
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    let source: &str = match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    match arguments.command {
        Some(Commands::Hash) => {
            let data_stream: DataStreamReference = match open_os_data_stream(source) {
                Ok(data_stream) => data_stream,
                Err(error) => {
                    println!("Unable to open file: {} with error: {}", source, error);
                    return ExitCode::FAILURE;
                }
            };
            let result: Option<FormatIdentifier> = match scan_for_storage_image_formats(
                &data_stream,
            ) {
                Ok(result) => result,
                Err(error) => {
                    println!(
                        "Unable to scan data stream for known storage media image format signatures with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
            let format_identifier: FormatIdentifier = match result {
                Some(format_identifier) => format_identifier,
                None => {
                    println!("No known storage media image format signatures found");
                    return ExitCode::FAILURE;
                }
            };
            let mut storage_media_image: StorageMediaImage = match &format_identifier {
                FormatIdentifier::Ewf => StorageMediaImage::Ewf(EwfImage::new()),
                FormatIdentifier::Qcow => StorageMediaImage::Qcow(QcowImage::new()),
                // TODO: add support for sparse bundle.
                FormatIdentifier::SparseImage => {
                    StorageMediaImage::SparseImage(SparseImageFile::new())
                }
                FormatIdentifier::Udif => StorageMediaImage::Udif(UdifFile::new()),
                FormatIdentifier::Vhd => StorageMediaImage::Vhd(VhdImage::new()),
                FormatIdentifier::Vhdx => StorageMediaImage::Vhdx(VhdxImage::new()),
                _ => {
                    println!("Unsupported format: {}", format_identifier.to_string());
                    return ExitCode::FAILURE;
                }
            };
            match storage_media_image.open(&arguments.source) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "Unable to open storage media image: {} with error: {}",
                        source, error
                    );
                    return ExitCode::FAILURE;
                }
            }
            let media_size: u64 = storage_media_image.get_media_size();

            let progress_bar_style: ProgressStyle = match ProgressStyle::with_template(
                "Hashing at {percent}% [{wide_bar}] {bytes}/{total_bytes} ({binary_bytes_per_sec}) elapsed: {elapsed_precise} (remaining: {eta_precise})",
            ) {
                Ok(style) => {
                    style.with_key("eta", |state: &ProgressState, writer: &mut dyn Write| {
                        write!(writer, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    })
                }
                Err(error) => {
                    println!(
                        "Unable to create progress bar style from template with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
            let progress_bar: ProgressBar = ProgressBar::new(media_size);
            progress_bar.set_style(progress_bar_style.progress_chars("#>-"));

            // TODO: add support for calculating multiple digest hashed concurrently.

            let mut media_offset: u64 = 0;
            let mut md5_context: Md5Context = Md5Context::new();
            let mut data: [u8; 65536] = [0; 65536];

            loop {
                let read_count = match storage_media_image.read(&mut data) {
                    Ok(read_count) => read_count,
                    Err(error) => {
                        println!(
                            "Unable to read data at offset {} with error: {}",
                            media_offset, error
                        );
                        return ExitCode::FAILURE;
                    }
                };
                if read_count == 0 {
                    break;
                }
                md5_context.update(&data[0..read_count]);

                media_offset += read_count as u64;

                progress_bar.set_position(media_offset);
            }
            progress_bar.finish();

            println!("");

            let md5_hash: Vec<u8> = md5_context.finalize();

            let hash_string: String = format_as_string(&md5_hash);
            println!("Calculated MD5 hash\t: {}", hash_string);

            match storage_media_image.get_md5_hash() {
                Some(stored_hash) => {
                    if stored_hash != [0; 16] {
                        let hash_string: String = format_as_string(stored_hash);
                        println!("Stored MD5 hash\t\t: {}", hash_string);
                    }
                }
                None => {}
            };
            match storage_media_image.get_sha1_hash() {
                Some(stored_hash) => {
                    if stored_hash != [0; 20] {
                        let hash_string: String = format_as_string(stored_hash);
                        println!("Stored SHA1 hash\t: {}", hash_string);
                    }
                }
                None => {}
            };
        }
        _ => {
            let mut vfs_scanner: VfsScanner = VfsScanner::new();

            match vfs_scanner.build() {
                Ok(_) => {}
                Err(error) => {
                    println!("{}", error);
                    return ExitCode::FAILURE;
                }
            };
            let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
            let vfs_location: VfsLocation = new_os_vfs_location(source);

            match vfs_scanner.scan(&mut vfs_scan_context, &vfs_location) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to scan: {} with error: {}", source, error);
                    return ExitCode::FAILURE;
                }
            };
            // TODO: print source type.

            match vfs_scan_context.root_node {
                Some(scan_node) => match print_scan_node(&scan_node, 0) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "Unable to print results of scan: {} with error: {}",
                            source, error
                        );
                        return ExitCode::FAILURE;
                    }
                },
                None => {}
            }
        }
    }
    ExitCode::SUCCESS
}
