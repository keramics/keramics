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
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};
use clap_num::maybe_hex;

use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_formats::{FormatIdentifier, FormatScanner};

mod enums;
mod formatters;
mod info;
mod range_stream;

use crate::enums::EncodingType;
use crate::info::{
    ApmInfo, EwfInfo, ExtInfo, FatInfo, GptInfo, MbrInfo, NtfsInfo, QcowInfo, SparseImageInfo,
    UdifInfo, VhdInfo, VhdxInfo,
};
use crate::range_stream::FileRangeDataStream;

#[derive(Parser)]
#[command(version, about = "Provides information about file formats", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Character encoding
    #[arg(long, value_enum)]
    encoding: Option<EncodingType>,

    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Offset within the source file
    offset: u64,

    /// Path of the source file
    source: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show the information about a specific entry
    Entry(EntryCommandArguments),

    /// Show the in-format hierarchy
    Hierarchy(HierarchyCommandArguments),

    /// Show the information about a specific path
    Path(PathCommandArguments),
}

#[derive(Args, Debug)]
struct EntryCommandArguments {
    /// Format specific entry identifier
    #[arg(value_parser=maybe_hex::<u64>)]
    entry: u64,
}

#[derive(Args, Debug)]
struct HierarchyCommandArguments {
    // TODO: allow to set the path component/segment separator
    // TODO: allow to set the data stream name separator
}

#[derive(Args, Debug)]
struct PathCommandArguments {
    /// Format specific path
    path: String,
}

/// Scans a data stream for format signatures.
fn scan_for_formats(
    data_stream: &DataStreamReference,
) -> Result<Option<FormatIdentifier>, ErrorTrace> {
    let mut format_scanner: FormatScanner = FormatScanner::new();
    format_scanner.add_apm_signatures();
    format_scanner.add_ext_signatures();
    format_scanner.add_ewf_signatures();
    format_scanner.add_fat_signatures();
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
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to build format scanner",
                error
            ));
        }
    }
    let mut scan_results: HashSet<FormatIdentifier> =
        match format_scanner.scan_data_stream(data_stream) {
            Ok(scan_results) => scan_results,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve scan results");
                return Err(error);
            }
        };
    if scan_results.len() > 1 {
        // Ignore VHD footer if additional format was detected.
        scan_results.remove(&FormatIdentifier::Vhd);

        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported multiple known format signatures"
            ));
        }
    }
    let mut result: Option<FormatIdentifier> = scan_results.drain().next();
    if result.is_none() {
        let mut format_scanner: FormatScanner = FormatScanner::new();
        format_scanner.add_mbr_signatures();

        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let mut scan_results: HashSet<FormatIdentifier> =
            match format_scanner.scan_data_stream(data_stream) {
                Ok(scan_results) => scan_results,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to scan data stream for known format signatures"
                    );
                    return Err(error);
                }
            };
        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported multiple known format signatures"
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
    let character_encoding: Option<CharacterEncoding> = match &arguments.encoding {
        Some(EncodingType::Ascii) => Some(CharacterEncoding::Ascii),
        Some(EncodingType::Iso8859_1) => Some(CharacterEncoding::Iso8859_1),
        Some(EncodingType::Iso8859_2) => Some(CharacterEncoding::Iso8859_2),
        Some(EncodingType::Iso8859_3) => Some(CharacterEncoding::Iso8859_3),
        Some(EncodingType::Iso8859_4) => Some(CharacterEncoding::Iso8859_4),
        Some(EncodingType::Iso8859_5) => Some(CharacterEncoding::Iso8859_5),
        Some(EncodingType::Iso8859_6) => Some(CharacterEncoding::Iso8859_6),
        Some(EncodingType::Iso8859_7) => Some(CharacterEncoding::Iso8859_7),
        Some(EncodingType::Iso8859_8) => Some(CharacterEncoding::Iso8859_8),
        Some(EncodingType::Iso8859_9) => Some(CharacterEncoding::Iso8859_9),
        Some(EncodingType::Iso8859_10) => Some(CharacterEncoding::Iso8859_10),
        Some(EncodingType::Iso8859_11) => Some(CharacterEncoding::Iso8859_11),
        Some(EncodingType::Iso8859_13) => Some(CharacterEncoding::Iso8859_13),
        Some(EncodingType::Iso8859_14) => Some(CharacterEncoding::Iso8859_14),
        Some(EncodingType::Iso8859_15) => Some(CharacterEncoding::Iso8859_15),
        Some(EncodingType::Iso8859_16) => Some(CharacterEncoding::Iso8859_16),
        Some(EncodingType::Utf8) => Some(CharacterEncoding::Utf8),
        None => None,
    };
    let mut file_range_stream: FileRangeDataStream = FileRangeDataStream::new(arguments.offset);

    match file_range_stream.open(source) {
        Ok(_) => {}
        Err(error) => {
            println!("Unable to open file with error:\n{}", error);
            return ExitCode::FAILURE;
        }
    };
    let data_stream: DataStreamReference = Arc::new(RwLock::new(file_range_stream));

    let result: Option<FormatIdentifier> = match scan_for_formats(&data_stream) {
        Ok(result) => result,
        Err(error) => {
            println!(
                "Unable to scan data stream for known format signatures with error:\n{}",
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

    let result: Result<(), ErrorTrace> = match arguments.command {
        Some(Commands::Entry(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => ExtInfo::print_file_entry_by_identifier(
                &data_stream,
                command_arguments.entry,
                character_encoding.as_ref(),
            ),
            FormatIdentifier::Fat => {
                FatInfo::print_file_entry_by_identifier(&data_stream, command_arguments.entry)
            }
            FormatIdentifier::Ntfs => {
                NtfsInfo::print_file_entry_by_identifier(&data_stream, command_arguments.entry)
            }
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier.to_string()
            ))),
        },
        Some(Commands::Hierarchy(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                ExtInfo::print_hierarchy(&data_stream, character_encoding.as_ref())
            }
            FormatIdentifier::Fat => FatInfo::print_hierarchy(&data_stream),
            FormatIdentifier::Ntfs => NtfsInfo::print_hierarchy(&data_stream),
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier.to_string()
            ))),
        },
        Some(Commands::Path(command_arguments)) => {
            // TODO: detect leading partion path component and suggest/check path exists without
            // it.
            let path_components: Vec<&str> = if command_arguments.path.is_empty() {
                vec![]
            } else if command_arguments.path == "/" {
                vec![""]
            } else {
                command_arguments.path.split('/').collect()
            };
            match &format_identifier {
                FormatIdentifier::Ext => ExtInfo::print_file_entry_by_path(
                    &data_stream,
                    &path_components,
                    character_encoding.as_ref(),
                ),
                FormatIdentifier::Fat => {
                    FatInfo::print_file_entry_by_path(&data_stream, &path_components)
                }
                FormatIdentifier::Ntfs => {
                    NtfsInfo::print_file_entry_by_path(&data_stream, &path_components)
                }
                _ => Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format: {}",
                    format_identifier.to_string()
                ))),
            }
        }
        None => match &format_identifier {
            FormatIdentifier::Apm => ApmInfo::print_volume_system(&data_stream),
            FormatIdentifier::Ext => {
                ExtInfo::print_file_system(&data_stream, character_encoding.as_ref())
            }
            FormatIdentifier::Ewf => EwfInfo::print_image(&arguments.source),
            FormatIdentifier::Fat => FatInfo::print_file_system(&data_stream),
            FormatIdentifier::Gpt => GptInfo::print_volume_system(&data_stream),
            FormatIdentifier::Mbr => MbrInfo::print_volume_system(&data_stream),
            FormatIdentifier::Ntfs => NtfsInfo::print_file_system(&data_stream),
            FormatIdentifier::Qcow => QcowInfo::print_file(&data_stream),
            // TODO: add support for sparse bundle.
            FormatIdentifier::SparseImage => SparseImageInfo::print_file(&data_stream),
            FormatIdentifier::Udif => UdifInfo::print_file(&data_stream),
            FormatIdentifier::Vhd => VhdInfo::print_file(&data_stream),
            FormatIdentifier::Vhdx => VhdxInfo::print_file(&data_stream),
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier.to_string()
            ))),
        },
    };
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            println!("Unable to provide information about: {}\n{}", source, error);
            ExitCode::FAILURE
        }
    }
}
