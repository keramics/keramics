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
use keramics_formats::ext::constants::*;
use keramics_formats::ext::{ExtFileEntry, ExtPath};
use keramics_formats::fat::{FatFileEntry, FatPath, FatString};
use keramics_formats::ntfs::{NtfsDataFork, NtfsFileEntry, NtfsPath};
use keramics_types::{ByteString, Ucs2String};

use super::apm::ApmFileEntry;
use super::data_fork::VfsDataFork;
use super::enums::VfsFileType;
use super::ewf::EwfFileEntry;
use super::fake::FakeFileEntry;
use super::gpt::GptFileEntry;
use super::iterators::VfsFileEntriesIterator;
use super::mbr::MbrFileEntry;
use super::os::OsFileEntry;
use super::path::VfsPath;
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
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_name() {
                Some(name) => Some(VfsString::Byte(name.clone())),
                None => None,
            },
            VfsFileEntry::Ewf(ewf_file_entry) => match ewf_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Gpt(gpt_file_entry) => match gpt_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Fat(fat_file_entry) => match fat_file_entry.get_name() {
                Some(name) => match name {
                    FatString::ByteString(byte_string) => Some(VfsString::Byte(byte_string)),
                    FatString::Ucs2String(ucs2_string) => Some(VfsString::Ucs2(ucs2_string)),
                },
                None => None,
            },
            VfsFileEntry::Mbr(mbr_file_entry) => match mbr_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_name() {
                Some(name) => Some(VfsString::Ucs2(name.clone())),
                None => None,
            },
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_name() {
                Some(name) => Some(VfsString::OsString(name.to_os_string())),
                None => None,
            },
            VfsFileEntry::Qcow(qcow_file_entry) => match qcow_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                match sparseimage_file_entry.get_name() {
                    Some(name) => Some(VfsString::String(name)),
                    None => None,
                }
            }
            VfsFileEntry::Udif(udif_file_entry) => match udif_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Vhd(vhd_file_entry) => match vhd_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
            VfsFileEntry::Vhdx(vhdx_file_entry) => match vhdx_file_entry.get_name() {
                Some(name) => Some(VfsString::String(name)),
                None => None,
            },
        }
    }

    /// Retrieves the size.
    pub fn get_size(&self) -> u64 {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ewf(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::SparseImage(_)
            | VfsFileEntry::Udif(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => 1,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_size(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.get_size(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_size(),
            VfsFileEntry::Os(_) => todo!(),
        }
    }

    /// Retrieves the symbolic link target.
    pub fn get_symbolic_link_target(&mut self) -> Result<Option<VfsPath>, ErrorTrace> {
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
                    Some(name) => {
                        let path_components: Vec<ByteString> = name
                            .elements
                            .split(|value| *value == 0x2f)
                            .map(|component| ByteString::from(component))
                            .collect::<Vec<ByteString>>();
                        Ok(Some(VfsPath::Ext(ExtPath {
                            components: path_components,
                        })))
                    }
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
                        let path_components: Vec<Ucs2String> = name
                            .elements
                            .split(|value| *value == 0x005c)
                            .skip(2) // Strip leading "\\??\\".
                            .map(|component| Ucs2String::from(component))
                            .collect::<Vec<Ucs2String>>();
                        Ok(Some(VfsPath::Ntfs(NtfsPath {
                            components: path_components,
                        })))
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
            VfsFileEntry::Os(_) => todo!(),
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
            VfsFileEntry::Apm(_) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => {
                if data_fork_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid data fork index: {}",
                        data_fork_index
                    )));
                }
                let result: Option<DataStreamReference> = match ext_file_entry.get_data_stream() {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve ext data stream"
                        );
                        return Err(error);
                    }
                };
                match result {
                    Some(data_stream) => VfsDataFork::Ext(data_stream),
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing ext data stream"));
                    }
                }
            }
            VfsFileEntry::Ewf(_) => todo!(),
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Fat(fat_file_entry) => {
                if data_fork_index != 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid data fork index: {}",
                        data_fork_index
                    )));
                }
                let result: Option<DataStreamReference> = match fat_file_entry.get_data_stream() {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve FAT data stream"
                        );
                        return Err(error);
                    }
                };
                match result {
                    Some(data_stream) => VfsDataFork::Fat(data_stream),
                    None => {
                        return Err(keramics_core::error_trace_new!("Missing FAT data stream"));
                    }
                }
            }
            VfsFileEntry::Gpt(_) => todo!(),
            VfsFileEntry::Mbr(_) => todo!(),
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
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(_) => todo!(),
            VfsFileEntry::SparseImage(_) => todo!(),
            VfsFileEntry::Udif(_) => todo!(),
            VfsFileEntry::Vhd(_) => todo!(),
            VfsFileEntry::Vhdx(_) => todo!(),
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
        name: Option<&str>,
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
                    Some(string) => Some(Ucs2String::from(string)),
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
            VfsFileEntry::Apm(apm_file_entry) => {
                match apm_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of APM sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
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
            VfsFileEntry::Ewf(ewf_file_entry) => {
                match ewf_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of EWF sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
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
            VfsFileEntry::Gpt(gpt_file_entry) => {
                match gpt_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of GPT sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Mbr(mbr_file_entry) => {
                match mbr_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of MBR sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
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
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(qcow_file_entry) => {
                match qcow_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of QCOW sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::SparseImage(sparseimage_file_entry) => {
                match sparseimage_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of sparseimage sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Udif(udif_file_entry) => {
                match udif_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of UDIF sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Vhd(vhd_file_entry) => {
                match vhd_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of VHD sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileEntry::Vhdx(vhdx_file_entry) => {
                match vhdx_file_entry.get_number_of_sub_file_entries() {
                    Ok(number_of_sub_file_entries) => number_of_sub_file_entries,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve number of VHDX sub file entries"
                        );
                        return Err(error);
                    }
                }
            }
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
            VfsFileEntry::Os(_) => todo!(),
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
        let number_of_sub_file_entries: usize = self.get_number_of_sub_file_entries()?;
        Ok(VfsFileEntriesIterator::new(
            self,
            number_of_sub_file_entries,
        ))
    }

    /// Determines if the file entry is the root directory.
    pub fn is_root_directory(&self) -> bool {
        match self {
            VfsFileEntry::Apm(apm_file_entry) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.is_root_directory(),
            VfsFileEntry::Ewf(ewf_file_entry) => todo!(),
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Fat(fat_file_entry) => fat_file_entry.is_root_directory(),
            VfsFileEntry::Gpt(gpt_file_entry) => todo!(),
            VfsFileEntry::Mbr(mbr_file_entry) => todo!(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.is_root_directory(),
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(qcow_file_entry) => todo!(),
            VfsFileEntry::SparseImage(sparseimage_file_entry) => todo!(),
            VfsFileEntry::Udif(udif_file_entry) => todo!(),
            VfsFileEntry::Vhd(vhd_file_entry) => todo!(),
            VfsFileEntry::Vhdx(vhdx_file_entry) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::{FatDate, FatTimeDate, FatTimeDate10Ms, PosixTime32};

    use crate::enums::{VfsFileType, VfsType};
    use crate::file_system::VfsFileSystem;
    use crate::location::{VfsLocation, new_os_vfs_location};
    use crate::path::VfsPath;
    use crate::types::VfsFileSystemReference;

    use crate::tests::get_test_data_path;

    fn get_parent_file_system() -> VfsFileSystemReference {
        VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os))
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

    fn get_ext_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ext);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("ext/ext2.raw").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

    fn get_fat_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Fat);

        let parent_file_system: VfsFileSystemReference = get_parent_file_system();
        let vfs_location: VfsLocation =
            new_os_vfs_location(get_test_data_path("fat/fat12.raw").as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_fat_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

    fn get_os_file_entry(path: &str) -> Result<VfsFileEntry, ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
            Some(file_entry) => Ok(file_entry),
            None => Err(keramics_core::error_trace_new!(format!(
                "No such file entry: {}",
                path
            ))),
        }
    }

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

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, path);
        match vfs_file_system.get_file_entry_by_path(&vfs_path)? {
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

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_access_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977482
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_access_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    // TODO: add test_get_access_time_with_fake

    #[test]
    fn test_get_access_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_access_time(),
            Some(&DateTime::FatDate(FatDate { date: 0x5b53 }))
        );
        Ok(())
    }

    #[test]
    fn test_get_access_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    // TODO: add test_get_access_time_with_ntfs

    #[test]
    fn test_get_access_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry =
            get_os_file_entry(get_test_data_path("file.txt").as_str())?;

        assert_ne!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx1")?;

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_change_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    // TODO: add test_get_change_time_with_fake

    #[test]
    fn test_get_change_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    // TODO: add test_get_change_time_with_ntfs

    #[test]
    fn test_get_change_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    // TODO: add test_get_change_time_with_os

    #[test]
    fn test_get_change_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    // TODO: add test_get_creation_time_with_fake

    #[test]
    fn test_get_creation_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_creation_time(),
            Some(&DateTime::FatTimeDate10Ms(FatTimeDate10Ms {
                date: 0x5b53,
                time: 0x958f,
                fraction: 0x7d,
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    // TODO: add test_get_creation_time_with_ntfs

    #[test]
    fn test_get_creation_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    // TODO: add test_get_creation_time_with_os

    #[test]
    fn test_get_creation_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add test_get_file_type_with_fake

    #[test]
    fn test_get_file_type_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/testdir1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add test_get_file_type_with_ntfs

    #[test]
    fn test_get_file_type_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add test_get_file_type_with_os

    #[test]
    fn test_get_file_type_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/sparseimage1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/udif1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add tests for get_group_identifier

    #[test]
    fn test_get_modification_time_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_apm_file_entry("/apm2")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ext_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_modification_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_ewf_file_entry("/ewf1")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add test_get_modification_time_with_fake

    #[test]
    fn test_get_modification_time_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_fat_file_entry("/testdir1/testfile1")?;

        assert_eq!(
            vfs_file_entry.get_modification_time(),
            Some(&DateTime::FatTimeDate(FatTimeDate {
                date: 0x5b53,
                time: 0x958f
            }))
        );
        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_gpt_file_entry("/gpt2")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add test_get_modification_time_with_ntfs

    #[test]
    fn test_get_modification_time_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_mbr_file_entry("/mbr2")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add test_get_modification_time_with_os

    #[test]
    fn test_get_modification_time_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_qcow_file_entry("/qcow1")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_sparseimage_file_entry("/sparseimage1")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_udif_file_entry("/udif1")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhd_file_entry("/vhd2")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_entry: VfsFileEntry = get_vhdx_file_entry("/vhdx2")?;

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add tests for get_name
    // TODO: add tests for get_number_of_links
    // TODO: add tests for get_size
    // TODO: add tests for get_symbolic_link_target
    // TODO: add tests for get_number_of_data_forks

    #[test]
    fn test_get_data_stream_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_data_stream_with_fake

    #[test]
    fn test_get_data_stream_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/testdir1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_data_stream_with_ntfs

    #[test]
    fn test_get_data_stream_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_data_stream_with_os

    #[test]
    fn test_get_data_stream_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/sparseimage1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/udif1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<DataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
    // TODO: add tests for sub_file_entries
}
