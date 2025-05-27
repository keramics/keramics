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

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use core::mediator::Mediator;
use vfs::{VfsPath, VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner};

mod writer;

#[derive(Parser)]
#[command(version, about = "Extract data streams from a storage media image", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    #[arg(short, long, default_value_t = 0)]
    /// Offset within the source file.
    offset: u64,

    /// Path of the storage media image
    source: PathBuf,

    #[arg(short, long)]
    /// Target (or destination) path of a directory where the extracted data stream should
    /// be written.
    target: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Path of the data stream to extract.
    Path(PathCommandArguments),
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,

    #[arg(long)]
    /// Name of the data stream to extract.
    name: Option<String>,
}

/// Export data stream from a scan node.
fn export_data_stream_from_scan_node(
    data_stream_writer: &mut writer::DataStreamWriter,
    vfs_scan_node: &VfsScanNode,
    path: &String,
    name: Option<&str>,
) -> io::Result<()> {
    if vfs_scan_node.is_empty() {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = vfs_scan_node.path.new_with_parent(path.as_str());
        match vfs_resolver.get_data_stream_by_path_and_name(&vfs_path, name)? {
            // TODO: pass sanitized file entry path and data stream name.
            Some(data_stream) => data_stream_writer.write_data_stream(&data_stream)?,
            None => {}
        };
    } else {
        for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
            export_data_stream_from_scan_node(data_stream_writer, sub_scan_node, path, name)?;
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

    // TODO: add scanner options (such as offset).
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

    let target: PathBuf = match arguments.target {
        Some(path) => path,
        None => PathBuf::from("."),
    };
    let mut data_stream_writer: writer::DataStreamWriter = writer::DataStreamWriter::new(&target);

    match arguments.command {
        Commands::Path(command_arguments) => {
            let name: Option<&str> = match command_arguments.name {
                Some(ref name) => Some(name.as_str()),
                None => None,
            };
            match export_data_stream_from_scan_node(
                &mut data_stream_writer,
                root_scan_node,
                &command_arguments.path,
                name,
            ) {
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
    if data_stream_writer.number_of_streams_written == 0 {
        println!("No data streams exported.");
    }
    ExitCode::SUCCESS
}
