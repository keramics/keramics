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

use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use core::mediator::Mediator;
use vfs::{VfsPath, VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner};

#[derive(Parser)]
#[command(version, about = "Extract data streams", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Path of the source file
    source: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Extracts a specific path
    Path(PathCommandArguments),
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,
}

/// Export data stream from a scan node.
fn export_data_stream_from_scan_node(vfs_scan_node: &VfsScanNode, path: &String) -> io::Result<()> {
    if vfs_scan_node.is_empty() {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = vfs_scan_node.path.new_with_parent(path.as_str());
        match vfs_resolver.get_data_stream_by_path_and_name(&vfs_path, None)? {
            Some(data_stream) => match data_stream.write() {
                Ok(mut data_stream) => {
                    // TODO: move write output to exporter structure.
                    // TODO: set output path and file name.
                    let mut output_file: File = File::create("/tmp/test.raw")?;

                    let mut data: Vec<u8> = vec![0; 65536];
                    while let Ok(read_count) = data_stream.read(&mut data) {
                        if read_count == 0 {
                            break;
                        }
                        output_file.write(&data[..read_count])?;
                    }
                }
                Err(error) => return Err(core::error_to_io_error!(error)),
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file entry: {}", path),
                ))
            }
        };
    } else {
        for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
            export_data_stream_from_scan_node(sub_scan_node, path)?;
        }
    }
    Ok(())
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
    let vfs_path: VfsPath = VfsPath::Os {
        location: source.to_string(),
    };

    // TODO: add scanner options.
    // TODO: add scanner mediator.
    let mut vfs_scanner: VfsScanner = VfsScanner::new();
    match vfs_scanner.build() {
        Ok(_) => {}
        Err(error) => {
            println!("{}", error);
            return ExitCode::FAILURE;
        }
    };
    let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
    match vfs_scanner.scan(&mut vfs_scan_context, &vfs_path) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to scan: {} with error: {}", source, error);
            return ExitCode::FAILURE;
        }
    };
    let root_scan_node: &VfsScanNode = match vfs_scan_context.root_node.as_ref() {
        Some(scan_node) => scan_node,
        None => {
            println!("Unable to scan: {} missing root scan node", source);
            return ExitCode::FAILURE;
        }
    };
    if root_scan_node.is_empty() {
        println!("No file system found in source.");
        return ExitCode::FAILURE;
    }
    Mediator {
        debug_output: arguments.debug,
    }
    .make_current();

    // TODO: create struct to assist with export.

    match arguments.command {
        Commands::Path(command_arguments) => {
            match export_data_stream_from_scan_node(root_scan_node, &command_arguments.path) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "Unable to export data stream from root scan node with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
        }
    };
    // TODO: print error if no data stream was found.

    ExitCode::SUCCESS
}
