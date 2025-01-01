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

use std::collections::HashMap;
use std::io;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use super::file_entry::VfsFileEntry;
use super::file_system::VfsFileSystem;
use super::path::VfsPath;
use super::types::VfsDataStreamReference;

/// Virtual File System (VFS) context.
pub struct VfsContext {
    /// File systems.
    file_systems: HashMap<Rc<VfsPath>, Weak<VfsFileSystem>>,

    /// Operating system (OS) file system path.
    os_vfs_path: Rc<VfsPath>,
}

impl VfsContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            file_systems: HashMap::new(),
            os_vfs_path: Rc::new(VfsPath::Os {
                location: "/".to_string(),
            }),
        }
    }

    /// Opens a data stream with the specified name.
    pub fn open_data_stream(
        &mut self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        let file_system: Rc<VfsFileSystem> = self.open_file_system(path)?;
        file_system.open_data_stream(path, name)
    }

    /// Opens a file entry.
    pub fn open_file_entry(&mut self, path: &VfsPath) -> io::Result<Option<VfsFileEntry>> {
        let file_system: Rc<VfsFileSystem> = self.open_file_system(path)?;
        file_system.open_file_entry(path)
    }

    /// Opens a file system.
    pub fn open_file_system(&mut self, path: &VfsPath) -> io::Result<Rc<VfsFileSystem>> {
        match path {
            VfsPath::Fake { .. } => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ));
            }
            _ => {}
        };
        let parent_path: Option<&Rc<VfsPath>> = path.get_parent();

        let lookup_key: &Rc<VfsPath> = match parent_path {
            Some(parent_path) => parent_path,
            None => &self.os_vfs_path,
        };
        let cached_file_system: Option<Rc<VfsFileSystem>> = match self.file_systems.get(lookup_key)
        {
            Some(file_system) => file_system.upgrade(),
            None => None,
        };
        match cached_file_system {
            Some(file_system) => Ok(file_system),
            None => {
                let parent_file_system: Option<Rc<VfsFileSystem>> = match parent_path {
                    Some(parent_path) => Some(self.open_file_system(parent_path)?),
                    None => None,
                };
                let file_system_path: &VfsPath = match parent_path {
                    Some(parent_path) => parent_path,
                    None => self.os_vfs_path.as_ref(),
                };
                let mut file_system: VfsFileSystem = VfsFileSystem::new(&path.get_path_type());
                file_system.open(parent_file_system, file_system_path)?;

                let lookup_key: Rc<VfsPath> = match parent_path {
                    Some(parent_path) => parent_path.clone(),
                    None => self.os_vfs_path.clone(),
                };
                let cached_file_system: Rc<VfsFileSystem> = Rc::new(file_system);

                self.file_systems
                    .insert(lookup_key, Rc::downgrade(&cached_file_system));

                Ok(cached_file_system)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsPathType;
    use crate::vfs::path::VfsPath;

    #[test]
    fn test_open_data_stream() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        let result: Option<VfsDataStreamReference> =
            vfs_context.open_data_stream(&vfs_path, None)?;
        assert!(result.is_some());

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        let result: Option<VfsDataStreamReference> =
            vfs_context.open_data_stream(&vfs_path, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        let result: Option<VfsFileEntry> = vfs_context.open_file_entry(&vfs_path)?;
        assert!(result.is_some());

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        let result: Option<VfsFileEntry> = vfs_context.open_file_entry(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        assert!(vfs_file_system.get_vfs_path_type() == VfsPathType::Os);

        Ok(())
    }
}
