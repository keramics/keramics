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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::{FileResolver, FileResolverReference, PathComponent};

use crate::file_entry::VfsFileEntry;
use crate::path::VfsPath;
use crate::types::VfsFileSystemReference;

pub struct VfsFileResolver {
    /// File system.
    file_system: VfsFileSystemReference,

    /// Base path.
    base_path: VfsPath,
}

impl VfsFileResolver {
    /// Creates a new file resolver.
    pub fn new(file_system: &VfsFileSystemReference, base_path: VfsPath) -> Self {
        Self {
            file_system: file_system.clone(),
            base_path: base_path,
        }
    }
}

impl FileResolver for VfsFileResolver {
    /// Retrieves a data stream with the specified path.
    fn get_data_stream(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Option<DataStreamReference>, ErrorTrace> {
        let vfs_path: VfsPath = match self.base_path.new_with_join(path_components) {
            Ok(path) => path,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create VFS path");
                return Err(error);
            }
        };
        let result: Option<VfsFileEntry> = match self.file_system.get_file_entry_by_path(&vfs_path)
        {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve file entry");
                return Err(error);
            }
        };
        match result {
            // TODO: replace by get_data_fork_by_name
            Some(file_entry) => file_entry.get_data_stream_by_name(None),
            None => Ok(None),
        }
    }
}

/// Creates a new  Virtual File System (VFS) file resolver.
pub fn new_vfs_file_resolver(
    file_system: &VfsFileSystemReference,
    base_path: VfsPath,
) -> Result<FileResolverReference, ErrorTrace> {
    let file_resolver: VfsFileResolver = VfsFileResolver::new(file_system, base_path);
    Ok(FileResolverReference::new(Box::new(file_resolver)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::VfsType;
    use crate::file_system::VfsFileSystem;

    #[test]
    fn test_get_data_stream() -> Result<(), ErrorTrace> {
        let file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Os, "../test_data");
        let file_resolver: FileResolverReference = new_vfs_file_resolver(&file_system, vfs_path)?;

        let path_components: [PathComponent; 1] = [PathComponent::from("file.txt")];
        let result: Option<DataStreamReference> =
            file_resolver.get_data_stream(&path_components)?;
        assert!(result.is_some());

        Ok(())
    }

    #[test]
    fn test_new_vfs_file_resolver() -> Result<(), ErrorTrace> {
        let file_system: VfsFileSystemReference =
            VfsFileSystemReference::new(VfsFileSystem::new(&VfsType::Os));
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Os, "../test_data");
        let _ = new_vfs_file_resolver(&file_system, vfs_path)?;

        Ok(())
    }
}
