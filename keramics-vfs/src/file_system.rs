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

use std::path::Path;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::ext::{ExtFileEntry, ExtFileSystem};
use keramics_formats::ntfs::{NtfsFileEntry, NtfsFileSystem};

use super::apm::{ApmFileEntry, ApmFileSystem};
use super::enums::VfsType;
use super::ewf::{EwfFileEntry, EwfFileSystem};
use super::fake::{FakeFileEntry, FakeFileSystem};
use super::file_entry::VfsFileEntry;
use super::gpt::{GptFileEntry, GptFileSystem};
use super::location::VfsLocation;
use super::mbr::{MbrFileEntry, MbrFileSystem};
use super::os::OsFileEntry;
use super::path::VfsPath;
use super::qcow::{QcowFileEntry, QcowFileSystem};
use super::sparseimage::{SparseImageFileEntry, SparseImageFileSystem};
use super::types::VfsFileSystemReference;
use super::udif::{UdifFileEntry, UdifFileSystem};
use super::vhd::{VhdFileEntry, VhdFileSystem};
use super::vhdx::{VhdxFileEntry, VhdxFileSystem};

/// Virtual File System (VFS) file system.
pub enum VfsFileSystem {
    Apm(ApmFileSystem),
    Ext(ExtFileSystem),
    Ewf(EwfFileSystem),
    Fake(FakeFileSystem),
    Gpt(GptFileSystem),
    Mbr(MbrFileSystem),
    Ntfs(NtfsFileSystem),
    Os,
    Qcow(QcowFileSystem),
    SparseImage(SparseImageFileSystem),
    Udif(UdifFileSystem),
    Vhd(VhdFileSystem),
    Vhdx(VhdxFileSystem),
}

impl VfsFileSystem {
    /// Creates a new file system.
    pub fn new(location_type: &VfsType) -> Self {
        match location_type {
            VfsType::Apm => VfsFileSystem::Apm(ApmFileSystem::new()),
            VfsType::Ext => VfsFileSystem::Ext(ExtFileSystem::new()),
            VfsType::Ewf => VfsFileSystem::Ewf(EwfFileSystem::new()),
            VfsType::Fake => VfsFileSystem::Fake(FakeFileSystem::new()),
            VfsType::Gpt => VfsFileSystem::Gpt(GptFileSystem::new()),
            VfsType::Mbr => VfsFileSystem::Mbr(MbrFileSystem::new()),
            VfsType::Ntfs => VfsFileSystem::Ntfs(NtfsFileSystem::new()),
            VfsType::Os => VfsFileSystem::Os,
            VfsType::Qcow => VfsFileSystem::Qcow(QcowFileSystem::new()),
            VfsType::SparseImage => VfsFileSystem::SparseImage(SparseImageFileSystem::new()),
            VfsType::Udif => VfsFileSystem::Udif(UdifFileSystem::new()),
            VfsType::Vhd => VfsFileSystem::Vhd(VhdFileSystem::new()),
            VfsType::Vhdx => VfsFileSystem::Vhdx(VhdxFileSystem::new()),
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, vfs_path: &VfsPath) -> Result<bool, ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                match apm_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if APM file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Ext(ext_file_system) => match vfs_path {
                VfsPath::Ext(ext_path) => {
                    let result: Option<ExtFileEntry> =
                        match ext_file_system.get_file_entry_by_path(&ext_path) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve ext file entry"
                                );
                                return Err(error);
                            }
                        };
                    match result {
                        Some(_) => Ok(true),
                        None => Ok(false),
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported ext VFS path type"
                )),
            },
            VfsFileSystem::Ewf(ewf_file_system) => {
                match ewf_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if EWF file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Fake(fake_file_system) => {
                match fake_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if fake file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Gpt(gpt_file_system) => {
                match gpt_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if GPT file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                match mbr_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if MBR file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => match vfs_path {
                VfsPath::Ntfs(ntfs_path) => {
                    let result: Option<NtfsFileEntry> =
                        match ntfs_file_system.get_file_entry_by_path(&ntfs_path) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve NTFS file entry"
                                );
                                return Err(error);
                            }
                        };
                    match result {
                        Some(_) => Ok(true),
                        None => Ok(false),
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported NTFS VFS path type"
                )),
            },
            VfsFileSystem::Os => match vfs_path {
                VfsPath::Os(string_path) => {
                    let os_path: &Path = Path::new(&string_path);

                    match os_path.try_exists() {
                        Ok(result) => Ok(result),
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to determine if OS file entry exists",
                                error
                            ));
                        }
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported OS VFS path type"
                )),
            },
            VfsFileSystem::Qcow(qcow_file_system) => {
                match qcow_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if QCOW file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                match sparseimage_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if sparseimage file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Udif(udif_file_system) => {
                match udif_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if UDIF file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                match vhd_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if VHD file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                match vhdx_file_system.file_entry_exists(vfs_path) {
                    Ok(result) => Ok(result),
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to determine if VHDX file entry exists"
                        );
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Retrieves a data stream with the specified path and name.
    #[inline(always)]
    pub fn get_data_stream_by_path_and_name(
        &self,
        vfs_path: &VfsPath,
        name: Option<&str>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self.get_file_entry_by_path(vfs_path)? {
            // TODO: replace by get_data_fork_by_name
            Some(file_entry) => file_entry.get_data_stream_by_name(name),
            None => Ok(None),
        }
    }

    /// Retrieves a file entry with the specified path.
    pub fn get_file_entry_by_path(
        &self,
        vfs_path: &VfsPath,
    ) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                let result: Option<ApmFileEntry> =
                    match apm_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve APM file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(apm_file_entry) => Ok(Some(VfsFileEntry::Apm(apm_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ext(ext_file_system) => match vfs_path {
                VfsPath::Ext(ext_path) => {
                    let result: Option<ExtFileEntry> =
                        match ext_file_system.get_file_entry_by_path(&ext_path) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve ext file entry"
                                );
                                return Err(error);
                            }
                        };
                    match result {
                        Some(file_entry) => Ok(Some(VfsFileEntry::Ext(file_entry))),
                        None => Ok(None),
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported ext VFS path type"
                )),
            },
            VfsFileSystem::Ewf(ewf_file_system) => {
                let result: Option<EwfFileEntry> =
                    match ewf_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve EWF file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(ewf_file_entry) => Ok(Some(VfsFileEntry::Ewf(ewf_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Fake(fake_file_system) => {
                let result: Option<Arc<FakeFileEntry>> =
                    match fake_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve fake file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(file_entry) => Ok(Some(VfsFileEntry::Fake(file_entry.clone()))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Gpt(gpt_file_system) => {
                let result: Option<GptFileEntry> =
                    match gpt_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve GPT file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(gpt_file_entry) => Ok(Some(VfsFileEntry::Gpt(gpt_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                let result: Option<MbrFileEntry> =
                    match mbr_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve MBR file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(mbr_file_entry) => Ok(Some(VfsFileEntry::Mbr(mbr_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => match vfs_path {
                VfsPath::Ntfs(ntfs_path) => {
                    let result: Option<NtfsFileEntry> =
                        match ntfs_file_system.get_file_entry_by_path(&ntfs_path) {
                            Ok(result) => result,
                            Err(mut error) => {
                                keramics_core::error_trace_add_frame!(
                                    error,
                                    "Unable to retrieve NTFS file entry"
                                );
                                return Err(error);
                            }
                        };
                    match result {
                        Some(file_entry) => Ok(Some(VfsFileEntry::Ntfs(file_entry))),
                        None => Ok(None),
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported NTFS VFS path type"
                )),
            },
            VfsFileSystem::Os => match vfs_path {
                VfsPath::Os(string_path) => {
                    let os_path: &Path = Path::new(&string_path);

                    let result: bool = match os_path.try_exists() {
                        Ok(result) => result,
                        Err(error) => {
                            return Err(keramics_core::error_trace_new_with_error!(
                                "Unable to determine if OS file entry exists",
                                error
                            ));
                        }
                    };
                    match result {
                        false => Ok(None),
                        true => {
                            let mut os_file_entry: OsFileEntry = OsFileEntry::new();

                            match os_file_entry.open(string_path.as_os_str()) {
                                Ok(_) => {}
                                Err(error) => {
                                    return Err(keramics_core::error_trace_new_with_error!(
                                        "Unable to open OS file entry",
                                        error
                                    ));
                                }
                            }
                            Ok(Some(VfsFileEntry::Os(os_file_entry)))
                        }
                    }
                }
                _ => Err(keramics_core::error_trace_new!(
                    "Unsupported OS VFS path type"
                )),
            },
            VfsFileSystem::Qcow(qcow_file_system) => {
                let result: Option<QcowFileEntry> =
                    match qcow_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve QCOW file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(qcow_file_entry) => Ok(Some(VfsFileEntry::Qcow(qcow_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                let result: Option<SparseImageFileEntry> =
                    match sparseimage_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve sparseimage file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(sparseimage_file_entry) => {
                        Ok(Some(VfsFileEntry::SparseImage(sparseimage_file_entry)))
                    }
                    None => Ok(None),
                }
            }
            VfsFileSystem::Udif(udif_file_system) => {
                let result: Option<UdifFileEntry> =
                    match udif_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve UDIF file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(udif_file_entry) => Ok(Some(VfsFileEntry::Udif(udif_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                let result: Option<VhdFileEntry> =
                    match vhd_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve VHD file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(vhd_file_entry) => Ok(Some(VfsFileEntry::Vhd(vhd_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                let result: Option<VhdxFileEntry> =
                    match vhdx_file_system.get_file_entry_by_path(vfs_path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve VHDX file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(vhdx_file_entry) => Ok(Some(VfsFileEntry::Vhdx(vhdx_file_entry))),
                    None => Ok(None),
                }
            }
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                let apm_file_entry: ApmFileEntry = match apm_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve APM root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Apm(apm_file_entry)))
            }
            VfsFileSystem::Ext(ext_file_system) => match ext_file_system.get_root_directory() {
                Ok(result) => match result {
                    Some(ext_file_entry) => Ok(Some(VfsFileEntry::Ext(ext_file_entry))),
                    None => Ok(None),
                },
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve ext root directory"
                    );
                    return Err(error);
                }
            },
            VfsFileSystem::Ewf(ewf_file_system) => {
                let ewf_file_entry: EwfFileEntry = match ewf_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve EWF root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Ewf(ewf_file_entry)))
            }
            VfsFileSystem::Fake(_) => todo!(),
            VfsFileSystem::Gpt(gpt_file_system) => {
                let gpt_file_entry: GptFileEntry = match gpt_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve GPT root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Gpt(gpt_file_entry)))
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                let mbr_file_entry: MbrFileEntry = match mbr_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve MBR root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Mbr(mbr_file_entry)))
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                let ntfs_file_entry: NtfsFileEntry = match ntfs_file_system.get_root_directory() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve NTFS root directory"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Ntfs(ntfs_file_entry)))
            }
            VfsFileSystem::Os => todo!(),
            VfsFileSystem::Qcow(qcow_file_system) => {
                let qcow_file_entry: QcowFileEntry = match qcow_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve QCOW root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Qcow(qcow_file_entry)))
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                let sparseimage_file_entry: SparseImageFileEntry =
                    match sparseimage_file_system.get_root_file_entry() {
                        Ok(file_entry) => file_entry,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve sparseimage root file entry"
                            );
                            return Err(error);
                        }
                    };
                Ok(Some(VfsFileEntry::SparseImage(sparseimage_file_entry)))
            }
            VfsFileSystem::Udif(udif_file_system) => {
                let udif_file_entry: UdifFileEntry = match udif_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve UDIF root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Udif(udif_file_entry)))
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                let vhd_file_entry: VhdFileEntry = match vhd_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve VHD root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Vhd(vhd_file_entry)))
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                let vhdx_file_entry: VhdxFileEntry = match vhdx_file_system.get_root_file_entry() {
                    Ok(file_entry) => file_entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to retrieve VHDX root file entry"
                        );
                        return Err(error);
                    }
                };
                Ok(Some(VfsFileEntry::Vhdx(vhdx_file_entry)))
            }
        }
    }

    /// Opens the file system.
    pub(super) fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                match apm_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open APM file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Ext(ext_file_system) => {
                let file_system: &VfsFileSystemReference = match parent_file_system {
                    Some(file_system) => file_system,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing parent file system"
                        ));
                    }
                };
                let vfs_path: &VfsPath = vfs_location.get_path();

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
                            "No such file: {}",
                            vfs_location.to_string()
                        )));
                    }
                };
                match ext_file_system.read_data_stream(&data_stream) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read ext file system from data stream"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Ewf(ewf_file_system) => {
                match ewf_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open EWF file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Fake(_) | VfsFileSystem::Os => {
                if parent_file_system.is_some() {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported parent file system"
                    ));
                }
            }
            VfsFileSystem::Gpt(gpt_file_system) => {
                match gpt_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open GPT file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                match mbr_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open MBR file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                let file_system: &VfsFileSystemReference = match parent_file_system {
                    Some(file_system) => file_system,
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing parent file system"
                        ));
                    }
                };
                let vfs_path: &VfsPath = vfs_location.get_path();

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
                            "No such file: {}",
                            vfs_location.to_string()
                        )));
                    }
                };
                match ntfs_file_system.read_data_stream(&data_stream) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read NTFS file system from data stream"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                match qcow_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open QCOW file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                match sparseimage_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open sparseimage file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Udif(udif_file_system) => {
                match udif_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open UDIF file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                match vhd_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open VHD file system"
                        );
                        return Err(error);
                    }
                }
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                match vhdx_file_system.open(parent_file_system, vfs_location) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to open VHDX file system"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsFileType;
    use crate::fake::FakeFileEntry;
    use crate::location::new_os_vfs_location;

    fn get_apm_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Apm);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/apm/apm.dmg");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ext);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/ext/ext2.raw");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_ewf_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ewf);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/ewf/ext2.E01");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_fake_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Fake);
        if let VfsFileSystem::Fake(fake_file_system) = &mut vfs_file_system {
            let data: [u8; 4] = [1, 2, 3, 4];
            let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&data);
            _ = fake_file_system.add_file_entry("/fake1", fake_file_entry);

            let data: [u8; 4] = [5, 6, 7, 8];
            let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file(&data);
            _ = fake_file_system.add_file_entry("/fake2", fake_file_entry);
        }
        Ok(vfs_file_system)
    }

    fn get_gpt_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Gpt);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/gpt/gpt.raw");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Mbr);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/mbr/mbr.raw");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    // TODO: add get_ntfs_file_system

    fn get_qcow_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Qcow);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_sparseimage_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SparseImage);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/sparseimage/hfsplus.sparseimage");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_udif_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Udif);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/udif/hfsplus_zlib.dmg");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhd);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/vhd/ntfs-differential.vhd");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhdx);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_location: VfsLocation =
            new_os_vfs_location("../test_data/vhdx/ntfs-differential.vhdx");
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1/testfile1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/fake2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/gpt2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    // TODO: add test_file_entry_exists_with_ntfs

    #[test]
    fn test_file_entry_exists_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "../test_data/file.txt");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "../test_data/bogus.txt");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/qcow1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/sparseimage1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/udif1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/vhd2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/vhdx2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    // TODO: add test for get_data_stream_by_path_and_name

    #[test]
    fn test_get_file_entry_by_path_with_apm_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/ewf1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ewf, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_file() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/fake2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fake, "/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests fir get_file_entry of fake root

    #[test]
    fn test_get_file_entry_by_path_with_gpt_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    // TODO: add test_get_file_entry_by_path_with_ntfs_non_existing
    // TODO: add test_get_file_entry_by_path_with_ntfs_partition
    // TODO: add test_get_file_entry_by_path_with_ntfs_root

    #[test]
    fn test_get_file_entry_by_path_with_qcow_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/sparseimage1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::SparseImage, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/udif1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Udif, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_unsupported_location_type() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/");
        let result = vfs_file_system.get_file_entry_by_path(&vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    // TODO: add tests for get_root_file_entry
}
