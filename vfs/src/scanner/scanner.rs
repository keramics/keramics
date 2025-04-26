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
use std::io;

use core::{DataStreamReference, FileResolverReference};
use sigscan::BuildError;

use formats::apm::ApmVolumeSystem;
use formats::gpt::GptVolumeSystem;
use formats::mbr::MbrVolumeSystem;
use formats::qcow::QcowImage;
use formats::sparseimage::SparseImageFile;
use formats::udif::UdifFile;
use formats::vhd::VhdImage;
use formats::vhdx::VhdxImage;
use formats::{FormatIdentifier, FormatScanner};

use crate::apm::ApmFileSystem;
use crate::enums::{VfsFileType, VfsPathType};
use crate::file_entry::VfsFileEntry;
use crate::file_resolver::open_vfs_file_resolver;
use crate::file_system::VfsFileSystem;
use crate::gpt::GptFileSystem;
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
        path: &'a VfsPath,
    ) -> io::Result<()> {
        let mut scan_node: VfsScanNode = VfsScanNode::new(path.clone());

        let file_system: VfsFileSystemReference = self.resolver.open_file_system(path)?;

        let file_entry: VfsFileEntry = match file_system.get_file_entry_by_path(path)? {
            Some(file_entry) => file_entry,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        };
        let file_type: VfsFileType = file_entry.get_file_type();
        match file_type {
            VfsFileType::BlockDevice | VfsFileType::CharacterDevice | VfsFileType::Device => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Devices are not supported",
                ));
            }
            VfsFileType::File => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported file type",
                ));
            }
        };
        self.scan_for_sub_nodes(&file_system, path, &mut scan_node)?;

        scan_context.root_node = Some(scan_node);

        Ok(())
    }

    /// Scans for a supported format.
    fn scan_for_format(
        &self,
        file_system: &VfsFileSystem,
        path: &VfsPath,
    ) -> io::Result<Option<VfsPathType>> {
        let result: Option<DataStreamReference> =
            file_system.get_data_stream_by_path_and_name(path, None)?;

        let data_stream: DataStreamReference = match result {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        };
        match path {
            VfsPath::Apm { .. } | VfsPath::Gpt { .. } | VfsPath::Mbr { .. } => {
                self.scan_for_file_system_format(&data_stream)
            }
            VfsPath::Fake { .. } | VfsPath::Os { .. } => {
                let mut result: Option<VfsPathType> =
                    self.scan_for_storage_media_image_format(&data_stream)?;

                if result.is_none() {
                    result = self.scan_for_volume_system_format(&data_stream)?;
                }
                if result.is_none() {
                    result = self.scan_for_file_system_format(&data_stream)?;
                }
                Ok(result)
            }
            VfsPath::Qcow { .. } | VfsPath::Vhd { .. } | VfsPath::Vhdx { .. } => {
                let mut result: Option<VfsPathType> =
                    self.scan_for_volume_system_format(&data_stream)?;

                if result.is_none() {
                    result = self.scan_for_file_system_format(&data_stream)?;
                }
                Ok(result)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            )),
        }
    }

    /// Scans a data stream for a supported file system format.
    fn scan_for_file_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> io::Result<Option<VfsPathType>> {
        let scan_results: HashSet<FormatIdentifier> =
            self.file_system_scanner.scan_data_stream(data_stream)?;

        if scan_results.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found multiple known file system format signatures"),
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Ext => Ok(Some(VfsPathType::Ext)),
                FormatIdentifier::Ntfs => Ok(Some(VfsPathType::Ntfs)),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Found unsupported file system format signature"),
                )),
            },
            None => Ok(None),
        }
    }

    /// Scans a data stream for a supported storage media image format.
    fn scan_for_storage_media_image_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> io::Result<Option<VfsPathType>> {
        let scan_results: HashSet<FormatIdentifier> = self
            .storage_media_image_scanner
            .scan_data_stream(data_stream)?;

        if scan_results.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found multiple known storage media image format signatures"),
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Qcow => Ok(Some(VfsPathType::Qcow)),
                FormatIdentifier::SparseImage => Ok(Some(VfsPathType::SparseImage)),
                FormatIdentifier::Udif => Ok(Some(VfsPathType::Udif)),
                FormatIdentifier::Vhd => Ok(Some(VfsPathType::Vhd)),
                FormatIdentifier::Vhdx => Ok(Some(VfsPathType::Vhdx)),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Found unsupported storage media image format signature"),
                )),
            },
            // TODO: handle (split) RAW images.
            None => Ok(None),
        }
    }

    /// Scans for storage media image sub nodes.
    fn scan_for_storage_media_image_sub_nodes(
        &self,
        file_system: &VfsFileSystem,
        path: &VfsPath,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_layers: usize,
    ) -> io::Result<()> {
        if number_of_layers == 0 {
            return Ok(());
        }
        let node_file_system_path: VfsPath = path.new_child(scan_node.path.get_path_type(), "/");
        let node_file_system: VfsFileSystemReference =
            self.resolver.open_file_system(&node_file_system_path)?;

        // TODO: add support for configuration driven scanning older image layers

        // TODO: use layer identifier in location?
        let location: String = format!("{}{}", path_prefix, number_of_layers);
        let node_path: VfsPath = path.new_child(scan_node.path.get_path_type(), location.as_str());
        match self.scan_for_format(&node_file_system, &node_path)? {
            Some(path_type) => {
                let sub_node_path: VfsPath = node_path.new_child(path_type, "/");
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
        path: &VfsPath,
        scan_node: &mut VfsScanNode,
    ) -> io::Result<()> {
        // TODO: handle image with both gpt and mbr volume systems
        match &scan_node.path {
            VfsPath::Apm { .. } => {
                let mut apm_volume_system: ApmVolumeSystem = ApmVolumeSystem::new();

                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => apm_volume_system.read_data_stream(&data_stream)?,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                };
                let number_of_partitions: usize = apm_volume_system.get_number_of_partitions();
                self.scan_for_volume_system_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    ApmFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsPath::Ext { .. } => {}
            VfsPath::Gpt { .. } => {
                let mut gpt_volume_system: GptVolumeSystem = GptVolumeSystem::new();

                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => gpt_volume_system.read_data_stream(&data_stream)?,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                };
                let number_of_partitions: usize = gpt_volume_system.get_number_of_partitions();
                self.scan_for_volume_system_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    GptFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsPath::Mbr { .. } => {
                let mut mbr_volume_system: MbrVolumeSystem = MbrVolumeSystem::new();

                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => mbr_volume_system.read_data_stream(&data_stream)?,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                };
                let number_of_partitions: usize = mbr_volume_system.get_number_of_partitions();
                self.scan_for_volume_system_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    MbrFileSystem::PATH_PREFIX,
                    number_of_partitions,
                )?;
            }
            VfsPath::Ntfs { .. } => {}
            VfsPath::Os { .. } => match self.scan_for_format(&file_system, &path)? {
                Some(path_type) => {
                    let sub_node_path: VfsPath = path.new_child(path_type, "/");
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_path);
                    self.scan_for_sub_nodes(file_system, path, &mut sub_scan_node)?;

                    scan_node.sub_nodes.push(sub_scan_node);
                }
                None => {}
            },
            VfsPath::Qcow { .. } => {
                let mut qcow_image: QcowImage = QcowImage::new();

                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, path.parent_directory())?;

                qcow_image.open(&file_resolver, path.get_file_name())?;

                let number_of_layers: usize = qcow_image.get_number_of_layers();
                self.scan_for_storage_media_image_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    QcowFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            VfsPath::SparseImage { .. } => {
                let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => sparseimage_file.read_data_stream(&data_stream)?,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                };
                self.scan_for_storage_media_image_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    SparseImageFileSystem::PATH_PREFIX,
                    1,
                )?;
            }
            VfsPath::Udif { .. } => {
                let mut udif_file: UdifFile = UdifFile::new();

                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => udif_file.read_data_stream(&data_stream)?,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                };
                self.scan_for_storage_media_image_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    UdifFileSystem::PATH_PREFIX,
                    1,
                )?;
            }
            VfsPath::Vhd { .. } => {
                let mut vhd_image: VhdImage = VhdImage::new();

                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, path.parent_directory())?;

                vhd_image.open(&file_resolver, path.get_file_name())?;

                let number_of_layers: usize = vhd_image.get_number_of_layers();
                self.scan_for_storage_media_image_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    VhdFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            VfsPath::Vhdx { .. } => {
                let mut vhdx_image: VhdxImage = VhdxImage::new();

                let file_resolver: FileResolverReference =
                    open_vfs_file_resolver(file_system, path.parent_directory())?;

                vhdx_image.open(&file_resolver, path.get_file_name())?;

                let number_of_layers: usize = vhdx_image.get_number_of_layers();
                self.scan_for_storage_media_image_sub_nodes(
                    file_system,
                    path,
                    scan_node,
                    VhdxFileSystem::PATH_PREFIX,
                    number_of_layers,
                )?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ))
            }
        };
        Ok(())
    }

    /// Scans a data stream for a supported volume system format.
    fn scan_for_volume_system_format(
        &self,
        data_stream: &DataStreamReference,
    ) -> io::Result<Option<VfsPathType>> {
        let scan_results: HashSet<FormatIdentifier> = self
            .phase1_volume_system_scanner
            .scan_data_stream(data_stream)?;

        if scan_results.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Found multiple known non-overlapping volume system format signatures"),
            ));
        }
        match scan_results.iter().next() {
            Some(format_identifier) => match format_identifier {
                FormatIdentifier::Apm => Ok(Some(VfsPathType::Apm)),
                FormatIdentifier::Gpt => Ok(Some(VfsPathType::Gpt)),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Found unsupported non-overlapping volume system format signature"),
                )),
            },
            None => {
                let scan_results: HashSet<FormatIdentifier> = self
                    .phase2_volume_system_scanner
                    .scan_data_stream(data_stream)?;

                if scan_results.len() > 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Found multiple exclusion volume system format signatures"),
                    ));
                }
                match scan_results.iter().next() {
                    Some(format_identifier) => match format_identifier {
                        FormatIdentifier::Ntfs => Ok(None),
                        _ => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Found unsupported exclusion volume system format signature"),
                        )),
                    },
                    None => {
                        let scan_results: HashSet<FormatIdentifier> = self
                            .phase3_volume_system_scanner
                            .scan_data_stream(data_stream)?;

                        if scan_results.len() > 1 {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!(
                                    "Found multiple overlapping volume system format signatures"
                                ),
                            ));
                        }
                        match scan_results.iter().next() {
                            Some(format_identifier) => match format_identifier {
                                FormatIdentifier::Mbr => Ok(Some(VfsPathType::Mbr)),
                                _ => Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("Found unsupported overlapping volume system format signature"),
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
        file_system: &VfsFileSystem,
        path: &VfsPath,
        scan_node: &mut VfsScanNode,
        path_prefix: &str,
        number_of_volumes: usize,
    ) -> io::Result<()> {
        let node_file_system_path: VfsPath = path.new_child(scan_node.path.get_path_type(), "/");
        let node_file_system: VfsFileSystemReference =
            self.resolver.open_file_system(&node_file_system_path)?;

        for volume_index in 0..number_of_volumes {
            // TODO: use volume identifier in location?
            let location: String = format!("{}{}", path_prefix, volume_index + 1);

            let node_path: VfsPath =
                path.new_child(scan_node.path.get_path_type(), location.as_str());
            let mut volume_scan_node: VfsScanNode = VfsScanNode::new(node_path);

            match self.scan_for_format(&node_file_system, &volume_scan_node.path)? {
                Some(path_type) => {
                    let sub_node_path: VfsPath = volume_scan_node.path.new_child(path_type, "/");
                    let mut sub_scan_node: VfsScanNode = VfsScanNode::new(sub_node_path);
                    self.scan_for_sub_nodes(
                        &node_file_system,
                        &volume_scan_node.path,
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

    fn get_data_stream(location: &str) -> io::Result<DataStreamReference> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: location.to_string(),
        };
        match vfs_context.get_data_stream_by_path_and_name(&vfs_path, None)? {
            Some(data_stream) => Ok(data_stream),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("No such file: {}", vfs_path.to_string()),
            )),
        }
    }

    fn get_file_system() -> io::Result<VfsFileSystemReference> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_file_system_path: VfsPath = VfsPath::Os {
            location: "/".to_string(),
        };
        vfs_context.open_file_system(&vfs_file_system_path)
    }

    #[test]
    fn test_build() -> Result<(), BuildError> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        format_scanner.build()
    }

    #[test]
    fn test_scan() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let mut scan_context: VfsScanContext = VfsScanContext::new();
        format_scanner.scan(&mut scan_context, &vfs_path)?;

        let scan_node: &VfsScanNode = scan_context.root_node.as_ref().unwrap();
        assert!(scan_node.path.get_path_type() == VfsPathType::Os);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        assert!(scan_node.path.get_path_type() == VfsPathType::Qcow);
        assert_eq!(scan_node.sub_nodes.len(), 1);

        let scan_node: &VfsScanNode = scan_node.sub_nodes.get(0).unwrap();
        assert!(scan_node.path.get_path_type() == VfsPathType::Ext);
        assert_eq!(scan_node.sub_nodes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_scan_for_format() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let vfs_file_system: VfsFileSystemReference = get_file_system()?;

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_path)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_storage_media_image() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let mut vfs_context: VfsContext = VfsContext::new();

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_file_system_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_path)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_format_with_volume_system() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let mut vfs_context: VfsContext = VfsContext::new();

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_file_system_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_context.open_file_system(&vfs_file_system_path)?;

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt1");
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_format(&vfs_file_system, &vfs_path)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Ext);

        Ok(())
    }

    // TODO: add test for scan_for_format with unsupported path type

    #[test]
    fn test_scan_for_file_system_format_with_ext() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/ext/ext2.raw")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Ext);

        Ok(())
    }

    #[test]
    fn test_scan_for_file_system_format_with_ntfs() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/ntfs/ntfs.raw")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_file_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Ntfs);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_qcow() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/qcow/ext2.qcow2")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Qcow);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_sparseimage() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/sparseimage/hfsplus.sparseimage")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::SparseImage);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_udif() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/udif/hfsplus_zlib.dmg")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Udif);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhd() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/vhd/ntfs-differential.vhd")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Vhd);

        Ok(())
    }

    #[test]
    fn test_scan_for_storage_media_image_format_with_vhdx() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference =
            get_data_stream("../test_data/vhdx/ntfs-differential.vhdx")?;
        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_storage_media_image_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Vhdx);

        Ok(())
    }

    // TODO: add tests for scan_for_storage_media_image_sub_nodes
    // TODO: add tests for scan_for_sub_nodes

    #[test]
    fn test_scan_for_volume_system_format_with_apm() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/apm/apm.dmg")?;

        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Apm);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_gpt() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/gpt/gpt.raw")?;

        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Gpt);

        Ok(())
    }

    #[test]
    fn test_scan_for_volume_system_format_with_mbr() -> io::Result<()> {
        let mut format_scanner: VfsScanner = VfsScanner::new();
        match format_scanner.build() {
            Ok(_) => {}
            Err(error) => return Err(core::error_to_io_error!(error)),
        }
        let data_stream: DataStreamReference = get_data_stream("../test_data/mbr/mbr.raw")?;

        let vfs_path_type: VfsPathType = format_scanner
            .scan_for_volume_system_format(&data_stream)?
            .unwrap();

        assert!(vfs_path_type == VfsPathType::Mbr);

        Ok(())
    }

    // TODO: add tests for scan_for_volume_system_sub_nodes
}
