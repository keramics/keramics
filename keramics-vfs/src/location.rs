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

use keramics_formats::Path;

use super::enums::VfsType;

/// Virtual File System (VFS) location.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VfsLocation {
    Base {
        path: Path,
        vfs_type: VfsType,
    },
    Layer {
        path: Path,
        parent: Arc<VfsLocation>,
        vfs_type: VfsType,
    },
}

impl VfsLocation {
    /// Creates a new location base.
    pub fn new_base(vfs_type: &VfsType, path: Path) -> Self {
        VfsLocation::Base {
            path,
            vfs_type: vfs_type.clone(),
        }
    }

    /// Creates a new location with an additional layer.
    pub fn new_with_layer(&self, vfs_type: &VfsType, path: Path) -> Self {
        VfsLocation::Layer {
            path,
            parent: Arc::new(self.clone()),
            vfs_type: vfs_type.clone(),
        }
    }

    /// Creates a new location from the path with the same parent.
    pub fn new_with_parent(&self, path: Path) -> Self {
        match self {
            VfsLocation::Base { vfs_type, .. } => VfsLocation::Base {
                path,
                vfs_type: vfs_type.clone(),
            },
            VfsLocation::Layer {
                parent, vfs_type, ..
            } => VfsLocation::Layer {
                path,
                parent: parent.clone(),
                vfs_type: vfs_type.clone(),
            },
        }
    }

    /// Retrieves the parent location.
    pub fn get_parent(&self) -> Option<&Self> {
        match self {
            VfsLocation::Base { .. } => None,
            VfsLocation::Layer { parent, .. } => Some(parent.as_ref()),
        }
    }

    /// Retrieves the path.
    pub fn get_path(&self) -> &Path {
        match self {
            VfsLocation::Base { path, .. } => &path,
            VfsLocation::Layer { path, .. } => &path,
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
                format!("{}: {}\n", vfs_type.as_str(), path.to_string())
            }
            VfsLocation::Layer {
                path,
                parent,
                vfs_type,
            } => {
                format!(
                    "{}{}: {}\n",
                    parent.to_string(),
                    vfs_type.as_str(),
                    path.to_string()
                )
            }
        }
    }
}

/// Creates a new OS VFS location.
pub fn new_os_vfs_location(path: &str) -> VfsLocation {
    VfsLocation::Base {
        path: Path::from(path),
        vfs_type: VfsType::Os,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::get_test_data_path;

    // TODO: add tests for new_base

    #[test]
    fn test_new_with_layer() {
        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let os_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        let path: Path = Path::from("/");
        let test_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, path);

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Qcow);

        let path: &Path = test_location.get_path();
        let expected_path: Path = Path::from("/");
        assert_eq!(path, &expected_path);
    }

    #[test]
    fn test_new_with_parent() {
        let path_string: String = get_test_data_path("directory/file.txt");
        let vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());

        let path_string: String = get_test_data_path("directory/bogus.txt");
        let path: Path = Path::from(path_string.as_str());
        let test_location: VfsLocation = vfs_location.new_with_parent(path);

        let path: &Path = test_location.get_path();
        let expected_path_string: String = get_test_data_path("directory/bogus.txt");
        let expected_path: Path = Path::from(expected_path_string.as_str());
        assert_eq!(path, &expected_path);

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Os);

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let os_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        let path: Path = Path::from("/");
        let vfs_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, path);

        let path: Path = Path::from("/qcow1");
        let test_location: VfsLocation = vfs_location.new_with_parent(path);

        let path: &Path = test_location.get_path();
        let expected_path: Path = Path::from("/qcow1");
        assert_eq!(path, &expected_path);

        let vfs_type: &VfsType = test_location.get_type();
        assert!(vfs_type == &VfsType::Qcow);
    }

    #[test]
    fn test_get_parent() {
        let path_string: String = get_test_data_path("directory/file.txt");
        let test_location: VfsLocation = new_os_vfs_location(path_string.as_str());

        let parent: Option<&VfsLocation> = test_location.get_parent();
        assert!(parent.is_none());

        let path_string: String = get_test_data_path("qcow/ext2.qcow2");
        let os_vfs_location: VfsLocation = new_os_vfs_location(path_string.as_str());
        let path: Path = Path::from("/");
        let test_location: VfsLocation = os_vfs_location.new_with_layer(&VfsType::Qcow, path);

        let parent: Option<&VfsLocation> = test_location.get_parent();
        assert!(parent.is_some());
    }

    // TODO: add tests for get_path
    // TODO: add tests for get_type
    // TODO: add tests for to_string
    // TODO: add tests for new_os_vfs_location
}
