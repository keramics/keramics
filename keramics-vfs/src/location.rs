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

use std::sync::Arc;

use super::enums::VfsType;
use super::path::VfsPath;

/// Virtual File System (VFS) location.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum VfsLocation {
    Base {
        path: VfsPath,
        vfs_type: VfsType,
    },
    Layer {
        path: VfsPath,
        parent: Arc<VfsLocation>,
        vfs_type: VfsType,
    },
}

impl VfsLocation {
    /// Creates a new location with an additional layer.
    pub fn new_with_layer(&self, vfs_type: &VfsType, path: VfsPath) -> Self {
        VfsLocation::Layer {
            path: path,
            parent: Arc::new(self.clone()),
            vfs_type: vfs_type.clone(),
        }
    }

    /// Creates a new location from the path with the same parent.
    pub fn new_with_parent(&self, path: VfsPath) -> Self {
        match self {
            VfsLocation::Base { vfs_type, .. } => VfsLocation::Base {
                path: path,
                vfs_type: vfs_type.clone(),
            },
            VfsLocation::Layer {
                parent, vfs_type, ..
            } => VfsLocation::Layer {
                path: path,
                parent: parent.clone(),
                vfs_type: vfs_type.clone(),
            },
        }
    }

    /// Retrieves the path.
    pub fn get_path(&self) -> &VfsPath {
        match self {
            VfsLocation::Base { path, .. } => &path,
            VfsLocation::Layer { path, .. } => &path,
        }
    }

    /// Retrieves the parent location.
    pub fn get_parent(&self) -> Option<&Self> {
        match self {
            VfsLocation::Base { .. } => None,
            VfsLocation::Layer { parent, .. } => Some(parent.as_ref()),
        }
    }

    /// Retrieves the type.
    pub fn get_type(&self) -> &VfsType {
        match self {
            VfsLocation::Base { vfs_type, .. } => &vfs_type,
            VfsLocation::Layer { vfs_type, .. } => &vfs_type,
        }
    }

    /// Retrieves a string representation of the location.
    pub fn to_string(&self) -> String {
        match self {
            VfsLocation::Base { path, vfs_type } => {
                format!("type: {}: path: {}\n", vfs_type.as_str(), path.to_string())
            }
            VfsLocation::Layer {
                path,
                parent,
                vfs_type,
            } => {
                format!(
                    "{}\ntype: {}: path: {}\n",
                    vfs_type.as_str(),
                    parent.to_string(),
                    path.to_string()
                )
            }
        }
    }
}

/// Creates a new OS VFS location.
pub fn new_os_vfs_location(path: &str) -> VfsLocation {
    VfsLocation::Base {
        path: VfsPath::new(&VfsType::Os, path),
        vfs_type: VfsType::Os,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_layer() {
        let os_vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Qcow, "/");
        let test_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, vfs_path);

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Qcow);

        let vfs_path: &VfsPath = test_location.get_path();
        assert_eq!(vfs_path.to_string(), "/");
    }

    #[test]
    fn test_new_with_parent() {
        let vfs_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Os, "../test_data/bogus.txt");
        let test_location: VfsLocation = vfs_location.new_with_parent(vfs_path);

        let vfs_path: &VfsPath = test_location.get_path();
        assert_eq!(vfs_path.to_string(), "../test_data/bogus.txt");

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Os);

        let os_vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Qcow, "/");
        let vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, vfs_path);

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Qcow, "/qcow1");
        let test_location: VfsLocation = vfs_location.new_with_parent(vfs_path);

        let vfs_path: &VfsPath = test_location.get_path();
        assert_eq!(vfs_path.to_string(), "/qcow1");

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Qcow);
    }

    // TODO: add tests for get_path

    #[test]
    fn test_get_parent() {
        let test_location: VfsLocation = new_os_vfs_location("../test_data/file.txt");

        let parent: Option<&VfsLocation> = test_location.get_parent();
        assert!(parent.is_none());

        let os_vfs_location: VfsLocation = new_os_vfs_location("../test_data/qcow/ext2.qcow2");
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Qcow, "/");
        let test_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, vfs_path);

        let parent: Option<&VfsLocation> = test_location.get_parent();
        assert!(parent.is_some());
    }

    // TODO: add tests for get_type
    // TODO: add tests for to_string
}
