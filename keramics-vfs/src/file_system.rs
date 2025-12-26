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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::ext::{ExtFileEntry, ExtFileSystem};
use keramics_formats::fat::{FatFileEntry, FatFileSystem};
use keramics_formats::ntfs::{NtfsFileEntry, NtfsFileSystem};
use keramics_formats::{Path, PathComponent};

use super::apm::{ApmFileEntry, ApmFileSystem};
use super::enums::VfsType;
use super::ewf::{EwfFileEntry, EwfFileSystem};
use super::fake::FakeFileSystem;
use super::file_entry::VfsFileEntry;
use super::gpt::{GptFileEntry, GptFileSystem};
use super::location::VfsLocation;
use super::mbr::{MbrFileEntry, MbrFileSystem};
use super::os::OsFileSystem;
use super::pdi::{PdiFileEntry, PdiFileSystem};
use super::qcow::{QcowFileEntry, QcowFileSystem};
use super::sparseimage::{SparseImageFileEntry, SparseImageFileSystem};
use super::splitraw::{SplitRawFileEntry, SplitRawFileSystem};
use super::types::VfsFileSystemReference;
use super::udif::{UdifFileEntry, UdifFileSystem};
use super::vhd::{VhdFileEntry, VhdFileSystem};
use super::vhdx::{VhdxFileEntry, VhdxFileSystem};
use super::vmdk::{VmdkFileEntry, VmdkFileSystem};

/// Virtual File System (VFS) file system.
pub enum VfsFileSystem {
    Apm(ApmFileSystem),
    Ewf(EwfFileSystem),
    Ext(ExtFileSystem),
    Fake(FakeFileSystem),
    Fat(FatFileSystem),
    Gpt(GptFileSystem),
    Mbr(MbrFileSystem),
    Ntfs(NtfsFileSystem),
    Os,
    Pdi(PdiFileSystem),
    Qcow(QcowFileSystem),
    SparseImage(SparseImageFileSystem),
    SplitRaw(SplitRawFileSystem),
    Udif(UdifFileSystem),
    Vhd(VhdFileSystem),
    Vhdx(VhdxFileSystem),
    Vmdk(VmdkFileSystem),
}

impl VfsFileSystem {
    /// Creates a new file system.
    pub fn new(location_type: &VfsType) -> Self {
        match location_type {
            VfsType::Apm => VfsFileSystem::Apm(ApmFileSystem::new()),
            VfsType::Ewf => VfsFileSystem::Ewf(EwfFileSystem::new()),
            VfsType::Ext => VfsFileSystem::Ext(ExtFileSystem::new()),
            VfsType::Fake => VfsFileSystem::Fake(FakeFileSystem::new()),
            VfsType::Fat => VfsFileSystem::Fat(FatFileSystem::new()),
            VfsType::Gpt => VfsFileSystem::Gpt(GptFileSystem::new()),
            VfsType::Mbr => VfsFileSystem::Mbr(MbrFileSystem::new()),
            VfsType::Ntfs => VfsFileSystem::Ntfs(NtfsFileSystem::new()),
            VfsType::Os => VfsFileSystem::Os,
            VfsType::Pdi => VfsFileSystem::Pdi(PdiFileSystem::new()),
            VfsType::Qcow => VfsFileSystem::Qcow(QcowFileSystem::new()),
            VfsType::SparseImage => VfsFileSystem::SparseImage(SparseImageFileSystem::new()),
            VfsType::SplitRaw => VfsFileSystem::SplitRaw(SplitRawFileSystem::new()),
            VfsType::Udif => VfsFileSystem::Udif(UdifFileSystem::new()),
            VfsType::Vhd => VfsFileSystem::Vhd(VhdFileSystem::new()),
            VfsType::Vhdx => VfsFileSystem::Vhdx(VhdxFileSystem::new()),
            VfsType::Vmdk => VfsFileSystem::Vmdk(VmdkFileSystem::new()),
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &Path) -> Result<bool, ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => Ok(apm_file_system.file_entry_exists(path)),
            VfsFileSystem::Ewf(ewf_file_system) => Ok(ewf_file_system.file_entry_exists(path)),
            VfsFileSystem::Ext(ext_file_system) => {
                let result: Option<ExtFileEntry> =
                    match ext_file_system.get_file_entry_by_path(path) {
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
            VfsFileSystem::Fake(fake_file_system) => {
                match fake_file_system.file_entry_exists(path) {
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
            VfsFileSystem::Fat(fat_file_system) => {
                let result: Option<FatFileEntry> =
                    match fat_file_system.get_file_entry_by_path(path) {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to retrieve FAT file entry"
                            );
                            return Err(error);
                        }
                    };
                match result {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            VfsFileSystem::Gpt(gpt_file_system) => Ok(gpt_file_system.file_entry_exists(path)),
            VfsFileSystem::Mbr(mbr_file_system) => Ok(mbr_file_system.file_entry_exists(path)),
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                let result: Option<NtfsFileEntry> =
                    match ntfs_file_system.get_file_entry_by_path(path) {
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
            VfsFileSystem::Os => OsFileSystem::file_entry_exists(path),
            VfsFileSystem::Pdi(pdi_file_system) => Ok(pdi_file_system.file_entry_exists(path)),
            VfsFileSystem::Qcow(qcow_file_system) => Ok(qcow_file_system.file_entry_exists(path)),
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                Ok(sparseimage_file_system.file_entry_exists(path))
            }
            VfsFileSystem::SplitRaw(splitraw_file_system) => {
                Ok(splitraw_file_system.file_entry_exists(path))
            }
            VfsFileSystem::Udif(udif_file_system) => Ok(udif_file_system.file_entry_exists(path)),
            VfsFileSystem::Vhd(vhd_file_system) => Ok(vhd_file_system.file_entry_exists(path)),
            VfsFileSystem::Vhdx(vhdx_file_system) => Ok(vhdx_file_system.file_entry_exists(path)),
            VfsFileSystem::Vmdk(vmdk_file_system) => Ok(vmdk_file_system.file_entry_exists(path)),
        }
    }

    /// Retrieves a data stream with the specified location.
    #[inline(always)]
    pub(crate) fn get_data_stream_by_location(
        &self,
        location: &VfsLocation,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self.get_file_entry_by_location(location) {
            Ok(Some(mut file_entry)) => file_entry.get_data_stream(),
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                Err(error)
            }
        }
    }

    /// Retrieves a data stream with the specified path.
    #[inline(always)]
    pub(crate) fn get_data_stream_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self.get_file_entry_by_path(path) {
            Ok(Some(mut file_entry)) => file_entry.get_data_stream(),
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                Err(error)
            }
        }
    }

    /// Retrieves a data stream with the specified path and name.
    #[inline(always)]
    pub fn get_data_stream_by_path_and_name(
        &self,
        path: &Path,
        name: Option<&PathComponent>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self.get_file_entry_by_path(path) {
            // TODO: replace by get_data_fork_by_name
            Ok(Some(mut file_entry)) => file_entry.get_data_stream_by_name(name),
            Ok(None) => Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                Err(error)
            }
        }
    }

    /// Retrieves a file entry with the specified loctions.
    #[inline(always)]
    pub(crate) fn get_file_entry_by_location(
        &self,
        location: &VfsLocation,
    ) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        let path: &Path = location.get_path();

        match self.get_file_entry_by_path(path) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to retrieve file entry by path"
                );
                Err(error)
            }
        }
    }

    /// Retrieves a file entry with the specified path.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        let result: Result<Option<VfsFileEntry>, ErrorTrace> = match self {
            VfsFileSystem::Apm(apm_file_system) => {
                match apm_file_system.get_file_entry_by_path(path)? {
                    Some(apm_file_entry) => Ok(Some(VfsFileEntry::Apm(apm_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ewf(ewf_file_system) => {
                match ewf_file_system.get_file_entry_by_path(path)? {
                    Some(ewf_file_entry) => Ok(Some(VfsFileEntry::Ewf(ewf_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ext(ext_file_system) => {
                match ext_file_system.get_file_entry_by_path(path)? {
                    Some(file_entry) => Ok(Some(VfsFileEntry::Ext(file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Fake(fake_file_system) => {
                match fake_file_system.get_file_entry_by_path(path)? {
                    Some(file_entry) => Ok(Some(VfsFileEntry::Fake(file_entry.clone()))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Fat(fat_file_system) => {
                match fat_file_system.get_file_entry_by_path(path)? {
                    Some(file_entry) => Ok(Some(VfsFileEntry::Fat(file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Gpt(gpt_file_system) => {
                match gpt_file_system.get_file_entry_by_path(path)? {
                    Some(gpt_file_entry) => Ok(Some(VfsFileEntry::Gpt(gpt_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                match mbr_file_system.get_file_entry_by_path(path)? {
                    Some(mbr_file_entry) => Ok(Some(VfsFileEntry::Mbr(mbr_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                match ntfs_file_system.get_file_entry_by_path(path)? {
                    Some(file_entry) => Ok(Some(VfsFileEntry::Ntfs(file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Os => match OsFileSystem::get_file_entry_by_path(path)? {
                Some(os_file_entry) => Ok(Some(VfsFileEntry::Os(os_file_entry))),
                None => Ok(None),
            },
            VfsFileSystem::Pdi(pdi_file_system) => {
                match pdi_file_system.get_file_entry_by_path(path)? {
                    Some(pdi_file_entry) => Ok(Some(VfsFileEntry::Pdi(pdi_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                match qcow_file_system.get_file_entry_by_path(path)? {
                    Some(qcow_file_entry) => Ok(Some(VfsFileEntry::Qcow(qcow_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                match sparseimage_file_system.get_file_entry_by_path(path)? {
                    Some(sparseimage_file_entry) => {
                        Ok(Some(VfsFileEntry::SparseImage(sparseimage_file_entry)))
                    }
                    None => Ok(None),
                }
            }
            VfsFileSystem::SplitRaw(splitraw_file_system) => {
                match splitraw_file_system.get_file_entry_by_path(path)? {
                    Some(splitraw_file_entry) => {
                        Ok(Some(VfsFileEntry::SplitRaw(splitraw_file_entry)))
                    }
                    None => Ok(None),
                }
            }
            VfsFileSystem::Udif(udif_file_system) => {
                match udif_file_system.get_file_entry_by_path(path)? {
                    Some(udif_file_entry) => Ok(Some(VfsFileEntry::Udif(udif_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                match vhd_file_system.get_file_entry_by_path(path)? {
                    Some(vhd_file_entry) => Ok(Some(VfsFileEntry::Vhd(vhd_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                match vhdx_file_system.get_file_entry_by_path(path)? {
                    Some(vhdx_file_entry) => Ok(Some(VfsFileEntry::Vhdx(vhdx_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Vmdk(vmdk_file_system) => {
                match vmdk_file_system.get_file_entry_by_path(path)? {
                    Some(vmdk_file_entry) => Ok(Some(VfsFileEntry::Vmdk(vmdk_file_entry))),
                    None => Ok(None),
                }
            }
        };
        match result {
            Ok(result) => Ok(result),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                Err(error)
            }
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                let apm_file_entry: ApmFileEntry = apm_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Apm(apm_file_entry)))
            }
            VfsFileSystem::Ewf(ewf_file_system) => {
                let ewf_file_entry: EwfFileEntry = ewf_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Ewf(ewf_file_entry)))
            }
            VfsFileSystem::Ext(ext_file_system) => match ext_file_system.get_root_directory() {
                Ok(Some(ext_file_entry)) => Ok(Some(VfsFileEntry::Ext(ext_file_entry))),
                Ok(None) => Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve ext root directory"
                    );
                    Err(error)
                }
            },
            VfsFileSystem::Fake(fake_file_system) => match fake_file_system.get_root_file_entry() {
                Ok(fake_file_entry) => Ok(Some(VfsFileEntry::Fake(fake_file_entry))),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve fake root directory"
                    );
                    Err(error)
                }
            },
            VfsFileSystem::Fat(fat_file_system) => match fat_file_system.get_root_directory() {
                Ok(fat_file_entry) => Ok(Some(VfsFileEntry::Fat(fat_file_entry))),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve FAT root directory"
                    );
                    Err(error)
                }
            },
            VfsFileSystem::Gpt(gpt_file_system) => {
                let gpt_file_entry: GptFileEntry = gpt_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Gpt(gpt_file_entry)))
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                let mbr_file_entry: MbrFileEntry = mbr_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Mbr(mbr_file_entry)))
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => match ntfs_file_system.get_root_directory() {
                Ok(ntfs_file_entry) => Ok(Some(VfsFileEntry::Ntfs(ntfs_file_entry))),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve NTFS root directory"
                    );
                    Err(error)
                }
            },
            VfsFileSystem::Os => match OsFileSystem::get_root_file_entry() {
                Ok(os_file_entry) => Ok(Some(VfsFileEntry::Os(os_file_entry))),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to retrieve OS root directory"
                    );
                    Err(error)
                }
            },
            VfsFileSystem::Pdi(pdi_file_system) => {
                let pdi_file_entry: PdiFileEntry = pdi_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Pdi(pdi_file_entry)))
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                let qcow_file_entry: QcowFileEntry = qcow_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Qcow(qcow_file_entry)))
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                let sparseimage_file_entry: SparseImageFileEntry =
                    sparseimage_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::SparseImage(sparseimage_file_entry)))
            }
            VfsFileSystem::SplitRaw(splitraw_file_system) => {
                let splitraw_file_entry: SplitRawFileEntry =
                    splitraw_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::SplitRaw(splitraw_file_entry)))
            }
            VfsFileSystem::Udif(udif_file_system) => {
                let udif_file_entry: UdifFileEntry = udif_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Udif(udif_file_entry)))
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                let vhd_file_entry: VhdFileEntry = vhd_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Vhd(vhd_file_entry)))
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                let vhdx_file_entry: VhdxFileEntry = vhdx_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Vhdx(vhdx_file_entry)))
            }
            VfsFileSystem::Vmdk(vmdk_file_system) => {
                let vmdk_file_entry: VmdkFileEntry = vmdk_file_system.get_root_file_entry();

                Ok(Some(VfsFileEntry::Vmdk(vmdk_file_entry)))
            }
        }
    }

    /// Opens the file system.
    pub(super) fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let result: Result<(), ErrorTrace> = match self {
            VfsFileSystem::Apm(apm_file_system) => {
                apm_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Ewf(ewf_file_system) => {
                ewf_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Ext(ext_file_system) => {
                Self::open_ext_file_system(ext_file_system, parent_file_system, vfs_location)
            }
            VfsFileSystem::Fake(_) | VfsFileSystem::Os => {
                if parent_file_system.is_some() {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported parent file system"
                    ));
                }
                Ok(())
            }
            VfsFileSystem::Fat(fat_file_system) => {
                Self::open_fat_file_system(fat_file_system, parent_file_system, vfs_location)
            }
            VfsFileSystem::Gpt(gpt_file_system) => {
                gpt_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                mbr_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                Self::open_ntfs_file_system(ntfs_file_system, parent_file_system, vfs_location)
            }
            VfsFileSystem::Pdi(pdi_file_system) => {
                pdi_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                qcow_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                sparseimage_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::SplitRaw(splitraw_file_system) => {
                splitraw_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Udif(udif_file_system) => {
                udif_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                vhd_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                vhdx_file_system.open(parent_file_system, vfs_location)
            }
            VfsFileSystem::Vmdk(vmdk_file_system) => {
                vmdk_file_system.open(parent_file_system, vfs_location)
            }
        };
        match result {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file system");
                Err(error)
            }
        }
    }

    /// Opens the ext file system.
    fn open_ext_file_system(
        ext_file_system: &mut ExtFileSystem,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let path: &Path = vfs_location.get_path();

        let data_stream: DataStreamReference =
            match file_system.get_data_stream_by_path_and_name(path, None) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing data stream: {}",
                        path,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        match ext_file_system.read_data_stream(&data_stream) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read ext file system from data stream"
                );
                Err(error)
            }
        }
    }

    /// Opens the FAT file system.
    fn open_fat_file_system(
        fat_file_system: &mut FatFileSystem,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let path: &Path = vfs_location.get_path();

        let data_stream: DataStreamReference =
            match file_system.get_data_stream_by_path_and_name(path, None) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing data stream: {}",
                        path,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        match fat_file_system.read_data_stream(&data_stream) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read FAT file system from data stream"
                );
                Err(error)
            }
        }
    }

    /// Opens the NTFS file system.
    fn open_ntfs_file_system(
        ntfs_file_system: &mut NtfsFileSystem,
        parent_file_system: Option<&VfsFileSystemReference>,
        vfs_location: &VfsLocation,
    ) -> Result<(), ErrorTrace> {
        let file_system: &VfsFileSystemReference = match parent_file_system {
            Some(file_system) => file_system,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing parent file system"
                ));
            }
        };
        let path: &Path = vfs_location.get_path();

        let data_stream: DataStreamReference =
            match file_system.get_data_stream_by_path_and_name(path, None) {
                Ok(Some(data_stream)) => data_stream,
                Ok(None) => {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Missing data stream: {}",
                        path,
                    )));
                }
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to retrieve data stream");
                    return Err(error);
                }
            };
        match ntfs_file_system.read_data_stream(&data_stream) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read NTFS file system from data stream"
                );
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsFileType;
    use crate::fake::FakeFileEntry;
    use crate::location::new_os_vfs_location;

    use crate::tests::get_test_data_path;

    // Tests with APM.

    fn get_apm_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Apm);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("apm/apm.dmg");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_apm() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from("/apm2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from("/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from("/apm2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with EWF.

    fn get_ewf_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ewf);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("ewf/ext2.E01");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_ewf() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from("/ewf1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from("/ewf1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ewf_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ewf_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with ext.

    fn get_ext_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ext);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("ext/ext2.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_ext() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_file() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with fake.

    fn get_fake_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Fake);

        if let VfsFileSystem::Fake(fake_file_system) = &mut vfs_file_system {
            let path: Path = Path::from("/");
            let fake_file_entry: FakeFileEntry = FakeFileEntry::new_directory("fake");
            fake_file_system.add_file_entry(path, fake_file_entry)?;

            let path: Path = Path::from("/fake");
            let test_data: [u8; 4] = [0x74, 0x65, 0x73, 0x74];
            let fake_file_entry: FakeFileEntry = FakeFileEntry::new_file("file.txt", &test_data);
            fake_file_system.add_file_entry(path, fake_file_entry)?;
        }
        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_fake() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let path: Path = Path::from("/fake/file.txt");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/fake/bogus.txt");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_file() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let path: Path = Path::from("/fake/file.txt");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let path: Path = Path::from("/fake/bogus.txt");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with FAT.

    fn get_fat_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Fat);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("fat/fat12.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_fat() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fat_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fat_file() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fat_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_fat_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with GPT.

    fn get_gpt_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Gpt);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("gpt/gpt.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_gpt() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from("/gpt2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from("/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from("/gpt2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with MBR.

    fn get_mbr_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Mbr);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("mbr/mbr.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_mbr() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from("/mbr2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from("/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_partition() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from("/mbr2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with NTFS.

    fn get_ntfs_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Ntfs);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("ntfs/ntfs.raw");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_ntfs() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ntfs_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ntfs_file() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let path: Path = Path::from("/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ntfs_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_ntfs_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with OS.

    #[test]
    fn test_file_entry_exists_with_os() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Os);

        let path_string: String = get_test_data_path("directory/file.txt");
        let path: Path = Path::from(path_string.as_str());
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path_string: String = get_test_data_path("directory/bogus.txt");
        let path: Path = Path::from(path_string.as_str());
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    // Tests with PDI.

    fn get_pdi_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Pdi);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("pdi/hfsplus.hdd/DiskDescriptor.xml");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_pdi() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_pdi_file_system()?;

        let path: Path = Path::from("/pdi1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_pdi_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_pdi_file_system()?;

        let path: Path = Path::from("/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_pdi_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_pdi_file_system()?;

        let path: Path = Path::from("/pdi1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_pdi_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_pdi_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with QCOW.

    fn get_qcow_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Qcow);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_qcow() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from("/qcow1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from("/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from("/qcow1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with sparse image.

    fn get_sparseimage_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SparseImage);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("sparseimage/hfsplus.sparseimage");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_sparseimage() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/sparseimage1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/sparseimage1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with split raw.

    fn get_splitraw_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::SplitRaw);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("splitraw/ext2.raw.000");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_splitraw() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_splitraw_file_system()?;

        let path: Path = Path::from("/raw1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_splitraw_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_splitraw_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_splitraw_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_splitraw_file_system()?;

        let path: Path = Path::from("/raw1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_splitraw_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_splitraw_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with UDIF.

    fn get_udif_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Udif);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("udif/hfsplus_zlib.dmg");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_udif() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from("/udif1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from("/bogus1");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from("/udif1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with VHD.

    fn get_vhd_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhd);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vhd/ntfs-differential.vhd");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_vhd() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from("/vhd2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from("/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from("/vhd2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with VHDX.

    fn get_vhdx_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vhdx);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vhdx/ntfs-differential.vhdx");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_vhdx() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from("/vhdx2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus2");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from("/bogus2");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from("/vhdx2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Tests with VMDK.

    fn get_vmdk_file_system() -> Result<VfsFileSystem, ErrorTrace> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsType::Vmdk);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        vfs_file_system.open(Some(&parent_file_system), &vfs_location)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_vmdk() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vmdk_file_system()?;

        let path: Path = Path::from("/vmdk1");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, true);

        let path: Path = Path::from("/bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vmdk_non_existing() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vmdk_file_system()?;

        let path: Path = Path::from("/bogus");
        let result: Option<VfsFileEntry> = vfs_file_system.get_file_entry_by_path(&path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vmdk_layer() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vmdk_file_system()?;

        let path: Path = Path::from("/vmdk1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vmdk_root() -> Result<(), ErrorTrace> {
        let vfs_file_system: VfsFileSystem = get_vmdk_file_system()?;

        let path: Path = Path::from("/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system.get_file_entry_by_path(&path)?.unwrap();

        let vfs_file_type: VfsFileType = vfs_file_entry.get_file_type();
        assert_eq!(vfs_file_type, VfsFileType::Directory);

        Ok(())
    }

    // Other tests.

    // TODO: add test for get_data_stream_by_path_and_name

    // TODO: add tests for get_root_file_entry
}
