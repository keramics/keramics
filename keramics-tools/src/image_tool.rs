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

use std::fmt;
use std::fmt::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::sync_channel;
use std::thread;

use clap::{Args, Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use sysinfo::System;

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::Path;
use keramics_formats::ntfs::NtfsAttribute;
use keramics_formats::ntfs::constants::NTFS_NAME_SPACE_DOS;
use keramics_hashes::{DigestHashContext, Md5Context, Sha1Context};
use keramics_types::Ucs2String;
use keramics_vfs::{
    VfsCredentialStore, VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFileType, VfsFinder,
    VfsLocation, VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanOptions,
    VfsScanner, VfsType,
};

#[cfg(feature = "debug-trace")]
use keramics_core::mediator::Mediator;

mod bodyfile;
mod display_path;
mod enums;
mod storage_media_image;

use crate::bodyfile::Bodyfile;
use crate::display_path::DisplayPath;
use crate::enums::DisplayPathType;
use crate::storage_media_image::StorageMediaImage;

pub const FILE_ATTRIBUTE_FLAG_READ_ONLY: u32 = 0x00000001;
pub const FILE_ATTRIBUTE_FLAG_SYSTEM: u32 = 0x00000004;

#[derive(Parser)]
#[command(version, about = "Analyzes the contents of a storage media image", long_about = None)]
struct CommandLineArguments {
    #[cfg(feature = "debug-trace")]
    #[arg(long, default_value_t = false)]
    /// Enable debug output
    debug: bool,

    #[arg(long)]
    /// Password to unlock storage media image
    password: Vec<String>,

    /// Path of the source file
    source: PathBuf,

    #[arg(long, default_value_t = false)]
    /// Stop when an error is encountered
    stop_on_error: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Output file entries in bodyfile format
    Bodyfile(BodyfileCommandArguments),

    /// Calculate digest hashes of a storage media image
    Hash(HashCommandArguments),

    /// Show the hierarchy of the volumes, partitions and file systems
    Hierarchy,
}

#[derive(Args, Debug)]
struct BodyfileCommandArguments {
    #[arg(long, default_value_t = false)]
    /// Calculate MD5 hashes of the content of file entries
    calculate_md5: bool,

    #[arg(long, default_value_t = 0)]
    /// Layer within the storage media image, where 1 represents the first layer
    image_layer: usize,

    /// Comma seperated list of partitions to include
    #[arg(long)]
    partitions: Option<String>,

    // TODO: allow to set the path component/segment separator
    // TODO: allow to set the data stream name separator
    /// Volume or partition path type
    #[arg(long, default_value_t = DisplayPathType::Index, value_enum)]
    volume_path_type: DisplayPathType,

    /// Comma seperated list of volumes to include
    #[arg(long)]
    volumes: Option<String>,
}

#[derive(Args, Debug)]
struct HashCommandArguments {
    #[arg(long, default_value_t = 0)]
    /// Layer within the storage media image, where 1 represents the first layer
    image_layer: usize,
}

/// File mode information.
struct FileModeInfo {
    /// Flags.
    file_mode: u32,
}

impl FileModeInfo {
    /// Creates new file mode information.
    fn new(file_mode: u32) -> Self {
        Self { file_mode }
    }

    /// Retrieves a file mode string representation.
    fn get_file_mode_string(file_mode: u32) -> String {
        let mut string_parts: Vec<&str> = vec!["-"; 10];

        if file_mode & 0x0001 != 0 {
            string_parts[9] = "x";
        }
        if file_mode & 0x0002 != 0 {
            string_parts[8] = "w";
        }
        if file_mode & 0x0004 != 0 {
            string_parts[7] = "r";
        }
        if file_mode & 0x0008 != 0 {
            string_parts[6] = "x";
        }
        if file_mode & 0x0010 != 0 {
            string_parts[5] = "w";
        }
        if file_mode & 0x0020 != 0 {
            string_parts[4] = "r";
        }
        if file_mode & 0x0040 != 0 {
            string_parts[3] = "x";
        }
        if file_mode & 0x0080 != 0 {
            string_parts[2] = "w";
        }
        if file_mode & 0x0100 != 0 {
            string_parts[1] = "r";
        }
        string_parts[0] = match file_mode & 0xf000 {
            0x1000 => "p",
            0x2000 => "c",
            0x4000 => "d",
            0x6000 => "b",
            0xa000 => "l",
            0xc000 => "s",
            _ => "-",
        };
        string_parts.join("")
    }
}

impl fmt::Display for FileModeInfo {
    /// Formats partition file mode information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: String = Self::get_file_mode_string(self.file_mode);

        write!(formatter, "{}", string)
    }
}

/// Tool for analyzing the contents of a storage media image.
struct ImageTool {
    /// The VFS resolver.
    vfs_resolver: VfsResolverReference,

    /// The display path.
    display_path: DisplayPath,

    /// Value to indicate to stop on error.
    pub stop_on_error: bool,
}

impl ImageTool {
    /// Creates a new tool.
    fn new(stop_on_error: bool) -> Self {
        let mut display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        // Escape | as \|
        display_path.translation_table.insert('|' as u32, "\\|");

        Self {
            vfs_resolver: VfsResolver::current(),
            display_path,
            stop_on_error,
        }
    }

    /// Output file entries in bodyfile format.
    fn generate_bodyfile(
        &self,
        source: &PathBuf,
        calculate_md5: bool,
        image_layer: usize,
        partitions: Option<&String>,
        volumes: Option<&String>,
    ) -> Result<(), ErrorTrace> {
        let mut vfs_scanner: VfsScanner = VfsScanner::new();

        match vfs_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build VFS scanner",
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
        if let Some(volumes_string) = volumes {
            match vfs_scan_options.parse_volumes(volumes_string.as_str()) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to parse volumes");
                    return Err(error);
                }
            }
        }
        // TODO: add scanner mediator.

        let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
        let vfs_location: VfsLocation = VfsLocation::from(source);

        match vfs_scanner.scan(&vfs_scan_options, &mut vfs_scan_context, &vfs_location) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to scan for file systems");
                return Err(error);
            }
        }
        println!("{}", Bodyfile::FILE_HEADER);

        match vfs_scan_context.root_node {
            Some(scan_node) => match self.print_scan_node_as_bodyfile(&scan_node, calculate_md5) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to print root scan node");
                    return Err(error);
                }
            },
            None => {}
        }
        Ok(())
    }

    /// Retrieves a file mode string representation of file attribute flags.
    fn get_file_mode_string_from_file_attribute_flags(
        file_type: &VfsFileType,
        file_attribute_flags: u32,
    ) -> String {
        let mut string_parts: Vec<&str> = vec!["-", "r", "w", "x", "r", "w", "x", "r", "w", "x"];

        string_parts[0] = match file_type {
            VfsFileType::Directory => "d",
            VfsFileType::SymbolicLink => "l",
            _ => "-",
        };
        if file_attribute_flags & FILE_ATTRIBUTE_FLAG_READ_ONLY != 0
            || file_attribute_flags & FILE_ATTRIBUTE_FLAG_SYSTEM != 0
        {
            string_parts[2] = "-";
            string_parts[5] = "-";
            string_parts[8] = "-";
        }
        string_parts.join("")
    }

    /// Retrieves a file mode string representation of a file type.
    fn get_file_mode_string_from_file_type(file_type: &VfsFileType) -> String {
        let mut string_parts: Vec<&str> = vec!["-", "r", "w", "x", "r", "w", "x", "r", "w", "x"];

        string_parts[0] = match file_type {
            VfsFileType::BlockDevice => "b",
            VfsFileType::CharacterDevice => "c",
            VfsFileType::Directory => "d",
            VfsFileType::NamedPipe => "p",
            VfsFileType::Socket => "s",
            VfsFileType::SymbolicLink => "l",
            VfsFileType::Whiteout => "w",
            _ => "-",
        };
        string_parts.join("")
    }

    /// Prints the file entry in bodyfile format.
    fn print_file_entry_as_bodyfile(
        &self,
        file_entry: &mut VfsFileEntry,
        file_system_display_path: &String,
        path: &Path,
        calculate_md5: bool,
    ) -> Result<(), ErrorTrace> {
        let md5: String = if !calculate_md5 {
            // TODO: consider changing to: String::from("N/A (skipped)")
            String::from("0")
        } else {
            // TODO: consider skipping $BadClus:$Bad
            match file_entry.get_data_stream() {
                Ok(Some(data_stream)) => match Bodyfile::calculate_md5(&data_stream) {
                    Ok(md5_string) => md5_string,
                    Err(mut error) => {
                        if self.stop_on_error {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to calculate MD5 of data stream"
                            );
                            return Err(error);
                        }
                        // TODO: consider changing to: String::from("N/A (error)")
                        String::from("00000000000000000000000000000000")
                    }
                },
                Ok(None) => {
                    // TODO: consider changing to: String::from("N/A (skipped)")
                    String::from("00000000000000000000000000000000")
                }
                Err(mut error) => {
                    if self.stop_on_error {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve data stream"
                        );
                        return Err(error);
                    }
                    // TODO: consider changing to: String::from("N/A (error)")
                    String::from("00000000000000000000000000000000")
                }
            }
        };
        let display_path: String = self.display_path.escape_path(path);

        let path_prefix: &str = if file_system_display_path.ends_with('/') {
            &file_system_display_path[..file_system_display_path.len() - 1]
        } else {
            file_system_display_path.as_str()
        };
        let path_suffix: String = match file_entry.get_symbolic_link_target() {
            Ok(Some(link_target)) => match file_entry {
                VfsFileEntry::Ntfs(_) => {
                    let display_link_target: String = link_target.components[1..]
                        .iter()
                        .map(|component| self.display_path.escape_path_component(component))
                        .collect::<Vec<String>>()
                        .join("/");

                    format!(" -> {}/{}", link_target.components[0], display_link_target)
                }
                _ => {
                    let display_link_target: String = self.display_path.escape_path(&link_target);

                    format!(" -> {}", display_link_target)
                }
            },
            Ok(None) => String::new(),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve symbolic link target"
                );
                return Err(error);
            }
        };
        let file_identifier: String = match file_entry {
            VfsFileEntry::Apfs(apfs_file_entry) => {
                format!("{}", apfs_file_entry.get_identifier())
            }
            VfsFileEntry::Ext(ext_file_entry) => {
                format!("{}", ext_file_entry.get_inode_number())
            }
            VfsFileEntry::Fat(fat_file_entry) => {
                format!("0x{:0x}", fat_file_entry.get_identifier())
            }
            VfsFileEntry::Hfs(hfs_file_entry) => {
                format!("{}", hfs_file_entry.get_identifier())
            }
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                // Note that the directory entry file reference can be differrent
                // from the values in the MFT entry.
                let file_reference: u64 = ntfs_file_entry.get_file_reference();

                format!(
                    "{}-{}",
                    file_reference & 0x0000ffffffffffff,
                    file_reference >> 48,
                )
            }
            _ => String::new(),
        };
        let file_type: VfsFileType = file_entry.get_file_type();

        let file_mode_string: String = match file_entry {
            VfsFileEntry::Apfs(_) | VfsFileEntry::Ext(_) | VfsFileEntry::Hfs(_) => {
                match file_entry.get_file_mode() {
                    Some(file_mode) => {
                        let file_mode_info: FileModeInfo = FileModeInfo::new(file_mode);

                        file_mode_info.to_string()
                    }
                    None => Self::get_file_mode_string_from_file_type(&file_type),
                }
            }
            VfsFileEntry::Fat(fat_file_entry) => {
                let file_attribute_flags: u8 = fat_file_entry.get_file_attribute_flags();

                Self::get_file_mode_string_from_file_attribute_flags(
                    &file_type,
                    file_attribute_flags as u32,
                )
            }
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let file_attribute_flags: u32 = ntfs_file_entry.get_file_attribute_flags();

                Self::get_file_mode_string_from_file_attribute_flags(
                    &file_type,
                    file_attribute_flags,
                )
            }
            _ => Self::get_file_mode_string_from_file_type(&file_type),
        };
        let owner_identifier: String = match file_entry.get_owner_identifier() {
            Some(owner_identifier) => format!("{}", owner_identifier),
            None => String::from(""),
        };
        let group_identifier: String = match file_entry.get_group_identifier() {
            Some(group_identifier) => format!("{}", group_identifier),
            None => String::from(""),
        };
        let size: u64 = file_entry.get_size();

        let access_time: String = match Bodyfile::format_as_timestamp(file_entry.get_access_time())
        {
            Ok(timestamp_string) => timestamp_string,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to format access time");
                return Err(error);
            }
        };
        let modification_time: String =
            match Bodyfile::format_as_timestamp(file_entry.get_modification_time()) {
                Ok(timestamp_string) => timestamp_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to format modification time"
                    );
                    return Err(error);
                }
            };
        let change_time: String = match Bodyfile::format_as_timestamp(file_entry.get_change_time())
        {
            Ok(timestamp_string) => timestamp_string,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to format change time");
                return Err(error);
            }
        };
        let creation_time: String =
            match Bodyfile::format_as_timestamp(file_entry.get_creation_time()) {
                Ok(timestamp_string) => timestamp_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to format creation time");
                    return Err(error);
                }
            };
        println!(
            "{}|{}{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            md5,
            path_prefix,
            display_path,
            path_suffix,
            file_identifier,
            file_mode_string,
            owner_identifier,
            group_identifier,
            size,
            access_time,
            modification_time,
            change_time,
            creation_time
        );
        let number_of_data_forks: usize = match file_entry.get_number_of_data_forks() {
            Ok(number_of_data_forks) => number_of_data_forks,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of data forks"
                );
                return Err(error);
            }
        };
        for data_fork_index in 0..number_of_data_forks {
            let data_fork: VfsDataFork = match file_entry.get_data_fork_by_index(data_fork_index) {
                Ok(number_of_data_forks) => number_of_data_forks,
                Err(mut error) => {
                    if self.stop_on_error {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve data fork: {}", data_fork_index)
                        );
                        return Err(error);
                    }
                    // TODO: report file entry containing error
                    continue;
                }
            };
            let data_fork_name: String = match &data_fork.get_name() {
                Some(name) => {
                    let escaped_name: String = self.display_path.escape_path_component(name);

                    format!(":{}", escaped_name)
                }
                None => continue,
            };
            let data_stream: &DataStreamReference = match data_fork.get_data_stream() {
                Ok(data_stream) => data_stream,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve data stream from data fork: {}",
                            data_fork_index
                        )
                    );
                    return Err(error);
                }
            };
            let md5: String = if !calculate_md5 {
                String::from("0")
            } else {
                match Bodyfile::calculate_md5(&data_stream) {
                    Ok(md5_string) => md5_string,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to calculate MD5 of data stream"
                        );
                        return Err(error);
                    }
                }
            };
            let data_stream_size: u64 = match data_stream.write() {
                Ok(mut data_stream) => match data_stream.get_size() {
                    Ok(size) => size,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to retrieve size");
                        return Err(error);
                    }
                },
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to obtain write lock on data stream",
                        error
                    ));
                }
            };
            println!(
                "{}|{}{}{}{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                md5,
                path_prefix,
                display_path,
                data_fork_name,
                path_suffix,
                file_identifier,
                file_mode_string,
                owner_identifier,
                group_identifier,
                data_stream_size,
                access_time,
                modification_time,
                change_time,
                creation_time
            );
        }
        match file_entry {
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                if let Some(parent_file_reference) = ntfs_file_entry.get_parent_file_reference() {
                    let name: Option<&Ucs2String> = ntfs_file_entry.get_name();
                    let number_of_attributes: usize = ntfs_file_entry.get_number_of_attributes();

                    // TODO: print index names
                    for attribute_index in 0..number_of_attributes {
                        let attribute: NtfsAttribute = match ntfs_file_entry
                            .get_attribute_by_index(attribute_index)
                        {
                            Ok(attribute) => attribute,
                            Err(mut error) => {
                                let file_reference: u64 = ntfs_file_entry.get_file_reference();

                                keramics_core::error_trace_add_frame!(
                                    error,
                                    format!(
                                        "Unable to retrieve NTFS MFT entry: {}-{} attribute: {}",
                                        file_reference & 0x0000ffffffffffff,
                                        file_reference >> 48,
                                        attribute_index
                                    )
                                );
                                return Err(error);
                            }
                        };
                        match attribute {
                            NtfsAttribute::FileName { file_name } => {
                                if file_name.get_parent_file_reference() != parent_file_reference
                                    || Some(file_name.get_name()) != name
                                {
                                    continue;
                                }
                                if file_name.get_name_space() == NTFS_NAME_SPACE_DOS {
                                    continue;
                                }
                                let file_name_access_time: String =
                                    match Bodyfile::format_as_timestamp(Some(
                                        file_name.get_access_time(),
                                    )) {
                                        Ok(timestamp_string) => timestamp_string,
                                        Err(mut error) => {
                                            keramics_core::error_trace_add_frame!(
                                                error,
                                                "Unable to format $FILE_NAME access time"
                                            );
                                            return Err(error);
                                        }
                                    };
                                let file_name_modification_time: String =
                                    match Bodyfile::format_as_timestamp(Some(
                                        file_name.get_modification_time(),
                                    )) {
                                        Ok(timestamp_string) => timestamp_string,
                                        Err(mut error) => {
                                            keramics_core::error_trace_add_frame!(
                                                error,
                                                "Unable to format $FILE_NAME modification time"
                                            );
                                            return Err(error);
                                        }
                                    };
                                let file_name_change_time: String =
                                    match Bodyfile::format_as_timestamp(Some(
                                        file_name.get_entry_modification_time(),
                                    )) {
                                        Ok(timestamp_string) => timestamp_string,
                                        Err(mut error) => {
                                            keramics_core::error_trace_add_frame!(
                                                error,
                                                "Unable to format $FILE_NAME entry modification time"
                                            );
                                            return Err(error);
                                        }
                                    };
                                let file_name_creation_time: String =
                                    match Bodyfile::format_as_timestamp(Some(
                                        file_name.get_creation_time(),
                                    )) {
                                        Ok(timestamp_string) => timestamp_string,
                                        Err(mut error) => {
                                            keramics_core::error_trace_add_frame!(
                                                error,
                                                "Unable to format $FILE_NAME creation time"
                                            );
                                            return Err(error);
                                        }
                                    };
                                println!(
                                    "{}|{}{} ($FILE_NAME)|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                                    md5,
                                    path_prefix,
                                    display_path,
                                    file_identifier,
                                    file_mode_string,
                                    owner_identifier,
                                    group_identifier,
                                    size,
                                    file_name_access_time,
                                    file_name_modification_time,
                                    file_name_change_time,
                                    file_name_creation_time
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Prints the scan node in bodyfile format.
    fn print_scan_node_as_bodyfile(
        &self,
        vfs_scan_node: &VfsScanNode,
        calculate_md5: bool,
    ) -> Result<(), ErrorTrace> {
        if vfs_scan_node.is_empty() {
            // Only process scan nodes that contain a file system.
            if !vfs_scan_node.is_file_system() {
                return Ok(());
            }
            let file_system: VfsFileSystemReference =
                match self.vfs_resolver.open_file_system(&vfs_scan_node.location) {
                    Ok(file_system) => file_system,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                        return Err(error);
                    }
                };
            let display_path: String = match vfs_scan_node.location.get_parent() {
                Some(parent_path) => match self.display_path.get_path(parent_path) {
                    Ok(path) => path,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve parent display path"
                        );
                        return Err(error);
                    }
                },
                None => String::new(),
            };
            let mut vfs_finder: VfsFinder = VfsFinder::new(&file_system);

            while let Some(result) = vfs_finder.next() {
                match result {
                    Ok((mut file_entry, path)) => {
                        match self.print_file_entry_as_bodyfile(
                            &mut file_entry,
                            &display_path,
                            &path,
                            calculate_md5,
                        ) {
                            Ok(_) => {}
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to print file entry"
                                );
                                return Err(error);
                            }
                        }
                    }
                    Err(mut error) => {
                        let path: &Path = vfs_finder.get_path();

                        if self.stop_on_error {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve file entry from finder: {}{}",
                                    display_path, path
                                )
                            );
                            return Err(error);
                        }
                        // TODO: report file entry containing error
                    }
                };
            }
        } else {
            for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
                match self.print_scan_node_as_bodyfile(sub_scan_node, calculate_md5) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to print sub scan node"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Retrieves a hierarchy prefix.
    fn get_hierarchy_prefix(&self, levels: &[bool]) -> String {
        let number_of_levels: usize = levels.len();
        let mut prefix: String = String::new();

        for (level, is_last) in levels[0..number_of_levels].iter().enumerate() {
            if level + 1 < number_of_levels {
                if *is_last {
                    prefix.push_str("    ");
                } else {
                    prefix.push_str("│   ");
                }
            } else {
                if *is_last {
                    prefix.push_str("└── ");
                } else {
                    prefix.push_str("├── ");
                }
            }
        }
        prefix
    }

    /// Prints the scan node as part of a hierarchy.
    fn print_scan_node_as_hierarchy(
        &self,
        vfs_scan_node: &VfsScanNode,
        levels: &mut Vec<bool>,
    ) -> Result<(), ErrorTrace> {
        let result: Option<VfsFileEntry> = match self
            .vfs_resolver
            .get_file_entry_by_location(&vfs_scan_node.location)
        {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        let prefix: String = self.get_hierarchy_prefix(levels);
        let path: &Path = vfs_scan_node.location.get_path();
        let vfs_type: &VfsType = vfs_scan_node.get_type();

        let path_string: String = match result.as_ref() {
            Some(file_entry) => match file_entry {
                VfsFileEntry::ApfsContainer(apfs_container_file_entry) => {
                    let path_string: String = match apfs_container_file_entry.get_volume_number() {
                        Some(volume_number) => format!("/apfs{}", volume_number),
                        _ => path.to_string(),
                    };
                    match apfs_container_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /apfs{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                VfsFileEntry::Gpt(gpt_file_entry) => {
                    let path_string: String = match gpt_file_entry.get_partition_number() {
                        Some(partition_number) => format!("/p{}", partition_number),
                        _ => path.to_string(),
                    };
                    match gpt_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /gpt{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                VfsFileEntry::LinuxLvm(lvm_file_entry) => {
                    let path_string: String = match lvm_file_entry.get_volume_number() {
                        Some(volume_number) => format!("/lvm{}", volume_number),
                        _ => path.to_string(),
                    };
                    match lvm_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /lvm{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry.get_partition_number() {
                    Some(partition_number) => format!("/p{}", partition_number),
                    None => path.to_string(),
                },
                VfsFileEntry::Pdi(pdi_file_entry) => {
                    let path_string: String = match pdi_file_entry.get_layer_number() {
                        Some(layer_number) => format!("/pdi{}", layer_number),
                        _ => path.to_string(),
                    };
                    match pdi_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /pdi{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                VfsFileEntry::Vhd(vhd_file_entry) => {
                    let path_string: String = match vhd_file_entry.get_layer_number() {
                        Some(layer_number) => format!("/vhd{}", layer_number),
                        _ => path.to_string(),
                    };
                    match vhd_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /vhd{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                VfsFileEntry::Vhdx(vhdx_file_entry) => {
                    let path_string: String = match vhdx_file_entry.get_layer_number() {
                        Some(layer_number) => format!("/vhdx{}", layer_number),
                        _ => path.to_string(),
                    };
                    match vhdx_file_entry.get_identifier() {
                        Some(identifier) => format!(
                            "{} (alias: /vhdx{{{}}})",
                            path_string,
                            identifier.to_string()
                        ),
                        _ => path_string,
                    }
                }
                _ => path.to_string(),
            },
            None => path.to_string(),
        };
        println!("{}{}: path: {}", prefix, vfs_type, path_string);

        let number_of_sub_nodes: usize = vfs_scan_node.sub_nodes.len();

        for (node_index, sub_scan_node) in vfs_scan_node.sub_nodes.iter().enumerate() {
            let is_last: bool = node_index + 1 == number_of_sub_nodes;

            levels.push(is_last);

            self.print_scan_node_as_hierarchy(sub_scan_node, levels)?;

            levels.pop();
        }
        Ok(())
    }

    /// Scans and prints the hierarchy of volumes, partitions and file systems.
    fn scan_for_hierarchy(&self, source: &PathBuf) -> Result<(), ErrorTrace> {
        let vfs_scan_options: VfsScanOptions = VfsScanOptions::new();

        let mut vfs_scanner: VfsScanner = VfsScanner::new();

        match vfs_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build VFS scanner",
                    error
                ));
            }
        }
        let mut vfs_scan_context: VfsScanContext = VfsScanContext::new();
        let vfs_location: VfsLocation = VfsLocation::from(source);

        match vfs_scanner.scan(&vfs_scan_options, &mut vfs_scan_context, &vfs_location) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to scan for file systems");
                return Err(error);
            }
        }
        // TODO: print source type.

        let mut levels: Vec<bool> = Vec::new();

        match vfs_scan_context.root_node {
            Some(scan_node) => match self.print_scan_node_as_hierarchy(&scan_node, &mut levels) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to print root scan node");
                    return Err(error);
                }
            },
            None => {}
        }
        println!();

        Ok(())
    }

    /// Sets the volume path type.
    pub fn set_volume_path_type(&mut self, volume_path_type: &DisplayPathType) {
        self.display_path.set_volume_path_type(volume_path_type);
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
    #[cfg(feature = "debug-trace")]
    {
        Mediator {
            debug_output: arguments.debug,
        }
        .make_current();
    }
    let vfs_credential_store: &VfsCredentialStore = VfsCredentialStore::current();

    for password in arguments.password.iter() {
        match vfs_credential_store.add_passphrase(password.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "Unable to add passphrase to credential store with error:\n{}",
                    error
                );
                return ExitCode::FAILURE;
            }
        }
    }
    let mut image_tool: ImageTool = ImageTool::new(arguments.stop_on_error);

    match arguments.command {
        Some(Commands::Bodyfile(command_arguments)) => {
            image_tool.set_volume_path_type(&command_arguments.volume_path_type);

            match image_tool.generate_bodyfile(
                &arguments.source,
                command_arguments.calculate_md5,
                command_arguments.image_layer,
                command_arguments.partitions.as_ref(),
                command_arguments.volumes.as_ref(),
            ) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "Unable to generate bodyfile of: {}\n{}",
                        source_string, error
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
        Some(Commands::Hash(command_arguments)) => {
            let storage_media_image: StorageMediaImage =
                match StorageMediaImage::open(&arguments.source, command_arguments.image_layer) {
                    Ok(storage_media_image) => storage_media_image,
                    Err(error) => {
                        println!(
                            "Unable to open storage media image: {} with error:\n{}",
                            source_string, error
                        );
                        return ExitCode::FAILURE;
                    }
                };
            let data_stream: DataStreamReference = match storage_media_image.get_data_stream() {
                Some(data_stream) => data_stream,
                None => {
                    println!("Unable to retrieve data stream\n");
                    return ExitCode::FAILURE;
                }
            };
            let media_size: u64 = match data_stream.write() {
                Ok(mut data_stream) => match data_stream.get_size() {
                    Ok(size) => size,
                    Err(error) => {
                        println!("Unable to determine media size with error:\n{}", error);
                        return ExitCode::FAILURE;
                    }
                },
                Err(error) => {
                    println!(
                        "Unable to obtain write lock on data stream with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
            let stored_md5_hash: Option<Vec<u8>> = match storage_media_image.get_md5_hash() {
                Ok(Some(stored_hash)) => {
                    if stored_hash != [0; 16] {
                        Some(stored_hash)
                    } else {
                        None
                    }
                }
                Ok(None) => None,
                Err(error) => {
                    println!("Unable to retrieve stored MD5 hash with error:\n{}", error);
                    return ExitCode::FAILURE;
                }
            };
            let stored_sha1_hash: Option<Vec<u8>> = match storage_media_image.get_sha1_hash() {
                Ok(Some(stored_hash)) => {
                    if stored_hash != [0; 20] {
                        Some(stored_hash)
                    } else {
                        None
                    }
                }
                Ok(None) => None,
                Err(error) => {
                    println!("Unable to retrieve stored SHA1 hash with error:\n{}", error);
                    return ExitCode::FAILURE;
                }
            };
            let calculate_md5_hash: bool = stored_md5_hash.is_some() || stored_sha1_hash.is_none();
            let calculate_sha1_hash: bool = stored_sha1_hash.is_some();

            let multi_progress: MultiProgress = MultiProgress::new();

            let reader_progress_bar_template: &str = concat!(
                "Reading at {percent}% [{wide_bar}] ",
                "{bytes}/{total_bytes} ({binary_bytes_per_sec}) ",
                "elapsed: {elapsed_precise} (remaining: {eta_precise})",
            );
            let reader_progress_bar_style: ProgressStyle =
                match ProgressStyle::with_template(reader_progress_bar_template) {
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
            let read_progress_bar: ProgressBar = multi_progress.add(ProgressBar::new(media_size));
            read_progress_bar.set_style(reader_progress_bar_style.progress_chars("#>-"));

            let md5_progress_bar: ProgressBar = if !calculate_md5_hash {
                ProgressBar::hidden()
            } else {
                let md5_progress_bar_template: &str = concat!(
                    "MD5 at {percent}% [{wide_bar}] ",
                    "{bytes}/{total_bytes} ({binary_bytes_per_sec}) ",
                    "elapsed: {elapsed_precise} (remaining: {eta_precise})",
                );
                let md5_progress_bar_style: ProgressStyle =
                    match ProgressStyle::with_template(md5_progress_bar_template) {
                        Ok(style) => style.with_key(
                            "eta",
                            |state: &ProgressState, writer: &mut dyn Write| {
                                write!(writer, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                            },
                        ),
                        Err(error) => {
                            println!(
                                "Unable to create progress bar style from template with error: {}",
                                error
                            );
                            return ExitCode::FAILURE;
                        }
                    };
                let progress_bar: ProgressBar = multi_progress.add(ProgressBar::new(media_size));
                progress_bar.set_style(md5_progress_bar_style.progress_chars("#>-"));
                progress_bar
            };
            let sha1_progress_bar: ProgressBar = if !calculate_sha1_hash {
                ProgressBar::hidden()
            } else {
                let sha1_progress_bar_template: &str = concat!(
                    "SHA1 at {percent}% [{wide_bar}] ",
                    "{bytes}/{total_bytes} ({binary_bytes_per_sec}) ",
                    "elapsed: {elapsed_precise} (remaining: {eta_precise})",
                );
                let sha1_progress_bar_style: ProgressStyle =
                    match ProgressStyle::with_template(sha1_progress_bar_template) {
                        Ok(style) => style.with_key(
                            "eta",
                            |state: &ProgressState, writer: &mut dyn Write| {
                                write!(writer, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                            },
                        ),
                        Err(error) => {
                            println!(
                                "Unable to create progress bar style from template with error: {}",
                                error
                            );
                            return ExitCode::FAILURE;
                        }
                    };
                let progress_bar: ProgressBar = multi_progress.add(ProgressBar::new(media_size));
                progress_bar.set_style(sha1_progress_bar_style.progress_chars("#>-"));
                progress_bar
            };
            let mut system: System = System::new_all();
            system.refresh_memory();

            // Limit channels at 80% of available memory.
            let channel_limit: usize =
                ((system.available_memory() as usize) * 80) / (65536 * 2 * 100);

            let (md5_sender, md5_receiver) = sync_channel::<Vec<u8>>(channel_limit);

            let md5_thread = thread::spawn(move || {
                let mut md5_context: Md5Context = Md5Context::new();

                while let Ok(buffer) = md5_receiver.recv() {
                    md5_context.update(&buffer);
                    md5_progress_bar.inc(buffer.len() as u64);
                }
                md5_progress_bar.finish();
                md5_context
            });
            let (sha1_sender, sha1_receiver) = sync_channel::<Vec<u8>>(channel_limit);

            let sha1_thread = thread::spawn(move || {
                let mut sha1_context: Sha1Context = Sha1Context::new();

                while let Ok(buffer) = sha1_receiver.recv() {
                    sha1_context.update(&buffer);
                    sha1_progress_bar.inc(buffer.len() as u64);
                }
                sha1_progress_bar.finish();
                sha1_context
            });
            let mut media_offset: u64 = 0;
            let mut data: [u8; 65536] = [0; 65536];

            match data_stream.write() {
                Ok(mut data_stream) => loop {
                    let read_count = match data_stream.read(&mut data) {
                        Ok(read_count) => read_count,
                        Err(error) => {
                            println!(
                                "Unable to read data at offset {} with error:\n{}",
                                media_offset, error
                            );
                            return ExitCode::FAILURE;
                        }
                    };
                    if read_count == 0 {
                        break;
                    }
                    if calculate_md5_hash {
                        _ = md5_sender.send(data[0..read_count].to_vec());
                    }
                    if calculate_sha1_hash {
                        _ = sha1_sender.send(data[0..read_count].to_vec());
                    }
                    media_offset += read_count as u64;

                    read_progress_bar.set_position(media_offset);
                },
                Err(error) => {
                    println!(
                        "Unable to obtain write lock on data stream with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
            read_progress_bar.finish();

            drop(md5_sender);
            drop(sha1_sender);

            // Both threads need to be completed before printing the results.
            let mut md5_context: Md5Context = md5_thread.join().expect("MD5 thread panicked");
            let mut sha1_context: Sha1Context = sha1_thread.join().expect("SHA1 thread panicked");

            let mut md5_hash_mismatch: bool = false;

            if calculate_md5_hash {
                let md5_hash: Vec<u8> = md5_context.finalize();

                let hash_string: String = format_as_string(&md5_hash);
                println!("\nCalculated MD5 hash\t: {}", hash_string);

                if let Some(stored_hash) = stored_md5_hash {
                    let hash_string: String = format_as_string(&stored_hash);
                    println!("Stored MD5 hash\t\t: {}", hash_string);

                    if stored_hash != md5_hash.as_slice() {
                        md5_hash_mismatch = true;
                    }
                }
            }
            let mut sha1_hash_mismatch: bool = false;

            if calculate_sha1_hash {
                let sha1_hash: Vec<u8> = sha1_context.finalize();

                let hash_string: String = format_as_string(&sha1_hash);
                println!("\nCalculated SHA1 hash\t: {}", hash_string);

                if let Some(stored_hash) = stored_sha1_hash {
                    let hash_string: String = format_as_string(&stored_hash);
                    println!("Stored SHA1 hash\t: {}", hash_string);

                    if stored_hash != sha1_hash.as_slice() {
                        sha1_hash_mismatch = true;
                    }
                }
            }
            if md5_hash_mismatch || sha1_hash_mismatch {
                println!("\nMismatch between calculated and stored hashes");
                return ExitCode::FAILURE;
            }
        }
        _ => match image_tool.scan_for_hierarchy(&arguments.source) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "Unable to determine hierarchy of: {}\n{}",
                    source_string, error
                );
                return ExitCode::FAILURE;
            }
        },
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_mode_string() {
        let string: String = FileModeInfo::get_file_mode_string(0x1000);
        assert_eq!(string, "p---------");

        let string: String = FileModeInfo::get_file_mode_string(0x2000);
        assert_eq!(string, "c---------");

        let string: String = FileModeInfo::get_file_mode_string(0x4000);
        assert_eq!(string, "d---------");

        let string: String = FileModeInfo::get_file_mode_string(0x6000);
        assert_eq!(string, "b---------");

        let string: String = FileModeInfo::get_file_mode_string(0xa000);
        assert_eq!(string, "l---------");

        let string: String = FileModeInfo::get_file_mode_string(0xc000);
        assert_eq!(string, "s---------");

        let string: String = FileModeInfo::get_file_mode_string(0x81ff);
        assert_eq!(string, "-rwxrwxrwx");
    }

    #[test]
    fn test_file_mode_information_fmt() {
        let test_struct: FileModeInfo = FileModeInfo::new(0x81a4);
        let string: String = test_struct.to_string();
        assert_eq!(string, "-rw-r--r--");
    }

    #[test]
    fn test_get_file_mode_string_from_file_attribute_flags() {
        let string: String = ImageTool::get_file_mode_string_from_file_attribute_flags(
            &VfsFileType::File,
            0x00000020,
        );
        assert_eq!(string, "-rwxrwxrwx");

        let string: String = ImageTool::get_file_mode_string_from_file_attribute_flags(
            &VfsFileType::File,
            0x00000006,
        );
        assert_eq!(string, "-r-xr-xr-x");

        let string: String = ImageTool::get_file_mode_string_from_file_attribute_flags(
            &VfsFileType::Directory,
            0x00000020,
        );
        assert_eq!(string, "drwxrwxrwx");

        let string: String = ImageTool::get_file_mode_string_from_file_attribute_flags(
            &VfsFileType::SymbolicLink,
            0x00000020,
        );
        assert_eq!(string, "lrwxrwxrwx");
    }

    #[test]
    fn test_get_file_mode_string_from_file_type() {
        let string: String =
            ImageTool::get_file_mode_string_from_file_type(&VfsFileType::BlockDevice);
        assert_eq!(string, "brwxrwxrwx");

        let string: String =
            ImageTool::get_file_mode_string_from_file_type(&VfsFileType::CharacterDevice);
        assert_eq!(string, "crwxrwxrwx");

        let string: String =
            ImageTool::get_file_mode_string_from_file_type(&VfsFileType::Directory);
        assert_eq!(string, "drwxrwxrwx");

        let string: String = ImageTool::get_file_mode_string_from_file_type(&VfsFileType::File);
        assert_eq!(string, "-rwxrwxrwx");

        let string: String =
            ImageTool::get_file_mode_string_from_file_type(&VfsFileType::NamedPipe);
        assert_eq!(string, "prwxrwxrwx");

        let string: String = ImageTool::get_file_mode_string_from_file_type(&VfsFileType::Socket);
        assert_eq!(string, "srwxrwxrwx");

        let string: String =
            ImageTool::get_file_mode_string_from_file_type(&VfsFileType::SymbolicLink);
        assert_eq!(string, "lrwxrwxrwx");

        let string: String = ImageTool::get_file_mode_string_from_file_type(&VfsFileType::Whiteout);
        assert_eq!(string, "wrwxrwxrwx");
    }

    // TODO: add more tests.
}
