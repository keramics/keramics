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

use keramics_core::{DataStreamReference, ErrorTrace};

use keramics_formats::apfs::{ApfsContainer, ApfsVolume};
use keramics_formats::apm::ApmVolumeSystem;
use keramics_formats::bde::BdeEncryptedVolume;
use keramics_formats::cdsaencr::{CdsaEncrContainer, CdsaEncrCredential};
use keramics_formats::ewf::EwfImage;
use keramics_formats::fat::FatFileSystem;
use keramics_formats::gpt::GptVolumeSystem;
use keramics_formats::linuxlvm::LinuxLvmVolumeSystem;
use keramics_formats::mbr::MbrVolumeSystem;
use keramics_formats::pdi::PdiImage;
use keramics_formats::qcow::QcowImage;
use keramics_formats::sgilabel::SgiDiskLabelVolumeSystem;
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::sparseimage::SparseImageFile;
use keramics_formats::splitraw::SplitRawImage;
use keramics_formats::udif::UdifImage;
use keramics_formats::vhd::VhdImage;
use keramics_formats::vhdx::VhdxImage;
use keramics_formats::vmdk::VmdkImage;
use keramics_formats::{FormatIdentifier, FormatScanner, PartitionIterator, Path};

use crate::apfs::ApfsContainerFileSystem;
use crate::bde::BdeFileSystem;
use crate::credential::VfsCredential;
use crate::credential_store::VfsCredentialStore;
use crate::enums::{VfsFileType, VfsType};
use crate::ewf::EwfFileSystem;
use crate::file_entry::VfsFileEntry;
use crate::linuxlvm::LinuxLvmFileSystem;
use crate::location::VfsLocation;
use crate::resolver::VfsResolver;
use crate::sparsebundle::SparseBundleFileSystem;
use crate::sparseimage::SparseImageFileSystem;
use crate::splitraw::SplitRawFileSystem;
use crate::traits::{VfsImage, VfsPartitionSystem};
use crate::types::{VfsFileSystemReference, VfsResolverReference};
use crate::udif::UdifFileSystem;
use crate::vmdk::VmdkFileSystem;

use super::scan_context::VfsScanContext;
use super::scan_node::VfsScanNode;
use super::scan_options::{VfsScanOptionGroup, VfsScanOptions};

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

    /// Sub volume system (or volume-system-in-volume-system) format signature scanner.
    sub_volume_system_scanner: FormatScanner,

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
            sub_volume_system_scanner: FormatScanner::new(),
            storage_media_image_scanner: FormatScanner::new(),
        }
    }

    /// Builds the scanner.
    pub fn build(&mut self) -> Result<(), ErrorTrace> {
        self.storage_media_image_scanner.add_cdsaencr_signatures();
        self.storage_media_image_scanner.add_ewf_signatures();
        self.storage_media_image_scanner.add_pdi_signatures();
        self.storage_media_image_scanner.add_qcow_signatures();
        self.storage_media_image_scanner
            .add_sparseimage_signatures();
        self.storage_media_image_scanner.add_udif_signatures();
        self.storage_media_image_scanner.add_vhd_signatures();
        self.storage_media_image_scanner.add_vhdx_signatures();
        self.storage_media_image_scanner.add_vmdk_signatures();

        match self.storage_media_image_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to build storage media image scanner"
                );
                return Err(error);
            }
        }
        // The scanner:
        // * first looks for non-overlapping volume system signatures (phase 1)
        // * next excludes overlapping signatures (phase 2)
        // * last looks for overlapping volume system signatures (phase 3)

        self.phase1_volume_system_scanner.add_apfs_signatures();
        self.phase1_volume_system_scanner.add_apm_signatures();
        self.phase1_volume_system_scanner.add_bde_signatures();
        self.phase1_volume_system_scanner.add_gpt_signatures();
        self.phase1_volume_system_scanner.add_linuxlvm_signatures();
        self.phase1_volume_system_scanner.add_luksde_signatures();
        self.phase1_volume_system_scanner.add_sgilabel_signatures();

        match self.phase1_volume_system_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to build phase 1 volume system scanner"
                );
                return Err(error);
            }
        }
        // The Master Boot Record (MBR) signatures are used in other volume system or file formats,
        // such as:
        // * BitLocker Drive Encryption (BDE)
        // * Extensible File Allocation Table (exFAT)
        // * File Allocation Table (FAT)
        // * New Technologies File System (NTFS)

        self.phase2_volume_system_scanner.add_bde_signatures();
        self.phase2_volume_system_scanner.add_exfat_signatures();
        self.phase2_volume_system_scanner.add_fat_signatures();
        self.phase2_volume_system_scanner.add_ntfs_signatures();

        match self.phase2_volume_system_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to build phase 2 volume system scanner"
                );
                return Err(error);
            }
        }
        self.phase3_volume_system_scanner.add_mbr_signatures();

        match self.phase3_volume_system_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to build phase 3 volume system scanner"
                );
                return Err(error);
            }
        }
        self.sub_volume_system_scanner.add_apfs_signatures();
        self.sub_volume_system_scanner.add_bde_signatures();
        self.sub_volume_system_scanner.add_linuxlvm_signatures();

        match self.sub_volume_system_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to build sub volume system scanner"
                );
                return Err(error);
            }
        }
        self.file_system_scanner.add_exfat_signatures();
        self.file_system_scanner.add_ext_signatures();
        self.file_system_scanner.add_fat_signatures();
        self.file_system_scanner.add_hfs_signatures();
        self.file_system_scanner.add_ntfs_signatures();
        self.file_system_scanner.add_xfs_signatures();

        match self.file_system_scanner.build() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build file system scanner");
                return Err(error);
            }
        }
        Ok(())
    }

    /// Retrieves a VFS type for a format identifier.
    fn get_vfs_type(format_identifier: &FormatIdentifier) -> Option<VfsType> {
        match format_identifier {
            FormatIdentifier::Apfs => Some(VfsType::ApfsContainer),
            FormatIdentifier::Apm => Some(VfsType::Apm),
            FormatIdentifier::Bde => Some(VfsType::Bde),
            FormatIdentifier::Ewf => Some(VfsType::Ewf),
            FormatIdentifier::ExFat => Some(VfsType::ExFat),
            FormatIdentifier::Ext => Some(VfsType::Ext),
            FormatIdentifier::Fat => Some(VfsType::Fat),
            FormatIdentifier::Gpt => Some(VfsType::Gpt),
            FormatIdentifier::Hfs => Some(VfsType::Hfs),
            FormatIdentifier::LinuxLvm => Some(VfsType::LinuxLvm),
            FormatIdentifier::Mbr => Some(VfsType::Mbr),
            FormatIdentifier::Ntfs => Some(VfsType::Ntfs),
            FormatIdentifier::Pdi => Some(VfsType::Pdi),
            FormatIdentifier::Qcow => Some(VfsType::Qcow),
            FormatIdentifier::SgiDiskLabel => Some(VfsType::SgiDiskLabel),
            FormatIdentifier::SparseBundle => Some(VfsType::SparseBundle),
            FormatIdentifier::SparseImage => Some(VfsType::SparseImage),
            FormatIdentifier::SplitRaw => Some(VfsType::SplitRaw),
            FormatIdentifier::Udif => Some(VfsType::Udif),
            FormatIdentifier::Vhd => Some(VfsType::Vhd),
            FormatIdentifier::Vhdx => Some(VfsType::Vhdx),
            FormatIdentifier::Vmdk => Some(VfsType::Vmdk),
            FormatIdentifier::Xfs => Some(VfsType::Xfs),
            _ => None,
        }
    }

    /// Scans a storage media image file for supported formats.
    pub fn scan<'a>(
        &self,
        scan_options: &VfsScanOptions,
        scan_context: &mut VfsScanContext<'a>,
        vfs_location: &'a VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let mut scan_node: VfsScanNode = VfsScanNode::new(vfs_location.clone());

        let file_system: VfsFileSystemReference = match self.resolver.open_file_system(vfs_location)
        {
            Ok(file_system) => file_system,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                return Err(error);
            }
        };
        let file_entry: VfsFileEntry = match file_system.get_file_entry_by_location(vfs_location) {
            Ok(Some(file_entry)) => file_entry,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing file entry: {}",
                    vfs_location
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve file entry: {}", vfs_location)
                );
                return Err(error);
            }
        };
        match file_entry.get_file_type() {
            VfsFileType::BlockDevice | VfsFileType::CharacterDevice | VfsFileType::Device => {
                return Err(keramics_core::error_trace_new!(
                    "Devices are currently not supported"
                ));
            }
            VfsFileType::Directory => {
                let path: &Path = vfs_location.get_path();

                match path.file_name() {
                    Some(file_name) => match file_name.extension() {
                        Ok(Some(extension)) => match extension.to_string().as_str() {
                            "sparsebundle" => {
                                let sub_node_path: Path = Path::from("/");
                                let sub_node_vfs_location: VfsLocation = vfs_location
                                    .new_with_layer(&VfsType::SparseBundle, sub_node_path);
                                let mut sub_scan_node: VfsScanNode =
                                    VfsScanNode::new(sub_node_vfs_location);

                                match self.scan_for_sub_nodes(
                                    scan_options,
                                    &file_system,
                                    vfs_location,
                                    &mut sub_scan_node,
                                ) {
                                    Ok(_) => {}
                                    Err(mut error) => {
                                        keramics_core::error_trace_add_frame!(
                                            error,
                                            "Unable to scan for sub nodes"
                                        );
                                        return Err(error);
                                    }
                                }
                                scan_node.sub_nodes.push(sub_scan_node);
                            }
                            _ => {}
                        },
                        Ok(None) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to retrieve extention of file entry: {}",
                                    vfs_location
                                )
                            );
                            return Err(error);
                        }
                    },
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unable to determine file name of file entry: {}",
                            vfs_location
                        )));
                    }
                }
            }
            VfsFileType::File => {
                match self.scan_for_sub_nodes(
                    scan_options,
                    &file_system,
                    vfs_location,
                    &mut scan_node,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan for sub nodes"
                        );
                        return Err(error);
                    }
                }
            }
            _ => {
                return Err(keramics_core::error_trace_new!("Unsupported file type"));
            }
        }
        scan_context.root_node = Some(scan_node);

        Ok(())
    }

    /// Scans for a supported format.
    fn scan_for_format(
        &self,
        file_system: &VfsFileSystemReference,
        vfs_location: &VfsLocation,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        let data_stream: DataStreamReference = match self
            .resolver
            .get_data_stream_by_location_and_name(vfs_location, None)
        {
            Ok(Some(data_stream)) => data_stream,
            Ok(None) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing data stream: {}",
                    vfs_location
                )));
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                return Err(error);
            }
        };
        match vfs_location.get_type() {
            VfsType::Apfs
            | VfsType::ApfsContainer
            | VfsType::ExFat
            | VfsType::Ext
            | VfsType::Fake
            | VfsType::Fat
            | VfsType::Hfs
            | VfsType::Ntfs
            | VfsType::Xfs => Err(keramics_core::error_trace_new!(
                "Unsupported VFS location type"
            )),
            VfsType::Apm
            | VfsType::Gpt
            | VfsType::LinuxLvm
            | VfsType::Mbr
            | VfsType::SgiDiskLabel => {
                let mut result: Option<FormatIdentifier> = match self
                    .scan_for_sub_volume_system_format(&data_stream)
                {
                    Ok(scan_results) => scan_results,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan data stream for volume-system-in-volume-system formats"
                        );
                        return Err(error);
                    }
                };
                if result.is_none() {
                    result = match self.scan_for_file_system_format(&data_stream) {
                        Ok(scan_results) => scan_results,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for file system formats"
                            );
                            return Err(error);
                        }
                    };
                }
                Ok(result)
            }
            VfsType::Bde => match self.scan_for_file_system_format(&data_stream) {
                Ok(scan_results) => Ok(scan_results),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to scan data stream for file system formats"
                    );
                    return Err(error);
                }
            },
            VfsType::Ewf
            | VfsType::SparseBundle
            | VfsType::SparseImage
            | VfsType::SplitRaw
            | VfsType::Pdi
            | VfsType::Qcow
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx
            | VfsType::Vmdk => {
                let mut result: Option<FormatIdentifier> =
                    match self.scan_for_volume_system_format(&data_stream) {
                        Ok(scan_results) => scan_results,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for volume system formats"
                            );
                            return Err(error);
                        }
                    };
                if result.is_none() {
                    result = match self.scan_for_file_system_format(&data_stream) {
                        Ok(scan_results) => scan_results,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for file system formats"
                            );
                            return Err(error);
                        }
                    };
                }
                Ok(result)
            }
            VfsType::Os => {
                let mut result: Option<FormatIdentifier> =
                    match self.scan_for_storage_media_image_format(&data_stream) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for storage media image formats"
                            );
                            return Err(error);
                        }
                    };
                if result.is_none() {
                    let mut splitraw_image: SplitRawImage = SplitRawImage::new();

                    let path: &Path = vfs_location.get_path();

                    result = match SplitRawFileSystem::open_image(
                        &mut splitraw_image,
                        file_system,
                        path,
                    ) {
                        Ok(_) => {
                            if splitraw_image.get_number_of_segments() > 1 {
                                Some(FormatIdentifier::SplitRaw)
                            } else {
                                None
                            }
                        }
                        Err(_) => None,
                    };
                }
                if result.is_none() {
                    result = match self.scan_for_volume_system_format(&data_stream) {
                        Ok(scan_results) => scan_results,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for volume system formats"
                            );
                            return Err(error);
                        }
                    };
                }
                if result.is_none() {
                    result = match self.scan_for_file_system_format(&data_stream) {
                        Ok(scan_results) => scan_results,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan data stream for file system formats"
                            );
                            return Err(error);
                        }
                    };
                }
                Ok(result)
            }
        }
    }

    /// Scans a data stream for a supported file system format.
    pub fn scan_for_file_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        match self.file_system_scanner.scan_data_stream(data_stream) {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple file system format signatures"
                    ));
                }
                Ok(scan_results.drain().next())
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for file system format signatures"
                );
                Err(error)
            }
        }
    }

    /// Scans a data stream for a supported storage media image format.
    pub fn scan_for_storage_media_image_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        let mut format_identifier: FormatIdentifier = match self
            .storage_media_image_scanner
            .scan_data_stream(data_stream)
        {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple storage media image format signatures"
                    ));
                }
                match scan_results.drain().next() {
                    Some(format_identifier) => format_identifier,
                    None => return Ok(None),
                }
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for storage media image format signatures"
                );
                return Err(error);
            }
        };
        if format_identifier == FormatIdentifier::CdsaEncr {
            let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

            match cdsaencr_container.read_data_stream(data_stream) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to open Mac OS Encrypted Encoding container"
                    );
                    return Err(error);
                }
            }
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<CdsaEncrCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(CdsaEncrCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match cdsaencr_container.unlock(&credentials) {
                Ok(false) => {}
                Ok(true) => {
                    let container_data_stream: DataStreamReference =
                        match cdsaencr_container.get_data_stream() {
                            Some(data_stream) => data_stream,
                            None => {
                                return Err(keramics_core::error_trace_new!(
                                    "Missing encrypted container data stream",
                                ));
                            }
                        };
                    format_identifier = match self
                        .storage_media_image_scanner
                        .scan_data_stream(&container_data_stream)
                    {
                        Ok(mut scan_results) => {
                            if scan_results.len() > 1 {
                                return Err(keramics_core::error_trace_new!(
                                    "Found multiple storage media image format signatures in encrypted container"
                                ));
                            }
                            match scan_results.drain().next() {
                                Some(format_identifier) => format_identifier,
                                // If no format was found treat the contents of the encrypted
                                // container as an encrypted uncompressed UDIF image.
                                None => FormatIdentifier::Udif,
                            }
                        }
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan encrypted container data stream for storage media image format signatures"
                            );
                            return Err(error);
                        }
                    };
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to unlock Mac OS Encrypted Encoding container"
                    );
                    return Err(error);
                }
            }
        }
        Ok(Some(format_identifier))
    }

    /// Scans for storage media image sub nodes.
    fn scan_for_storage_media_image_sub_nodes(
        &self,
        scan_options: &VfsScanOptions,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_layers: usize,
    ) -> Result<(), ErrorTrace> {
        if number_of_layers == 0 {
            return Ok(());
        }
        let vfs_type: &VfsType = scan_node.get_type();

        let path: Path = Path::from("/");
        let file_system_vfs_location: VfsLocation = vfs_location.new_with_layer(vfs_type, path);
        let node_file_system: VfsFileSystemReference =
            match self.resolver.open_file_system(&file_system_vfs_location) {
                Ok(file_system) => file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };
        // TODO: add support for configuration driven scanning older image layers

        // TODO: invoke mediator to ask which image layers to include.
        for layer_index in 0..number_of_layers {
            if scan_options.image_layer != 0 && scan_options.image_layer != layer_index + 1 {
                continue;
            }
            let vfs_type: &VfsType = scan_node.get_type();

            // TODO: use layer identifier in location?
            let layer_path: String = format!("{}{}", path_prefix, layer_index + 1);

            let node_path: Path = Path::from(layer_path.as_str());
            let node_vfs_location: VfsLocation = vfs_location.new_with_layer(vfs_type, node_path);
            let mut layer_scan_node: VfsScanNode = VfsScanNode::new(node_vfs_location);

            match self.scan_for_format(&node_file_system, &layer_scan_node.location)? {
                Some(format_identifier) => {
                    let sub_node_vfs_type: VfsType = match Self::get_vfs_type(&format_identifier) {
                        Some(vfs_type) => vfs_type,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Found unsupported format signature: {}",
                                format_identifier
                            )));
                        }
                    };
                    let sub_node_path: Path = Path::from("/");
                    let sub_node_vfs_location: VfsLocation = layer_scan_node
                        .location
                        .new_with_layer(&sub_node_vfs_type, sub_node_path);
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_vfs_location);

                    match self.scan_for_sub_nodes(
                        scan_options,
                        &node_file_system,
                        &layer_scan_node.location,
                        &mut sub_scan_node,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan for sub nodes"
                            );
                            return Err(error);
                        }
                    }
                    layer_scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            }
            scan_node.sub_nodes.push(layer_scan_node);
        }
        Ok(())
    }

    /// Scans a node for supported formats.
    fn scan_for_sub_nodes(
        &self,
        scan_options: &VfsScanOptions,
        file_system: &VfsFileSystemReference,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
    ) -> Result<(), ErrorTrace> {
        let path: &Path = vfs_location.get_path();

        // TODO: handle image with both GPT and MBR volume systems.
        match scan_node.get_type() {
            VfsType::Apfs
            | VfsType::ExFat
            | VfsType::Ext
            | VfsType::Fat
            | VfsType::Hfs
            | VfsType::Ntfs
            | VfsType::Xfs => {}
            VfsType::ApfsContainer => {
                let mut apfs_container: ApfsContainer = ApfsContainer::new();

                match ApfsContainerFileSystem::open_container(
                    &mut apfs_container,
                    file_system,
                    path,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open APFS container"
                        );
                        return Err(error);
                    }
                }
                // TODO: invoke mediator to ask which volumes to include.
                for (volume_index, result) in apfs_container.volumes().enumerate() {
                    let apfs_volume: ApfsVolume = match result {
                        Ok(volume) => volume,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to open APFS volume: {}", volume_index),
                            );
                            return Err(error);
                        }
                    };
                    if scan_options.volumes != VfsScanOptionGroup::NotSet
                        && !scan_options.volumes.contains_index(volume_index + 1)
                    {
                        continue;
                    }
                    let volume_path: String = format!(
                        "{}{}",
                        ApfsContainerFileSystem::PATH_PREFIX,
                        volume_index + 1
                    );
                    let volume_path: Path = Path::from(volume_path.as_str());
                    let volume_vfs_location: VfsLocation =
                        vfs_location.new_with_layer(&VfsType::ApfsContainer, volume_path);

                    let file_system_path: Path = Path::from("/");
                    let file_system_vfs_location: VfsLocation =
                        volume_vfs_location.new_with_layer(&VfsType::Apfs, file_system_path);
                    let file_system_scan_node: VfsScanNode =
                        VfsScanNode::new(file_system_vfs_location);

                    let mut volume_scan_node: VfsScanNode = VfsScanNode::new(volume_vfs_location);
                    volume_scan_node.is_locked = apfs_volume.is_locked();
                    volume_scan_node.sub_nodes.push(file_system_scan_node);

                    scan_node.sub_nodes.push(volume_scan_node);
                }
            }
            VfsType::Apm => {
                let mut apm_volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

                match apm_volume_system.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open APM volume system"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();

                match self.scan_for_volume_system_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    ApmVolumeSystem::PATH_PREFIX,
                    number_of_partitions,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan APM volume system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::Bde => {
                let mut bde_encrypted_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

                match BdeFileSystem::open_encrypted_volume(
                    &mut bde_encrypted_volume,
                    file_system,
                    path,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open BDE encrypted volume"
                        );
                        return Err(error);
                    }
                }
                // TODO: add support for ToGo placeholder FAT file system.

                if bde_encrypted_volume.is_locked() {
                    scan_node.is_locked = true;
                } else {
                    let number_of_partitions: usize = 1;

                    match self.scan_for_volume_system_sub_nodes(
                        scan_options,
                        vfs_location,
                        scan_node,
                        BdeFileSystem::PATH_PREFIX,
                        number_of_partitions,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan BDE unlocked volume"
                            );
                            return Err(error);
                        }
                    }
                }
            }
            VfsType::Ewf => {
                let mut ewf_image: EwfImage = EwfImage::new();

                match EwfFileSystem::open_image(&mut ewf_image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    EwfFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan EWF image");
                        return Err(error);
                    }
                }
            }
            VfsType::Fake => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported VFS location type"
                ));
            }
            VfsType::Gpt => {
                let mut gpt_volume_system: GptVolumeSystem = GptVolumeSystem::new();

                match gpt_volume_system.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open GPT volume system"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();

                match self.scan_for_volume_system_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    GptVolumeSystem::PATH_PREFIX,
                    number_of_partitions,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan GPT volume system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::LinuxLvm => {
                let mut lvm_volume_system: LinuxLvmVolumeSystem = LinuxLvmVolumeSystem::new();

                match LinuxLvmFileSystem::open_volume_system(
                    &mut lvm_volume_system,
                    file_system,
                    path,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open Linux LVM volume system"
                        );
                        return Err(error);
                    }
                }
                let number_of_volumes: usize = lvm_volume_system.get_number_of_volumes();

                match self.scan_for_volume_system_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    LinuxLvmFileSystem::PATH_PREFIX,
                    number_of_volumes,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan Linux LVM volume system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::Mbr => {
                let mut mbr_volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

                match mbr_volume_system.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open MBR volume system"
                        );
                        return Err(error);
                    }
                }
                // TODO: handle mbr_volume_system.bytes_per_sector == 0
                // Use options or invoke mediator to get sector size
                let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();

                match self.scan_for_volume_system_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    MbrVolumeSystem::PATH_PREFIX,
                    number_of_partitions,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan MBR volume system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::Os => match self.scan_for_format(&file_system, vfs_location)? {
                Some(FormatIdentifier::CdsaEncr) => {
                    // TODO: Set VfsType::SparseImage based on extension?
                    scan_node.add_locked_sub_node(&VfsType::Udif);
                }
                Some(format_identifier) => {
                    let sub_node_vfs_type: VfsType = match Self::get_vfs_type(&format_identifier) {
                        Some(vfs_type) => vfs_type,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Found unsupported format signature: {}",
                                format_identifier
                            )));
                        }
                    };
                    let sub_node_path: Path = Path::from("/");
                    let sub_node_vfs_location: VfsLocation =
                        vfs_location.new_with_layer(&sub_node_vfs_type, sub_node_path);
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_vfs_location);

                    match self.scan_for_sub_nodes(
                        scan_options,
                        file_system,
                        vfs_location,
                        &mut sub_scan_node,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(error, "Unable to scan OS");
                            return Err(error);
                        }
                    }
                    scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            },
            VfsType::Pdi => {
                let mut pdi_image: PdiImage = PdiImage::new();

                match pdi_image.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open PDI image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = pdi_image.get_number_of_layers();

                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    PdiImage::PATH_PREFIX,
                    number_of_layers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan PDI image");
                        return Err(error);
                    }
                }
            }
            VfsType::Qcow => {
                let mut qcow_image: QcowImage = QcowImage::new();

                match qcow_image.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open QCOW image");
                        return Err(error);
                    }
                }
                if qcow_image.is_locked() {
                    scan_node.is_locked = true;
                } else {
                    let number_of_layers: usize = qcow_image.get_number_of_layers();

                    match self.scan_for_storage_media_image_sub_nodes(
                        scan_options,
                        vfs_location,
                        scan_node,
                        QcowImage::PATH_PREFIX,
                        number_of_layers,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan QCOW image"
                            );
                            return Err(error);
                        }
                    }
                }
            }
            VfsType::SgiDiskLabel => {
                let mut sgilabel_volume_system: SgiDiskLabelVolumeSystem =
                    SgiDiskLabelVolumeSystem::new();

                match sgilabel_volume_system.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sgilabel volume system"
                        );
                        return Err(error);
                    }
                }
                let number_of_partitions: usize = sgilabel_volume_system.get_number_of_partitions();

                match self.scan_for_volume_system_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    SgiDiskLabelVolumeSystem::PATH_PREFIX,
                    number_of_partitions,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan sgilabel volume system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::SparseBundle => {
                let mut sparsebundle_image: SparseBundleImage = SparseBundleImage::new();

                match SparseBundleFileSystem::open_image(&mut sparsebundle_image, file_system, path)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sparsebundle image"
                        );
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    SparseBundleFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan sparsebundle image"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::SparseImage => {
                let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

                match SparseImageFileSystem::open_file(&mut sparseimage_file, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sparseimage file"
                        );
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    SparseImageFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan sparseimage file"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::SplitRaw => {
                let mut splitraw_image: SplitRawImage = SplitRawImage::new();

                match SplitRawFileSystem::open_image(&mut splitraw_image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open split raw image"
                        );
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    SplitRawFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to scan split raw image"
                        );
                        return Err(error);
                    }
                }
            }
            VfsType::Udif => {
                let mut udif_image: UdifImage = UdifImage::new();

                match UdifFileSystem::open_image(&mut udif_image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open UDIF image");
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    UdifFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan UDIF image");
                        return Err(error);
                    }
                }
            }
            VfsType::Vhd => {
                let mut vhd_image: VhdImage = VhdImage::new();

                match vhd_image.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHD image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = vhd_image.get_number_of_layers();

                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    VhdImage::PATH_PREFIX,
                    number_of_layers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan VHD image");
                        return Err(error);
                    }
                }
            }
            VfsType::Vhdx => {
                let mut vhdx_image: VhdxImage = VhdxImage::new();

                match vhdx_image.open_from_vfs(file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VHDX image");
                        return Err(error);
                    }
                }
                let number_of_layers: usize = vhdx_image.get_number_of_layers();

                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    VhdxImage::PATH_PREFIX,
                    number_of_layers,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan VHDX image");
                        return Err(error);
                    }
                }
            }
            VfsType::Vmdk => {
                let mut vmdk_image: VmdkImage = VmdkImage::new();

                match VmdkFileSystem::open_image(&mut vmdk_image, file_system, path) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                        return Err(error);
                    }
                }
                match self.scan_for_storage_media_image_sub_nodes(
                    scan_options,
                    vfs_location,
                    scan_node,
                    VmdkFileSystem::PATH_PREFIX,
                    1,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to scan VMDK image");
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Scans a data stream for a supported volume-system-in-volume-system format.
    fn scan_for_sub_volume_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        match self.sub_volume_system_scanner.scan_data_stream(data_stream) {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple volume-system-in-volume-system format signatures"
                    ));
                }
                Ok(scan_results.drain().next())
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for non-overlapping volume system format signatures"
                );
                Err(error)
            }
        }
    }

    /// Scans a data stream for a supported volume system format.
    pub fn scan_for_volume_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<Option<FormatIdentifier>, ErrorTrace> {
        let format_identifier: Option<FormatIdentifier> = match self
            .phase1_volume_system_scanner
            .scan_data_stream(data_stream)
        {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple non-overlapping volume system format signatures"
                    ));
                }
                scan_results.drain().next()
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for non-overlapping volume system format signatures"
                );
                return Err(error);
            }
        };
        match &format_identifier {
            Some(FormatIdentifier::Apfs) => return Ok(Some(FormatIdentifier::Apfs)),
            Some(FormatIdentifier::Apm) => return Ok(Some(FormatIdentifier::Apm)),
            Some(FormatIdentifier::Bde) => return Ok(Some(FormatIdentifier::Bde)),
            Some(FormatIdentifier::Gpt) => return Ok(Some(FormatIdentifier::Gpt)),
            Some(FormatIdentifier::LinuxLvm) => return Ok(Some(FormatIdentifier::LinuxLvm)),
            Some(FormatIdentifier::Luks) => return Ok(Some(FormatIdentifier::Luks)),
            Some(FormatIdentifier::SgiDiskLabel) => {
                return Ok(Some(FormatIdentifier::SgiDiskLabel));
            }
            Some(format_identifier) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Found unsupported non-overlapping volume system format signature: {}",
                    format_identifier
                )));
            }
            None => {}
        }
        let format_identifier: Option<FormatIdentifier> = match self
            .phase2_volume_system_scanner
            .scan_data_stream(data_stream)
        {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple exclusion volume system format signatures"
                    ));
                }
                scan_results.drain().next()
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for exclusion volume system format signatures"
                );
                return Err(error);
            }
        };
        match &format_identifier {
            Some(FormatIdentifier::ExFat) => return Ok(None),
            Some(FormatIdentifier::Fat) => return Ok(None),
            Some(FormatIdentifier::Ntfs) => return Ok(None),
            Some(format_identifier) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Found unsupported exclusion volume system format signature: {}",
                    format_identifier
                )));
            }
            None => {}
        }
        let format_identifier: FormatIdentifier = match self
            .phase3_volume_system_scanner
            .scan_data_stream(data_stream)
        {
            Ok(mut scan_results) => {
                if scan_results.len() > 1 {
                    return Err(keramics_core::error_trace_new!(
                        "Found multiple overlapping volume system format signatures"
                    ));
                }
                match scan_results.drain().next() {
                    Some(format_identifier) => format_identifier,
                    None => return Ok(None),
                }
            }
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to scan data stream for overlapping volume system format signatures"
                );
                return Err(error);
            }
        };
        match &format_identifier {
            FormatIdentifier::Mbr => {
                // FAT does not have unique signatures.
                let mut fat_file_system: FatFileSystem = FatFileSystem::new();

                match fat_file_system.read_data_stream(data_stream) {
                    Ok(_) => Ok(Some(FormatIdentifier::Fat)),
                    Err(_) => Ok(Some(FormatIdentifier::Mbr)),
                }
            }
            _ => Err(keramics_core::error_trace_new!(
                "Found unsupported overlapping volume system format signature"
            )),
        }
    }

    /// Scans for volume system sub nodes.
    fn scan_for_volume_system_sub_nodes(
        &self,
        scan_options: &VfsScanOptions,
        vfs_location: &VfsLocation,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_volumes: usize,
    ) -> Result<(), ErrorTrace> {
        let vfs_type: &VfsType = scan_node.get_type();

        let path: Path = Path::from("/");
        let file_system_vfs_location: VfsLocation = vfs_location.new_with_layer(vfs_type, path);
        let node_file_system: VfsFileSystemReference =
            match self.resolver.open_file_system(&file_system_vfs_location) {
                Ok(file_system) => file_system,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                    return Err(error);
                }
            };

        match vfs_type {
            VfsType::Apm | VfsType::Gpt | VfsType::Mbr | VfsType::SgiDiskLabel => {
                if scan_options.partitions == VfsScanOptionGroup::NotSet {
                    // TODO: invoke mediator to ask which partitions to include.
                }
            }
            VfsType::LinuxLvm => {
                if scan_options.volumes == VfsScanOptionGroup::NotSet {
                    // TODO: invoke mediator to ask which volumes to include.
                }
            }
            _ => {}
        };
        for volume_index in 0..number_of_volumes {
            let vfs_type: &VfsType = scan_node.get_type();

            match vfs_type {
                VfsType::Apm | VfsType::Gpt | VfsType::Mbr | VfsType::SgiDiskLabel => {
                    if scan_options.partitions != VfsScanOptionGroup::NotSet
                        && !scan_options.partitions.contains_index(volume_index + 1)
                    {
                        continue;
                    }
                }
                VfsType::LinuxLvm => {
                    if scan_options.volumes != VfsScanOptionGroup::NotSet
                        && !scan_options.volumes.contains_index(volume_index + 1)
                    {
                        continue;
                    }
                }
                _ => {}
            };
            // TODO: use volume identifier in location?
            let volume_path: String = format!("{}{}", path_prefix, volume_index + 1);

            let node_path: Path = Path::from(volume_path.as_str());
            let node_vfs_location: VfsLocation = vfs_location.new_with_layer(vfs_type, node_path);
            let mut volume_scan_node: VfsScanNode = VfsScanNode::new(node_vfs_location);

            match self.scan_for_format(&node_file_system, &volume_scan_node.location)? {
                Some(format_identifier) => {
                    let sub_node_vfs_type: VfsType = match Self::get_vfs_type(&format_identifier) {
                        Some(vfs_type) => vfs_type,
                        None => {
                            return Err(keramics_core::error_trace_new!(format!(
                                "Found unsupported format signature: {}",
                                format_identifier
                            )));
                        }
                    };
                    let sub_node_path: Path = Path::from("/");
                    let sub_node_vfs_location: VfsLocation = volume_scan_node
                        .location
                        .new_with_layer(&sub_node_vfs_type, sub_node_path);
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_vfs_location);

                    match self.scan_for_sub_nodes(
                        scan_options,
                        &node_file_system,
                        &volume_scan_node.location,
                        &mut sub_scan_node,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to scan for sub nodes"
                            );
                            return Err(error);
                        }
                    }
                    volume_scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            }
            scan_node.sub_nodes.push(volume_scan_node);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::VfsContext;

    use crate::tests::get_test_data_path;

    fn get_data_stream(path: &str) -> Result<DataStreamReference, ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_location: VfsLocation = VfsLocation::from(path);
        match vfs_context.get_data_stream_by_location_and_name(&vfs_location, None)? {
            Some(data_stream) => Ok(data_stream),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing data stream: {}",
                vfs_location
            ))),
        }
    }

    fn get_file_system() -> Result<VfsFileSystemReference, ErrorTrace> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsLocation = VfsLocation::from("/");
        vfs_context.open_file_system(&vfs_file_system_path)
    }

    fn get_format_scanner() -> Result<VfsScanner, ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();

        match format_scanner.build() {
            Ok(_) => Ok(format_scanner),
            Err(error) => Err(keramics_core::error_trace_new_with_error!(
                "Unable to build format scanner",
                error
            )),
        }
    }

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        format_scanner.build()
    }

    #[test]
    fn test_scan() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let scan_options: VfsScanOptions = VfsScanOptions::new();

        let mut scan_context: VfsScanContext = VfsScanContext::new();
        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        format_scanner.scan(&scan_options, &mut scan_context, &vfs_location)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Gpt);
        assert_eq!(scan_node.sub_nodes.len(), 2);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Gpt);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Ext);
        assert_eq!(scan_node.sub_nodes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_with_options() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let mut scan_options: VfsScanOptions = VfsScanOptions::new();
        scan_options.parse_partitions("1")?;

        let mut scan_context: VfsScanContext = VfsScanContext::new();
        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        format_scanner.scan(&scan_options, &mut scan_context, &vfs_location)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Gpt);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Gpt);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Ext);
        assert_eq!(scan_node.sub_nodes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_with_apfs() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let mut scan_options: VfsScanOptions = VfsScanOptions::new();
        scan_options.parse_partitions("1")?;

        let mut scan_context: VfsScanContext = VfsScanContext::new();
        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        format_scanner.scan(&scan_options, &mut scan_context, &vfs_location)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::ApfsContainer);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::ApfsContainer);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Apfs);
        assert_eq!(scan_node.sub_nodes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_with_sparsebundle() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let mut scan_options: VfsScanOptions = VfsScanOptions::new();
        scan_options.parse_partitions("1")?;

        let mut scan_context: VfsScanContext = VfsScanContext::new();
        let path_string: String = get_test_data_path("sparsebundle/hfsplus.sparsebundle");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        format_scanner.scan(&scan_options, &mut scan_context, &vfs_location)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        let vfs_type: &VfsType = scan_node.get_type();
        assert_eq!(vfs_type, &VfsType::SparseBundle);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_pdi() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;
        let vfs_file_system: VfsFileSystemReference = get_file_system()?;

        let path_string: String = get_test_data_path("pdi/hfsplus.hdd/DiskDescriptor.xml");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Pdi);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_qcow() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;
        let vfs_file_system: VfsFileSystemReference = get_file_system()?;

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_splitraw() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;
        let vfs_file_system: VfsFileSystemReference = get_file_system()?;

        let path_string: String = get_test_data_path("splitraw/ext2.raw.000");
        let vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::SplitRaw);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_storage_media_image() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;
        let mut vfs_context: VfsContext = VfsContext::new();

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let os_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let path: Path = Path::from("/");
        let vfs_file_system_path: VfsLocation =
            os_vfs_location.new_with_layer(&VfsType::Qcow, path);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let path: Path = Path::from("/qcow1");
        let vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, path);
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_volume_system() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;
        let mut vfs_context: VfsContext = VfsContext::new();

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let os_vfs_location: VfsLocation = VfsLocation::from(&path_string);
        let path: Path = Path::from("/");
        let vfs_file_system_path: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Gpt, path);
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let path: Path = Path::from("/gpt1");
        let vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Gpt, path);
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_location)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Ext);

        Ok(())
    }

    // TODO: add test for scan_for_format with unsupported path type

    #[test]
    fn test_scan_for_file_system_format_with_exfat() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("exfat/exfat.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::ExFat);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_ext() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("ext/ext2.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_fat() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("fat/fat12.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Fat);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_hfs() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("hfs/hfs.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Hfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_hfsplus() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Hfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_ntfs() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Ntfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_xfs() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("xfs/xfs.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Xfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_ewf() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("ewf/ext2.E01");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Ewf);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_qcow() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_sparseimage() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::SparseImage);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_udif() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Udif);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhd() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Vhd);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhdx() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Vhdx);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vmdk() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Vmdk);

        Ok(())
    }

    // TODO: add tests for scan_for_storage_media_image_sub_nodes
    // TODO: add tests for scan_for_sub_nodes

    #[test]
    fn test_scan_for_volume_system_format_with_apfs() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("apfs/apfs.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Apfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_apm() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("apm/apm.dmg");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Apm);

        Ok(())
    }

    // TODO: add test for BDE

    #[test]
    fn test_scan_for_volume_system_format_with_gpt() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Gpt);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_linuxlvm() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("linuxlvm/lvm2.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::LinuxLvm);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_mbr() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::Mbr);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_sgilabel() -> Result<(), ErrorTrace> {
        let format_scanner: VfsScanner = get_format_scanner()?;

        let path_string: String = get_test_data_path("sgilabel/sgilabel.raw");
        let data_stream: DataStreamReference = get_data_stream(path_string.as_str())?;
        let format_identifier: FormatIdentifier = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert_eq!(format_identifier, FormatIdentifier::SgiDiskLabel);

        Ok(())
    }

    // TODO: add tests for scan_for_volume_system_sub_nodes
}
