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
use std::path::Path;
use std::rc::Rc;

use crate::formats::apm::ApmVolumeSystem;
use crate::formats::ext::ExtFileSystem;
use crate::formats::gpt::GptVolumeSystem;
use crate::formats::mbr::MbrVolumeSystem;
use crate::formats::qcow::QcowImage;
use crate::formats::vhd::VhdImage;
use crate::formats::vhdx::VhdxImage;
use crate::types::SharedValue;

use super::enums::VfsPathType;
use super::fake::FakeFileSystem;
use super::file_entry::VfsFileEntry;
use super::os::OsFileEntry;
use super::path::VfsPath;
use super::types::VfsDataStreamReference;

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
    pub fn file_entry_exists(&self, path: &VfsPath) -> io::Result<bool> {
        match self {
            VfsFileSystem::Apm(apm_volume_system) => match path {
                VfsPath::Apm { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match apm_volume_system.get_partition_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
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
            VfsFileSystem::Gpt(gpt_volume_system) => match path {
                VfsPath::Gpt { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match gpt_volume_system.get_partition_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Mbr(mbr_volume_system) => match path {
                VfsPath::Mbr { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match mbr_volume_system.get_partition_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
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
            VfsFileSystem::Qcow(qcow_image) => match path {
                VfsPath::Qcow { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match qcow_image.get_layer_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Vhd(vhd_image) => match path {
                VfsPath::Vhd { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match vhd_image.get_layer_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Vhdx(vhdx_image) => match path {
                VfsPath::Vhdx { location, .. } => {
                    if location == "/" {
                        return Ok(true);
                    }
                    match vhdx_image.get_layer_index_by_path(&location) {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    }
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
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
        parent_file_system: Option<Rc<VfsFileSystem>>,
        path: &VfsPath,
    ) -> io::Result<()> {
        match self {
            VfsFileSystem::Apm(apm_volume_system) => match parent_file_system {
                Some(vfs_file_system) => apm_volume_system.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Ext(ext_file_system) => match parent_file_system {
                Some(vfs_file_system) => ext_file_system.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Fake(_) | VfsFileSystem::Os => {
                if parent_file_system.is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Unsupported parent file system",
                    ));
                }
                Ok(())
            }
            VfsFileSystem::Gpt(gpt_volume_system) => match parent_file_system {
                Some(vfs_file_system) => gpt_volume_system.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Mbr(mbr_volume_system) => match parent_file_system {
                Some(vfs_file_system) => mbr_volume_system.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Qcow(qcow_image) => match parent_file_system {
                Some(vfs_file_system) => qcow_image.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Vhd(vhd_image) => match parent_file_system {
                Some(vfs_file_system) => vhd_image.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
            VfsFileSystem::Vhdx(vhdx_image) => match parent_file_system {
                Some(vfs_file_system) => vhdx_image.open(&vfs_file_system, path),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing parent file system",
                )),
            },
        }
    }

    /// Opens a data stream with the specified path and name.
    #[inline(always)]
    pub fn open_data_stream(
        &self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        match self.open_file_entry(path)? {
            Some(file_entry) => file_entry.open_data_stream(name),
            None => Ok(None),
        }
    }

    /// Opens a file entry with the specified path.
    pub fn open_file_entry(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntry>> {
        match self {
            VfsFileSystem::Apm(apm_volume_system) => match path {
                VfsPath::Apm { location, .. } => {
                    let result: Option<VfsFileEntry> =
                        match apm_volume_system.get_partition_by_path(&location) {
                            Ok(result) => match result {
                                Some(apm_partition) => Some(VfsFileEntry::Apm(Some(
                                    SharedValue::new(Box::new(apm_partition)),
                                ))),
                                None => Some(VfsFileEntry::Apm(None)),
                            },
                            Err(_) => None,
                        };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
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
            VfsFileSystem::Gpt(gpt_volume_system) => match path {
                VfsPath::Gpt { location, .. } => {
                    let result: Option<VfsFileEntry> =
                        match gpt_volume_system.get_partition_by_path(&location) {
                            Ok(result) => match result {
                                Some(gpt_partition) => Some(VfsFileEntry::Gpt(Some(
                                    SharedValue::new(Box::new(gpt_partition)),
                                ))),
                                None => Some(VfsFileEntry::Gpt(None)),
                            },
                            Err(_) => None,
                        };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Mbr(mbr_volume_system) => match path {
                VfsPath::Mbr { location, .. } => {
                    let result: Option<VfsFileEntry> =
                        match mbr_volume_system.get_partition_by_path(&location) {
                            Ok(result) => match result {
                                Some(mbr_partition) => Some(VfsFileEntry::Mbr(Some(
                                    SharedValue::new(Box::new(mbr_partition)),
                                ))),
                                None => Some(VfsFileEntry::Mbr(None)),
                            },
                            Err(_) => None,
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
                            os_file_entry.initialize(&location)?;
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
            VfsFileSystem::Qcow(qcow_image) => match path {
                VfsPath::Qcow { location, .. } => {
                    let result: Option<VfsFileEntry> = match qcow_image.get_layer_by_path(&location)
                    {
                        Ok(result) => match result {
                            Some(qcow_layer) => Some(VfsFileEntry::Qcow(Some(SharedValue::new(
                                Box::new(qcow_layer),
                            )))),
                            None => Some(VfsFileEntry::Qcow(None)),
                        },
                        Err(_) => None,
                    };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Vhd(vhd_image) => match path {
                VfsPath::Vhd { location, .. } => {
                    let result: Option<VfsFileEntry> = match vhd_image.get_layer_by_path(&location)
                    {
                        Ok(result) => match result {
                            Some(vhd_layer) => Some(VfsFileEntry::Vhd(Some(SharedValue::new(
                                Box::new(vhd_layer),
                            )))),
                            None => Some(VfsFileEntry::Vhd(None)),
                        },
                        Err(_) => None,
                    };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
            VfsFileSystem::Vhdx(vhdx_image) => match path {
                VfsPath::Vhdx { location, .. } => {
                    let result: Option<VfsFileEntry> = match vhdx_image.get_layer_by_path(&location)
                    {
                        Ok(result) => match result {
                            Some(vhdx_layer) => Some(VfsFileEntry::Vhdx(Some(SharedValue::new(
                                Box::new(vhdx_layer),
                            )))),
                            None => Some(VfsFileEntry::Vhdx(None)),
                        },
                        Err(_) => None,
                    };
                    Ok(result)
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                )),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;

    use crate::vfs::enums::VfsFileType;
    use crate::vfs::fake::FakeFileEntry;
    use crate::vfs::path::VfsPath;

    fn get_parent_file_system() -> Rc<VfsFileSystem> {
        Rc::new(VfsFileSystem::new(&VfsPathType::Os))
    }

    fn get_apm_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Apm);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/apm/apm.dmg", None);
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Ext);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/ext/ext2.raw", None);
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

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

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/gpt/gpt.raw", None);
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Mbr);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/mbr/mbr.raw", None);
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Qcow);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhd);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        );
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhdx);

        let parent_file_system: Rc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        );
        vfs_file_system.open(Some(parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_file_entry_exists_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/apm/apm.dmg",
            None,
        ));

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Apm, "/apm2", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Apm, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/ext/ext2.raw",
            None,
        ));

        let vfs_path: VfsPath =
            VfsPath::new(VfsPathType::Ext, "/passwords.txt", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_fake() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "/fake2", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "./bogus", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/gpt/gpt.raw",
            None,
        ));

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/gpt2", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/mbr/mbr.raw",
            None,
        ));

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/mbr2", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_os() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/qcow1", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        ));
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhd, "/vhd2", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhd, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

        Ok(())
    }

    #[test]
    fn test_file_entry_exists_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;
        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        ));
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, true);

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "./bogus", Some(&os_vfs_path));
        assert_eq!(vfs_file_system.file_entry_exists(&vfs_path)?, false);

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

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/apm/apm.dmg",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Apm, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_apm_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/apm/apm.dmg",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Apm, "/apm2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_apm_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/apm/apm.dmg",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Apm, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/ext/ext2.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/ext/ext2.raw",
            None,
        ));
        let test_vfs_path: VfsPath =
            VfsPath::new(VfsPathType::Ext, "/passwords.txt", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_ext_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/ext/ext2.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Ext, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_fake_file() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "/fake2", None);
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_fake_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_fake_file_system()?;

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "/bogus", None);
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    // TODO: add tests fir open_file_entry of fake root

    #[test]
    fn test_open_file_entry_with_gpt_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/gpt/gpt.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_gpt_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/gpt/gpt.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/gpt2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_gpt_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/gpt/gpt.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Gpt, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/mbr/mbr.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_partition() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/mbr/mbr.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/mbr2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_mbr_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/mbr/mbr.raw",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Mbr, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/qcow1", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_qcow_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhd, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhd, "/vhd2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhd_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhd/ntfs-differential.vhd",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhd, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_non_existing() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/bogus", Some(&os_vfs_path));
        let result: Option<VfsFileEntry> = vfs_file_system.open_file_entry(&test_vfs_path)?;

        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_layer() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_vhdx_root() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/vhdx/ntfs-differential.vhdx",
            None,
        ));
        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Vhdx, "/", Some(&os_vfs_path));
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.open_file_entry(&test_vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        Ok(())
    }

    #[test]
    fn test_open_file_entry_with_unsupported_path_type() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Os);

        let test_vfs_path: VfsPath = VfsPath::new(VfsPathType::Fake, "/", None);

        let result = vfs_file_system.open_file_entry(&test_vfs_path);
        assert!(result.is_err());

        Ok(())
    }
}
