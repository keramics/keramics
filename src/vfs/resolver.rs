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
use std::sync::{Arc, RwLock};

use super::context::VfsContext;
use super::path::VfsPath;
use super::types::{
    VfsDataStreamReference, VfsFileEntryReference, VfsFileSystemReference, VfsResolverReference,
};

/// Virtual File System (VFS) resolver.
pub struct VfsResolver {
    /// VFS context.
    context: RwLock<VfsContext>,
}

impl VfsResolver {
    /// Creates a new resolver.
    fn new() -> Arc<Self> {
        Arc::new(Self {
            context: RwLock::new(VfsContext::new()),
        })
    }

    /// Retrieves a reference to the resolver.
    pub fn current() -> VfsResolverReference {
        CURRENT_RESOLVER.with(|resolver| resolver.clone())
    }

    /// Opens a data stream with the specified name.
    pub fn open_data_stream(
        &self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        match self.context.write() {
            Ok(mut context) => context.open_data_stream(path, name),
            Err(error) => Err(crate::error_to_io_error!(error)),
        }
    }

    /// Opens a file entry.
    pub fn open_file_entry(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntryReference>> {
        match self.context.write() {
            Ok(mut context) => context.open_file_entry(path),
            Err(error) => Err(crate::error_to_io_error!(error)),
        }
    }

    /// Opens a file system.
    pub fn open_file_system(&self, path: &VfsPath) -> io::Result<VfsFileSystemReference> {
        match self.context.write() {
            Ok(mut context) => context.open_file_system(path),
            Err(error) => Err(crate::error_to_io_error!(error)),
        }
    }
}

thread_local! {
    static CURRENT_RESOLVER: VfsResolverReference = VfsResolver::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::enums::VfsPathType;

    // TODO: add tests for open_data_stream

    #[test]
    fn test_open_file_entry() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "test_data/file.txt", None);
        let result: Option<VfsFileEntryReference> = vfs_resolver.open_file_entry(&vfs_path)?;
        assert!(result.is_some());

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "test_data/bogus.txt", None);
        let result: Option<VfsFileEntryReference> = vfs_resolver.open_file_entry(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "/", None);
        let vfs_file_system: VfsFileSystemReference = vfs_resolver.open_file_system(&vfs_path)?;

        let vfs_path_type: VfsPathType = match vfs_file_system.with_read_lock() {
            Ok(file_system) => file_system.get_vfs_path_type(),
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        assert!(vfs_path_type == VfsPathType::Os);

        Ok(())
    }
}
