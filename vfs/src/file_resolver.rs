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

use core::{DataStreamReference, FileResolver, FileResolverReference};

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
    fn get_data_stream<'a>(
        &'a self,
        path_components: &mut Vec<&'a str>,
    ) -> io::Result<Option<DataStreamReference>> {
        let path: VfsPath = self.base_path.append_components(path_components);

        match self.file_system.get_file_entry_by_path(&path)? {
            // TODO: replace by get_data_fork_by_name
            Some(file_entry) => file_entry.get_data_stream_by_name(None),
            None => Ok(None),
        }
    }
}

/// Opens a new  Virtual File System (VFS) file resolver.
pub fn open_vfs_file_resolver(
    file_system: &VfsFileSystemReference,
    base_path: VfsPath,
) -> io::Result<FileResolverReference> {
    let file_resolver: VfsFileResolver = VfsFileResolver::new(file_system, base_path);
    Ok(FileResolverReference::new(Box::new(file_resolver)))
}

// TODO: add tests
