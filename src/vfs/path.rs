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
    pub fn new(
        path_type: VfsPathType,
        location: &str,
        parent: Option<&VfsPathReference>,
    ) -> VfsPathReference {
        Rc::new(Self {
            path_type: path_type.clone(),
            location: location.to_string(),
            parent: parent.cloned(),
        })
    }

    /// Creates a new path from another path.
    pub fn new_from_path(path: &VfsPathReference, location: &str) -> VfsPathReference {
        Rc::new(Self {
            path_type: path.path_type.clone(),
            location: location.to_string(),
            parent: path.get_parent(),
        })
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
        let test_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        assert!(test_path.path_type == VfsPathType::Os);
        assert_eq!(test_path.location, "./test_data/file.txt");
        assert!(test_path.parent.is_none());

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));

        assert!(test_path.path_type == VfsPathType::Qcow);
        assert_eq!(test_path.location, "/");
        assert!(test_path.parent.is_some());
    }

    #[test]
    fn test_new_from_path() {
        let vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        let test_path: VfsPathReference =
            VfsPath::new_from_path(&vfs_path, "./test_data/bogus.txt");

        assert!(test_path.path_type == VfsPathType::Os);
        assert_eq!(test_path.location, "./test_data/bogus.txt");
        assert!(test_path.parent.is_none());
    }

    #[test]
    fn test_get_parent() {
        let test_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        let parent: Option<VfsPathReference> = test_path.get_parent();
        assert!(parent.is_none());

        let os_vfs_path: VfsPathReference =
            VfsPath::new(VfsPathType::Os, "./test_data/qcow/ext2.qcow2", None);
        let test_path: VfsPathReference = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));

        let parent: Option<VfsPathReference> = test_path.get_parent();
        assert!(parent.is_some());
    }
}
