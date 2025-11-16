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

use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::{Path, PathComponent};

use super::enums::VfsType;

/// Virtual File System (VFS) path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VfsPath {
    Os(PathBuf),
    Path(Path),
}

impl VfsPath {
    /// Creates a new path based on the string.
    pub fn from_string(vfs_type: &VfsType, path: &str) -> Self {
        match vfs_type {
            VfsType::Apm
            | VfsType::Ewf
            | VfsType::Ext
            | VfsType::Fake
            | VfsType::Fat
            | VfsType::Gpt
            | VfsType::Mbr
            | VfsType::Ntfs
            | VfsType::Qcow
            | VfsType::SparseImage
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx => VfsPath::Path(Path::from(path)),
            VfsType::Os => VfsPath::Os(PathBuf::from(path)),
        }
    }

    /// Creates a new path of the current path joined with the path.
    pub fn new_with_join(&self, path: &VfsPath) -> Result<Self, ErrorTrace> {
        let vfs_path: VfsPath = match self {
            VfsPath::Os(path_buf) => todo!(),
            VfsPath::Path(path) => VfsPath::Path(path.new_with_join(path)),
        };
        Ok(vfs_path)
    }

    /// Creates a new path of the current path and additional path components.
    pub fn new_with_join_path_components(
        &self,
        path_components: &[PathComponent],
    ) -> Result<Self, ErrorTrace> {
        let vfs_path: VfsPath = match self {
            VfsPath::Os(path_buf) => {
                let mut new_path_buf: PathBuf = path_buf.clone();

                for path_component in path_components {
                    let string: String = match path_component {
                        PathComponent::String(string) => string.clone(),
                        _ => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported path component"
                            ));
                        }
                    };
                    new_path_buf.push(string);
                }
                VfsPath::Os(new_path_buf)
            }
            VfsPath::Path(path) => {
                VfsPath::Path(path.new_with_join_path_components(path_components))
            }
        };
        Ok(vfs_path)
    }

    /// Creates a new path of the parent directory of the current path.
    pub fn new_with_parent_directory(&self) -> Self {
        match self {
            VfsPath::Os(path_buf) => {
                let mut new_path_buf: PathBuf = path_buf.clone();
                new_path_buf.pop();

                VfsPath::Os(new_path_buf)
            }
            VfsPath::Path(path) => VfsPath::Path(path.new_with_parent_directory()),
        }
    }

    /// Retrieves the file name.
    pub fn get_file_name(&self) -> Option<PathComponent> {
        match self {
            VfsPath::Os(path_buf) => match path_buf.file_name() {
                Some(os_str) => Some(PathComponent::String(os_str.to_str().unwrap().to_string())),
                None => None,
            },
            VfsPath::Path(path) => path.file_name().cloned(),
        }
    }

    /// Retrieves a numeric path component suffix.
    pub(crate) fn get_numeric_suffix(
        path_component: &PathComponent,
        prefix: &str,
    ) -> Option<usize> {
        let string: String = path_component.to_string();

        if !string.starts_with(prefix) {
            return None;
        }
        match string[prefix.len()..].parse::<usize>() {
            Ok(numeric_suffix) => Some(numeric_suffix),
            Err(_) => None,
        }
    }

    /// Determines if the path is relative.
    pub fn is_relative(&self) -> bool {
        match self {
            VfsPath::Os(path_buf) => path_buf.is_relative(),
            VfsPath::Path(path) => path.is_relative(),
        }
    }

    /// Determines if the path represents the root.
    pub fn is_root(&self) -> bool {
        match self {
            VfsPath::Os(path_buf) => path_buf.has_root() && path_buf.parent().is_none(),
            VfsPath::Path(path) => path.is_root(),
        }
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        match self {
            // TODO: change to_string_lossy to a non-lossy conversion
            VfsPath::Os(path_buf) => path_buf.to_string_lossy().to_string(),
            VfsPath::Path(path) => path.to_string(),
        }
    }
}

impl From<Path> for VfsPath {
    /// Converts a [`Path`] into a [`VfsPath`]
    #[inline(always)]
    fn from(path: Path) -> Self {
        Self::Path(path)
    }
}

impl From<PathBuf> for VfsPath {
    /// Converts a [`PathBuf`] into a [`VfsPath`]
    #[inline(always)]
    fn from(path_buf: PathBuf) -> Self {
        Self::Os(path_buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Apm, "apm1");
        assert!(matches!(vfs_path, VfsPath::Path(_)));

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "os1");
        assert!(matches!(vfs_path, VfsPath::Os(_)));
    }

    #[test]
    fn test_new_with_join_path_components() -> Result<(), ErrorTrace> {
        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Apm, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("apm1")];
        let test_path: VfsPath = vfs_path.new_with_join_path_components(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/apm1");

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("os1")];
        let test_path: VfsPath = vfs_path.new_with_join_path_components(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/os1");

        Ok(())
    }

    // TODO: add tests for new_with_parent_directory

    #[test]
    fn test_is_relative() {
        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Apm, "apm1");
        assert_eq!(vfs_path.is_relative(), true);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "os1");
        assert_eq!(vfs_path.is_relative(), true);
    }

    #[test]
    fn test_is_root() {
        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Apm, "apm1");
        assert_eq!(vfs_path.is_root(), false);

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "os1");
        assert_eq!(vfs_path.is_root(), false);
    }

    #[test]
    fn test_get_file_name() {
        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Apm, "/apm1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(result, Some(PathComponent::from("apm1")));

        let vfs_path: VfsPath = VfsPath::from_string(&VfsType::Os, "/os1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(result, Some(PathComponent::String(String::from("os1"))));
    }

    // TODO: add tests for to_string
    // TODO: add tests for from path
    // TODO: add tests for from path_buf
}
