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

use std::ffi::OsStr;
use std::io;
use std::path::Path;

use core::DataStreamReference;
use formats::ext::{ExtFileEntry, ExtFileSystem};
use formats::ntfs::{NtfsFileEntry, NtfsFileSystem};

use super::apm::{ApmFileEntry, ApmFileSystem};
use super::enums::VfsPathType;
use super::fake::FakeFileSystem;
use super::file_entry::VfsFileEntry;
use super::gpt::{GptFileEntry, GptFileSystem};
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
    pub fn new(path_type: &VfsPathType) -> Self {
        match path_type {
            VfsPathType::Apm => VfsFileSystem::Apm(ApmFileSystem::new()),
            VfsPathType::Ext => VfsFileSystem::Ext(ExtFileSystem::new()),
            VfsPathType::Fake => VfsFileSystem::Fake(FakeFileSystem::new()),
            VfsPathType::Gpt => VfsFileSystem::Gpt(GptFileSystem::new()),
            VfsPathType::Mbr => VfsFileSystem::Mbr(MbrFileSystem::new()),
            VfsPathType::Ntfs => VfsFileSystem::Ntfs(NtfsFileSystem::new()),
            VfsPathType::Os => VfsFileSystem::Os,
            VfsPathType::Qcow => VfsFileSystem::Qcow(QcowFileSystem::new()),
            VfsPathType::SparseImage => VfsFileSystem::SparseImage(SparseImageFileSystem::new()),
            VfsPathType::Udif => VfsFileSystem::Udif(UdifFileSystem::new()),
            VfsPathType::Vhd => VfsFileSystem::Vhd(VhdFileSystem::new()),
            VfsPathType::Vhdx => VfsFileSystem::Vhdx(VhdxFileSystem::new()),
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => apm_file_system.file_entry_exists(path),
            VfsFileSystem::Ext(ext_file_system) => match path {
                VfsPath::Ext { ext_path, .. } => {
                    match ext_file_system.get_file_entry_by_path(&ext_path)? {
                        Some(_) => Ok(true),
                        None => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Fake(fake_file_system) => match path {
                VfsPath::Fake { location, .. } => fake_file_system.file_entry_exists(&location),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Gpt(gpt_file_system) => gpt_file_system.file_entry_exists(path),
            VfsFileSystem::Mbr(mbr_file_system) => mbr_file_system.file_entry_exists(path),
            VfsFileSystem::Ntfs(ntfs_file_system) => match path {
                VfsPath::Ntfs { ntfs_path, .. } => {
                    match ntfs_file_system.get_file_entry_by_path(&ntfs_path)? {
                        Some(_) => Ok(true),
                        None => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Os => match path {
                VfsPath::Os { location, .. } => {
                    let os_path: &Path = Path::new(&location);

                    os_path.try_exists()
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Qcow(qcow_file_system) => qcow_file_system.file_entry_exists(path),
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                sparseimage_file_system.file_entry_exists(path)
            }
            VfsFileSystem::Udif(udif_file_system) => udif_file_system.file_entry_exists(path),
            VfsFileSystem::Vhd(vhd_file_system) => vhd_file_system.file_entry_exists(path),
            VfsFileSystem::Vhdx(vhdx_file_system) => vhdx_file_system.file_entry_exists(path),
        }
    }

    /// Retrieves a data stream with the specified path and name.
    #[inline(always)]
    pub fn get_data_stream_by_path_and_name(
        &self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<Option<DataStreamReference>> {
        match self.get_file_entry_by_path(path)? {
            // TODO: replace by get_data_fork_by_name
            Some(file_entry) => file_entry.get_data_stream_by_name(name),
            None => Ok(None),
        }
    }

    /// Retrieves a file entry with the specified path.
    pub fn get_file_entry_by_path(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntry>> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                match apm_file_system.get_file_entry_by_path(path)? {
                    Some(apm_file_entry) => Ok(Some(VfsFileEntry::Apm(apm_file_entry))),
                    None => Ok(None),
                }
            }
            VfsFileSystem::Ext(ext_file_system) => match path {
                VfsPath::Ext { ext_path, .. } => {
                    let result: Option<VfsFileEntry> =
                        match ext_file_system.get_file_entry_by_path(&ext_path)? {
                            Some(file_entry) => Some(VfsFileEntry::Ext(file_entry)),
                            None => None,
                        };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Fake(fake_file_system) => match path {
                VfsPath::Fake { location, .. } => {
                    let result: Option<VfsFileEntry> =
                        match fake_file_system.get_file_entry_by_path(&location)? {
                            Some(file_entry) => Some(VfsFileEntry::Fake(file_entry.clone())),
                            None => None,
                        };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
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
            VfsFileSystem::Ntfs(ntfs_file_system) => match path {
                VfsPath::Ntfs { ntfs_path, .. } => {
                    let result: Option<VfsFileEntry> =
                        match ntfs_file_system.get_file_entry_by_path(&ntfs_path)? {
                            Some(file_entry) => Some(VfsFileEntry::Ntfs(file_entry)),
                            None => None,
                        };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Os => match path {
                VfsPath::Os { location, .. } => {
                    let os_path: &Path = Path::new(&location);

                    let result: Option<VfsFileEntry> = match os_path.try_exists()? {
                        false => None,
                        true => {
                            let mut os_file_entry: OsFileEntry = OsFileEntry::new();
                            os_file_entry.open(OsStr::new(location.as_str()))?;
                            Some(VfsFileEntry::Os(os_file_entry))
                        }
                    };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
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
        }
    }

    /// Retrieves the root file entry.
    pub fn get_root_file_entry(&self) -> io::Result<Option<VfsFileEntry>> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => {
                let apm_file_entry: ApmFileEntry = apm_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Apm(apm_file_entry)))
            }
            VfsFileSystem::Ext(ext_file_system) => {
                let ext_file_entry: ExtFileEntry = ext_file_system.get_root_directory()?;
                Ok(Some(VfsFileEntry::Ext(ext_file_entry)))
            }
            VfsFileSystem::Fake(_) => todo!(),
            VfsFileSystem::Gpt(gpt_file_system) => {
                let gpt_file_entry: GptFileEntry = gpt_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Gpt(gpt_file_entry)))
            }
            VfsFileSystem::Mbr(mbr_file_system) => {
                let mbr_file_entry: MbrFileEntry = mbr_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Mbr(mbr_file_entry)))
            }
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                let ntfs_file_entry: NtfsFileEntry = ntfs_file_system.get_root_directory()?;
                Ok(Some(VfsFileEntry::Ntfs(ntfs_file_entry)))
            }
            VfsFileSystem::Os => todo!(),
            VfsFileSystem::Qcow(qcow_file_system) => {
                let qcow_file_entry: QcowFileEntry = qcow_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Qcow(qcow_file_entry)))
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                let sparseimage_file_entry: SparseImageFileEntry =
                    sparseimage_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::SparseImage(sparseimage_file_entry)))
            }
            VfsFileSystem::Udif(udif_file_system) => {
                let udif_file_entry: UdifFileEntry = udif_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Udif(udif_file_entry)))
            }
            VfsFileSystem::Vhd(vhd_file_system) => {
                let vhd_file_entry: VhdFileEntry = vhd_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Vhd(vhd_file_entry)))
            }
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                let vhdx_file_entry: VhdxFileEntry = vhdx_file_system.get_root_file_entry()?;
                Ok(Some(VfsFileEntry::Vhdx(vhdx_file_entry)))
            }
        }
    }

    /// Retrieves the path type.
    pub fn get_vfs_path_type(&self) -> VfsPathType {
        match self {
            VfsFileSystem::Apm(_) => VfsPathType::Apm,
            VfsFileSystem::Ext(_) => VfsPathType::Ext,
            VfsFileSystem::Fake(_) => VfsPathType::Fake,
            VfsFileSystem::Gpt(_) => VfsPathType::Gpt,
            VfsFileSystem::Mbr(_) => VfsPathType::Mbr,
            VfsFileSystem::Ntfs(_) => VfsPathType::Ntfs,
            VfsFileSystem::Os => VfsPathType::Os,
            VfsFileSystem::Qcow(_) => VfsPathType::Qcow,
            VfsFileSystem::SparseImage(_) => VfsPathType::SparseImage,
            VfsFileSystem::Udif(_) => VfsPathType::Udif,
            VfsFileSystem::Vhd(_) => VfsPathType::Vhd,
            VfsFileSystem::Vhdx(_) => VfsPathType::Vhdx,
        }
    }

    /// Opens the file system.
    pub(super) fn open(
        &mut self,
        parent_file_system: Option<&VfsFileSystemReference>,
        path: &VfsPath,
    ) -> io::Result<()> {
        match self {
            VfsFileSystem::Apm(apm_file_system) => apm_file_system.open(parent_file_system, path),
            VfsFileSystem::Ext(ext_file_system) => {
                let file_system: &VfsFileSystemReference = match parent_file_system {
                    Some(file_system) => file_system,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing parent file system",
                        ))
                    }
                };
                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => ext_file_system.read_data_stream(&data_stream),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                }
            }
            VfsFileSystem::Fake(_) | VfsFileSystem::Os => {
                if parent_file_system.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Unsupported parent file system",
                    ));
                }
                Ok(())
            }
            VfsFileSystem::Gpt(gpt_file_system) => gpt_file_system.open(parent_file_system, path),
            VfsFileSystem::Mbr(mbr_file_system) => mbr_file_system.open(parent_file_system, path),
            VfsFileSystem::Ntfs(ntfs_file_system) => {
                let file_system: &VfsFileSystemReference = match parent_file_system {
                    Some(file_system) => file_system,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Missing parent file system",
                        ))
                    }
                };
                match file_system.get_data_stream_by_path_and_name(path, None)? {
                    Some(data_stream) => ntfs_file_system.read_data_stream(&data_stream),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("No such file: {}", path.to_string()),
                        ))
                    }
                }
            }
            VfsFileSystem::Qcow(qcow_file_system) => {
                qcow_file_system.open(parent_file_system, path)
            }
            VfsFileSystem::SparseImage(sparseimage_file_system) => {
                sparseimage_file_system.open(parent_file_system, path)
            }
            VfsFileSystem::Udif(udif_file_system) => {
                udif_file_system.open(parent_file_system, path)
            }
            VfsFileSystem::Vhd(vhd_file_system) => vhd_file_system.open(parent_file_system, path),
            VfsFileSystem::Vhdx(vhdx_file_system) => {
                vhdx_file_system.open(parent_file_system, path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsFileType;
    use crate::fake::FakeFileEntry;
    use crate::path::VfsPath;

    fn get_apm_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Apm);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Ext);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_fake_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Fake);
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

    fn get_gpt_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Gpt);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Mbr);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Qcow);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_sparseimage_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::SparseImage);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/sparseimage/hfsplus.sparseimage".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_udif_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Udif);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhd);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhdx);

        let parent_file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_fake() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let vfs_path: VfsPath = VfsPath::Fake {
            location: "/fake2".to_string(),
        };
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::Fake {
            location: "./bogus".to_string(),
        };
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_os() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/file.txt".to_string(),
        };
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/bogus.txt".to_string(),
        };
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_sparseimage() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/sparseimage/hfsplus.sparseimage".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::SparseImage, "/sparseimage1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::SparseImage, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_udif() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Udif, "/udif1");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Udif, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "./bogus");
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_apm_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_ext_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_file() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::Fake {
            location: "/fake2".to_string(),
        };
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_fake_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::Fake {
            location: "/bogus".to_string(),
        };
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests fir get_file_entry of fake root

    #[test]
    fn test_get_file_entry_by_path_with_gpt_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_gpt_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_mbr_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_qcow_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/sparseimage/hfsplus.sparseimage".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::SparseImage, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/sparseimage/hfsplus.sparseimage".to_string(),
        };
        let test_vfs_path: VfsPath =
            os_vfs_path.new_child(VfsPathType::SparseImage, "/sparseimage1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_sparseimage_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_sparseimage_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/sparseimage/hfsplus.sparseimage".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::SparseImage, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Udif, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Udif, "/udif1");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_udif_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_udif_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/udif/hfsplus_zlib.dmg".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Udif, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhd_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/bogus");
        let result: Option<VfsFileEntry> =
            vfs_file_system.get_file_entry_by_path(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_vhdx_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let test_vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry = vfs_file_system
            .get_file_entry_by_path(&test_vfs_path)?
            .unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path_with_unsupported_path_type() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let test_vfs_path: VfsPath = VfsPath::Fake {
            location: "/".to_string(),
        };

        let result = vfs_file_system.get_file_entry_by_path(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }

    // TODO: add tests for get_root_file_entry

    #[test]
    fn test_get_vfs_path_type_with_os() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let vfs_path_type: VfsPathType = vfs_file_system.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Os);

        Ok(())
    }
}
