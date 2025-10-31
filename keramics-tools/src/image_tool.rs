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
use std::io::SeekFrom;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use keramics_core::formatters::format_as_string;
use keramics_core::mediator::Mediator;
use keramics_core::{DataStream, DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::ewf::EwfImage;
use keramics_formats::ntfs::NtfsAttribute;
use keramics_formats::qcow::{QcowImage, QcowImageLayer};
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::udif::UdifFile;
use keramics_formats::vhd::{VhdImage, VhdImageLayer};
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_formats::{FormatIdentifier, FormatScanner};
use keramics_hashes::{DigestHashContext, Md5Context};
use keramics_types::Ucs2String;
use keramics_vfs::{
    VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFileType, VfsFinder, VfsLocation,
    VfsPath, VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner, VfsString,
    VfsType, new_os_vfs_location,
};

mod bodyfile;
mod display_path;
mod enums;

use crate::bodyfile::Bodyfile;
use crate::display_path::DisplayPath;
use crate::enums::DisplayPathType;

pub const FILE_ATTRIBUTE_FLAG_READ_ONLY: u32 = 0x00000001;
pub const FILE_ATTRIBUTE_FLAG_SYSTEM: u32 = 0x00000004;

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
    /// Output file entries in bodyfile format
    Bodyfile(BodyfileCommandArguments),

    /// Calculate digest hashes of a storage media image
    Hash,

    /// Show the hierarchy of the volumes, partitions and file systems
    Hierarchy,
}

#[derive(Args, Debug)]
struct BodyfileCommandArguments {
    #[arg(long, default_value_t = false)]
    /// Calculate MD5 hashes of the content of file entries
    calculate_md5: bool,

    // TODO: allow to set the path component/segment separator

    // TODO: allow to set the data stream name separator
    /// Volume or partition path type
    #[arg(long, default_value_t = DisplayPathType::Index, value_enum)]
    volume_path_type: DisplayPathType,
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
    fn get_base_path_and_file_name<'a>(
        path: &'a PathBuf,
    ) -> Result<(PathBuf, &'a str), ErrorTrace> {
        let mut base_path: PathBuf = path.clone();
        base_path.pop();

        let file_name: &str = match path.file_name() {
            Some(file_name_path) => match file_name_path.to_str() {
                Some(path_string) => path_string,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported source - invalid file name"
                    ));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported source - missing file name"
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
    fn open(&mut self, path: &PathBuf) -> Result<(), ErrorTrace> {
        match self {
            StorageMediaImage::Ewf(ewf_image) => {
                let (base_path, file_name) =
                    match StorageMediaImage::get_base_path_and_file_name(path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            // TODO: get printable version of path instead of using display().
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine base path and file name of path: {}",
                                    path.display()
                                )
                            );
                            return Err(error);
                        }
                    };
                let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
                    Ok(file_resolver) => file_resolver,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to create file resolver for path: {}",
                                base_path.display()
                            )
                        );
                        return Err(error);
                    }
                };
                let path_component: PathComponent = PathComponent::from(file_name);
                match ewf_image.open(&file_resolver, &path_component) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                        return Err(error);
                    }
                }
            }
            StorageMediaImage::Qcow(qcow_image) => {
                let (base_path, file_name) =
                    match StorageMediaImage::get_base_path_and_file_name(path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            // TODO: get printable version of path instead of using display().
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine base path and file name of path: {}",
                                    path.display()
                                )
                            );
                            return Err(error);
                        }
                    };
                let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
                    Ok(file_resolver) => file_resolver,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to create file resolver for base path: {}",
                                base_path.display()
                            )
                        );
                        return Err(error);
                    }
                };
                let path_component: PathComponent = PathComponent::from(file_name);
                match qcow_image.open(&file_resolver, &path_component) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                        return Err(error);
                    }
                }
            }
            StorageMediaImage::SparseImage(file) => {
                let data_stream: DataStreamReference = match open_os_data_stream(path) {
                    Ok(data_stream) => data_stream,
                    Err(mut error) => {
                        // TODO: get printable version of path instead of using display().
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open data stream: {}", path.display())
                        );
                        return Err(error);
                    }
                };
                match file.read_data_stream(&data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read sparseimage from data stream"
                        );
                        return Err(error);
                    }
                }
            }
            StorageMediaImage::Udif(file) => {
                let data_stream: DataStreamReference = match open_os_data_stream(path) {
                    Ok(data_stream) => data_stream,
                    Err(mut error) => {
                        // TODO: get printable version of path instead of using display().
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to open data stream: {}", path.display())
                        );
                        return Err(error);
                    }
                };
                match file.read_data_stream(&data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read UDIF image from data stream"
                        );
                        return Err(error);
                    }
                }
            }
            StorageMediaImage::Vhd(vhd_image) => {
                let (base_path, file_name) =
                    match StorageMediaImage::get_base_path_and_file_name(path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            // TODO: get printable version of path instead of using display().
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine base path and file name of path: {}",
                                    path.display()
                                )
                            );
                            return Err(error);
                        }
                    };
                let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
                    Ok(file_resolver) => file_resolver,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to create file resolver for base path: {}",
                                base_path.display()
                            )
                        );
                        return Err(error);
                    }
                };
                let path_component: PathComponent = PathComponent::from(file_name);
                match vhd_image.open(&file_resolver, &path_component) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHD image");
                        return Err(error);
                    }
                }
            }
            StorageMediaImage::Vhdx(vhdx_image) => {
                let (base_path, file_name) =
                    match StorageMediaImage::get_base_path_and_file_name(path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            // TODO: get printable version of path instead of using display().
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine base path and file name of path: {}",
                                    path.display()
                                )
                            );
                            return Err(error);
                        }
                    };
                let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
                    Ok(file_resolver) => file_resolver,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to create file resolver for base path: {}",
                                base_path.display()
                            )
                        );
                        return Err(error);
                    }
                };
                let path_component: PathComponent = PathComponent::from(file_name);
                match vhdx_image.open(&file_resolver, &path_component) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}

impl DataStream for StorageMediaImage {
    /// Retrieves the size of the data.
    fn get_size(&mut self) -> Result<u64, ErrorTrace> {
        match self {
            StorageMediaImage::Ewf(image) => image.get_size(),
            // TODO: add Qcow layer support.
            StorageMediaImage::SparseImage(file) => file.get_size(),
            StorageMediaImage::Udif(file) => file.get_size(),
            // TODO: add Vhd layer support.
            // TODO: add Vhdx layer support.
            _ => todo!(),
        }
    }

    /// Reads data at the current position.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ErrorTrace> {
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

    /// Sets the current position of the data.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ErrorTrace> {
        match self {
            StorageMediaImage::Ewf(image) => image.seek(pos),
            // TODO: add Qcow layer support.
            StorageMediaImage::SparseImage(file) => file.seek(pos),
            StorageMediaImage::Udif(file) => file.seek(pos),
            // TODO: add Vhd layer support.
            // TODO: add Vhdx layer support.
            _ => todo!(),
        }
    }
}

/// Tool for analyzing the contents of a storage media image.
struct ImageTool {
    /// The display path.
    display_path: DisplayPath,
}

impl ImageTool {
    /// Creates a new tool.
    fn new() -> Self {
        let mut display_path: DisplayPath = DisplayPath::new(&DisplayPathType::Index);

        // Escape | as \|
        display_path
            .translation_table
            .insert('|' as u32, String::from("\\|"));

        Self {
            display_path: display_path,
        }
    }

    /// Output file entries in bodyfile format.
    fn generate_bodyfile(&self, source: &str, calculate_md5: bool) -> Result<(), ErrorTrace> {
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
        let vfs_location: VfsLocation = new_os_vfs_location(source);

        match vfs_scanner.scan(&mut vfs_scan_context, &vfs_location) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to scan file system");
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

    /// Retrieves a file mode string representation.
    fn get_file_mode_string(file_mode: u16) -> String {
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
        path_components: &Vec<VfsString>,
        calculate_md5: bool,
    ) -> Result<(), ErrorTrace> {
        let md5: String = if !calculate_md5 {
            String::from("0")
        } else {
            let result: Option<DataStreamReference> = match file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
            match result {
                Some(data_stream) => match Bodyfile::calculate_md5(&data_stream) {
                    Ok(md5_string) => md5_string,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to calculate MD5 of data stream"
                        );
                        return Err(error);
                    }
                },
                None => String::from("00000000000000000000000000000000"),
            }
        };
        let display_path: String = self.display_path.join_path_components(path_components);

        let result: Option<VfsPath> = match file_entry.get_symbolic_link_target() {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve symbolic link target"
                );
                return Err(error);
            }
        };
        let path_prefix: &str = if file_system_display_path.ends_with('/') {
            &file_system_display_path[..file_system_display_path.len() - 1]
        } else {
            file_system_display_path.as_str()
        };
        // TODO: escape symbolic link target.
        let path_suffix: String = match result {
            Some(symbolic_link_target) => format!(" -> {}", symbolic_link_target.to_string()),
            None => String::new(),
        };
        let file_identifier: String = match file_entry {
            VfsFileEntry::Ext(ext_file_entry) => {
                format!("{}", ext_file_entry.inode_number)
            }
            VfsFileEntry::Fat(fat_file_entry) => {
                format!("0x{:08x}", fat_file_entry.identifier)
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
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_mode: u16 = ext_file_entry.get_file_mode();

                Self::get_file_mode_string(file_mode)
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
        let owner_identifier: String = match file_entry {
            VfsFileEntry::Ext(ext_file_entry) => {
                let owner_identifier: u32 = ext_file_entry.get_owner_identifier();

                format!("{}", owner_identifier)
            }
            _ => String::from(""),
        };
        let group_identifier: String = match file_entry {
            VfsFileEntry::Ext(ext_file_entry) => {
                let group_identifier: u32 = ext_file_entry.get_group_identifier();

                format!("{}", group_identifier)
            }
            _ => String::from(""),
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
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve data fork: {}", data_fork_index)
                    );
                    return Err(error);
                }
            };
            let data_fork_name: String = match data_fork.get_name() {
                Some(name) => format!(":{}", self.display_path.escape_string(&name)),
                None => continue,
            };
            let data_stream: DataStreamReference = match data_fork.get_data_stream() {
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
                        let attribute: NtfsAttribute =
                            match ntfs_file_entry.get_attribute_by_index(attribute_index) {
                                Ok(attribute) => attribute,
                                Err(mut error) => {
                                    keramics_core::error_trace_add_frame!(
                                        error,
                                        format!(
                                            "Unable to retrieve NTFS MFT entry: {} attribute: {}",
                                            ntfs_file_entry.mft_entry_number, attribute_index
                                        )
                                    );
                                    return Err(error);
                                }
                            };
                        match attribute {
                            NtfsAttribute::FileName { file_name } => {
                                if file_name.parent_file_reference != parent_file_reference
                                    || Some(&file_name.name) != name
                                {
                                    continue;
                                }
                                if file_name.name_space == 0x02 {
                                    continue;
                                }
                                let file_name_access_time: String =
                                    match Bodyfile::format_as_timestamp(Some(
                                        &file_name.access_time,
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
                                        &file_name.modification_time,
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
                                        &file_name.entry_modification_time,
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
                                        &file_name.creation_time,
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

    /// Prints information about a scan node.
    fn print_scan_node(&self, vfs_scan_node: &VfsScanNode, depth: usize) -> Result<(), ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let result: Option<VfsFileEntry> =
            match vfs_resolver.get_file_entry_by_location(&vfs_scan_node.location) {
                Ok(file_entry) => file_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                    return Err(error);
                }
            };
        let indentation: String = vec![" "; depth * 4].join("");
        let vfs_path: &VfsPath = vfs_scan_node.location.get_path();

        let vfs_type: &VfsType = vfs_scan_node.get_type();

        let suffix: String = match result.as_ref() {
            Some(file_entry) => match file_entry {
                VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry.get_identifier() {
                    Some(identifier) => format!(" (alias: /gpt{{{}}})", identifier.to_string()),
                    _ => String::new(),
                },
                _ => String::new(),
            },
            None => String::new(),
        };
        println!(
            "{}{}: path: {}{}",
            indentation,
            vfs_type.as_str(),
            vfs_path.to_string(),
            suffix,
        );
        for sub_scan_node in vfs_scan_node.sub_nodes.iter() {
            self.print_scan_node(sub_scan_node, depth + 1)?;
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
            match vfs_scan_node.get_type() {
                VfsType::Ext { .. } | VfsType::Fat { .. } | VfsType::Ntfs { .. } => {}
                _ => return Ok(()),
            }
            let vfs_resolver: VfsResolverReference = VfsResolver::current();

            let file_system: VfsFileSystemReference =
                match vfs_resolver.open_file_system(&vfs_scan_node.location) {
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
            for result in VfsFinder::new(&file_system) {
                match result {
                    Ok((mut file_entry, path_components)) => match self
                        .print_file_entry_as_bodyfile(
                            &mut file_entry,
                            &display_path,
                            &path_components,
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
                    },
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve file entry from finder"
                        );
                        return Err(error);
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

    /// Scans a data stream for storage media image format signatures.
    fn scan_for_storage_image_formats(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
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
        let result: Option<FormatIdentifier> = scan_results.drain().next();

        Ok(result)
    }

    /// Scans and prints the hierarchy of volumes, partitions and file systems.
    fn scan_for_hierarchy(&self, source: &str) -> Result<(), ErrorTrace> {
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
        let vfs_location: VfsLocation = new_os_vfs_location(source);

        match vfs_scanner.scan(&mut vfs_scan_context, &vfs_location) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to scan file system");
                return Err(error);
            }
        }
        // TODO: print source type.

        match vfs_scan_context.root_node {
            Some(scan_node) => match self.print_scan_node(&scan_node, 0) {
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

    /// Sets the volume path type.
    pub fn set_volume_path_type(&mut self, volume_path_type: &DisplayPathType) {
        self.display_path.set_volume_path_type(volume_path_type);
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

    let mut image_tool: ImageTool = ImageTool::new();

    match arguments.command {
        Some(Commands::Bodyfile(command_arguments)) => {
            image_tool.set_volume_path_type(&command_arguments.volume_path_type);

            match image_tool.generate_bodyfile(source, command_arguments.calculate_md5) {
                Ok(_) => {}
                Err(error) => {
                    println!("Unable to generate bodyfile of: {}\n{}", source, error);
                    return ExitCode::FAILURE;
                }
            }
        }
        Some(Commands::Hash) => {
            let data_stream: DataStreamReference = match open_os_data_stream(&arguments.source) {
                Ok(data_stream) => data_stream,
                Err(error) => {
                    println!("Unable to open file: {} with error:\n{}", source, error);
                    return ExitCode::FAILURE;
                }
            };
            let result: Option<FormatIdentifier> = match image_tool
                .scan_for_storage_image_formats(&data_stream)
            {
                Ok(result) => result,
                Err(error) => {
                    println!(
                        "Unable to scan data stream for known storage media image format signatures with error:\n{}",
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
                        "Unable to open storage media image: {} with error:\n{}",
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
                            "Unable to read data at offset {} with error:\n{}",
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
            // TODO: compare MD5 hashes.
            // TODO: compare SHA1 hashes.
        }
        _ => match image_tool.scan_for_hierarchy(source) {
            Ok(_) => {}
            Err(error) => {
                println!("Unable to determine hierarchy of: {}\n{}", source, error);
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
        let string: String = ImageTool::get_file_mode_string(0x1000);
        assert_eq!(string, "p---------");

        let string: String = ImageTool::get_file_mode_string(0x2000);
        assert_eq!(string, "c---------");

        let string: String = ImageTool::get_file_mode_string(0x4000);
        assert_eq!(string, "d---------");

        let string: String = ImageTool::get_file_mode_string(0x6000);
        assert_eq!(string, "b---------");

        let string: String = ImageTool::get_file_mode_string(0xa000);
        assert_eq!(string, "l---------");

        let string: String = ImageTool::get_file_mode_string(0xc000);
        assert_eq!(string, "s---------");

        let string: String = ImageTool::get_file_mode_string(0x81ff);
        assert_eq!(string, "-rwxrwxrwx");
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
