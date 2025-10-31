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
use keramics_formats::PathComponent;
use keramics_formats::ext::ExtPath;
use keramics_formats::fat::{FatPath, FatString};
use keramics_formats::ntfs::NtfsPath;
use keramics_types::{ByteString, Ucs2String};

use super::enums::VfsType;
use super::string_path::StringPath;

/// Virtual File System (VFS) path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VfsPath {
    Ext(ExtPath),
    Fat(FatPath),
    Ntfs(NtfsPath),
    Os(PathBuf),
    String(StringPath),
}

impl VfsPath {
    /// Creates a new VFS path based on the path string.
    pub fn from_path(vfs_type: &VfsType, path: &str) -> Self {
        match vfs_type {
            VfsType::Apm
            | VfsType::Ewf
            | VfsType::Fake
            | VfsType::Gpt
            | VfsType::Mbr
            | VfsType::Qcow
            | VfsType::SparseImage
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx => VfsPath::String(StringPath::from(path)),
            VfsType::Ext => VfsPath::Ext(ExtPath::from(path)),
            VfsType::Fat => VfsPath::Fat(FatPath::from(path)),
            VfsType::Ntfs => VfsPath::Ntfs(NtfsPath::from(path)),
            VfsType::Os => VfsPath::Os(PathBuf::from(path)),
        }
    }

    /// Creates a new VFS path based on the path components.
    pub fn from_path_components(vfs_type: &VfsType, path_components: &[&str]) -> Self {
        match vfs_type {
            VfsType::Apm
            | VfsType::Ewf
            | VfsType::Fake
            | VfsType::Gpt
            | VfsType::Mbr
            | VfsType::Qcow
            | VfsType::SparseImage
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx => VfsPath::String(StringPath::from(path_components)),
            VfsType::Ext => VfsPath::Ext(ExtPath::from(path_components)),
            VfsType::Fat => VfsPath::Fat(FatPath::from(path_components)),
            VfsType::Ntfs => VfsPath::Ntfs(NtfsPath::from(path_components)),
            VfsType::Os => VfsPath::Os(PathBuf::from_iter(path_components)),
        }
    }

    /// Creates a new VFS path of the current path and additional path components.
    pub fn new_with_join(&self, path_components: &[PathComponent]) -> Result<Self, ErrorTrace> {
        let vfs_path: VfsPath = match self {
            VfsPath::Ext(ext_path) => {
                let mut ext_path_components: Vec<ByteString> = ext_path.components.clone();

                for path_component in path_components {
                    let byte_string: ByteString = match path_component {
                        PathComponent::ByteString(byte_string) => byte_string.clone(),
                        PathComponent::String(string) => ByteString::from(string),
                        _ => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported path component"
                            ));
                        }
                    };
                    ext_path_components.push(byte_string);
                }
                VfsPath::Ext(ExtPath {
                    components: ext_path_components,
                })
            }
            VfsPath::Fat(fat_path) => {
                let mut fat_path_components: Vec<FatString> = fat_path.components.clone();

                for path_component in path_components {
                    let fat_string: FatString = match path_component {
                        PathComponent::ByteString(byte_string) => {
                            FatString::ByteString(byte_string.clone())
                        }
                        PathComponent::Ucs2String(ucs2_string) => {
                            FatString::Ucs2String(ucs2_string.clone())
                        }
                        PathComponent::String(string) => FatString::from(string),
                    };
                    fat_path_components.push(fat_string);
                }
                VfsPath::Fat(FatPath {
                    components: fat_path_components,
                })
            }
            VfsPath::Ntfs(ntfs_path) => {
                let mut ntfs_path_components: Vec<Ucs2String> = ntfs_path.components.clone();

                for path_component in path_components {
                    let ucs2_string: Ucs2String = match path_component {
                        PathComponent::String(string) => Ucs2String::from(string),
                        PathComponent::Ucs2String(ucs2_string) => ucs2_string.clone(),
                        _ => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported path component"
                            ));
                        }
                    };
                    ntfs_path_components.push(ucs2_string);
                }
                VfsPath::Ntfs(NtfsPath {
                    components: ntfs_path_components,
                })
            }
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
            VfsPath::String(string_path) => {
                let mut string_path_components: Vec<String> = string_path.components.clone();

                for path_component in path_components {
                    let string: String = match path_component {
                        PathComponent::String(string) => string.clone(),
                        _ => {
                            return Err(keramics_core::error_trace_new!(
                                "Unsupported path component"
                            ));
                        }
                    };
                    string_path_components.push(string);
                }
                VfsPath::String(StringPath {
                    components: string_path_components,
                })
            }
        };
        Ok(vfs_path)
    }

    /// Creates a new VFS path of the parent directory of the current path.
    pub fn new_with_parent_directory(&self) -> Self {
        match self {
            VfsPath::Ext(ext_path) => {
                let parent_ext_path: ExtPath = ext_path.new_with_parent_directory();
                VfsPath::Ext(parent_ext_path)
            }
            VfsPath::Fat(fat_path) => {
                let parent_fat_path: FatPath = fat_path.new_with_parent_directory();
                VfsPath::Fat(parent_fat_path)
            }
            VfsPath::Ntfs(ntfs_path) => {
                let parent_ntfs_path: NtfsPath = ntfs_path.new_with_parent_directory();
                VfsPath::Ntfs(parent_ntfs_path)
            }
            VfsPath::Os(path_buf) => {
                let mut new_path_buf: PathBuf = path_buf.clone();
                new_path_buf.pop();

                VfsPath::Os(new_path_buf)
            }
            VfsPath::String(string_path) => {
                let parent_string_path: StringPath = string_path.new_with_parent_directory();
                VfsPath::String(parent_string_path)
            }
        }
    }

    /// Retrieves the file name.
    pub fn get_file_name(&self) -> Option<PathComponent> {
        match self {
            VfsPath::Ext(ext_path) => match ext_path.file_name() {
                Some(byte_string) => Some(PathComponent::ByteString(byte_string.clone())),
                None => None,
            },
            VfsPath::Fat(fat_path) => match fat_path.file_name() {
                Some(fat_string) => match fat_string {
                    FatString::ByteString(byte_string) => {
                        Some(PathComponent::ByteString(byte_string.clone()))
                    }
                    FatString::Ucs2String(ucs2_string) => {
                        Some(PathComponent::Ucs2String(ucs2_string.clone()))
                    }
                },
                None => None,
            },
            VfsPath::Ntfs(ntfs_path) => match ntfs_path.file_name() {
                Some(ucs2_string) => Some(PathComponent::Ucs2String(ucs2_string.clone())),
                None => None,
            },
            VfsPath::Os(path_buf) => match path_buf.file_name() {
                Some(os_str) => Some(PathComponent::String(os_str.to_str().unwrap().to_string())),
                None => None,
            },
            VfsPath::String(string_path) => match string_path.file_name() {
                Some(string) => Some(PathComponent::String(string.clone())),
                None => None,
            },
        }
    }

    /// Retrieves a string representation of the VFS path.
    pub fn to_string(&self) -> String {
        match self {
            VfsPath::Ext(ext_path) => ext_path.to_string(),
            VfsPath::Fat(fat_path) => fat_path.to_string(),
            VfsPath::Ntfs(ntfs_path) => ntfs_path.to_string(),
            // TODO: change to_string_lossy to a non-lossy conversion
            VfsPath::Os(path_buf) => path_buf.to_string_lossy().to_string(),
            VfsPath::String(string_path) => string_path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/ext1");
        assert!(matches!(vfs_path, VfsPath::Ext(_)));

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/fat1");
        assert!(matches!(vfs_path, VfsPath::Fat(_)));

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ntfs, "\\ntfs1");
        assert!(matches!(vfs_path, VfsPath::Ntfs(_)));

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "/os1");
        assert!(matches!(vfs_path, VfsPath::Os(_)));

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm1");
        assert!(matches!(vfs_path, VfsPath::String(_)));
    }

    #[test]
    fn test_from_path_components() {
        let path_components: [&str; 1] = ["ext1"];
        let vfs_path: VfsPath = VfsPath::from_path_components(&VfsType::Ext, &path_components);
        assert!(matches!(vfs_path, VfsPath::Ext(_)));

        let path_components: [&str; 1] = ["fat1"];
        let vfs_path: VfsPath = VfsPath::from_path_components(&VfsType::Fat, &path_components);
        assert!(matches!(vfs_path, VfsPath::Fat(_)));

        let path_components: [&str; 1] = ["ntfs1"];
        let vfs_path: VfsPath = VfsPath::from_path_components(&VfsType::Ntfs, &path_components);
        assert!(matches!(vfs_path, VfsPath::Ntfs(_)));

        let path_components: [&str; 1] = ["os1"];
        let vfs_path: VfsPath = VfsPath::from_path_components(&VfsType::Os, &path_components);
        assert!(matches!(vfs_path, VfsPath::Os(_)));

        let path_components: [&str; 1] = ["apm1"];
        let vfs_path: VfsPath = VfsPath::from_path_components(&VfsType::Apm, &path_components);
        assert!(matches!(vfs_path, VfsPath::String(_)));
    }

    #[test]
    fn test_new_with_join() -> Result<(), ErrorTrace> {
        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("ext1")];
        let test_path: VfsPath = vfs_path.new_with_join(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/ext1");

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("fat1")];
        let test_path: VfsPath = vfs_path.new_with_join(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/fat1");

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ntfs, "\\");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("ntfs1")];
        let test_path: VfsPath = vfs_path.new_with_join(&test_path_components)?;
        assert_eq!(test_path.to_string(), "\\ntfs1");

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("os1")];
        let test_path: VfsPath = vfs_path.new_with_join(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/os1");

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/");

        let test_path_components: [PathComponent; 1] = [PathComponent::from("apm1")];
        let test_path: VfsPath = vfs_path.new_with_join(&test_path_components)?;
        assert_eq!(test_path.to_string(), "/apm1");

        Ok(())
    }

    // TODO: add tests for new_with_parent_directory

    #[test]
    fn test_get_file_name() {
        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ext, "/ext1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(
            result,
            Some(PathComponent::ByteString(ByteString::from("ext1")))
        );

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Fat, "/fat1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from("fat1")))
        );

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Ntfs, "\\ntfs1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(
            result,
            Some(PathComponent::Ucs2String(Ucs2String::from("ntfs1")))
        );

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Os, "/os1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(result, Some(PathComponent::String(String::from("os1"))));

        let vfs_path: VfsPath = VfsPath::from_path(&VfsType::Apm, "/apm1");
        let result: Option<PathComponent> = vfs_path.get_file_name();
        assert_eq!(result, Some(PathComponent::String(String::from("apm1"))));
    }

    // TODO: add tests for to_string
}
