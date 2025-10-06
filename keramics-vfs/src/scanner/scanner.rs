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

use keramics_core::{DataStreamReference, ErrorTrace, FileResolverReference};
use keramics_sigscan::BuildError;

use keramics_formats::apm::ApmVolumeSystem;
use keramics_formats::ewf::EwfImage;
use keramics_formats::gpt::GptVolumeSystem;
use keramics_formats::mbr::MbrVolumeSystem;
use keramics_formats::qcow::QcowImage;
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::udif::UdifFile;
use keramics_formats::vhd::VhdImage;
use keramics_formats::vhdx::VhdxImage;
use keramics_formats::{FormatIdentifier, FormatScanner};

use crate::apm::ApmFileSystem;
use crate::enums::{VfsFileType, VfsType};
use crate::ewf::EwfFileSystem;
use crate::file_entry::VfsFileEntry;
use crate::file_resolver::open_vfs_file_resolver;
use crate::file_system::VfsFileSystem;
use crate::gpt::GptFileSystem;
use crate::location::VfsLocation;
use crate::mbr::MbrFileSystem;
use crate::path::VfsPath;
use crate::qcow::QcowFileSystem;
use crate::resolver::VfsResolver;
use crate::sparseimage::SparseImageFileSystem;
use crate::types::{VfsFileSystemReference, VfsResolverReference};
use crate::udif::UdifFileSystem;
use crate::vhd::VhdFileSystem;
use crate::vhdx::VhdxFileSystem;

use super::scan_context::VfsScanContext;
use super::scan_node::VfsScanNode;

/// Virtual File System (VFS) scanner.
pub struct VfsScanner {
    /// Resolver.
    resolver: VfsResolverReference,

    /// File system format signature scanner.
    file_system_scanner: FormatScanner,

    /// Phase 1 volume system format signature scanner.
    phase1_volume_system_scanner: FormatScanner,

    /// Phase 2 volume system format signature scanner.
    phase2_volume_system_scanner: FormatScanner,

    /// Phase 3 volume system format signature scanner.
    phase3_volume_system_scanner: FormatScanner,

    /// Storage media image format signature scanner.
    storage_media_image_scanner: FormatScanner,
}

impl VfsScanner {
    /// Creates a new scanner.
    pub fn new() -> Self {
        Self {
            resolver: VfsResolver::current(),
            file_system_scanner: FormatScanner::new(),
            phase1_volume_system_scanner: FormatScanner::new(),
            phase2_volume_system_scanner: FormatScanner::new(),
            phase3_volume_system_scanner: FormatScanner::new(),
            storage_media_image_scanner: FormatScanner::new(),
        }
    }

    /// Builds the scanner.
    pub fn build(&mut self) -> Result<(), BuildError> {
        self.storage_media_image_scanner.add_ewf_signatures();
        self.storage_media_image_scanner.add_qcow_signatures();
        self.storage_media_image_scanner
            .add_sparseimage_signatures();
        self.storage_media_image_scanner.add_udif_signatures();
        self.storage_media_image_scanner.add_vhd_signatures();
        self.storage_media_image_scanner.add_vhdx_signatures();
        self.storage_media_image_scanner.build()?;

        // The Master Boot Record (MBR) signatures are used in other volume
        // system formats, such as BitLocker drive encryption (BDE) and
        // New Technologies File System (NTFS).

        // The scanner:
        // * first looks for non-overlapping volume system signatures (phase 1)
        // * next excludes overlapping signatures (phase 2)
        // * last looks for overlapping volume system signatures (phase 3)

        self.phase1_volume_system_scanner.add_apm_signatures();
        self.phase1_volume_system_scanner.add_gpt_signatures();
        self.phase1_volume_system_scanner.build()?;

        self.phase2_volume_system_scanner.add_ntfs_signatures();
        self.phase2_volume_system_scanner.build()?;

        self.phase3_volume_system_scanner.add_mbr_signatures();
        self.phase3_volume_system_scanner.build()?;

        self.file_system_scanner.add_ext_signatures();
        self.file_system_scanner.add_ntfs_signatures();
        self.file_system_scanner.build()?;

        Ok(())
    }

    /// Scans a storage media image file for supported formats.
    pub fn scan<'a>(
        &self,
        scan_context: &mut VfsScanContext<'a>,
        vfs_location: &'a VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let mut scan_node: VfsScanNode = VfsScanNode::new(vfs_location.clone());

        let file_system: VfsFileSystemReference = self.resolver.open_file_system(vfs_location)?;

        let vfs_path: &VfsPath = vfs_location.get_path();
        let file_entry: VfsFileEntry = match file_system.get_file_entry_by_path(vfs_path)? {
            Some(file_entry) => file_entry,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No such file: {}",
                    vfs_location.to_string()
                )));
            }
        };
        let file_type: VfsFileType = file_entry.get_file_type();
        match file_type {
            VfsFileType::BlockDevice | VfsFileType::CharacterDevice | VfsFileType::Device => {
                return Err(keramics_core::error_trace_new!("Devices are not supported"));
            }
            VfsFileType::File => {}
            _ => {
                return Err(keramics_core::error_trace_new!("Unsupported file type"));
            }
        };
        self.scan_for_sub_nodes(&file_system, vfs_location, &mut scan_node)?;

        scan_context.root_node = Some(scan_node);

        Ok(())
    }

    /// Scans for a supported format.
    fn scan_for_format(
        &self,
        file_system: &VfsFileSystem,
        vfs_location: &VfsLocation,
    ) -> Result<Option<VfsType>, ErrorTrace> {
        let vfs_path: &VfsPath = vfs_location.get_path();
        let result: Option<DataStreamReference> =
            file_system.get_data_stream_by_path_and_name(vfs_path, None)?;

        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "No such file: {}",
                    vfs_location.to_string()
                )));
            }
        };
        match vfs_location.get_type() {
            VfsType::Apm { .. } | VfsType::Gpt { .. } | VfsType::Mbr { .. } => {
                self.scan_for_file_system_format(&data_stream)
            }
            VfsType::Ewf { .. }
            | VfsType::Qcow { .. }
            | VfsType::Vhd { .. }
            | VfsType::Vhdx { .. } => {
                let mut result: Option<VfsType> =
                    self.scan_for_volume_system_format(&data_stream)?;

                if result.is_none() {
                    result = self.scan_for_file_system_format(&data_stream)?;
                }
                Ok(result)
            }
            VfsType::Fake { .. } | VfsType::Os { .. } => {
                let mut result: Option<VfsType> =
                    self.scan_for_storage_media_image_format(&data_stream)?;

                if result.is_none() {
                    result = self.scan_for_volume_system_format(&data_stream)?;
                }
                if result.is_none() {
                    result = self.scan_for_file_system_format(&data_stream)?;
                }
                Ok(result)
            }
            _ => Err(keramics_core::error_trace_new!(
                "Unsupported VFS location type"
            )),
        }
    }

    /// Scans a data stream for a supported file system format.
    fn scan_for_file_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<VfsType>, ErrorTrace> {
        let scan_results: HashSet<FormatIdentifier> =
            match self.file_system_scanner.scan_data_stream(data_stream) {
                Ok(scan_results) => scan_results,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to scan data stream for known file system format signatures"
                    );
                    return Err(error);
                }
            };
        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Found multiple known file system format signatures"
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Ext => Ok(Some(VfsType::Ext)),
                FormatIdentifier::Ntfs => Ok(Some(VfsType::Ntfs)),
                _ => Err(keramics_core::error_trace_new!(
                    "Found unsupported file system format signature"
                )),
            },
            None => Ok(None),
        }
    }

    /// Scans a data stream for a supported storage media image format.
    fn scan_for_storage_media_image_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<VfsType>, ErrorTrace> {
        let scan_results: HashSet<FormatIdentifier> = match self
            .storage_media_image_scanner
            .scan_data_stream(data_stream)
        {
            Ok(scan_results) => scan_results,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for known storage media image format signatures"
                );
                return Err(error);
            }
        };
        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Found multiple known storage media image format signatures"
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Ewf => Ok(Some(VfsType::Ewf)),
                FormatIdentifier::Qcow => Ok(Some(VfsType::Qcow)),
                FormatIdentifier::SparseImage => Ok(Some(VfsType::SparseImage)),
                FormatIdentifier::Udif => Ok(Some(VfsType::Udif)),
                FormatIdentifier::Vhd => Ok(Some(VfsType::Vhd)),
                FormatIdentifier::Vhdx => Ok(Some(VfsType::Vhdx)),
                _ => Err(keramics_core::error_trace_new!(
                    "Found unsupported storage media image format signature"
                )),
            },
            // TODO: handle (split) RAW images.
            None => Ok(None),
        }
    }

    /// Scans for storage media image sub nodes.
    fn scan_for_storage_media_image_sub_nodes(
        &self,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_layers: usize,
    ) -> Result<(), ErrorTrace> {
        if number_of_layers == 0 {
            return Ok(());
        }
        let vfs_type: &VfsType = scan_node.get_type();
        let node_file_system_path: VfsLocation = vfs_location.new_child(vfs_type, "/");
        let node_file_system: VfsFileSystemReference =
            self.resolver.open_file_system(&node_file_system_path)?;

        // TODO: add support for configuration driven scanning older image layers

        // TODO: use layer identifier in location?
        let vfs_type: &VfsType = scan_node.get_type();
        let location: String = format!("{}{}", path_prefix, number_of_layers);
        let node_path: VfsLocation = vfs_location.new_child(vfs_type, location.as_str());
        match self.scan_for_format(&node_file_system, &node_path)? {
            Some(vfs_type) => {
                let sub_node_path: VfsLocation = node_path.new_child(&vfs_type, "/");
                let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_path);
                self.scan_for_sub_nodes(&node_file_system, &node_path, &mut sub_scan_node)?;

                scan_node.sub_nodes.push(sub_scan_node);
            }
            None => {}
        };
        Ok(())
    }

    /// Scans a node for supported formats.
    fn scan_for_sub_nodes(
        &self,
        file_system: &VfsFileSystemReference,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
    ) -> Result<(), ErrorTrace> {
        let vfs_path: &VfsPath = vfs_location.get_path();

        // TODO: handle image with both GPT and MBR volume systems.
        match scan_node.get_type() {
            VfsType::Apm { .. } => {
                let result: Option<DataStreamReference> =
                    match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            vfs_path.to_string()
                        )));
                    }
                };
                let mut apm_volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

                match apm_volume_system.read_data_stream(&data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read APM volume system from data stream"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();

                self.scan_for_volume_system_sub_nodes(
                    vfs_location,
                    scan_node,
                    ApmFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsType::Ext { .. } => {}
            VfsType::Ewf { .. } => {
                let mut ewf_image: EwfImage = EwfImage::new();

                let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, parent_vfs_path)?;

                match ewf_image.open(&file_resolver, vfs_path.get_file_name()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                        return Err(error);
                    }
                }
                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    EwfFileSystem::PATH_PREFIX,
                    1,
                )?;
            }
            VfsType::Gpt { .. } => {
                let result: Option<DataStreamReference> =
                    match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            vfs_path.to_string()
                        )));
                    }
                };
                let mut gpt_volume_system: GptVolumeSystem = GptVolumeSystem::new();

                match gpt_volume_system.read_data_stream(&data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read GPT volume system from data stream"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();

                self.scan_for_volume_system_sub_nodes(
                    vfs_location,
                    scan_node,
                    GptFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsType::Mbr { .. } => {
                let result: Option<DataStreamReference> =
                    match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            vfs_path.to_string()
                        )));
                    }
                };
                let mut mbr_volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

                match mbr_volume_system.read_data_stream(&data_stream) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read MBR volume system from data stream"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();

                self.scan_for_volume_system_sub_nodes(
                    vfs_location,
                    scan_node,
                    MbrFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsType::Ntfs { .. } => {}
            VfsType::Os { .. } => match self.scan_for_format(&file_system, vfs_location)? {
                Some(vfs_type) => {
                    let sub_node_path: VfsLocation = vfs_location.new_child(&vfs_type, "/");
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_path);
                    self.scan_for_sub_nodes(file_system, vfs_location, &mut sub_scan_node)?;

                    scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            },
            VfsType::Qcow { .. } => {
                let mut qcow_image: QcowImage = QcowImage::new();

                let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, parent_vfs_path)?;

                match qcow_image.open(&file_resolver, vfs_path.get_file_name()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = qcow_image.get_number_of_layers();

                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    QcowFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            VfsType::SparseImage { .. } => {
                let result: Option<DataStreamReference> =
                    match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            vfs_path.to_string()
                        )));
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
                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    SparseImageFileSystem::PATH_PREFIX,
                    1,
                )?;
            }
            VfsType::Udif { .. } => {
                let result: Option<DataStreamReference> =
                    match file_system.get_data_stream_by_path_and_name(vfs_path, None) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve data stream"
                            );
                            return Err(error);
                        }
                    };
                let data_stream: DataStreamReference = match result {
                    Some(data_stream) => data_stream,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing data stream: {}",
                            vfs_path.to_string()
                        )));
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
                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    UdifFileSystem::PATH_PREFIX,
                    1,
                )?;
            }
            VfsType::Vhd { .. } => {
                let mut vhd_image: VhdImage = VhdImage::new();

                let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, parent_vfs_path)?;

                match vhd_image.open(&file_resolver, vfs_path.get_file_name()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHD image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = vhd_image.get_number_of_layers();

                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    VhdFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            VfsType::Vhdx { .. } => {
                let mut vhdx_image: VhdxImage = VhdxImage::new();

                let parent_vfs_path: VfsPath = vfs_path.new_with_parent_directory();
                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, parent_vfs_path)?;

                match vhdx_image.open(&file_resolver, vfs_path.get_file_name()) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = vhdx_image.get_number_of_layers();

                self.scan_for_storage_media_image_sub_nodes(
                    vfs_location,
                    scan_node,
                    VhdxFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported VFS location type"
                ));
            }
        };
        Ok(())
    }

    /// Scans a data stream for a supported volume system format.
    fn scan_for_volume_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<VfsType>, ErrorTrace> {
        let scan_results: HashSet<FormatIdentifier> = match self
            .phase1_volume_system_scanner
            .scan_data_stream(data_stream)
        {
            Ok(scan_results) => scan_results,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for known volume system format signatures"
                );
                return Err(error);
            }
        };
        if scan_results.len() > 1 {
            return Err(keramics_core::error_trace_new!(
                "Found multiple known non-overlapping volume system format signatures"
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Apm => Ok(Some(VfsType::Apm)),
                FormatIdentifier::Gpt => Ok(Some(VfsType::Gpt)),
                _ => Err(keramics_core::error_trace_new!(
                    "Found unsupported non-overlapping volume system format signature"
                )),
            },
            None => {
                let scan_results: HashSet<FormatIdentifier> = match self
                    .phase2_volume_system_scanner
                    .scan_data_stream(data_stream)
                {
                    Ok(scan_results) => scan_results,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan data stream for known volume system format signatures"
                        );
                        return Err(error);
                    }
                };
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple exclusion volume system format signatures"
                    ));
                }
                match scan_results.iter().next() {
                    Some(format_identifier) => match format_identifier {
                        FormatIdentifier::Ntfs => Ok(None),
                        _ => Err(keramics_core::error_trace_new!(
                            "Found unsupported exclusion volume system format signature"
                        )),
                    },
                    None => {
                        let scan_results: HashSet<FormatIdentifier> = match self
                            .phase3_volume_system_scanner
                            .scan_data_stream(data_stream)
                        {
                            Ok(scan_results) => scan_results,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to scan data stream for known volume system format signatures"
                                );
                                return Err(error);
                            }
                        };
                        if scan_results.len() > 1 {
                            return Err(keramics_core::error_trace_new!(
                                "Found multiple overlapping volume system format signatures"
                            ));
                        }
                        match scan_results.iter().next() {
                            Some(format_identifier) => match format_identifier {
                                FormatIdentifier::Mbr => Ok(Some(VfsType::Mbr)),
                                _ => Err(keramics_core::error_trace_new!(
                                    "Found unsupported overlapping volume system format signature"
                                )),
                            },
                            None => Ok(None),
                        }
                    }
                }
            }
        }
    }

    /// Scans for volume system sub nodes.
    fn scan_for_volume_system_sub_nodes(
        &self,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_volumes: usize,
    ) -> Result<(), ErrorTrace> {
        let vfs_type: &VfsType = scan_node.get_type();
        let node_file_system_path: VfsLocation = vfs_location.new_child(vfs_type, "/");
        let node_file_system: VfsFileSystemReference =
            self.resolver.open_file_system(&node_file_system_path)?;

        for volume_index in 0..number_of_volumes {
            // TODO: use volume identifier in location?
            let location: String = format!("{}{}", path_prefix, volume_index + 1);

            let vfs_type: &VfsType = scan_node.get_type();
            let node_path: VfsLocation = vfs_location.new_child(vfs_type, location.as_str());
            let mut volume_scan_node: VfsScanNode = VfsScanNode::new(node_path);

            match self.scan_for_format(&node_file_system, &volume_scan_node.location)? {
                Some(vfs_type) => {
                    let sub_node_path: VfsLocation =
                        volume_scan_node.location.new_child(&vfs_type, "/");
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_path);
                    self.scan_for_sub_nodes(
                        &node_file_system,
                        &volume_scan_node.location,
                        &mut sub_scan_node,
                    )?;

                    volume_scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            };
            scan_node.sub_nodes.push(volume_scan_node);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::VfsContext;
    use crate::location::new_os_vfs_location;

    fn get_data_stream(path: &str) -> Result<DataStreamReference, ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = new_os_vfs_location(path);
        match vfs_context.get_data_stream_by_path_and_name(&vfs_location, None)? {
            Some(data_stream) => Ok(data_stream),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file: {}",
                vfs_location.to_string()
            ))),
        }
    }

    fn get_file_system() -> Result<VfsFileSystemReference, ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsLocation = new_os_vfs_location("/");
        vfs_context.open_file_system(&vfs_file_system_path)
    }

    #[test]
    fn test_build() -> Result<(), BuildError> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        format_scanner.build()
    }

    #[test]
    fn test_scan() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let mut scan_context: VfsScanContext = VfsScanContext::new();
        format_scanner.scan(&mut scan_context, &vfs_location)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert!(vfs_type == &VfsType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert!(vfs_type == &VfsType::Qcow);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert!(vfs_type == &VfsType::Ext);
        assert_eq!(scan_node.sub_nodes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_for_format() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let vfs_file_system: VfsFileSystemReference = get_file_system()?;

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let vfs_type: VfsType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert!(vfs_type == VfsType::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_storage_media_image() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let mut vfs_context: VfsContext = VfsContext::new();

        let os_vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let vfs_file_system_path: VfsLocation = os_vfs_location.new_child(&VfsType::Qcow, "/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let vfs_location: VfsLocation = os_vfs_location.new_child(&VfsType::Qcow, "/qcow1");
        let vfs_type: VfsType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert!(vfs_type == VfsType::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_volume_system() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let mut vfs_context: VfsContext = VfsContext::new();

        let os_vfs_location: VfsLocation = new_os_vfs_location("../test_data/gpt/gpt.raw");
        let vfs_file_system_path: VfsLocation = os_vfs_location.new_child(&VfsType::Gpt, "/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let vfs_location: VfsLocation = os_vfs_location.new_child(&VfsType::Gpt, "/gpt1");
        let vfs_type: VfsType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert!(vfs_type == VfsType::Ext);

        Ok(())
    }

    // TODO: add test for scan_for_format with unsupported path type

    #[test]
    fn test_scan_for_file_system_format_with_ext() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/ext/ext2.raw")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_ewf() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/ewf/ext2.E01")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Ewf);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_ntfs() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/ntfs/ntfs.raw")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Ntfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_qcow() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/qcow/ext2.qcow2")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::SparseImage);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_udif() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/udif/hfsplus_zlib.dmg")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Udif);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhd() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/vhd/ntfs-differential.vhd")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Vhd);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhdx() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
        let vfs_type: VfsType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Vhdx);

        Ok(())
    }

    // TODO: add tests for scan_for_storage_media_image_sub_nodes
    // TODO: add tests for scan_for_sub_nodes

    #[test]
    fn test_scan_for_volume_system_format_with_apm() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/apm/apm.dmg")?;

        let vfs_type: VfsType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Apm);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_gpt() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/gpt/gpt.raw")?;

        let vfs_type: VfsType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Gpt);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_mbr() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to build format scanner",
                    error
                ));
            }
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/mbr/mbr.raw")?;

        let vfs_type: VfsType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_type == VfsType::Mbr);

        Ok(())
    }

    // TODO: add tests for scan_for_volume_system_sub_nodes
}
