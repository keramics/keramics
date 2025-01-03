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
use std::sync::{Arc, RwLock};

use super::context::VfsContext;
use super::file_entry::VfsFileEntry;
use super::file_system::VfsFileSystem;
use super::path::VfsPath;
use super::types::{VfsDataStreamReference, VfsResolverReference};

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

    /// Retrieves a data stream with the specified path and name.
    pub fn get_data_stream_by_path_and_name(
        &self,
        path: &VfsPath,
        name: Option<&str>,
    ) -> io::Result<Option<VfsDataStreamReference>> {
        match self.context.write() {
            Ok(mut context) => context.get_data_stream_by_path_and_name(path, name),
            Err(error) => Err(crate::error_to_io_error!(error)),
        }
    }

    /// Retrieves a file entry with the specified path.
    pub fn get_file_entry_by_path(&self, path: &VfsPath) -> io::Result<Option<VfsFileEntry>> {
        match self.context.write() {
            Ok(mut context) => context.get_file_entry_by_path(path),
            Err(error) => Err(crate::error_to_io_error!(error)),
        }
    }

    /// Opens a file system.
    pub fn open_file_system(&self, path: &VfsPath) -> io::Result<Rc<VfsFileSystem>> {
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
    use crate::vfs::path::VfsPath;

    #[test]
    fn test_get_data_stream_by_path_and_name() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/file.txt".to_string(),
        };
        let result: Option<VfsDataStreamReference> =
            vfs_resolver.get_data_stream_by_path_and_name(&vfs_path, None)?;
        assert!(result.is_some());

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/bogus.txt".to_string(),
        };
        let result: Option<VfsDataStreamReference> =
            vfs_resolver.get_data_stream_by_path_and_name(&vfs_path, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/file.txt".to_string(),
        };
        let result: Option<VfsFileEntry> = vfs_resolver.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_some());

        let vfs_path: VfsPath = VfsPath::Os {
            location: "./test_data/bogus.txt".to_string(),
        };
        let result: Option<VfsFileEntry> = vfs_resolver.get_file_entry_by_path(&vfs_path)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> io::Result<()> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "/".to_string(),
        };
        let vfs_file_system: Rc<VfsFileSystem> = vfs_resolver.open_file_system(&vfs_path)?;

        assert!(vfs_file_system.get_vfs_path_type() == VfsPathType::Os);

        Ok(())
    }
}
