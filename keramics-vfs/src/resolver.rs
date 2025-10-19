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

use std::sync::{Arc, RwLock};

use keramics_core::{DataStreamReference, ErrorTrace};

use super::context::VfsContext;
use super::file_entry::VfsFileEntry;
use super::location::VfsLocation;
use super::types::{VfsFileSystemReference, VfsResolverReference};

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
        vfs_location: &VfsLocation,
        name: Option<&str>,
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        match self.context.write() {
            Ok(mut context) => context.get_data_stream_by_path_and_name(vfs_location, name),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on context",
                    error
                ));
            }
        }
    }

    /// Retrieves a file entry with the specified location.
    pub fn get_file_entry_by_location(
        &self,
        vfs_location: &VfsLocation,
    ) -> Result<Option<VfsFileEntry>, ErrorTrace> {
        match self.context.write() {
            Ok(mut context) => context.get_file_entry_by_location(vfs_location),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on context",
                    error
                ));
            }
        }
    }

    /// Opens a file system.
    pub fn open_file_system(
        &self,
        vfs_location: &VfsLocation,
    ) -> Result<VfsFileSystemReference, ErrorTrace> {
        match self.context.write() {
            Ok(mut context) => context.open_file_system(vfs_location),
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to obtain write lock on context",
                    error
                ));
            }
        }
    }
}

thread_local! {
    static CURRENT_RESOLVER: VfsResolverReference = VfsResolver::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::file_system::VfsFileSystem;
    use crate::location::new_os_vfs_location;

    #[test]
    fn test_get_data_stream_by_path_and_name() -> Result<(), ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");
        let result: Option<DataStreamReference> =
            vfs_resolver.get_data_stream_by_path_and_name(&vfs_location, None)?;
        assert!(result.is_some());

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/bogus.txt");
        let result: Option<DataStreamReference> =
            vfs_resolver.get_data_stream_by_path_and_name(&vfs_location, None)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_location() -> Result<(), ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");
        let result: Option<VfsFileEntry> =
            vfs_resolver.get_file_entry_by_location(&vfs_location)?;
        assert!(result.is_some());

        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/bogus.txt");
        let result: Option<VfsFileEntry> =
            vfs_resolver.get_file_entry_by_location(&vfs_location)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_open_file_system() -> Result<(), ErrorTrace> {
        let vfs_resolver: VfsResolverReference = VfsResolver::current();

        let vfs_location: VfsLocation = new_os_vfs_location("/");
        let vfs_file_system: VfsFileSystemReference =
            vfs_resolver.open_file_system(&vfs_location)?;

        assert!(matches!(*vfs_file_system, VfsFileSystem::Os { .. }));

        Ok(())
    }
}
