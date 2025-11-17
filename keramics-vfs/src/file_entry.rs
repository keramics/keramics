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

use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_formats::ext::ExtFileEntry;
use keramics_formats::ext::constants::*;
use keramics_formats::fat::FatFileEntry;
use keramics_formats::ntfs::{NtfsDataFork, NtfsFileEntry};
use keramics_formats::{Path, PathComponent};
use keramics_types::Ucs2String;

use super::apm::ApmFileEntry;
use super::data_fork::VfsDataFork;
use super::enums::VfsFileType;
use super::ewf::EwfFileEntry;
use super::fake::FakeFileEntry;
use super::gpt::GptFileEntry;
use super::iterators::VfsFileEntriesIterator;
use super::mbr::MbrFileEntry;
use super::os::OsFileEntry;
use super::qcow::QcowFileEntry;
use super::sparseimage::SparseImageFileEntry;
use super::string::VfsString;
use super::udif::UdifFileEntry;
use super::vhd::VhdFileEntry;
use super::vhdx::VhdxFileEntry;

/// Virtual File System (VFS) file entry.
pub enum VfsFileEntry {
    Apm(ApmFileEntry),
    Ext(ExtFileEntry),
    Ewf(EwfFileEntry),
    Fake(Arc<FakeFileEntry>),
    Fat(FatFileEntry),
    Gpt(GptFileEntry),
    Mbr(MbrFileEntry),
    Ntfs(NtfsFileEntry),
    Os(OsFileEntry),
    Qcow(QcowFileEntry),
    SparseImage(SparseImageFileEntry),
    Udif(UdifFileEntry),
    Vhd(VhdFileEntry),
    Vhdx(VhdxFileEntry),
}

impl VfsFileEntry {
    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_access_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_access_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_access_time(),
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
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_change_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_change_time(),
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
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_creation_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_creation_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_creation_time(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_creation_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_creation_time(),
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_file_type(),
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_mode: u16 = ext_file_entry.get_file_mode();
                let file_type: u16 = file_mode & 0xf000;
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
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_file_type(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_file_type(),
            VfsFileEntry::Fat(fat_file_entry) => {
                if fat_file_entry.is_directory() {
                    VfsFileType::Directory
                } else {
                    VfsFileType::File
                }
            }
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_file_type(),
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
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_file_type(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.get_file_type()
            }
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_file_type(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_file_type(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_file_type(),
        }
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_modification_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_modification_time(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_modification_time(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_modification_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_modification_time(),
        }
    }

    /// Retrieves the name.
    pub fn get_name(&self) -> Option<VfsString> {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => match apm_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Ewf(ewf_file_entry) => match ewf_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Fake(fake_file_entry) => match fake_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Fat(fat_file_entry) => match fat_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Qcow(qcow_file_entry) => match qcow_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                match sparseimage_file_entry.get_name() {
                    Some(name) => Some(VfsString::from(name)),
                    None => None,
                }
            }
            VfsFileEntry::Udif(udif_file_entry) => match udif_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Vhd(vhd_file_entry) => match vhd_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
            VfsFileEntry::Vhdx(vhdx_file_entry) => match vhdx_file_entry.get_name() {
                Some(name) => Some(VfsString::from(name)),
                None => None,
            },
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_size(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_size(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_size(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_size(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_size(),
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_size(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.get_size(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_size(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_size(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_size(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => sparseimage_file_entry.get_size(),
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_size(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_size(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_size(),
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
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => Ok(None),
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
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_mode: u16 = ext_file_entry.get_file_mode();
                if file_mode & 0xf000 != EXT_FILE_MODE_TYPE_REGULAR_FILE {
                    0
                } else {
                    1
                }
            }
            VfsFileEntry::Ewf(ewf_file_entry) => match ewf_file_entry {
                EwfFileEntry::Layer { .. } => 1,
                EwfFileEntry::Root { .. } => 0,
            },
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
            VfsFileEntry::Qcow(qcow_file_entry) => match qcow_file_entry {
                QcowFileEntry::Layer { .. } => 1,
                QcowFileEntry::Root { .. } => 0,
            },
            VfsFileEntry::SparseImage(sparseimage_file_entry) => match sparseimage_file_entry {
                SparseImageFileEntry::Layer { .. } => 1,
                SparseImageFileEntry::Root { .. } => 0,
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
        };
        Ok(result)
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(
        &self,
        data_fork_index: usize,
    ) -> Result<VfsDataFork<'_>, ErrorTrace> {
        let data_fork: VfsDataFork = match self {
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let ntfs_data_fork: NtfsDataFork =
                    match ntfs_file_entry.get_data_fork_by_index(data_fork_index) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve NTFS data stream"
                            );
                            return Err(error);
                        }
                    };
                VfsDataFork::Ntfs(ntfs_data_fork)
            }
            _ => {
                if data_fork_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid data fork index: {}",
                        data_fork_index
                    )));
                }
                match self.get_data_stream() {
                    Ok(Some(data_stream)) => VfsDataFork::DataStream(data_stream),
                    Ok(None) => {
                        return Err(keramics_core::error_trace_new!("Missing data stream"));
                    }
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve data stream"
                        );
                        return Err(error);
                    }
                }
            }
        };
        Ok(data_fork)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let result: Option<DataStreamReference> = match self {
            VfsFileEntry::Apm(apm_file_entry) => match apm_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve APM data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve ext data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Ewf(ewf_file_entry) => match ewf_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve EWF data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Fake(fake_file_entry) => match fake_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve fake data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Fat(fat_file_entry) => match fat_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve FAT data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve GPT data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve MBR data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve NTFS data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve OS data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Qcow(qcow_file_entry) => match qcow_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve QCOW data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                match sparseimage_file_entry.get_data_stream() {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve sparseimage data stream"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Udif(udif_file_entry) => match udif_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve UDIF data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Vhd(vhd_file_entry) => match vhd_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve VHD data stream"
                    );
                    return Err(error);
                }
            },
            VfsFileEntry::Vhdx(vhdx_file_entry) => match vhdx_file_entry.get_data_stream() {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve VHDX data stream"
                    );
                    return Err(error);
                }
            },
        };
        Ok(result)
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: Option<&VfsString>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let result: Option<DataStreamReference> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ext(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Fat(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => match name {
                Some(_) => None,
                None => match self.get_data_stream() {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve data stream"
                        );
                        return Err(error);
                    }
                },
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let ntfs_name: Option<Ucs2String> = match name {
                    Some(vfs_string) => Some(vfs_string.to_ucs2string()),
                    None => None,
                };
                match ntfs_file_entry.get_data_stream_by_name(&ntfs_name) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve NTFS data stream"
                        );
                        return Err(error);
                    }
                }
            }
        };
        Ok(result)
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> Result<usize, ErrorTrace> {
        let number_of_sub_file_entries: usize = match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Ext(ext_file_entry) => {
                match ext_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of ext sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Fat(fat_file_entry) => {
                match fat_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of FAT sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                match ntfs_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of NTFS sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Os(os_file_entry) => {
                match os_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of OS sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.get_number_of_sub_file_entries()
            }
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.get_number_of_sub_file_entries(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.get_number_of_sub_file_entries(),
        };
        Ok(number_of_sub_file_entries)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> Result<VfsFileEntry, ErrorTrace> {
        let sub_file_entry: VfsFileEntry = match self {
            VfsFileEntry::Apm(apm_file_entry) => {
                match apm_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Apm(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve APM sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Ext(ext_file_entry) => {
                match ext_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Ext(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve ext sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Ewf(ewf_file_entry) => {
                match ewf_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Ewf(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve EWF sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Fat(fat_file_entry) => {
                match fat_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Fat(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve FAT sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Gpt(gpt_file_entry) => {
                match gpt_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Gpt(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve GPT sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Mbr(mbr_file_entry) => {
                match mbr_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Mbr(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve MBR sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                match ntfs_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Ntfs(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve NTFS sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Os(os_file_entry) => {
                match os_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Os(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve OS sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Qcow(qcow_file_entry) => {
                match qcow_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Qcow(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve QCOW sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                match sparseimage_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::SparseImage(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve sparseimage sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Udif(udif_file_entry) => {
                match udif_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Udif(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve UDIF sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Vhd(vhd_file_entry) => {
                match vhd_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Vhd(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve VHD sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Vhdx(vhdx_file_entry) => {
                match vhdx_file_entry.get_sub_file_entry_by_index(sub_file_entry_index) {
                    Ok(sub_file_entry) => VfsFileEntry::Vhdx(sub_file_entry),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve VHDX sub file entry: {}",
                                sub_file_entry_index
                            )
                        );
                        return Err(error);
                    }
                }
            }
        };
        Ok(sub_file_entry)
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> Result<VfsFileEntriesIterator<'_>, ErrorTrace> {
        let number_of_sub_file_entries: usize = match self.get_number_of_sub_file_entries() {
            Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine number of sub file entries"
                );
                return Err(error);
            }
        };
        Ok(VfsFileEntriesIterator::new(
            self,
            number_of_sub_file_entries,
        ))
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => apm_file_entry.is_root_file_entry(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.is_root_directory(),
            VfsFileEntry::Ewf(ewf_file_entry) => ewf_file_entry.is_root_file_entry(),
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.is_root_directory(),
            VfsFileEntry::Gpt(gpt_file_entry) => gpt_file_entry.is_root_file_entry(),
            VfsFileEntry::Mbr(mbr_file_entry) => mbr_file_entry.is_root_file_entry(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.is_root_directory(),
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(qcow_file_entry) => qcow_file_entry.is_root_file_entry(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                sparseimage_file_entry.is_root_file_entry()
            }
            VfsFileEntry::Udif(udif_file_entry) => udif_file_entry.is_root_file_entry(),
            VfsFileEntry::Vhd(vhd_file_entry) => vhd_file_entry.is_root_file_entry(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => vhdx_file_entry.is_root_file_entry(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;
    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_datetime::{FatDate, FatTimeDate, FatTimeDate10Ms, Filetime, PosixTime32};
    use keramics_formats::ext::ExtFileSystem;
    use keramics_formats::fat::FatFileSystem;
    use keramics_formats::ntfs::NtfsFileSystem;
    use keramics_types::ByteString;

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
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("apm/apm.dmg").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_apm_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("apm2")));

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
    fn test_get_data_stream_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_apm

    // Tests with ext.

    fn get_ext_file_system() -> Result<ExtFileSystem, ErrorTrace> {
        let mut file_system: ExtFileSystem = ExtFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ext/ext2.raw").as_str());
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
                "No such file entry: {}",
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
    fn test_get_file_type_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from(ByteString::from("testfile1"))));
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
                    PathComponent::ByteString(ByteString::from("")),
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
    fn test_get_data_stream_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for test_get_number_of_sub_file_entries_with_ext
    // TODO: add tests for test_get_sub_file_entry_by_index_with_ext

    // Tests with EWF.

    fn get_ewf_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ewf);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("ewf/ext2.E01").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ewf_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("ewf1")));

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
    fn test_get_data_stream_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_ewf

    // Tests with fake.

    fn get_fake_file_entry() -> VfsFileEntry {
        let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];

        VfsFileEntry::Fake(Arc::new(FakeFileEntry::new_file("file.txt", &test_data)))
    }

    #[test]
    fn test_get_access_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_file_type_with_fake

    #[test]
    fn test_get_modification_time_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fake_file_entry();

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("file.txt")));

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

    // TODO: add test_get_data_stream_with_fake

    // TODO: add tests for test_get_number_of_sub_file_entries_with_fake
    // TODO: add tests for test_get_sub_file_entry_by_index_with_fake

    // Tests with FAT.

    fn get_fat_file_system() -> Result<FatFileSystem, ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("fat/fat12.raw").as_str());
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
                "No such file entry: {}",
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
    fn test_get_file_type_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from(Ucs2String::from("testfile1"))));
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
    fn test_get_data_stream_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for test_get_number_of_sub_file_entries_with_fat
    // TODO: add tests for test_get_sub_file_entry_by_index_with_fat

    // Tests with GPT.

    fn get_gpt_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Gpt);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("gpt/gpt.raw").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_gpt_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("gpt2")));

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
    fn test_get_data_stream_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_gpt

    // Tests with MBR.

    fn get_mbr_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Mbr);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("mbr/mbr.raw").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("mbr2")));

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
    fn test_get_data_stream_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_mbr

    // Tests with NTFS.

    fn get_ntfs_file_system() -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
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
                "No such file entry: {}",
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
    fn test_get_file_type_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from(Ucs2String::from("testfile1"))));
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
    fn test_get_data_stream_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_ntfs_file_entry("/testdir1/testfile1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for test_get_number_of_sub_file_entries_with_ntfs
    // TODO: add tests for test_get_sub_file_entry_by_index_with_ntfs

    // Tests with OS.

    fn get_os_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

    #[test]
    fn test_get_access_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let result: Option<&DateTime> = vfs_file_entry.get_access_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let result: Option<&DateTime> = vfs_file_entry.get_change_time();
        if cfg!(windows) {
            assert!(result.is_none());
        } else {
            assert!(result.is_some());
        }
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let result: Option<&DateTime> = vfs_file_entry.get_creation_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory").as_str())?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let result: Option<&DateTime> = vfs_file_entry.get_modification_time();
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_name_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from(OsString::from("file.txt"))));

        Ok(())
    }

    #[test]
    fn test_get_size_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("directory/file.txt").as_str())?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 202);

        Ok(())
    }

    // TODO: add test_get_symbolic_link_target_with_os

    // TODO: add test_get_data_stream_with_os

    // TODO: add tests for test_get_number_of_sub_file_entries_with_os
    // TODO: add tests for test_get_sub_file_entry_by_index_with_os

    // Tests with QCOW.

    fn get_qcow_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Qcow);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("qcow/ext2.qcow2").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("qcow1")));

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
    fn test_get_data_stream_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_qcow

    // Tests with sparse image.

    fn get_sparseimage_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SparseImage);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("sparseimage/hfsplus.sparseimage").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_sparseimage_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let path: Path = Path::from("/sparseimage1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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
    fn test_get_name_with_sparseiamge() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("sparseimage1")));
        Ok(())
    }

    #[test]
    fn test_get_size_with_sparseiamge() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let size: u64 = vfs_file_entry.get_size();
        assert_eq!(size, 4194304);

        Ok(())
    }

    #[test]
    fn test_get_symbolic_link_target_with_sparseiamge() -> Result<(), ErrorTrace> {
        let mut vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let link_target: Option<Path> = vfs_file_entry.get_symbolic_link_target()?;
        assert_eq!(link_target, None);

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_sparseimage

    // Tests with UDIF.

    fn get_udif_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Udif);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("udif/hfsplus_zlib.dmg").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_udif_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("udif1")));

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
    fn test_get_data_stream_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_udif

    // Tests with VHD.

    fn get_vhd_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhd);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("vhd/ntfs-differential.vhd").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("vhd2")));

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
    fn test_get_data_stream_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_vhd

    // Tests with VHDX.

    fn get_vhdx_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhdx);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("vhdx/ntfs-differential.vhdx").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from(path);
        match vfs_file_system.get_file_entry_by_path(&path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
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
    fn test_get_file_type_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

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

        let name: Option<VfsString> = vfs_file_entry.get_name();
        assert_eq!(name, Some(VfsString::from("vhdx2")));

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
    fn test_get_data_stream_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

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

    // TODO: add tests for test_get_sub_file_entry_by_index_with_vhdx

    // Other tests.

    // TODO: add tests for get_group_identifier

    // TODO: add tests for get_number_of_data_forks

    // TODO: add tests for sub_file_entries
}
