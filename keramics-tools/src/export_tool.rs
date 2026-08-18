/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use clap_num::maybe_hex;

use keramics_core::ErrorTrace;
use keramics_formats::{Path, PathComponent};
use keramics_vfs::{
    VfsLocation, VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanOptions,
    VfsScanner,
};

#[cfg(feature = "debug-trace")]
use keramics_core::mediator::Mediator;

mod writer;

use crate::writer::DataStreamWriter;

#[derive(Parser)]
#[command(version, about = "Extract data streams from a storage media image", long_about = None)]
struct CommandLineArguments {
    #[cfg(feature = "debug-trace")]
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    #[arg(long, default_value_t = 0)]
    /// Layer within the storage media image, where 1 represents the first layer
    image_layer: usize,

    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Offset within the storage media
    offset: u64,

    /// Comma seperated list of partitions to include
    #[arg(long)]
    partitions: Option<String>,

    /// Path of the storage media image
    source: PathBuf,

    #[arg(short, long)]
    /// Target (or destination) path of a directory where the extracted data stream should
    /// be written
    target: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Path of the data stream to extract
    Path(PathCommandArguments),
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,

    #[arg(long)]
    /// Name of the data stream to extract
    name: Option<String>,
}

/// Tool for extracting data streams from a storage media image.
struct ExportTool {
    /// Data stream writer.
    data_stream_writer: DataStreamWriter,

    /// Output path.
    output_path: PathBuf,
}

impl ExportTool {
    /// Creates a new tool.
    fn new(output_path: PathBuf) -> Self {
        Self {
            data_stream_writer: DataStreamWriter::new(),
            output_path,
        }
    }

    /// Export data stream from a scan node.
    fn export_data_stream_from_scan_node_with_path(
        &mut self,
        vfs_scan_node: &VfsScanNode,
        path: &Path,
        name: Option<&PathComponent>,
    ) -> Result<(), ErrorTrace> {
        if vfs_scan_node.is_empty() {
            let vfs_resolver: VfsResolverReference = VfsResolver::current();

            let vfs_location: VfsLocation = vfs_scan_node.location.new_with_parent(path.clone());

            match vfs_resolver.get_data_stream_by_location_and_name(&vfs_location, name) {
                Ok(Some(data_stream)) => {
                    // TODO: sanitize output path, file name and data stream name.
                    match create_dir_all(&self.output_path) {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to create output directory",
                                error
                            ));
                        }
                    }
                    let file_name: String = match path.file_name() {
                        Some(file_name) => file_name.to_string(),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to retrieve file name from path"
                            ));
                        }
                    };
                    // TODO: include data stream name in output file name.
                    let mut output_path: PathBuf = self.output_path.clone();

                    output_path.push(file_name.as_str());

                    match self
                        .data_stream_writer
                        .write_data_stream(&data_stream, &output_path)
                    {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to export data stream"
                            );
                            return Err(error);
                        }
                    }
                }
                Ok(None) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        } else {
            for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
                match self.export_data_stream_from_scan_node_with_path(sub_scan_node, path, name) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to export data stream from sub scan node"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Retrieves the number of data streams written.
    fn get_number_of_streams_written(&self) -> usize {
        self.data_stream_writer.number_of_streams_written
    }

    /// Scans the source for file systems.
    fn scan_for_file_systems<'a>(
        &self,
        vfs_location: &'a VfsLocation,
        image_layer: usize,
        partitions: Option<&String>,
        vfs_scan_context: &mut VfsScanContext<'a>,
    ) -> Result<(), ErrorTrace> {
        let mut vfs_scanner: VfsScanner = VfsScanner::new();

        match vfs_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let mut vfs_scan_options: VfsScanOptions = VfsScanOptions::new();

        vfs_scan_options.image_layer = image_layer;

        if let Some(partitions_string) = partitions {
            match vfs_scan_options.parse_partitions(partitions_string.as_str()) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to parse partitions");
                    return Err(error);
                }
            }
        }
        // TODO: add scanner mediator.

        match vfs_scanner.scan(&vfs_scan_options, vfs_scan_context, vfs_location) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to scan for file systems");
                return Err(error);
            }
        }
        Ok(())
    }
}

fn main() -> ExitCode {
    let arguments = CommandLineArguments::parse();

    let source_string: &str = match arguments.source.to_str() {
        Some(value) => value,
        None => {
            println!("Missing source");
            return ExitCode::FAILURE;
        }
    };
    let source_file_name: &str = match arguments.source.file_name() {
        Some(os_str) => match os_str.to_str() {
            Some(value) => value,
            None => {
                println!("Unsuppported source file name");
                return ExitCode::FAILURE;
            }
        },
        None => {
            println!("Unable to retrieve source file name");
            return ExitCode::FAILURE;
        }
    };
    let mut target: PathBuf = match arguments.target {
        Some(path) => path,
        None => PathBuf::from("."),
    };
    target.push(format!("{}.export", source_file_name));

    let mut export_tool: ExportTool = ExportTool::new(target);

    let vfs_location: VfsLocation = VfsLocation::from(&arguments.source);
    let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();

    match export_tool.scan_for_file_systems(
        &vfs_location,
        arguments.image_layer,
        arguments.partitions.as_ref(),
        &mut vfs_scan_context,
    ) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "Unable to scan: {} for file systems\n{}",
                source_string, error
            );
            return ExitCode::FAILURE;
        }
    }
    let root_scan_node: &VfsScanNode = match vfs_scan_context.root_node.as_ref() {
        Some(scan_node) => scan_node,
        None => {
            println!("Unable to scan: {} missing root scan node", source_string);
            return ExitCode::FAILURE;
        }
    };
    if root_scan_node.is_empty() {
        println!("No file system found in source");
        return ExitCode::FAILURE;
    }
    #[cfg(feature = "debug-trace")]
    {
        Mediator {
            debug_output: arguments.debug,
        }
        .make_current();
    }
    match arguments.command {
        Commands::Path(command_arguments) => {
            let name: Option<PathComponent> = match command_arguments.name {
                Some(ref name) => Some(PathComponent::from(name)),
                None => None,
            };
            let path: Path = Path::from(&command_arguments.path);

            match export_tool.export_data_stream_from_scan_node_with_path(
                root_scan_node,
                &path,
                name.as_ref(),
            ) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "Unable to export data stream from: {} with error:\n{}",
                        source_string, error
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    if export_tool.get_number_of_streams_written() == 0 {
        println!("No data streams exported");
    }
    ExitCode::SUCCESS
}
