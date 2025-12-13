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
use std::fmt;
use std::fmt::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use keramics_core::formatters::format_as_string;
use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace, open_os_data_stream};
use keramics_formats::ewf::EwfImage;
use keramics_formats::ntfs::NtfsAttribute;
use keramics_formats::qcow::{QcowImage, QcowImageLayer};
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::splitraw::SplitRawImage;
use keramics_formats::udif::UdifFile;
use keramics_formats::vhd::{VhdImage, VhdImageLayer};
use keramics_formats::vhdx::{VhdxImage, VhdxImageLayer};
use keramics_formats::vmdk::VmdkImage;
use keramics_formats::{
    FileResolverReference, FormatIdentifier, FormatScanner, Path, PathComponent,
    open_os_file_resolver,
};
use keramics_hashes::{DigestHashContext, Md5Context, Sha1Context};
use keramics_types::Ucs2String;
use keramics_vfs::{
    VfsDataFork, VfsFileEntry, VfsFileSystemReference, VfsFileType, VfsFinder, VfsLocation,
    VfsResolver, VfsResolverReference, VfsScanContext, VfsScanNode, VfsScanner, VfsType,
    new_os_vfs_location,
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

/// File mode information.
struct FileModeInfo {
    /// Flags.
    file_mode: u16,
}

impl FileModeInfo {
    /// Creates new file mode information.
    fn new(file_mode: u16) -> Self {
        Self { file_mode }
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
}

impl fmt::Display for FileModeInfo {
    /// Formats partition file mode information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: String = Self::get_file_mode_string(self.file_mode);

        write!(formatter, "{} (0o{:0o})", string, self.file_mode)
    }
}

/// Storage media image.
enum StorageMediaImage {
    Ewf {
        ewf_image: Arc<RwLock<EwfImage>>,
    },
    Qcow {
        qcow_layer: QcowImageLayer,
    },
    SparseImage {
        sparseimage_file: Arc<RwLock<SparseImageFile>>,
    },
    SplitRaw {
        splitraw_image: Arc<RwLock<SplitRawImage>>,
    },
    Udif {
        udif_file: Arc<RwLock<UdifFile>>,
    },
    Vhd {
        vhd_layer: VhdImageLayer,
    },
    Vhdx {
        vhdx_layer: VhdxImageLayer,
    },
    Vmdk {
        vmdk_image: Arc<RwLock<VmdkImage>>,
    },
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

    /// Retrieves a data stream.
    fn get_data_stream(&self) -> DataStreamReference {
        match self {
            Self::Ewf { ewf_image } => ewf_image.clone(),
            Self::Qcow { qcow_layer, .. } => qcow_layer.clone(),
            Self::SparseImage { sparseimage_file } => sparseimage_file.clone(),
            Self::SplitRaw { splitraw_image } => splitraw_image.clone(),
            Self::Udif { udif_file } => udif_file.clone(),
            Self::Vhd { vhd_layer, .. } => vhd_layer.clone(),
            Self::Vhdx { vhdx_layer, .. } => vhdx_layer.clone(),
            Self::Vmdk { vmdk_image } => vmdk_image.clone(),
        }
    }

    /// Retrieves the stored MD5 hash.
    fn get_md5_hash(&self) -> Result<Option<Vec<u8>>, ErrorTrace> {
        match self {
            Self::Ewf { ewf_image } => match ewf_image.read() {
                Ok(image) => Ok(Some(image.md5_hash.to_vec())),
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on EWF image",
                    error
                )),
            },
            _ => Ok(None),
        }
    }

    /// Retrieves the stored SHA1 hash.
    fn get_sha1_hash(&self) -> Result<Option<Vec<u8>>, ErrorTrace> {
        match self {
            Self::Ewf { ewf_image } => match ewf_image.read() {
                Ok(image) => Ok(Some(image.sha1_hash.to_vec())),
                Err(error) => Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain read lock on EWF image",
                    error
                )),
            },
            _ => Ok(None),
        }
    }

    /// Opens a storage media image.
    fn open(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let data_stream: DataStreamReference = match open_os_data_stream(path) {
            Ok(data_stream) => data_stream,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open data stream");
                return Err(error);
            }
        };
        match Self::scan_for_storage_image_formats(&data_stream) {
            Ok(Some(format_identifier)) => match format_identifier {
                FormatIdentifier::Ewf => Self::open_ewf_image(path),
                FormatIdentifier::Qcow => Self::open_qcow_image(path),
                FormatIdentifier::SparseImage => Self::open_sparseimage_file(path),
                FormatIdentifier::Udif => Self::open_udif_file(path),
                FormatIdentifier::Vhd => Self::open_vhd_image(path),
                FormatIdentifier::Vhdx => Self::open_vhdx_image(path),
                FormatIdentifier::Vmdk => Self::open_vmdk_image(path),
                _ => Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format: {}",
                    format_identifier.to_string()
                ))),
            },
            Ok(None) => {
                match Self::open_splitraw_image(path) {
                    Ok(storage_media_image) => Ok(storage_media_image),
                    Err(_) => {
                        // TODO: scan for known volume and file system formats to detect raw
                        // storage media image format.
                        Err(keramics_core::error_trace_new!(
                            "No known storage media image formats found"
                        ))
                    }
                }
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for known storage media image format signatures"
                );
                Err(error)
            }
        }
    }

    /// Opens an EWF image.
    fn open_ewf_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut ewf_image: EwfImage = EwfImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match ewf_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                return Err(error);
            }
        }
        Ok(Self::Ewf {
            ewf_image: Arc::new(RwLock::new(ewf_image)),
        })
    }

    /// Opens a QCOW image.
    fn open_qcow_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut qcow_image: QcowImage = QcowImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match qcow_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                return Err(error);
            }
        }
        let number_of_layers: usize = qcow_image.get_number_of_layers();

        let qcow_layer: QcowImageLayer = match qcow_image.get_layer_by_index(number_of_layers - 1) {
            Ok(layer) => layer,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve QCOW image upper layer"
                );
                return Err(error);
            }
        };
        Ok(Self::Qcow { qcow_layer })
    }

    /// Opens a sparseimage file.
    fn open_sparseimage_file(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
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
        let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

        match sparseimage_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read sparseimage file from data stream"
                );
                return Err(error);
            }
        }
        Ok(Self::SparseImage {
            sparseimage_file: Arc::new(RwLock::new(sparseimage_file)),
        })
    }

    /// Opens a split raw image.
    fn open_splitraw_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut splitraw_image: SplitRawImage = SplitRawImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match splitraw_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open split raw image");
                return Err(error);
            }
        }
        Ok(Self::SplitRaw {
            splitraw_image: Arc::new(RwLock::new(splitraw_image)),
        })
    }

    /// Opens an UDIF file.
    fn open_udif_file(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
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
        let mut udif_file: UdifFile = UdifFile::new();

        match udif_file.read_data_stream(&data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read UDIF file from data stream"
                );
                return Err(error);
            }
        }
        Ok(Self::Udif {
            udif_file: Arc::new(RwLock::new(udif_file)),
        })
    }

    /// Opens a VHD image.
    fn open_vhd_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut vhd_image: VhdImage = VhdImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vhd_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHD image");
                return Err(error);
            }
        }
        let number_of_layers: usize = vhd_image.get_number_of_layers();

        let vhd_layer: VhdImageLayer = match vhd_image.get_layer_by_index(number_of_layers - 1) {
            Ok(layer) => layer,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve VHD image upper layer"
                );
                return Err(error);
            }
        };
        Ok(Self::Vhd {
            vhd_layer: vhd_layer,
        })
    }

    /// Opens a VHDX image.
    fn open_vhdx_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut vhdx_image: VhdxImage = VhdxImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vhdx_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                return Err(error);
            }
        }
        let number_of_layers: usize = vhdx_image.get_number_of_layers();

        let vhdx_layer: VhdxImageLayer = match vhdx_image.get_layer_by_index(number_of_layers - 1) {
            Ok(layer) => layer,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve VHDX image upper layer"
                );
                return Err(error);
            }
        };
        Ok(Self::Vhdx {
            vhdx_layer: vhdx_layer,
        })
    }

    /// Opens a VMDK image.
    fn open_vmdk_image(path: &PathBuf) -> Result<StorageMediaImage, ErrorTrace> {
        let (base_path, file_name) = match Self::get_base_path_and_file_name(path) {
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
        let mut vmdk_image: VmdkImage = VmdkImage::new();

        let path_component: PathComponent = PathComponent::from(file_name);

        match vmdk_image.open(&file_resolver, &path_component) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                return Err(error);
            }
        }
        Ok(Self::Vmdk {
            vmdk_image: Arc::new(RwLock::new(vmdk_image)),
        })
    }

    /// Scans a data stream for storage media image format signatures.
    fn scan_for_storage_image_formats(
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
        format_scanner.add_vmdk_signatures();

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
}

/// Tool for analyzing the contents of a storage media image.
struct ImageTool {
    /// The VFS resolver.
    vfs_resolver: VfsResolverReference,

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
            vfs_resolver: VfsResolver::current(),
            display_path,
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
        let display_path: String = self.display_path.escape_path(path);

        let result: Option<Path> = match file_entry.get_symbolic_link_target() {
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
                format!("{}", ext_file_entry.get_inode_number())
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
                let file_mode_info: FileModeInfo = FileModeInfo::new(file_mode);

                file_mode_info.to_string()
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
        let indentation: String = vec![" "; depth * 4].join("");
        let path: &Path = vfs_scan_node.location.get_path();

        let vfs_type: &VfsType = vfs_scan_node.get_type();

        let path_string: String = match result.as_ref() {
            Some(file_entry) => match file_entry {
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
                VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry.get_partition_number() {
                    Some(partition_number) => format!("/p{}", partition_number),
                    None => path.to_string(),
                },
                _ => path.to_string(),
            },
            None => path.to_string(),
        };
        println!("{}{}: path: {}", indentation, vfs_type, path_string);

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
                VfsType::Ext | VfsType::Fat | VfsType::Ntfs => {}
                _ => return Ok(()),
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
            let storage_media_image: StorageMediaImage =
                match StorageMediaImage::open(&arguments.source) {
                    Ok(storage_media_image) => storage_media_image,
                    Err(error) => {
                        println!(
                            "Unable to open storage media image: {} with error:\n{}",
                            source, error
                        );
                        return ExitCode::FAILURE;
                    }
                };
            let data_stream: DataStreamReference = storage_media_image.get_data_stream();

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
            let progress_bar_template: &str = concat!(
                "Hashing at {percent}% [{wide_bar}] ",
                "{bytes}/{total_bytes} ({binary_bytes_per_sec}) ",
                "elapsed: {elapsed_precise} (remaining: {eta_precise})",
            );
            let progress_bar_style: ProgressStyle =
                match ProgressStyle::with_template(progress_bar_template) {
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

            let mut media_offset: u64 = 0;
            let mut data: [u8; 65536] = [0; 65536];

            let mut md5_context: Md5Context = Md5Context::new();
            let mut sha1_context: Sha1Context = Sha1Context::new();

            let calculate_md5_hash: bool = stored_md5_hash.is_some() || stored_sha1_hash.is_none();

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
                        md5_context.update(&data[0..read_count]);
                    }
                    if stored_sha1_hash.is_some() {
                        sha1_context.update(&data[0..read_count]);
                    }
                    media_offset += read_count as u64;

                    progress_bar.set_position(media_offset);
                },
                Err(error) => {
                    println!(
                        "Unable to obtain write lock on data stream with error: {}",
                        error
                    );
                    return ExitCode::FAILURE;
                }
            };
            progress_bar.finish();

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

            if let Some(stored_hash) = stored_sha1_hash {
                let sha1_hash: Vec<u8> = sha1_context.finalize();

                let hash_string: String = format_as_string(&sha1_hash);
                println!("\nCalculated SHA1 hash\t: {}", hash_string);

                let hash_string: String = format_as_string(&stored_hash);
                println!("Stored SHA1 hash\t: {}", hash_string);

                if stored_hash != sha1_hash.as_slice() {
                    sha1_hash_mismatch = true;
                }
            }
            if md5_hash_mismatch || sha1_hash_mismatch {
                println!("\nMismatch between calculated and stored hashes");
                return ExitCode::FAILURE;
            }
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
        assert_eq!(string, "-rw-r--r-- (0o100644)");
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
