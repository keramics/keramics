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
use std::rc::Rc;

use super::file_entry::VfsFileEntry;
use super::file_system::VfsFileSystem;
use super::types::{VfsDataStreamReference, VfsPathReference};

/// Virtual File System (VFS) context.
pub struct VfsContext {
    /// File systems.
    file_systems: HashMap<String, Rc<VfsFileSystem>>,
}

impl VfsContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            file_systems: HashMap::new(),
        }
    }

    /// Opens a data stream with the specified name.
    pub fn open_data_stream(
        &mut self,
        path: &VfsPathReference,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        let file_system: Rc<VfsFileSystem> = self.open_file_system(path)?;
        file_system.open_data_stream(path, name)
    }

    /// Opens a file entry.
    pub fn open_file_entry(&mut self, path: &VfsPathReference) -> io::Result<Option<VfsFileEntry>> {
        let file_system: Rc<VfsFileSystem> = self.open_file_system(path)?;
        file_system.open_file_entry(path)
    }

    /// Opens a file system.
    pub fn open_file_system(&mut self, path: &VfsPathReference) -> io::Result<Rc<VfsFileSystem>> {
        // TODO: ensure the lookup key is unique for nested VFS paths.
        let parent_path: Option<VfsPathReference> = path.get_parent();
        let lookup_key: &str = match parent_path.as_ref() {
            Some(parent_path) => &parent_path.location,
            None => "",
        };
        match self.file_systems.get(lookup_key) {
            Some(value) => return Ok(value.clone()),
            None => {}
        };
        let parent_file_system: Option<Rc<VfsFileSystem>> = match parent_path.as_ref() {
            Some(parent_path) => Some(self.open_file_system(parent_path)?),
            None => None,
        };
        let file_system_path: &VfsPathReference = match parent_path.as_ref() {
            Some(parent_path) => parent_path,
            None => path,
        };
        let mut file_system: VfsFileSystem = VfsFileSystem::new(&path.path_type);
        file_system.open(parent_file_system, file_system_path)?;

        self.file_systems
            .insert(lookup_key.to_string(), Rc::new(file_system));

        match self.file_systems.get(lookup_key) {
            Some(value) => return Ok(value.clone()),
            None => Err(io::Error::new(io::ErrorKind::Other, "Missing file system")),
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

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        let result: Option<VfsDataStreamReference> =
            vfs_context.open_data_stream(&vfs_path, None)?;
        assert!(result.is_some());

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        let result: Option<VfsDataStreamReference> =
            vfs_context.open_data_stream(&vfs_path, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);
        let result: Option<VfsFileEntry> = vfs_context.open_file_entry(&vfs_path)?;
        assert!(result.is_some());

        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/bogus.txt", None);
        let result: Option<VfsFileEntry> = vfs_context.open_file_entry(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPathReference = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: Rc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        assert!(vfs_file_system.get_vfs_path_type() == VfsPathType::Os);

        Ok(())
    }
}
