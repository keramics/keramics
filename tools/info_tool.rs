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

use std::collections::HashSet;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use keramics::enums::FormatIdentifier;
use keramics::format_scanner::FormatScanner;
use keramics::mediator::Mediator;
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
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,
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
    let mut format_scanner: FormatScanner = FormatScanner::new();
    format_scanner.add_apm_signatures();
    format_scanner.add_ext_signatures();
    format_scanner.add_gpt_signatures();
    // TODO: support for MBR.
    format_scanner.add_qcow_signatures();
    // TODO: support for sparse bundle.
    format_scanner.add_sparseimage_signatures();
    format_scanner.add_udif_signatures();
    format_scanner.add_vhd_signatures();
    format_scanner.add_vhdx_signatures();

    match format_scanner.build() {
        Ok(_) => {}
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

    let result: Option<VfsDataStreamReference> = match vfs_file_system.with_write_lock() {
        Ok(file_system) => match file_system.open_data_stream(&vfs_path, None) {
            Ok(result) => result,
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
    let vfs_data_stream: VfsDataStreamReference = match result {
        Some(data_stream) => data_stream,
        None => {
            println!("No such file: {}", source);
            return ExitCode::FAILURE;
        }
    };
    let scan_results: HashSet<FormatIdentifier> =
        match format_scanner.scan_data_stream(&vfs_data_stream) {
            Ok(scan_results) => scan_results,
            Err(error) => {
                println!(
                    "Unable to scan data stream for known format signatures with error: {}",
                    error
                );
                return ExitCode::FAILURE;
            }
        };
    if scan_results.len() > 1 {
        println!("Found multiple known format signatures",);
        return ExitCode::FAILURE;
    }
    let format_identifier: FormatIdentifier = match scan_results.iter().next() {
        Some(format_identifier) => format_identifier.clone(),
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
                return info::print_entry_ext_file_system(
                    &vfs_file_system,
                    &vfs_path,
                    command_arguments.entry,
                )
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                return ExitCode::FAILURE;
            }
        },
        Some(Commands::Hierarchy(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                return info::print_hierarcy_ext_file_system(
                    &vfs_file_system,
                    &vfs_path,
                    command_arguments.bodyfile,
                )
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                return ExitCode::FAILURE;
            }
        },
        Some(Commands::Path(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                return info::print_path_ext_file_system(
                    &vfs_file_system,
                    &vfs_path,
                    &command_arguments.path,
                )
            }
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                return ExitCode::FAILURE;
            }
        },
        None => match &format_identifier {
            FormatIdentifier::Apm => {
                return info::print_apm_volume_system(&vfs_file_system, &vfs_path)
            }
            FormatIdentifier::Ext => {
                return info::print_ext_file_system(&vfs_file_system, &vfs_path)
            }
            FormatIdentifier::Gpt => {
                return info::print_gpt_volume_system(&vfs_file_system, &vfs_path)
            }
            FormatIdentifier::Qcow => return info::print_qcow_file(&vfs_file_system, &vfs_path),
            FormatIdentifier::SparseImage => {
                return info::print_sparseimage_file(&vfs_file_system, &vfs_path)
            }
            FormatIdentifier::Udif => return info::print_udif_file(&vfs_file_system, &vfs_path),
            FormatIdentifier::Vhd => return info::print_vhd_file(&vfs_file_system, &vfs_path),
            FormatIdentifier::Vhdx => return info::print_vhdx_file(&vfs_file_system, &vfs_path),
            _ => {
                println!("Unsupported format: {}", format_identifier.to_string());
                return ExitCode::FAILURE;
            }
        },
    }
}
