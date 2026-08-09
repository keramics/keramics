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

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};
use clap_num::maybe_hex;

use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_encodings::CharacterEncoding;
use keramics_formats::{FormatIdentifier, FormatScanner, Path};

mod enums;
mod formatters;
mod info;
mod range_stream;
mod storage_media_image;

use crate::enums::EncodingType;
use crate::info::{
    ApfsInfo, ApmInfo, EwfInfo, ExtInfo, FatInfo, GptInfo, HfsInfo, MbrInfo, NtfsInfo, PdiInfo,
    QcowInfo, SparseBundleInfo, SparseImageInfo, UdifInfo, VhdInfo, VhdxInfo, VmdkInfo,
};
use crate::range_stream::RangeDataStream;
use crate::storage_media_image::StorageMediaImage;

#[derive(Parser)]
#[command(version, about = "Provides information about file formats", long_about = None)]
struct CommandLineArguments {
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    /// Character encoding
    #[arg(long, value_enum)]
    encoding: Option<EncodingType>,

    #[arg(long, default_value_t = false)]
    /// Process storage media image contents
    image: bool,

    #[arg(short, long, default_value_t = 0, value_parser=maybe_hex::<u64>)]
    /// Offset within the source file or storage media
    offset: u64,

    #[arg(long)]
    /// Password to unlock format
    password: Vec<String>,

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

/// Tool for providing information about file formats.
struct InfoTool {
    /// Character encoding.
    character_encoding: Option<CharacterEncoding>,

    /// Image mode.
    image_mode: bool,

    /// Offset.
    offset: u64,
}

impl InfoTool {
    /// Creates a new info tool.
    fn new(encoding_type: &Option<EncodingType>, image_mode: bool, offset: u64) -> InfoTool {
        let character_encoding: Option<CharacterEncoding> = match encoding_type {
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
            Some(EncodingType::Koi8R) => Some(CharacterEncoding::Koi8R),
            Some(EncodingType::Koi8U) => Some(CharacterEncoding::Koi8U),
            Some(EncodingType::Utf8) => Some(CharacterEncoding::Utf8),
            Some(EncodingType::Windows874) => Some(CharacterEncoding::Windows874),
            Some(EncodingType::Windows932) => Some(CharacterEncoding::Windows932),
            Some(EncodingType::Windows936) => Some(CharacterEncoding::Windows936),
            Some(EncodingType::Windows949) => Some(CharacterEncoding::Windows949),
            Some(EncodingType::Windows950) => Some(CharacterEncoding::Windows950),
            Some(EncodingType::Windows1250) => Some(CharacterEncoding::Windows1250),
            Some(EncodingType::Windows1251) => Some(CharacterEncoding::Windows1251),
            Some(EncodingType::Windows1252) => Some(CharacterEncoding::Windows1252),
            Some(EncodingType::Windows1253) => Some(CharacterEncoding::Windows1253),
            Some(EncodingType::Windows1254) => Some(CharacterEncoding::Windows1254),
            Some(EncodingType::Windows1255) => Some(CharacterEncoding::Windows1255),
            Some(EncodingType::Windows1256) => Some(CharacterEncoding::Windows1256),
            Some(EncodingType::Windows1257) => Some(CharacterEncoding::Windows1257),
            Some(EncodingType::Windows1258) => Some(CharacterEncoding::Windows1258),
            None => None,
        };
        InfoTool {
            character_encoding,
            image_mode,
            offset,
        }
    }

    /// Retrieves a data stream.
    pub fn get_data_stream(
        &self,
        path: &PathBuf,
        passwords: &Vec<String>,
    ) -> Result<DataStreamReference, ErrorTrace> {
        let data_stream: DataStreamReference = if self.image_mode {
            match StorageMediaImage::open(path, passwords) {
                Ok(storage_media_image) => storage_media_image.get_data_stream(),
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to open storage media image",
                        error
                    ));
                }
            }
        } else {
            match open_os_data_stream(path) {
                Ok(data_stream) => data_stream,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
                    return Err(error);
                }
            }
        };
        if self.offset == 0 {
            return Ok(data_stream);
        }
        let mut range_data_stream: RangeDataStream = RangeDataStream::new(data_stream, self.offset);

        match range_data_stream.open() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open range data stream");
                return Err(error);
            }
        }
        Ok(Arc::new(RwLock::new(range_data_stream)))
    }

    /// Scans a data stream for format signatures.
    fn scan_for_formats(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        let mut format_scanner: FormatScanner = FormatScanner::new();

        if !self.image_mode {
            format_scanner.add_ewf_signatures();
            format_scanner.add_pdi_signatures();
            format_scanner.add_qcow_signatures();
            // TODO: add support for sparse bundle Info.plist.
            format_scanner.add_sparseimage_signatures();
            format_scanner.add_udif_signatures();
            format_scanner.add_vhd_signatures();
            format_scanner.add_vhdx_signatures();
            format_scanner.add_vmdk_signatures();
            // TODO: add support for individual VMDK sparse file.
            // TODO: add support for individual VMDK sparse COWD file.
        }
        format_scanner.add_apfs_signatures();
        format_scanner.add_apm_signatures();
        format_scanner.add_ext_signatures();
        format_scanner.add_fat_signatures();
        format_scanner.add_hfs_signatures();
        format_scanner.add_gpt_signatures();
        format_scanner.add_ntfs_signatures();

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
        let mut result: Option<FormatIdentifier> = None;

        if scan_results.len() > 1 {
            if scan_results.contains(&FormatIdentifier::Udif) {
                // Check if UDIF footer was detected.
                let mut scan_results_copy: HashSet<FormatIdentifier> = scan_results.clone();
                scan_results_copy.remove(&FormatIdentifier::Udif);

                if scan_results_copy.len() == 1 {
                    result = match scan_results_copy.iter().next() {
                        Some(format_identifier) => {
                            if format_identifier == &FormatIdentifier::Unknown {
                                None
                            } else if !format_identifier.is_storage_media_image_format() {
                                Some(FormatIdentifier::Udif)
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                }
            } else if scan_results.contains(&FormatIdentifier::Vhd) {
                // Check if VHD footer was detected.
                let mut scan_results_copy: HashSet<FormatIdentifier> = scan_results.clone();
                scan_results_copy.remove(&FormatIdentifier::Vhd);

                if scan_results_copy.len() == 1 {
                    result = match scan_results_copy.iter().next() {
                        Some(format_identifier) => {
                            if format_identifier == &FormatIdentifier::Unknown {
                                None
                            } else if !format_identifier.is_storage_media_image_format() {
                                Some(FormatIdentifier::Vhd)
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                }
            }
            if result.is_none() {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported multiple known format signatures {:?}",
                    scan_results
                )));
            }
        } else {
            result = scan_results.iter().next().cloned();
        }
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
            scan_results = match format_scanner.scan_data_stream(data_stream) {
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
            result = scan_results.iter().next().cloned();
        }
        Ok(result)
    }
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
    let info_tool: InfoTool = InfoTool::new(&arguments.encoding, arguments.image, arguments.offset);

    // TODO: bundle all credentials into 1 credential store argument.
    let data_stream: DataStreamReference =
        match info_tool.get_data_stream(&arguments.source, &arguments.password) {
            Ok(data_stream) => data_stream,
            Err(error) => {
                println!("Unable to open data stream with error:\n{}", error);
                return ExitCode::FAILURE;
            }
        };
    let format_identifier: FormatIdentifier = if !arguments.image
        && arguments.source.is_dir()
        && arguments.source.extension() == Some("sparsebundle".as_ref())
    {
        FormatIdentifier::SparseBundle
    } else {
        match info_tool.scan_for_formats(&data_stream) {
            Ok(Some(format_identifier)) => format_identifier,
            Ok(None) => {
                println!("No known format signatures found");
                return ExitCode::FAILURE;
            }
            Err(error) => {
                println!(
                    "Unable to scan data stream for known format signatures with error:\n{}",
                    error
                );
                return ExitCode::FAILURE;
            }
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
                info_tool.character_encoding.as_ref(),
            ),
            FormatIdentifier::Fat => {
                FatInfo::print_file_entry_by_identifier(&data_stream, command_arguments.entry)
            }
            FormatIdentifier::Hfs => {
                HfsInfo::print_file_entry_by_identifier(&data_stream, command_arguments.entry)
            }
            FormatIdentifier::Ntfs => {
                NtfsInfo::print_file_entry_by_identifier(&data_stream, command_arguments.entry)
            }
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier
            ))),
        },
        Some(Commands::Hierarchy(command_arguments)) => match &format_identifier {
            FormatIdentifier::Ext => {
                ExtInfo::print_hierarchy(&data_stream, info_tool.character_encoding.as_ref())
            }
            FormatIdentifier::Fat => FatInfo::print_hierarchy(&data_stream),
            FormatIdentifier::Hfs => HfsInfo::print_hierarchy(&data_stream),
            FormatIdentifier::Ntfs => NtfsInfo::print_hierarchy(&data_stream),
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier
            ))),
        },
        Some(Commands::Path(command_arguments)) => {
            // TODO: detect leading partion path component and suggest/check path exists without
            // it.
            let path: Path = Path::from(&command_arguments.path);

            match &format_identifier {
                FormatIdentifier::Ext => ExtInfo::print_file_entry_by_path(
                    &data_stream,
                    &path,
                    info_tool.character_encoding.as_ref(),
                ),
                FormatIdentifier::Fat => FatInfo::print_file_entry_by_path(&data_stream, &path),
                FormatIdentifier::Hfs => HfsInfo::print_file_entry_by_path(&data_stream, &path),
                FormatIdentifier::Ntfs => NtfsInfo::print_file_entry_by_path(&data_stream, &path),
                _ => Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format: {}",
                    format_identifier
                ))),
            }
        }
        None => match &format_identifier {
            FormatIdentifier::Apfs => ApfsInfo::print_container(&data_stream),
            FormatIdentifier::Apm => ApmInfo::print_volume_system(&data_stream),
            // TODO: add support for individual EWF segment file.
            FormatIdentifier::Ewf => EwfInfo::print_image(&arguments.source),
            FormatIdentifier::Ext => {
                ExtInfo::print_file_system(&data_stream, info_tool.character_encoding.as_ref())
            }
            FormatIdentifier::Fat => FatInfo::print_file_system(&data_stream),
            FormatIdentifier::Hfs => HfsInfo::print_file_system(&data_stream),
            FormatIdentifier::Gpt => GptInfo::print_volume_system(&data_stream),
            FormatIdentifier::Mbr => MbrInfo::print_volume_system(&data_stream),
            FormatIdentifier::Ntfs => NtfsInfo::print_file_system(&data_stream),
            // TODO: add support for individual sparse Pdi file.
            FormatIdentifier::Pdi => PdiInfo::print_image(&arguments.source),
            // TODO: add support for QCOW image.
            FormatIdentifier::Qcow => QcowInfo::print_file(&data_stream),
            FormatIdentifier::SparseBundle => SparseBundleInfo::print_image(&arguments.source),
            FormatIdentifier::SparseImage => SparseImageInfo::print_file(&data_stream),
            // TODO: bundle all credentials into 1 credential store argument.
            FormatIdentifier::Udif => UdifInfo::print(&arguments.source, &arguments.password),
            // TODO: add support for VHD image.
            FormatIdentifier::Vhd => VhdInfo::print_file(&data_stream),
            // TODO: add support for VHDX image.
            FormatIdentifier::Vhdx => VhdxInfo::print_file(&data_stream),
            // TODO: add support for individual VMDK file.
            FormatIdentifier::Vmdk => VmdkInfo::print_image(&arguments.source),
            _ => Err(keramics_core::error_trace_new!(format!(
                "Unsupported format: {}",
                format_identifier
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
