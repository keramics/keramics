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

use std::rc::Rc;

use super::enums::VfsPathType;
use super::types::VfsPathReference;

/// Virtual File System (VFS) path.
pub struct VfsPath {
    /// Path type.
    pub path_type: VfsPathType,

    /// Location.
    pub location: String,

    /// Parent.
    parent: Option<VfsPathReference>,
}

impl VfsPath {
    /// Creates a new path.
    pub fn new(path_type: VfsPathType, location: &str, parent: Option<VfsPath>) -> Self {
        let parent_reference: Option<VfsPathReference> = match parent {
            Some(value) => Some(Rc::new(value)),
            None => None,
        };
        Self {
            path_type: path_type.clone(),
            location: location.to_string(),
            parent: parent_reference,
        }
    }

    /// Creates a new path from another path.
    pub fn new_from_path(path: &VfsPath, location: &str) -> Self {
        Self {
            path_type: path.path_type.clone(),
            location: location.to_string(),
            parent: path.get_parent(),
        }
    }

    /// Retrieves the parent path.
    pub fn get_parent(&self) -> Option<VfsPathReference> {
        match &self.parent {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        assert!(vfs_path.path_type == VfsPathType::Os);
        assert_eq!(vfs_path.location, "./test_data/file.txt");
        assert!(vfs_path.parent.is_none());
    }
}
