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
use std::rc::Rc;

use crate::datetime::DateTime;
use crate::formats::apm::ApmPartition;
use crate::formats::ext::constants::*;
use crate::formats::ext::ExtFileEntry;
use crate::formats::gpt::GptPartition;
use crate::formats::mbr::MbrPartition;
use crate::formats::qcow::QcowLayer;
use crate::formats::vhd::VhdLayer;
use crate::formats::vhdx::VhdxLayer;

use super::enums::VfsFileType;
use super::fake::FakeFileEntry;
use super::file_entries::OsVfsFileEntry;
use super::types::VfsDataStreamReference;

/// Virtual File System (VFS) file entry.
pub enum VfsFileEntry {
    Apm(Option<VfsDataStreamReference>),
    Ext(ExtFileEntry),
    Fake(Rc<FakeFileEntry>),
    Gpt(Option<VfsDataStreamReference>),
    Mbr(Option<VfsDataStreamReference>),
    Os(OsVfsFileEntry),
    Qcow(Option<VfsDataStreamReference>),
    Vhd(Option<VfsDataStreamReference>),
    Vhdx(Option<VfsDataStreamReference>),
}

impl VfsFileEntry {
    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_access_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_access_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_access_time(),
        }
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_change_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_change_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_change_time(),
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_creation_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_creation_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_creation_time(),
        }
    }

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_modification_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_modification_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_modification_time(),
        }
    }

    /// Retrieves the file type.
    pub fn get_vfs_file_type(&self) -> VfsFileType {
        match self {
            VfsFileEntry::Apm(apm_partition) => match apm_partition {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_mode: u16 = ext_file_entry.get_file_mode();
                match file_mode & 0xf000 {
                    EXT_FILE_MODE_TYPE_FIFO => VfsFileType::NamedPipe,
                    EXT_FILE_MODE_TYPE_CHARACTER_DEVICE => VfsFileType::CharacterDevice,
                    EXT_FILE_MODE_TYPE_DIRECTORY => VfsFileType::Directory,
                    EXT_FILE_MODE_TYPE_BLOCK_DEVICE => VfsFileType::BlockDevice,
                    EXT_FILE_MODE_TYPE_REGULAR_FILE => VfsFileType::File,
                    EXT_FILE_MODE_TYPE_SYMBOLIC_LINK => VfsFileType::SymbolicLink,
                    EXT_FILE_MODE_TYPE_SOCKET => VfsFileType::Socket,
                    _ => VfsFileType::Unknown,
                }
            }
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_vfs_file_type(),
            VfsFileEntry::Gpt(gpt_partition) => match gpt_partition {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Mbr(mbr_partition) => match mbr_partition {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_vfs_file_type(),
            VfsFileEntry::Qcow(qcow_layer) => match qcow_layer {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Vhd(vhd_layer) => match vhd_layer {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Vhdx(vhdx_layer) => match vhdx_layer {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
        }
    }

    /// Opens a data stream with the specified name.
    pub fn open_data_stream(
        &self,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        match self {
            VfsFileEntry::Apm(apm_partition) => match apm_partition {
                Some(partition) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(partition.clone())),
                },
                None => Ok(None),
            },
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.open_data_stream(name),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.open_data_stream(name),
            VfsFileEntry::Gpt(gpt_partition) => match gpt_partition {
                Some(partition) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(partition.clone())),
                },
                None => Ok(None),
            },
            VfsFileEntry::Mbr(mbr_partition) => match mbr_partition {
                Some(partition) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(partition.clone())),
                },
                None => Ok(None),
            },
            VfsFileEntry::Os(os_file_entry) => os_file_entry.open_data_stream(name),
            VfsFileEntry::Qcow(qcow_layer) => match qcow_layer {
                Some(layer) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(layer.clone())),
                },
                None => Ok(None),
            },
            VfsFileEntry::Vhd(vhd_layer) => match vhd_layer {
                Some(layer) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(layer.clone())),
                },
                None => Ok(None),
            },
            VfsFileEntry::Vhdx(vhdx_layer) => match vhdx_layer {
                Some(layer) => match name {
                    Some(_) => Ok(None),
                    None => Ok(Some(layer.clone())),
                },
                None => Ok(None),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::datetime::PosixTime32;
    use crate::types::SharedValue;
    use crate::vfs::enums::{VfsFileType, VfsPathType};
    use crate::vfs::file_system::VfsFileSystem;
    use crate::vfs::path::VfsPath;
    use crate::vfs::types::{VfsFileSystemReference, VfsPathReference};

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
    fn test_get_access_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_access_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1626962852
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_change_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1626962852
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Ext, "/passwords.txt", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_modification_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1626962852
            }))
        );

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_vfs_file_type_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_vfs_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Apm, "/apm2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Gpt, "/gpt2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Mbr, "/mbr2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/qcow1", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhd, "/vhd2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_data_stream_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_none());

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Vhdx, "/vhdx2", None);
        let vfs_file_entry: VfsFileEntry = vfs_file_system.open_file_entry(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.open_data_stream(None)?;
        assert!(result.is_some());

        let result: Option<VfsDataStreamReference> =
            vfs_file_entry.open_data_stream(Some("bogus"))?;
        assert!(result.is_none());

        Ok(())
    }
}
