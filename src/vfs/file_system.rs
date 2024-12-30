/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use std::io;
use std::path::{Path, MAIN_SEPARATOR_STR};

use crate::formats::apm::{ApmPartition, ApmVolumeSystem};
use crate::formats::ext::{ExtFileSystem, ExtPath};
use crate::formats::gpt::{GptPartition, GptVolumeSystem};
use crate::formats::mbr::{MbrPartition, MbrVolumeSystem};
use crate::formats::qcow::{QcowImage, QcowLayer};
use crate::formats::vhd::{VhdImage, VhdLayer};
use crate::formats::vhdx::{VhdxImage, VhdxLayer};

use super::enums::VfsPathType;
use super::fake::FakeFileSystem;
use super::file_entries::{OsVfsFileEntry, WrapperVfsFileEntry};
use super::types::{
    VfsDataStreamReference, VfsFileEntryReference, VfsFileSystemReference, VfsPathReference,
};

/// Virtual File System (VFS) file system.
pub enum VfsFileSystem {
    Apm(ApmVolumeSystem),
    Ext(ExtFileSystem),
    Fake(FakeFileSystem),
    Gpt(GptVolumeSystem),
    Mbr(MbrVolumeSystem),
    Os,
    Qcow(QcowImage),
    Vhd(VhdImage),
    Vhdx(VhdxImage),
}

impl VfsFileSystem {
    /// Creates a new file system.
    pub fn new(path_type: &VfsPathType) -> Self {
        match path_type {
            VfsPathType::Apm => VfsFileSystem::Apm(ApmVolumeSystem::new()),
            VfsPathType::Ext => VfsFileSystem::Ext(ExtFileSystem::new()),
            VfsPathType::Fake => VfsFileSystem::Fake(FakeFileSystem::new()),
            VfsPathType::Gpt => VfsFileSystem::Gpt(GptVolumeSystem::new()),
            VfsPathType::Mbr => VfsFileSystem::Mbr(MbrVolumeSystem::new()),
            VfsPathType::Os => VfsFileSystem::Os,
            VfsPathType::Qcow => VfsFileSystem::Qcow(QcowImage::new()),
            VfsPathType::Vhd => VfsFileSystem::Vhd(VhdImage::new()),
            VfsPathType::Vhdx => VfsFileSystem::Vhdx(VhdxImage::new()),
        }
    }

    /// Determines if the file entry with the specified path exists.
    pub fn file_entry_exists(&self, path: &VfsPathReference) -> io::Result<bool> {
        let path_type: VfsPathType = self.get_vfs_path_type();
        if path.path_type != path_type {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        match self {
            VfsFileSystem::Apm(apm_volume_system) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match apm_volume_system.get_partition_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            VfsFileSystem::Ext(ext_file_system) => {
                let ext_path: ExtPath = ExtPath::from(&path.location);

                match ext_file_system.get_file_entry_by_path(&ext_path)? {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            VfsFileSystem::Fake(fake_file_system) => {
                fake_file_system.file_entry_exists(&path.location)
            }
            VfsFileSystem::Gpt(gpt_volume_system) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match gpt_volume_system.get_partition_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            VfsFileSystem::Mbr(mbr_volume_system) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match mbr_volume_system.get_partition_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            VfsFileSystem::Os => {
                let os_path: &Path = Path::new(&path.location);

                os_path.try_exists()
            }
            VfsFileSystem::Qcow(qcow_image) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match qcow_image.get_layer_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            VfsFileSystem::Vhd(vhd_image) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match vhd_image.get_layer_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            VfsFileSystem::Vhdx(vhdx_image) => {
                if path.location == "/" {
                    return Ok(true);
                }
                match vhdx_image.get_layer_index_by_path(&path.location) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// Retrieves the directory name of the specified location.
    pub fn get_directory_name<'a>(&self, location: &'a str) -> &'a str {
        let separator: &str = match self {
            VfsFileSystem::Os => MAIN_SEPARATOR_STR,
            _ => "/",
        };
        let directory_name: &str = match location.rsplit_once(separator) {
            Some(path_components) => path_components.0,
            None => "",
        };
        if directory_name == "" {
            separator
        } else {
            directory_name
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
            VfsFileSystem::Os => VfsPathType::Os,
            VfsFileSystem::Qcow(_) => VfsPathType::Qcow,
            VfsFileSystem::Vhd(_) => VfsPathType::Vhd,
            VfsFileSystem::Vhdx(_) => VfsPathType::Vhdx,
        }
    }

    /// Opens the file system.
    pub(super) fn open(
        &mut self,
        parent_file_system: &VfsFileSystemReference,
        path: &VfsPathReference,
    ) -> io::Result<()> {
        match self {
            VfsFileSystem::Apm(apm_volume_system) => {
                apm_volume_system.open(parent_file_system, path)
            }
            VfsFileSystem::Ext(ext_file_system) => ext_file_system.open(parent_file_system, path),
            VfsFileSystem::Fake(_) | VfsFileSystem::Os => {
                if parent_file_system.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Unsupported parent file system",
                    ));
                }
                Ok(())
            }
            VfsFileSystem::Gpt(gpt_volume_system) => {
                gpt_volume_system.open(parent_file_system, path)
            }
            VfsFileSystem::Mbr(mbr_volume_system) => {
                mbr_volume_system.open(parent_file_system, path)
            }
            VfsFileSystem::Qcow(qcow_image) => qcow_image.open(parent_file_system, path),
            VfsFileSystem::Vhd(vhd_image) => vhd_image.open(parent_file_system, path),
            VfsFileSystem::Vhdx(vhdx_image) => vhdx_image.open(parent_file_system, path),
        }
    }

    /// Opens a data stream with the specified path and name.
    #[inline(always)]
    pub fn open_data_stream(
        &self,
        path: &VfsPathReference,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        match self.open_file_entry(path)? {
            Some(file_entry) => file_entry.open_data_stream(name),
            None => Ok(None),
        }
    }

    /// Opens a file entry with the specified path.
    pub fn open_file_entry(
        &self,
        path: &VfsPathReference,
    ) -> io::Result<Option<VfsFileEntryReference>> {
        let path_type: VfsPathType = self.get_vfs_path_type();
        if path.path_type != path_type {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported path type",
            ));
        }
        match self {
            VfsFileSystem::Apm(apm_volume_system) => {
                let result: Option<VfsFileEntryReference> =
                    match apm_volume_system.get_partition_by_path(&path.location) {
                        Ok(apm_partition) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<ApmPartition>(apm_partition);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Ext(ext_file_system) => {
                let ext_path: ExtPath = ExtPath::from(&path.location);

                let result: Option<VfsFileEntryReference> =
                    match ext_file_system.get_file_entry_by_path(&ext_path)? {
                        Some(file_entry) => Some(Box::new(file_entry)),
                        None => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Fake(fake_file_system) => {
                fake_file_system.open_file_entry(&path.location)
            }
            VfsFileSystem::Gpt(gpt_volume_system) => {
                let result: Option<VfsFileEntryReference> =
                    match gpt_volume_system.get_partition_by_path(&path.location) {
                        Ok(gpt_partition) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<GptPartition>(gpt_partition);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Mbr(mbr_volume_system) => {
                let result: Option<VfsFileEntryReference> =
                    match mbr_volume_system.get_partition_by_path(&path.location) {
                        Ok(mbr_partition) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<MbrPartition>(mbr_partition);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Os => {
                let os_path: &Path = Path::new(&path.location);

                let result: Option<VfsFileEntryReference> = match os_path.try_exists()? {
                    false => None,
                    true => {
                        let mut file_entry: OsVfsFileEntry = OsVfsFileEntry::new();
                        file_entry.initialize(path)?;
                        Some(Box::new(file_entry))
                    }
                };
                Ok(result)
            }
            VfsFileSystem::Qcow(qcow_image) => {
                let result: Option<VfsFileEntryReference> =
                    match qcow_image.get_layer_by_path(&path.location) {
                        Ok(qcow_layer) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<QcowLayer>(qcow_layer);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Vhd(vhd_image) => {
                let result: Option<VfsFileEntryReference> =
                    match vhd_image.get_layer_by_path(&path.location) {
                        Ok(vhd_layer) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<VhdLayer>(vhd_layer);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
            VfsFileSystem::Vhdx(vhdx_image) => {
                let result: Option<VfsFileEntryReference> =
                    match vhdx_image.get_layer_by_path(&path.location) {
                        Ok(vhdx_layer) => {
                            let mut file_entry: WrapperVfsFileEntry =
                                WrapperVfsFileEntry::new::<VhdxLayer>(vhdx_layer);
                            file_entry.initialize(path)?;

                            Some(Box::new(file_entry))
                        }
                        Err(_) => None,
                    };
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::SharedValue;
    use crate::vfs::enums::VfsFileType;
    use crate::vfs::path::VfsPath;

    fn get_apm_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Apm);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Ext);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_gpt_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Gpt);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Mbr);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Qcow);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhd);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        );
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhdx);

        let parent_file_system: VfsFileSystemReference =
            SharedValue::new(VfsFileSystem::new(&VfsPathType::Os));
        let vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        vfs_file_system.open(&parent_file_system, &vfs_path)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_os() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_get_directory_name_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let directory_name: &str = vfs_file_system.get_directory_name("/gpt1");
        assert_eq!(directory_name, "/");

        Ok(())
    }

    #[test]
    fn test_get_vfs_path_type_with_os() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let vfs_path_type: VfsPathType = vfs_file_system.get_vfs_path_type();
        assert!(vfs_path_type == VfsPathType::Os);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_apm_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_apm_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/apm2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_apm_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Apm, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Ext, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Ext, "/passwords.txt", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Ext, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_gpt_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Gpt, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_gpt_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Gpt, "/gpt2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_gpt_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Gpt, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Mbr, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Mbr, "/mbr2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Mbr, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/qcow1", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhd, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhd, "/vhd2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhd, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhdx, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntryReference> =
            vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhdx, "/vhdx2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPathReference = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        let test_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Vhdx, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntryReference =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let test_vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Fake, "/", None);

        let result = vfs_file_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
