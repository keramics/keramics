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

use std::io;
use std::rc::Rc;

use crate::datetime::DateTime;
use crate::formats::ext::constants::*;
use crate::formats::ext::ExtFileEntry;
use crate::formats::ntfs::{NtfsDataFork, NtfsFileEntry};
use crate::types::Ucs2String;

use super::data_fork::VfsDataFork;
use super::enums::VfsFileType;
use super::fake::FakeFileEntry;
use super::iterators::VfsFileEntriesIterator;
use super::os::OsFileEntry;
use super::string::VfsString;
use super::types::VfsDataStreamReference;

/// Virtual File System (VFS) file entry.
pub enum VfsFileEntry {
    Apm(Option<VfsDataStreamReference>),
    Ext(ExtFileEntry),
    Fake(Rc<FakeFileEntry>),
    Gpt(Option<VfsDataStreamReference>),
    Mbr(Option<VfsDataStreamReference>),
    Ntfs(NtfsFileEntry),
    Os(OsFileEntry),
    Qcow(Option<VfsDataStreamReference>),
    Vhd(Option<VfsDataStreamReference>),
    Vhdx(Option<VfsDataStreamReference>),
}

impl VfsFileEntry {
    /// Retrieves the access time.
    pub fn get_access_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_access_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_access_time(),
            VfsFileEntry::Gpt(_) => None,
            VfsFileEntry::Mbr(_) => None,
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_access_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_access_time(),
            VfsFileEntry::Qcow(_) => None,
            VfsFileEntry::Vhd(_) => None,
            VfsFileEntry::Vhdx(_) => None,
        }
    }

    /// Retrieves the change time.
    pub fn get_change_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_change_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_change_time(),
            VfsFileEntry::Gpt(_) => None,
            VfsFileEntry::Mbr(_) => None,
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_change_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_change_time(),
            VfsFileEntry::Qcow(_) => None,
            VfsFileEntry::Vhd(_) => None,
            VfsFileEntry::Vhdx(_) => None,
        }
    }

    /// Retrieves the creation time.
    pub fn get_creation_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_creation_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_creation_time(),
            VfsFileEntry::Gpt(_) => None,
            VfsFileEntry::Mbr(_) => None,
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_creation_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_creation_time(),
            VfsFileEntry::Qcow(_) => None,
            VfsFileEntry::Vhd(_) => None,
            VfsFileEntry::Vhdx(_) => None,
        }
    }

    /// Retrieves the file type.
    pub fn get_file_type(&self) -> VfsFileType {
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
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_file_type(),
            VfsFileEntry::Gpt(gpt_partition) => match gpt_partition {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Mbr(mbr_partition) => match mbr_partition {
                Some(_) => VfsFileType::File,
                None => VfsFileType::Directory,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let file_attribute_flags: u32 = ntfs_file_entry.get_file_attribute_flags();
                // FILE_ATTRIBUTE_DEVICE is not used by NTFS.
                if file_attribute_flags & 0x00000400 != 0 {
                    VfsFileType::SymbolicLink
                }
                // FILE_ATTRIBUTE_DIRECTORY is not used by NTFS.
                else if ntfs_file_entry.has_directory_entries() {
                    VfsFileType::Directory
                } else {
                    VfsFileType::File
                }
            }
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_file_type(),
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

    /// Retrieves the modification time.
    pub fn get_modification_time(&self) -> Option<&DateTime> {
        match self {
            VfsFileEntry::Apm(_) => None,
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_modification_time(),
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_modification_time(),
            VfsFileEntry::Gpt(_) => None,
            VfsFileEntry::Mbr(_) => None,
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_modification_time(),
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_modification_time(),
            VfsFileEntry::Qcow(_) => None,
            VfsFileEntry::Vhd(_) => None,
            VfsFileEntry::Vhdx(_) => None,
        }
    }

    /// Retrieves the name.
    pub fn get_name(&mut self) -> Option<VfsString> {
        match self {
            VfsFileEntry::Apm(_) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => match ext_file_entry.get_name() {
                Some(name) => Some(VfsString::Byte(name.clone())),
                None => None,
            },
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Gpt(_) => todo!(),
            VfsFileEntry::Mbr(_) => todo!(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => match ntfs_file_entry.get_name() {
                Some(name) => Some(VfsString::Ucs2(name.clone())),
                None => None,
            },
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(_) => todo!(),
            VfsFileEntry::Vhd(_) => todo!(),
            VfsFileEntry::Vhdx(_) => todo!(),
        }
    }

    /// Retrieves the number of data forks.
    pub fn get_number_of_data_forks(&self) -> io::Result<usize> {
        let result: usize = match self {
            VfsFileEntry::Apm(apm_partition) => match apm_partition {
                Some(_) => 1,
                None => 0,
            },
            VfsFileEntry::Ext(ext_file_entry) => {
                let file_mode: u16 = ext_file_entry.get_file_mode();
                if file_mode & 0xf000 != EXT_FILE_MODE_TYPE_REGULAR_FILE {
                    0
                } else {
                    1
                }
            }
            VfsFileEntry::Fake(fake_file_entry) => match fake_file_entry.get_file_type() {
                VfsFileType::File => 1,
                _ => 0,
            },
            VfsFileEntry::Gpt(gpt_partition) => match gpt_partition {
                Some(_) => 1,
                None => 0,
            },
            VfsFileEntry::Mbr(mbr_partition) => match mbr_partition {
                Some(_) => 1,
                None => 0,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_number_of_data_forks()?,
            VfsFileEntry::Os(os_file_entry) => match os_file_entry.get_file_type() {
                VfsFileType::File => 1,
                _ => 0,
            },
            VfsFileEntry::Qcow(qcow_layer) => match qcow_layer {
                Some(_) => 1,
                None => 0,
            },
            VfsFileEntry::Vhd(vhd_layer) => match vhd_layer {
                Some(_) => 1,
                None => 0,
            },
            VfsFileEntry::Vhdx(vhdx_layer) => match vhdx_layer {
                Some(_) => 1,
                None => 0,
            },
        };
        Ok(result)
    }

    /// Retrieves a specific data fork.
    pub fn get_data_fork_by_index(&self, data_fork_index: usize) -> io::Result<VfsDataFork> {
        let data_fork: VfsDataFork = match self {
            VfsFileEntry::Apm(_) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => {
                if data_fork_index != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid data fork index: {}", data_fork_index),
                    ));
                }
                match ext_file_entry.get_data_stream()? {
                    Some(data_stream) => VfsDataFork::Ext(data_stream),
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("Missing data stream"),
                        ))
                    }
                }
            }
            VfsFileEntry::Fake(_) => todo!(),
            VfsFileEntry::Gpt(_) => todo!(),
            VfsFileEntry::Mbr(_) => todo!(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let ntfs_data_fork: NtfsDataFork =
                    ntfs_file_entry.get_data_fork_by_index(data_fork_index)?;
                VfsDataFork::Ntfs(ntfs_data_fork)
            }
            VfsFileEntry::Os(_) => todo!(),
            VfsFileEntry::Qcow(_) => todo!(),
            VfsFileEntry::Vhd(_) => todo!(),
            VfsFileEntry::Vhdx(_) => todo!(),
        };
        Ok(data_fork)
    }

    /// Retrieves the default data stream.
    pub fn get_data_stream(&self) -> io::Result<Option<VfsDataStreamReference>> {
        let result: Option<VfsDataStreamReference> = match self {
            VfsFileEntry::Apm(apm_partition) => match apm_partition {
                Some(partition) => Some(partition.clone()),
                None => None,
            },
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_data_stream()?,
            VfsFileEntry::Fake(fake_file_entry) => fake_file_entry.get_data_stream()?,
            VfsFileEntry::Gpt(gpt_partition) => match gpt_partition {
                Some(partition) => Some(partition.clone()),
                None => None,
            },
            VfsFileEntry::Mbr(mbr_partition) => match mbr_partition {
                Some(partition) => Some(partition.clone()),
                None => None,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => ntfs_file_entry.get_data_stream()?,
            VfsFileEntry::Os(os_file_entry) => os_file_entry.get_data_stream()?,
            VfsFileEntry::Qcow(qcow_layer) => match qcow_layer {
                Some(layer) => Some(layer.clone()),
                None => None,
            },
            VfsFileEntry::Vhd(vhd_layer) => match vhd_layer {
                Some(layer) => Some(layer.clone()),
                None => None,
            },
            VfsFileEntry::Vhdx(vhdx_layer) => match vhdx_layer {
                Some(layer) => Some(layer.clone()),
                None => None,
            },
        };
        Ok(result)
    }

    /// Retrieves a data stream with the specified name.
    pub fn get_data_stream_by_name(
        &self,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        let result: Option<VfsDataStreamReference> = match self {
            VfsFileEntry::Apm(_)
            | VfsFileEntry::Ext(_)
            | VfsFileEntry::Fake(_)
            | VfsFileEntry::Gpt(_)
            | VfsFileEntry::Mbr(_)
            | VfsFileEntry::Os(_)
            | VfsFileEntry::Qcow(_)
            | VfsFileEntry::Vhd(_)
            | VfsFileEntry::Vhdx(_) => match name {
                Some(_) => None,
                None => self.get_data_stream()?,
            },
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                let ntfs_name: Option<Ucs2String> = match name {
                    Some(string) => Some(Ucs2String::from_string(string)),
                    None => None,
                };
                ntfs_file_entry.get_data_stream_by_name(&ntfs_name)?
            }
        };
        Ok(result)
    }

    /// Retrieves the number of sub file entries.
    pub fn get_number_of_sub_file_entries(&mut self) -> io::Result<usize> {
        let number_of_sub_file_entries: usize = match self {
            VfsFileEntry::Apm(_) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => ext_file_entry.get_number_of_sub_file_entries()?,
            VfsFileEntry::Fake(fake_file_entry) => todo!(),
            VfsFileEntry::Gpt(_) => todo!(),
            VfsFileEntry::Mbr(_) => todo!(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => {
                ntfs_file_entry.get_number_of_sub_file_entries()?
            }
            VfsFileEntry::Os(os_file_entry) => todo!(),
            VfsFileEntry::Qcow(_) => todo!(),
            VfsFileEntry::Vhd(_) => todo!(),
            VfsFileEntry::Vhdx(_) => todo!(),
        };
        Ok(number_of_sub_file_entries)
    }

    /// Retrieves a specific sub file entry.
    pub fn get_sub_file_entry_by_index(
        &mut self,
        sub_file_entry_index: usize,
    ) -> io::Result<VfsFileEntry> {
        let sub_file_entry: VfsFileEntry = match self {
            VfsFileEntry::Apm(_) => todo!(),
            VfsFileEntry::Ext(ext_file_entry) => {
                VfsFileEntry::Ext(ext_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?)
            }
            VfsFileEntry::Fake(fake_file_entry) => todo!(),
            VfsFileEntry::Gpt(_) => todo!(),
            VfsFileEntry::Mbr(_) => todo!(),
            VfsFileEntry::Ntfs(ntfs_file_entry) => VfsFileEntry::Ntfs(
                ntfs_file_entry.get_sub_file_entry_by_index(sub_file_entry_index)?,
            ),
            VfsFileEntry::Os(os_file_entry) => todo!(),
            VfsFileEntry::Qcow(_) => todo!(),
            VfsFileEntry::Vhd(_) => todo!(),
            VfsFileEntry::Vhdx(_) => todo!(),
        };
        Ok(sub_file_entry)
    }

    /// Retrieves a sub file entries iterator.
    pub fn sub_file_entries(&mut self) -> io::Result<VfsFileEntriesIterator> {
        let number_of_sub_file_entries: usize = self.get_number_of_sub_file_entries()?;
        Ok(VfsFileEntriesIterator::new(
            self,
            number_of_sub_file_entries,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use crate::datetime::PosixTime32;
    use crate::vfs::enums::{VfsFileType, VfsPathType};
    use crate::vfs::file_system::VfsFileSystem;
    use crate::vfs::path::VfsPath;

    fn get_parent_file_system() -> Arc<VfsFileSystem> {
        Arc::new(VfsFileSystem::new(&VfsPathType::Os))
    }

    fn get_apm_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Apm);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_ext_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Ext);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_gpt_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Gpt);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_mbr_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Mbr);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_qcow_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Qcow);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhd_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhd);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    fn get_vhdx_file_system() -> io::Result<VfsFileSystem> {
        let mut vfs_file_system: VfsFileSystem = VfsFileSystem::new(&VfsPathType::Vhdx);

        let parent_file_system: Arc<VfsFileSystem> = get_parent_file_system();
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        vfs_file_system.open(Some(&parent_file_system), &vfs_path)?;

        Ok(vfs_file_system)
    }

    #[test]
    fn test_get_access_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_access_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977482
            }))
        );

        Ok(())
    }

    // TODO: add test_get_access_time_with_fake

    #[test]
    fn test_get_access_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    // TODO: add test_get_access_time_with_os

    #[test]
    fn test_get_access_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_access_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_access_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_change_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );

        Ok(())
    }

    // TODO: add test_get_change_time_with_fake

    #[test]
    fn test_get_change_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    // TODO: add test_get_change_time_with_os

    #[test]
    fn test_get_change_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_change_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_change_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    // TODO: add test_get_creation_time_with_fake

    #[test]
    fn test_get_creation_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    // TODO: add test_get_creation_time_with_os

    #[test]
    fn test_get_creation_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_creation_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_creation_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add test_get_file_type_with_ext
    // TODO: add test_get_file_type_with_fake

    #[test]
    fn test_get_file_type_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    // TODO: add test_get_file_type_with_os

    #[test]
    fn test_get_file_type_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_file_type_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::Directory);

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert!(vfs_file_entry.get_file_type() == VfsFileType::File);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_ext() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_ext_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ext/ext2.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Ext, "/testdir1/testfile1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(
            vfs_file_entry.get_modification_time(),
            Some(&DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            }))
        );

        Ok(())
    }

    // TODO: add test_get_modification_time_with_fake

    #[test]
    fn test_get_modification_time_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add test_get_modification_time_with_os

    #[test]
    fn test_get_modification_time_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    #[test]
    fn test_get_modification_time_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        assert_eq!(vfs_file_entry.get_modification_time(), None);

        Ok(())
    }

    // TODO: add tests for get_name
    // TODO: add tests for get_number_of_data_forks

    #[test]
    fn test_get_data_stream_with_apm() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_apm_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/apm/apm.dmg".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Apm, "/apm2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_data_stream_with_ext
    // TODO: add test_get_data_stream_with_fake

    #[test]
    fn test_get_data_stream_with_gpt() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_gpt_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/gpt/gpt.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Gpt, "/gpt2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_mbr() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_mbr_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/mbr/mbr.raw".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Mbr, "/mbr2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add test_get_data_stream_with_os

    #[test]
    fn test_get_data_stream_with_qcow() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_qcow_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/qcow1");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhd() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhd_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhd/ntfs-differential.vhd".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhd, "/vhd2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_get_data_stream_with_vhdx() -> io::Result<()> {
        let vfs_file_system: VfsFileSystem = get_vhdx_file_system()?;

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/vhdx/ntfs-differential.vhdx".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_none());

        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Vhdx, "/vhdx2");
        let vfs_file_entry: VfsFileEntry =
            vfs_file_system.get_file_entry_by_path(&vfs_path)?.unwrap();

        let result: Option<VfsDataStreamReference> = vfs_file_entry.get_data_stream()?;
        assert!(result.is_some());

        Ok(())
    }

    // TODO: add tests for get_number_of_sub_file_entries
    // TODO: add tests for get_sub_file_entry_by_index
    // TODO: add tests for sub_file_entries
}
