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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::ext::ExtFileEntry;
use keramics_formats::ext::constants::*;
use keramics_formats::fat::{FatFileEntry, FatString};
use keramics_formats::hfs::constants::*;
use keramics_formats::hfs::{HfsFileEntry, HfsString};
use keramics_formats::ntfs::NtfsFileEntry;
use keramics_formats::{Path, PathComponent};
use keramics_types::Ucs2String;

use super::apm::ApmFileEntry;
use super::data_fork::VfsDataFork;
use super::data_forks::VfsDataForksIterator;
use super::enums::VfsFileType;
use super::ewf::EwfFileEntry;
use super::extended_attribute::VfsExtendedAttribute;
use super::extended_attributes::VfsExtendedAttributesIterator;
use super::fake::FakeFileEntry;
use super::file_entries::VfsFileEntriesIterator;
use super::gpt::GptFileEntry;
use super::mbr::MbrFileEntry;
use super::os::OsFileEntry;
use super::pdi::PdiFileEntry;
use super::qcow::QcowFileEntry;
use super::sparseimage::SparseImageFileEntry;
use super::splitraw::SplitRawFileEntry;
use super::udif::UdifFileEntry;
use super::vhd::VhdFileEntry;
use super::vhdx::VhdxFileEntry;
use super::vmdk::VmdkFileEntry;

/// Virtual File System (VFS) file entry.
pub enum VfsFileEntry {
    Apm(ApmFileEntry),
    Ext(ExtFileEntry),
    Ewf(EwfFileEntry),
    Fake(Arc<FakeFileEntry>),
    Fat(FatFileEntry),
    Gpt(GptFileEntry),
    Hfs(HfsFileEntry),
    Mbr(MbrFileEntry),
    Ntfs(NtfsFileEntry),
    Os(OsFileEntry),
    Pdi(PdiFileEntry),
    Qcow(QcowFileEntry),
    SparseImage(SparseImageFileEntry),
    SplitRaw(SplitRawFileEntry),
    Udif(UdifFileEntry),
    Vhd(VhdFileEntry),
    Vhdx(VhdxFileEntry),
    Vmdk(VmdkFileEntry),
}

impl VfsFileEntry {
    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_access_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_access_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_access_time(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_access_time(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_access_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_access_time(),
        }
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_change_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_change_time(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_change_time(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_change_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_change_time(),
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_creation_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_creation_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_creation_time(),
            VfsFileEntry::Hfs(hfs_file_entry) => Some(hfs_file_entry.get_creation_time()),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_creation_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_creation_time(),
        }
    }

    /// Retrieves the device identifier.
    pub fn get_device_identifier(&self) -> Result<Option<u64>, ErrorTrace> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Hfs(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => Ok(None),
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_device_identifier() {
                Ok(Some(device_identifier)) => Ok(Some(device_identifier as u64)),
                Ok(None) => Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve ext device identifier"
                    );
                    Err(error)
                }
            },
            VfsFileEntry::Os(os_file_entry) => Ok(os_file_entry.get_device_identifier()),
        }
    }

    /// Retrieves the file mode.
    pub fn get_file_mode(&self) -> Option<u32> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => Some(ext_file_entry.get_file_mode() as u32),
            VfsFileEntry::Hfs(hfs_file_entry) => match hfs_file_entry.get_file_mode() {
                Some(file_mode) => Some(*file_mode as u32),
                None => None,
            },
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_file_mode(),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_file_type(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_file_type(),
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_type: u16 = ext_file_entry.get_file_mode() & 0xf000;
                match file_type {
                    EXT_FILE_MODE_TYPE_FIFO => VfsFileType::NamedPipe,
                    EXT_FILE_MODE_TYPE_CHARACTER_DEVICE => VfsFileType::CharacterDevice,
                    EXT_FILE_MODE_TYPE_DIRECTORY => VfsFileType::Directory,
                    EXT_FILE_MODE_TYPE_BLOCK_DEVICE => VfsFileType::BlockDevice,
                    EXT_FILE_MODE_TYPE_REGULAR_FILE => VfsFileType::File,
                    EXT_FILE_MODE_TYPE_SYMBOLIC_LINK => VfsFileType::SymbolicLink,
                    EXT_FILE_MODE_TYPE_SOCKET => VfsFileType::Socket,
                    _ => VfsFileType::Unknown(file_type),
                }
            }
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_file_type(),
            VfsFileEntry::Fat(fat_file_entry) => {
                if fat_file_entry.is_directory() {
                    VfsFileType::Directory
                } else {
                    VfsFileType::File
                }
            }
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_file_type(),
            VfsFileEntry::Hfs(hfs_file_entry) => {
                match hfs_file_entry.get_file_mode() {
                    Some(file_mode) => {
                        let file_type: u16 = *file_mode & 0xf000;
                        match file_type {
                            HFS_FILE_MODE_TYPE_FIFO => VfsFileType::NamedPipe,
                            HFS_FILE_MODE_TYPE_CHARACTER_DEVICE => VfsFileType::CharacterDevice,
                            HFS_FILE_MODE_TYPE_DIRECTORY => VfsFileType::Directory,
                            HFS_FILE_MODE_TYPE_BLOCK_DEVICE => VfsFileType::BlockDevice,
                            HFS_FILE_MODE_TYPE_REGULAR_FILE => VfsFileType::File,
                            HFS_FILE_MODE_TYPE_SYMBOLIC_LINK => VfsFileType::SymbolicLink,
                            HFS_FILE_MODE_TYPE_SOCKET => VfsFileType::Socket,
                            HFS_FILE_MODE_TYPE_WHITEOUT => VfsFileType::Whiteout,
                            _ => VfsFileType::Unknown(file_type),
                        }
                    }
                    // TODO: determine file type by other means
                    None => VfsFileType::Unknown(0),
                }
            }
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.get_file_type(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                if ntfs_file_entry.is_symbolic_link() {
                    VfsFileType::SymbolicLink
                } else if ntfs_file_entry.is_directory() {
                    VfsFileType::Directory
                } else {
                    VfsFileType::File
                }
            }
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_file_type(),
            VfsFileEntry::Pdi(pdi_file_entry) => pdi_file_entry.get_file_type(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_file_type(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.get_file_type()
            }
            VfsFileEntry::SplitRaw(splitraw_file_entry) => splitraw_file_entry.get_file_type(),
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_file_type(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_file_type(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_file_type(),
            VfsFileEntry::Vmdk(vmdk_file_entry) => vmdk_file_entry.get_file_type(),
        }
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_modification_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_modification_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_modification_time(),
            VfsFileEntry::Hfs(hfs_file_entry) => Some(hfs_file_entry.get_modification_time()),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_modification_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_modification_time(),
        }
    }

    /// Retrieves the group identifier.
    pub fn get_group_identifier(&self) -> Option<u32> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => Some(ext_file_entry.get_group_identifier()),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_group_identifier().cloned(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_group_identifier(),
        }
    }

    /// Retrieves the inode number.
    pub fn get_inode_number(&self) -> Option<u64> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => Some(ext_file_entry.get_inode_number() as u64),
            VfsFileEntry::Hfs(hfs_file_entry) => Some(hfs_file_entry.get_identifier() as u64),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_inode_number(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<PathComponent> {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => Some(apm_file_entry.get_name()),
            VfsFileEntry::Ewf(ewf_file_entry) => Some(ewf_file_entry.get_name()),
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_name() {
                Some(name) => Some(PathComponent::from(name)),
                None => None,
            },
            VfsFileEntry::Fake(fake_file_entry) => {
                let path_component: &PathComponent = fake_file_entry.get_name();

                Some(path_component.clone())
            }
            VfsFileEntry::Fat(fat_file_entry) => match fat_file_entry.get_name() {
                Some(FatString::ByteString(byte_string)) => Some(PathComponent::from(byte_string)),
                Some(FatString::Ucs2String(ucs2_string)) => Some(PathComponent::from(ucs2_string)),
                None => None,
            },
            VfsFileEntry::Gpt(gpt_file_entry) => Some(gpt_file_entry.get_name()),
            VfsFileEntry::Hfs(hfs_file_entry) => match hfs_file_entry.get_name() {
                Some(HfsString::ByteString(byte_string)) => Some(PathComponent::from(byte_string)),
                Some(HfsString::Utf16String(utf16_string)) => {
                    Some(PathComponent::from(utf16_string))
                }
                None => None,
            },
            VfsFileEntry::Mbr(mbr_file_entry) => Some(mbr_file_entry.get_name()),
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_name() {
                Some(name) => Some(PathComponent::from(name)),
                None => None,
            },
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_name() {
                Some(name) => Some(PathComponent::from(name)),
                None => None,
            },
            VfsFileEntry::Pdi(pdi_file_entry) => Some(pdi_file_entry.get_name()),
            VfsFileEntry::Qcow(qcow_file_entry) => Some(qcow_file_entry.get_name()),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                Some(sparseimage_file_entry.get_name())
            }
            VfsFileEntry::SplitRaw(splitraw_file_entry) => Some(splitraw_file_entry.get_name()),
            VfsFileEntry::Udif(udif_file_entry) => Some(udif_file_entry.get_name()),
            VfsFileEntry::Vhd(vhd_file_entry) => Some(vhd_file_entry.get_name()),
            VfsFileEntry::Vhdx(vhdx_file_entry) => Some(vhdx_file_entry.get_name()),
            VfsFileEntry::Vmdk(vmdk_file_entry) => Some(vmdk_file_entry.get_name()),
        }
    }

    /// Retrieves the number of links.
    pub fn get_number_of_links(&self) -> Option<u64> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => Some(ext_file_entry.get_number_of_links() as u64),
            VfsFileEntry::Hfs(hfs_file_entry) => Some(hfs_file_entry.get_number_of_links() as u64),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_number_of_links(),
        }
    }

    /// Retrieves the owner identifier.
    pub fn get_owner_identifier(&self) -> Option<u32> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => Some(ext_file_entry.get_owner_identifier()),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_owner_identifier().cloned(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_owner_identifier(),
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_size(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_size(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_size(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_size(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_size(),
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_size(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_size(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.get_size(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_size(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_size(),
            VfsFileEntry::Pdi(pdi_file_entry) => pdi_file_entry.get_size(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_size(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => sparseimage_file_entry.get_size(),
            VfsFileEntry::SplitRaw(splitraw_file_entry) => splitraw_file_entry.get_size(),
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_size(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_size(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_size(),
            VfsFileEntry::Vmdk(vmdk_file_entry) => vmdk_file_entry.get_size(),
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> Result<Option<Path>, ErrorTrace> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => Ok(None),
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_symbolic_link_target() {
                Ok(result) => match result {
                    Some(symbolic_link_target) => Ok(Some(Path::from(symbolic_link_target))),
                    None => Ok(None),
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve ext symbolic link target"
                    );
                    Err(error)
                }
            },
            VfsFileEntry::Hfs(hfs_file_entry) => match hfs_file_entry.get_symbolic_link_target() {
                Ok(result) => match result {
                    Some(symbolic_link_target) => Ok(Some(Path::from(symbolic_link_target))),
                    None => Ok(None),
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve HFS symbolic link target"
                    );
                    Err(error)
                }
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_symbolic_link_target()
            {
                Ok(result) => match result {
                    Some(name) => {
                        let path_components: Vec<PathComponent> = name
                            .elements
                            .split(|value| *value == 0x005c)
                            .skip(2) // Strip leading "\\??\\".
                            .map(|component| PathComponent::Ucs2String(Ucs2String::from(component)))
                            .collect::<Vec<PathComponent>>();

                        Ok(Some(Path::from(path_components)))
                    }
                    None => Ok(None),
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve NTFS symbolic link target"
                    );
                    Err(error)
                }
            },
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_symbolic_link_target() {
                Some(link_target) => Ok(Some(Path::from(&link_target))),
                None => Ok(None),
            },
        }
    }

    /// Retrieves the number of data forks.
    pub fn get_number_of_data_forks(&self) -> Result<usize, ErrorTrace> {
        let result: usize = match self {
            VfsFileEntry::Apm(apm_file_entry) => match apm_file_entry {
                ApmFileEntry::Partition { .. } => 1,
                ApmFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Ewf(ewf_file_entry) => match ewf_file_entry {
                EwfFileEntry::Layer { .. } => 1,
                EwfFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_type: u16 = ext_file_entry.get_file_mode() & 0xf000;

                if file_type != EXT_FILE_MODE_TYPE_REGULAR_FILE {
                    0
                } else {
                    1
                }
            }
            VfsFileEntry::Fake(fake_file_entry) => match fake_file_entry.get_file_type() {
                VfsFileType::File => 1,
                _ => 0,
            },
            VfsFileEntry::Fat(fat_file_entry) => {
                if fat_file_entry.is_directory() {
                    0
                } else {
                    1
                }
            }
            VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry {
                GptFileEntry::Partition { .. } => 1,
                GptFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Hfs(hfs_file_entry) => {
                let has_data_fork: bool = hfs_file_entry.has_data_fork();
                let has_resource_fork: bool = hfs_file_entry.has_resource_fork();

                if has_data_fork && has_resource_fork {
                    2
                } else if has_data_fork || has_resource_fork {
                    1
                } else {
                    0
                }
            }
            VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry {
                MbrFileEntry::Partition { .. } => 1,
                MbrFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                match ntfs_file_entry.get_number_of_data_forks() {
                    Ok(number_of_data_forks) => number_of_data_forks,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of data forks"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_file_type() {
                VfsFileType::File => 1,
                _ => 0,
            },
            VfsFileEntry::Pdi(pdi_file_entry) => match pdi_file_entry {
                PdiFileEntry::Layer { .. } => 1,
                PdiFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Qcow(qcow_file_entry) => match qcow_file_entry {
                QcowFileEntry::Layer { .. } => 1,
                QcowFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::SparseImage(sparseimage_file_entry) => match sparseimage_file_entry {
                SparseImageFileEntry::Layer { .. } => 1,
                SparseImageFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::SplitRaw(splitraw_file_entry) => match splitraw_file_entry {
                SplitRawFileEntry::Layer { .. } => 1,
                SplitRawFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Udif(udif_file_entry) => match udif_file_entry {
                UdifFileEntry::Layer { .. } => 1,
                UdifFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Vhd(vhd_file_entry) => match vhd_file_entry {
                VhdFileEntry::Layer { .. } => 1,
                VhdFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Vhdx(vhdx_file_entry) => match vhdx_file_entry {
                VhdxFileEntry::Layer { .. } => 1,
                VhdxFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::Vmdk(vmdk_file_entry) => match vmdk_file_entry {
                VmdkFileEntry::Layer { .. } => 1,
                VmdkFileEntry::Root { .. } => 0,
            },
        };
        Ok(result)
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(
        &mut self,
        data_fork_index: usize,
    ) -> Result<VfsDataFork, ErrorTrace> {
        let result: Result<Option<VfsDataFork>, ErrorTrace> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Ext(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => {
                if data_fork_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid data fork index: {}",
                        data_fork_index
                    )));
                }
                match self.get_data_stream()? {
                    Some(data_stream) => Ok(Some(VfsDataFork::DataStream(data_stream))),
                    None => Ok(None),
                }
            }
            VfsFileEntry::Hfs(hfs_file_entry) => {
                let has_data_fork: bool = hfs_file_entry.has_data_fork();
                let resource_fork_index: usize = if has_data_fork { 1 } else { 0 };

                if has_data_fork && data_fork_index == 0 {
                    match hfs_file_entry.get_data_fork()? {
                        Some(hfs_fork) => Ok(Some(VfsDataFork::Hfs(hfs_fork))),
                        None => Ok(None),
                    }
                } else if hfs_file_entry.has_resource_fork()
                    && data_fork_index == resource_fork_index
                {
                    match hfs_file_entry.get_resource_fork()? {
                        Some(hfs_fork) => Ok(Some(VfsDataFork::Hfs(hfs_fork))),
                        None => Ok(None),
                    }
                } else {
                    Err(keramics_core::error_trace_new!(format!(
                        "Invalid data fork index: {}",
                        data_fork_index
                    )))
                }
            }
            VfsFileEntry::Ntfs(ntfs_file_entry) => Ok(Some(VfsDataFork::Ntfs(
                ntfs_file_entry.get_data_fork_by_index(data_fork_index)?,
            ))),
        };
        match result {
            Ok(Some(data_fork)) => Ok(data_fork),
            Ok(None) => Err(keramics_core::error_trace_new!(format!(
                "Missing data fork: {}",
                data_fork_index
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve data fork: {}", data_fork_index)
                );
                Err(error)
            }
        }
    }

    // TODO: add get_data_fork_by_name

    /// Retrieves a data fork iterator.
    pub fn data_forks(&mut self) -> VfsDataForksIterator<'_> {
        VfsDataForksIterator::new(self)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&mut self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let result: Result<Option<DataStreamReference>, ErrorTrace> = match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_data_stream(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_data_stream(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_data_stream(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_data_stream(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_data_stream(),
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_data_stream(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_data_stream(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.get_data_stream(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_data_stream(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_data_stream(),
            VfsFileEntry::Pdi(pdi_file_entry) => pdi_file_entry.get_data_stream(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_data_stream(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.get_data_stream()
            }
            VfsFileEntry::SplitRaw(splitraw_file_entry) => splitraw_file_entry.get_data_stream(),
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_data_stream(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_data_stream(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_data_stream(),
            VfsFileEntry::Vmdk(vmdk_file_entry) => vmdk_file_entry.get_data_stream(),
        };
        match result {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                return Err(error);
            }
        }
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &mut self,
        name: Option<&PathComponent>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let result: Result<Option<DataStreamReference>, ErrorTrace> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Ext(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Hfs(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => match name {
                Some(_) => Ok(None),
                None => self.get_data_stream(),
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_data_stream_by_name(name),
        };
        match result {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                Err(error)
            }
        }
    }

    /// Retrieves the number of extended attributes.
    pub fn get_number_of_extended_attributes(&mut self) -> Result<usize, ErrorTrace> {
        let result: Result<usize, ErrorTrace> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => Ok(0),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_number_of_extended_attributes(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_number_of_extended_attributes(),
        };
        match result {
            Ok(number_of_extended_attributes) => Ok(number_of_extended_attributes),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of extended attributes"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_index(
        &mut self,
        extended_attribute_index: usize,
    ) -> Result<VfsExtendedAttribute, ErrorTrace> {
        let result: Result<Option<VfsExtendedAttribute>, ErrorTrace> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => Ok(None),
            VfsFileEntry::Ext(ext_file_entry) => Ok(Some(VfsExtendedAttribute::Ext(
                ext_file_entry.get_extended_attribute_by_index(extended_attribute_index)?,
            ))),
            VfsFileEntry::Hfs(hfs_file_entry) => Ok(Some(VfsExtendedAttribute::Hfs(
                hfs_file_entry.get_extended_attribute_by_index(extended_attribute_index)?,
            ))),
        };
        match result {
            Ok(Some(extended_attribute)) => Ok(extended_attribute),
            Ok(None) => Err(keramics_core::error_trace_new!(format!(
                "Missing extended attribute: {}",
                extended_attribute_index
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve extended attribute: {}",
                        extended_attribute_index
                    )
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific extended attribute.
    pub fn get_extended_attribute_by_name(
        &mut self,
        extended_attribute_name: &PathComponent,
    ) -> Result<Option<VfsExtendedAttribute>, ErrorTrace> {
        let result: Result<Option<VfsExtendedAttribute>, ErrorTrace> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Ntfs(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Pdi(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::SplitRaw(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_)
            | VfsFileEntry::Vmdk(_) => Ok(None),
            VfsFileEntry::Ext(ext_file_entry) => {
                match ext_file_entry.get_extended_attribute_by_name(extended_attribute_name)? {
                    Some(ext_extended_attribute) => {
                        Ok(Some(VfsExtendedAttribute::Ext(ext_extended_attribute)))
                    }
                    None => Ok(None),
                }
            }
            VfsFileEntry::Hfs(hfs_file_entry) => {
                match hfs_file_entry.get_extended_attribute_by_name(extended_attribute_name)? {
                    Some(hfs_extended_attribute) => {
                        Ok(Some(VfsExtendedAttribute::Hfs(hfs_extended_attribute)))
                    }
                    None => Ok(None),
                }
            }
        };
        match result {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve extended attribute"
                );
                Err(error)
            }
        }
    }

    /// Retrieves an extended attributes iterator.
    pub fn extended_attributes(&mut self) -> VfsExtendedAttributesIterator<'_> {
        VfsExtendedAttributesIterator::new(self)
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        let result: Result<usize, ErrorTrace> = match self {
            VfsFileEntry::Apm(apm_file_entry) => {
                Ok(apm_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Ewf(ewf_file_entry) => {
                Ok(ewf_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Fake(fake_file_entry) => {
                Ok(fake_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Gpt(gpt_file_entry) => {
                Ok(gpt_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Mbr(mbr_file_entry) => {
                Ok(mbr_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Pdi(pdi_file_entry) => {
                Ok(pdi_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Qcow(qcow_file_entry) => {
                Ok(qcow_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                Ok(sparseimage_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::SplitRaw(splitraw_file_entry) => {
                Ok(splitraw_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Udif(udif_file_entry) => {
                Ok(udif_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Vhd(vhd_file_entry) => {
                Ok(vhd_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Vhdx(vhdx_file_entry) => {
                Ok(vhdx_file_entry.get_number_of_sub_file_entries())
            }
            VfsFileEntry::Vmdk(vmdk_file_entry) => {
                Ok(vmdk_file_entry.get_number_of_sub_file_entries())
            }
        };
        match result {
            Ok(number_of_sub_file_entries) => Ok(number_of_sub_file_entries),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve number of sub file entries"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<VfsFileEntry, ErrorTrace> {
        let result: Result<VfsFileEntry, ErrorTrace> = match self {
            VfsFileEntry::Apm(apm_file_entry) => Ok(VfsFileEntry::Apm(
                apm_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Ewf(ewf_file_entry) => Ok(VfsFileEntry::Ewf(
                ewf_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Ext(ext_file_entry) => Ok(VfsFileEntry::Ext(
                ext_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Fake(fake_file_entry) => Ok(VfsFileEntry::Fake(
                fake_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Fat(fat_file_entry) => Ok(VfsFileEntry::Fat(
                fat_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Gpt(gpt_file_entry) => Ok(VfsFileEntry::Gpt(
                gpt_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Hfs(hfs_file_entry) => Ok(VfsFileEntry::Hfs(
                hfs_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Mbr(mbr_file_entry) => Ok(VfsFileEntry::Mbr(
                mbr_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Ntfs(ntfs_file_entry) => Ok(VfsFileEntry::Ntfs(
                ntfs_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Os(os_file_entry) => Ok(VfsFileEntry::Os(
                os_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Pdi(pdi_file_entry) => Ok(VfsFileEntry::Pdi(
                pdi_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Qcow(qcow_file_entry) => Ok(VfsFileEntry::Qcow(
                qcow_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => Ok(VfsFileEntry::SparseImage(
                sparseimage_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::SplitRaw(splitraw_file_entry) => Ok(VfsFileEntry::SplitRaw(
                splitraw_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Udif(udif_file_entry) => Ok(VfsFileEntry::Udif(
                udif_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Vhd(vhd_file_entry) => Ok(VfsFileEntry::Vhd(
                vhd_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Vhdx(vhdx_file_entry) => Ok(VfsFileEntry::Vhdx(
                vhdx_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
            VfsFileEntry::Vmdk(vmdk_file_entry) => Ok(VfsFileEntry::Vmdk(
                vmdk_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            )),
        };
        match result {
            Ok(sub_file_entry) => Ok(sub_file_entry),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve sub file entry: {}",
                        sub_file_entry_index
                    )
                );
                Err(error)
            }
        }
    }

    // TODO: add get sub_file_entry_by_name

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> VfsFileEntriesIterator<'_> {
        VfsFileEntriesIterator::new(self)
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.is_root_file_entry(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.is_root_file_entry(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.is_root_directory(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.is_root_file_entry(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.is_root_directory(),
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.is_root_file_entry(),
            VfsFileEntry::Hfs(hfs_file_entry) => hfs_file_entry.is_root_directory(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.is_root_file_entry(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.is_root_directory(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.is_root_file_entry(),
            VfsFileEntry::Pdi(pdi_file_entry) => pdi_file_entry.is_root_file_entry(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.is_root_file_entry(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.is_root_file_entry()
            }
            VfsFileEntry::SplitRaw(splitraw_file_entry) => splitraw_file_entry.is_root_file_entry(),
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.is_root_file_entry(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.is_root_file_entry(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.is_root_file_entry(),
            VfsFileEntry::Vmdk(vmdk_file_entry) => vmdk_file_entry.is_root_file_entry(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;
    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::{
        FatDate, FatTimeDate, FatTimeDate10Ms, Filetime, HfsTime, PosixTime32,
    };
    use keramics_encodings::CharacterEncoding;
    use keramics_formats::ext::ExtFileSystem;
    use keramics_formats::fat::FatFileSystem;
    use keramics_formats::hfs::HfsFileSystem;
    use keramics_formats::ntfs::NtfsFileSystem;
    use keramics_types::{ByteString, Utf16String};

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::{VfsLocation, new_os_vfs_location};
    use crate::types::VfsFileSystemReference;

    use crate::tests::get_test_data_path;

    fn get_parent_file_system() -> VfsFileSystemReference {
        VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os))
    }

    // Tests with APM.

    fn get_apm_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Apm);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_apm_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("apm2")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 8192);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("apm1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_apm() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with EWF.

    fn get_ewf_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ewf);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("ewf/ext2.E01");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ewf_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("ewf1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("ewf1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_ewf() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with ext.

    fn get_ext_file_system() -> Result<ExtFileSystem, ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_string: String = get_test_data_path("ext/ext2.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_ext_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let ext_file_system: ExtFileSystem = get_ext_file_system()?;

        let path: Path = Path::from(path_string);
        match ext_file_system.get_file_entry_by_path(&path)? {
            Some(ext_file_entry) => Ok(VfsFileEntry::Ext(ext_file_entry)),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(
            result,
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977482
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(
            result,
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, Some(0o100644));

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, Some(1000));

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, Some(14));

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(
            result,
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(ByteString::from("testfile1")))
        );

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, Some(2));

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, Some(1000));

        Ok(())
    }

    #[test]
    fn test_get_size_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/file_symboliclink1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;

        assert_eq!(
            link_target,
            Some(Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::ByteString(ByteString::from("mnt")),
                    PathComponent::ByteString(ByteString::from("keramics")),
                    PathComponent::ByteString(ByteString::from("testdir1")),
                    PathComponent::ByteString(ByteString::from("testfile1")),
                ],
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 1);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let extended_attribute: VfsExtendedAttribute =
            vfs_file_entry.get_extended_attribute_by_index(0)?;
        let expected_name: PathComponent = PathComponent::ByteString(ByteString {
            encoding: CharacterEncoding::Ascii,
            elements: vec![
                115, 101, 99, 117, 114, 105, 116, 121, 46, 115, 101, 108, 105, 110, 117, 120,
            ],
        });
        assert_eq!(extended_attribute.get_name(), expected_name,);

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let name: PathComponent = PathComponent::from("security.selinux");
        let extended_attribute: VfsExtendedAttribute = vfs_file_entry
            .get_extended_attribute_by_name(&name)?
            .unwrap();
        let expected_name: PathComponent = PathComponent::ByteString(ByteString {
            encoding: CharacterEncoding::Ascii,
            elements: vec![
                115, 101, 99, 117, 114, 105, 116, 121, 46, 115, 101, 108, 105, 110, 117, 120,
            ],
        });
        assert_eq!(extended_attribute.get_name(), expected_name);

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 10);

        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(ByteString::from("TestFile2")))
        );

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_ext() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(9).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with fake.

    fn get_fake_file_entry() -> VfsFileEntry {
        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];

        VfsFileEntry::Fake(Arc::new(FakeFileEntry::new_file("file.txt", &test_data)))
    }

    #[test]
    fn test_get_access_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("file.txt")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_fake() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    // TODO: add test_get_number_of_data_forks_with_fake
    // TODO: add test_data_forks_with_fake
    // TODO: add test_get_data_stream_with_fake
    // TODO: add test_get_data_stream_by_name_with_fake

    // TODO: add test_get_number_of_extended_attributes_with_fake
    // TODO: add test_get_extended_attribute_by_index_with_fake
    // TODO: add test_get_extended_attribute_by_name_with_fake
    // TODO: add test_extended_attributes_with_fake
    // TODO: add test_get_number_of_sub_file_entries_with_fake
    // TODO: add test_get_sub_file_entry_by_index_with_fake
    // TODO: add test_sub_file_entries_with_fake

    // Tests with FAT.

    fn get_fat_file_system() -> Result<FatFileSystem, ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_string: String = get_test_data_path("fat/fat12.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_fat_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let fat_file_system: FatFileSystem = get_fat_file_system()?;

        let path: Path = Path::from(path_string);
        match fat_file_system.get_file_entry_by_path(&path)? {
            Some(fat_file_entry) => Ok(VfsFileEntry::Fat(fat_file_entry)),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, Some(&DateTime::FatDate(FatDate { date: 0x5b53 })));
        Ok(())
    }

    #[test]
    fn test_get_change_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(
            result,
            Some(&DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5b53,
                time: 0x958f,
                fraction: 0x7d,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(
            result,
            Some(&DateTime::FatTimeDate(FatTimeDate {
                date: 0x5b53,
                time: 0x958f
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Ucs2String::from("testfile1")))
        );

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 3);

        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Ucs2String::from(
                "My long, very long file name, so very long"
            )))
        );

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_fat() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(2).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with GPT.

    fn get_gpt_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Gpt);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_gpt_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("gpt2")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 1572864);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("gpt1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_gpt() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with HFS.

    fn get_hfs_file_system() -> Result<HfsFileSystem, ErrorTrace> {
        let mut file_system: HfsFileSystem = HfsFileSystem::new();

        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_hfs_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let hfs_file_system: HfsFileSystem = get_hfs_file_system()?;

        let path: Path = Path::from(path_string);
        match hfs_file_system.get_file_entry_by_path(&path)? {
            Some(hfs_file_entry) => Ok(VfsFileEntry::Hfs(hfs_file_entry)),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(
            result,
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3814701237
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_change_time_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(
            result,
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3814701242
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(
            result,
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3814701220
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, Some(0o100644));

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, Some(20));

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, Some(21));

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(
            result,
            Some(&DateTime::HfsTime(HfsTime {
                timestamp: 3814701220
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Utf16String::from("testfile1")))
        );

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, Some(1));

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, Some(501));

        Ok(())
    }

    #[test]
    fn test_get_size_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/file_symboliclink1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;

        assert_eq!(
            link_target,
            Some(Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::ByteString(ByteString::from("Volumes")),
                    PathComponent::ByteString(ByteString::from("hfsplus_test")),
                    PathComponent::ByteString(ByteString::from("testdir1")),
                    PathComponent::ByteString(ByteString::from("testfile1")),
                ],
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_hfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/xattr1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 1);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/xattr1")?;

        let extended_attribute: VfsExtendedAttribute =
            vfs_file_entry.get_extended_attribute_by_index(0)?;
        let expected_name: PathComponent = PathComponent::Utf16String(Utf16String {
            elements: vec![109, 121, 120, 97, 116, 116, 114, 49],
        });
        assert_eq!(extended_attribute.get_name(), expected_name,);

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/xattr1")?;

        let name: PathComponent = PathComponent::from("myxattr1");
        let extended_attribute: VfsExtendedAttribute = vfs_file_entry
            .get_extended_attribute_by_name(&name)?
            .unwrap();
        let expected_name: PathComponent = PathComponent::Utf16String(Utf16String {
            elements: vec![109, 121, 120, 97, 116, 116, 114, 49],
        });
        assert_eq!(extended_attribute.get_name(), expected_name);

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/xattr1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 5);

        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1/testfile1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Utf16String::from("large_xattr")))
        );

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_hfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_hfs_file_entry("/testdir1")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(9).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with MBR.

    fn get_mbr_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Mbr);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("mbr2")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 1573376);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("mbr1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_mbr() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with NTFS.

    fn get_ntfs_file_system() -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    fn get_ntfs_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let ntfs_file_system: NtfsFileSystem = get_ntfs_file_system()?;

        let path: Path = Path::from(path_string);
        match ntfs_file_system.get_file_entry_by_path(&path)? {
            Some(ntfs_file_entry) => Ok(VfsFileEntry::Ntfs(ntfs_file_entry)),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path_string
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(
            result,
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba6892474
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(
            result,
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba689275d
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(
            result,
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba6892474
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(
            result,
            Some(&DateTime::Filetime(Filetime {
                timestamp: 0x1db5e8ba689275d
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_name_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Ucs2String::from("testfile1")))
        );

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 9);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        // TODO: add test with link target

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/$UpCase")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("$Info"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 3);

        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(
            name,
            Some(PathComponent::from(Ucs2String::from(
                "My long, very long file name, so very long"
            )))
        );

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_ntfs() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(2).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with OS.

    fn get_os_file_entry(path_string: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let test_data_path_string: String = get_test_data_path(path_string);
        let path: Path = Path::from(test_data_path_string.as_str());
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        if cfg!(windows) {
            assert_eq!(result, None);
        } else {
            // Note that the value can vary.
            assert!(result.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        if cfg!(windows) {
            assert_eq!(file_mode, None);
        } else {
            // Note that the value can vary.
            assert!(file_mode.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_file_type_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        if cfg!(windows) {
            assert_eq!(group_identifier, None);
        } else {
            // Note that the value can vary.
            assert!(group_identifier.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        if cfg!(windows) {
            assert_eq!(inode_number, None);
        } else {
            // Note that the value can vary.
            assert!(inode_number.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        // Note that the value can vary.
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from(OsString::from("file.txt"))));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        if cfg!(windows) {
            assert_eq!(number_of_links, None);
        } else {
            assert_eq!(number_of_links, Some(1));
        }
        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        if cfg!(windows) {
            assert_eq!(owner_identifier, None);
        } else {
            // Note that the value can vary.
            assert!(owner_identifier.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_size_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 202);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/symbolic_link")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(
            link_target,
            Some(Path {
                components: vec![PathComponent::OsString(OsString::from("file.txt")),],
            })
        );
        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory/file.txt")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        // Note that the value can vary.
        assert!(name.is_some());

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_os() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_os_file_entry("directory")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with PDI.

    fn get_pdi_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Pdi);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd/DiskDescriptor.xml");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_pdi_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_pdi_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("pdi1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 33554432);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/pdi1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("pdi1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_pdi() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_pdi_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with QCOW.

    fn get_qcow_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Qcow);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("qcow1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("qcow1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_qcow() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with sparse image.

    fn get_sparseimage_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SparseImage);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_sparseimage_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let path: Path = Path::from("/sparseimage1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("sparseimage1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("sparseimage1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_sparseimage() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with split raw.

    fn get_splitraw_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SplitRaw);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("splitraw/ext2.raw.000");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_splitraw_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_splitraw_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("raw1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/raw1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("raw1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_splitraw() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_splitraw_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with UDIF.

    fn get_udif_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Udif);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_udif_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("udif1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 1964032);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("udif1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_udif() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with VHD.

    fn get_vhd_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhd);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vhd2")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vhd1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_vhd() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with VHDX.

    fn get_vhdx_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhdx);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vhdx2")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 2);

        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vhdx1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_vhdx() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> =
            sub_file_entries_iterator.skip(1).next();
        assert!(result.is_none());

        Ok(())
    }

    // Tests with VMDK.

    fn get_vmdk_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vmdk);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vmdk_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vmdk_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_device_identifier_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let device_identifier: Option<u64> = vfs_file_entry.get_device_identifier()?;
        assert_eq!(device_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_file_mode_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let file_mode: Option<u32> = vfs_file_entry.get_file_mode();
        assert_eq!(file_mode, None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_group_identifier_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let group_identifier: Option<u32> = vfs_file_entry.get_group_identifier();
        assert_eq!(group_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_inode_number_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let inode_number: Option<u64> = vfs_file_entry.get_inode_number();
        assert_eq!(inode_number, None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_get_name_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let name: Option<PathComponent> = vfs_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vmdk1")));

        Ok(())
    }

    #[test]
    fn test_get_number_of_links_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let number_of_links: Option<u64> = vfs_file_entry.get_number_of_links();
        assert_eq!(number_of_links, None);

        Ok(())
    }

    #[test]
    fn test_get_owner_identifier_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let owner_identifier: Option<u32> = vfs_file_entry.get_owner_identifier();
        assert_eq!(owner_identifier, None);

        Ok(())
    }

    #[test]
    fn test_get_size_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_number_of_data_forks_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 0);

        let vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let number_of_data_forks: usize = vfs_file_entry.get_number_of_data_forks()?;
        assert_eq!(number_of_data_forks, 1);

        Ok(())
    }

    #[test]
    fn test_data_forks_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let mut data_forks_iterator: VfsDataForksIterator = vfs_file_entry.data_forks();

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsDataFork, ErrorTrace>> = data_forks_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_by_name_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let name: Option<PathComponent> = None;
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_some());

        let name: Option<PathComponent> = Some(PathComponent::from("bogus"));
        let result: Option<DataStreamReference> =
            vfs_file_entry.get_data_stream_by_name(name.as_ref())?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_extended_attributes_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let number_of_extended_attributes: usize =
            vfs_file_entry.get_number_of_extended_attributes()?;
        assert_eq!(number_of_extended_attributes, 0);

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_index_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let result: Result<VfsExtendedAttribute, ErrorTrace> =
            vfs_file_entry.get_extended_attribute_by_index(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_get_extended_attribute_by_name_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let name: PathComponent = PathComponent::from("bogus");
        let result: Option<VfsExtendedAttribute> =
            vfs_file_entry.get_extended_attribute_by_name(&name)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_extended_attributes_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let mut extended_attributes_iterator: VfsExtendedAttributesIterator =
            vfs_file_entry.extended_attributes();

        let result: Option<Result<VfsExtendedAttribute, ErrorTrace>> =
            extended_attributes_iterator.next();
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_sub_file_entries_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 1);

        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/vmdk1")?;

        let number_of_sub_file_entries: usize = vfs_file_entry.get_number_of_sub_file_entries()?;
        assert_eq!(number_of_sub_file_entries, 0);

        Ok(())
    }

    #[test]
    fn test_test_get_sub_file_entry_by_index_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let sub_file_entry: VfsFileEntry = vfs_file_entry.get_sub_file_entry_by_index(0)?;

        let name: Option<PathComponent> = sub_file_entry.get_name();
        assert_eq!(name, Some(PathComponent::from("vmdk1")));

        let result: Result<VfsFileEntry, ErrorTrace> =
            vfs_file_entry.get_sub_file_entry_by_index(99);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_sub_file_entries_with_vmdk() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_vmdk_file_entry("/")?;

        let mut sub_file_entries_iterator: VfsFileEntriesIterator =
            vfs_file_entry.sub_file_entries();

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        let result: Option<Result<VfsFileEntry, ErrorTrace>> = sub_file_entries_iterator.next();
        assert!(result.is_none());

        Ok(())
    }
}
