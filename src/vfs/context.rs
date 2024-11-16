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

use crate::types::SharedValue;

use super::enums::VfsPathType;
use super::file_systems::*;
use super::path::VfsPath;
use super::traits::VfsFileSystem;
use super::types::{VfsFileSystemReference, VfsPathReference};

/// Virtual File System (VFS) context.
pub struct VfsContext {
    /// File systems.
    file_systems: HashMap<String, VfsFileSystemReference>,
}

impl VfsContext {
    /// Creates a new context.
    pub fn new() -> Self {
        Self {
            file_systems: HashMap::new(),
        }
    }

    /// Opens a file system.
    pub fn open_file_system(&mut self, path: &VfsPath) -> io::Result<VfsFileSystemReference> {
        // TODO: ensure the lookup key is unique for nested VFS paths.
        let parent_path: Option<VfsPathReference> = path.get_parent();
        let lookup_key: &str = match &parent_path {
            Some(parent_path) => &(*parent_path.location),
            None => "",
        };
        match self.file_systems.get(lookup_key) {
            Some(value) => return Ok(value.clone()),
            None => {}
        }
        let mut file_system: Box<dyn VfsFileSystem> = match path.path_type {
            VfsPathType::Os => Box::new(OsVfsFileSystem::new()),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unsupported path type",
                ));
            }
        };
        file_system.open_with_resolver(path)?;

        self.file_systems
            .insert(lookup_key.to_string(), SharedValue::new(file_system));

        match self.file_systems.get(lookup_key) {
            Some(value) => return Ok(value.clone()),
            None => Err(io::Error::new(io::ErrorKind::Other, "Missing file system")),
        }
    }
}
