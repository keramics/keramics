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
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};

use keramics_core::DataStreamReference;
use keramics_core::mediator::Mediator;
use keramics_formats::{FormatIdentifier, FormatScanner};

mod formatters;
mod info;
mod range_stream;

use range_stream::FileRangeDataStream;

#[derive(Parser)]
#[command(version, about = "Provides information about file formats", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    #[arg(short, long, default_value_t = 0)]
    /// Offset within the source file.
    offset: u64,

    /// Path of the source file
    source: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Shows the information about a specific entry
    Entry(EntryCommandArguments),

    /// Shows the in-format hierarchy
    Hierarchy(HierarchyCommandArguments),

    /// Shows the information about a specific path
    Path(PathCommandArguments),
}

#[derive(Args, Debug)]
struct EntryCommandArguments {
    /// Format specific entry identifier
    entry: u64,
}

#[derive(Args, Debug)]
struct HierarchyCommandArguments {
    #[arg(long, default_value_t = false)]
    /// Output as a bodyfile
    bodyfile: bool,
    // TODO: allow to set the path component/segment separator
    // TODO: allow to set the data stream name separator
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,
}

/// Scans a data stream for format signatures.
fn scan_for_formats(data_stream: &DataStreamReference) -> io::Result<Option<FormatIdentifier>> {
    let mut format_scanner: FormatScanner = FormatScanner::new();
    format_scanner.add_apm_signatures();
    format_scanner.add_ext_signatures();
    format_scanner.add_gpt_signatures();
    format_scanner.add_ntfs_signatures();
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

    if scan_results.len() > 1 {
        // Ignore VHD footer if additional format was detected.
        scan_results.remove(&FormatIdentifier::Vhd);

        if scan_results.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found multiple known format signatures"),
            ));
        }
    }
    let mut result: Option<FormatIdentifier> = scan_results.drain().next();
    if result.is_none() {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_mbr_signatures();

        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        let mut scan_results: HashSet<FormatIdentifier> =
            format_scanner.scan_data_stream(data_stream)?;

        if scan_results.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found multiple known format signatures"),
            ));
        }
        result = scan_results.drain().next();
    }
    Ok(result)
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    // TODO: add option to list supported formats

    let source: &str = match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    let mut file_range_stream: FileRangeDataStream = FileRangeDataStream::new(arguments.offset);

    match file_range_stream.open(source) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open file with error: {}", error);
            return ExitCode::FAILURE;
        }
    };
    let data_stream: DataStreamReference = Arc::new(RwLock::new(file_range_stream));

    let result: Option<FormatIdentifier> = match scan_for_formats(&data_stream) {
        Ok(result) => result,
        Err(error) => {
            println!(
                "Unable to scan data stream for known format signatures with error: {}",
                error
            );
            return ExitCode::FAILURE;
        }
    };
    let format_identifier: FormatIdentifier = match result {
        Some(format_identifier) => format_identifier,
        None => {
            println!("No known format signatures found");
            return ExitCode::FAILURE;
        }
    };
    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    match arguments.command {
        Some(Commands::Entry(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                info::print_entry_ext_file_system(&data_stream, command_arguments.entry)
            }
            FormatIdentifier::Ntfs => {
                info::print_entry_ntfs_file_system(&data_stream, command_arguments.entry)
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                ExitCode::FAILURE
            }
        },
        Some(Commands::Hierarchy(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                info::print_hierarcy_ext_file_system(&data_stream, command_arguments.bodyfile)
            }
            FormatIdentifier::Ntfs => {
                info::print_hierarcy_ntfs_file_system(&data_stream, command_arguments.bodyfile)
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                ExitCode::FAILURE
            }
        },
        Some(Commands::Path(command_arguments)) => match &format_identifier {
            // TODO: detect leading partion path component and suggest/check path exists without
            // it.
            FormatIdentifier::Ext => {
                info::print_path_ext_file_system(&data_stream, &command_arguments.path)
            }
            FormatIdentifier::Ntfs => {
                info::print_path_ntfs_file_system(&data_stream, &command_arguments.path)
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                ExitCode::FAILURE
            }
        },
        None => match &format_identifier {
            FormatIdentifier::Apm => info::print_apm_volume_system(&data_stream),
            FormatIdentifier::Ext => info::print_ext_file_system(&data_stream),
            FormatIdentifier::Gpt => info::print_gpt_volume_system(&data_stream),
            FormatIdentifier::Mbr => info::print_mbr_volume_system(&data_stream),
            FormatIdentifier::Ntfs => info::print_ntfs_file_system(&data_stream),
            FormatIdentifier::Qcow => info::print_qcow_file(&data_stream),
            // TODO: add support for sparse bundle.
            FormatIdentifier::SparseImage => info::print_sparseimage_file(&data_stream),
            FormatIdentifier::Udif => info::print_udif_file(&data_stream),
            FormatIdentifier::Vhd => info::print_vhd_file(&data_stream),
            FormatIdentifier::Vhdx => info::print_vhdx_file(&data_stream),
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                ExitCode::FAILURE
            }
        },
    }
}
