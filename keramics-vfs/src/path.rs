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

use std::path::MAIN_SEPARATOR_STR;

use keramics_formats::ext::ExtPath;
use keramics_formats::ntfs::NtfsPath;
use keramics_types::{ByteString, Ucs2String};

use super::enums::VfsType;

/// Retrieves the directory name of the path.
fn get_directory_name<'a>(path: &'a str, separator: &'a str) -> &'a str {
    match path.rsplit_once(separator) {
        Some(path_components) => {
            if path_components.0.is_empty() {
                separator
            } else {
                path_components.0
            }
        }
        None => path,
    }
}

/// Retrieves the file name of the path.
fn get_file_name<'a>(path: &'a str, separator: &'a str) -> &'a str {
    match path.rsplit_once(separator) {
        Some(path_components) => {
            if path_components.1.is_empty() {
                separator
            } else {
                path_components.1
            }
        }
        None => path,
    }
}

/// Virtual File System (VFS) path.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum VfsPath {
    Ext(ExtPath),
    Ntfs(NtfsPath),
    Os(String),
    String(Vec<String>),
}

impl VfsPath {
    const COMPONENT_SEPARATOR: &'static str = "/";

    /// Creates a new path.
    pub fn new(vfs_type: &VfsType, path: &str) -> Self {
        match vfs_type {
            VfsType::Apm
            | VfsType::Fake
            | VfsType::Gpt
            | VfsType::Mbr
            | VfsType::Qcow
            | VfsType::SparseImage
            | VfsType::Udif
            | VfsType::Vhd
            | VfsType::Vhdx => {
                let string_path_components: Vec<String> = if path == VfsPath::COMPONENT_SEPARATOR {
                    // Splitting "/" results in ["", ""]
                    vec![String::new()]
                } else {
                    path.split(VfsPath::COMPONENT_SEPARATOR)
                        .map(|component| component.to_string())
                        .collect::<Vec<String>>()
                };
                VfsPath::String(string_path_components)
            }
            VfsType::Ext => VfsPath::Ext(ExtPath::from(path)),
            VfsType::Ntfs => VfsPath::Ntfs(NtfsPath::from(path)),
            VfsType::Os => VfsPath::Os(path.to_string()),
        }
    }

    /// Creates a new path of the current path and additional path components.
    pub fn new_with_join<'a>(&'a self, path_components: &mut Vec<&'a str>) -> Self {
        match self {
            VfsPath::Ext(ext_path) => {
                let mut new_path_components: Vec<ByteString> = ext_path.components.clone();
                new_path_components.append(
                    &mut path_components
                        .iter()
                        .map(|component| ByteString::from_string(component))
                        .collect::<Vec<ByteString>>(),
                );
                VfsPath::Ext(ExtPath::from(&new_path_components))
            }
            VfsPath::Ntfs(ntfs_path) => {
                let mut new_path_components: Vec<Ucs2String> = ntfs_path.components.clone();
                new_path_components.append(
                    &mut path_components
                        .iter()
                        .map(|component| Ucs2String::from_string(component))
                        .collect::<Vec<Ucs2String>>(),
                );
                VfsPath::Ntfs(NtfsPath::from(&new_path_components))
            }
            VfsPath::Os(string_path) => {
                let mut new_path_components: Vec<&str> = vec![string_path.as_str()];
                new_path_components.append(path_components);
                VfsPath::Os(new_path_components.join(MAIN_SEPARATOR_STR))
            }
            VfsPath::String(string_path_components) => {
                let mut new_path_components: Vec<String> = string_path_components.clone();
                new_path_components.append(
                    &mut path_components
                        .iter()
                        .map(|component| component.to_string())
                        .collect::<Vec<String>>(),
                );
                VfsPath::String(new_path_components)
            }
        }
    }

    /// Creates a new path of the parent directory of the current path.
    pub fn new_with_parent_directory(&self) -> Self {
        match self {
            VfsPath::Ext(ext_path) => {
                let parent_ext_path: ExtPath = ext_path.new_with_parent_directory();
                VfsPath::Ext(parent_ext_path)
            }
            VfsPath::Ntfs(ntfs_path) => {
                let parent_ntfs_path: NtfsPath = ntfs_path.new_with_parent_directory();
                VfsPath::Ntfs(parent_ntfs_path)
            }
            VfsPath::Os(string_path) => {
                let parent_string_path: &str =
                    get_directory_name(string_path.as_str(), MAIN_SEPARATOR_STR);
                VfsPath::Os(parent_string_path.to_string())
            }
            VfsPath::String(string_path_components) => {
                let mut number_of_components: usize = string_path_components.len();
                if number_of_components > 1 {
                    number_of_components -= 1;
                }
                VfsPath::String(string_path_components[0..number_of_components].to_vec())
            }
        }
    }

    /// Retrieves the file name.
    pub fn get_file_name(&self) -> &str {
        match self {
            VfsPath::Ext(_) => todo!(),
            VfsPath::Ntfs(_) => todo!(),
            VfsPath::Os(string_path) => get_file_name(string_path.as_str(), MAIN_SEPARATOR_STR),
            VfsPath::String(string_path_components) => {
                let mut number_of_components: usize = string_path_components.len();
                if number_of_components > 1 {
                    number_of_components -= 1;
                }
                string_path_components[number_of_components].as_str()
            }
        }
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        match self {
            VfsPath::Ext(ext_path) => ext_path.to_string(),
            VfsPath::Ntfs(ntfs_path) => ntfs_path.to_string(),
            VfsPath::Os(string_path) => string_path.clone(),
            VfsPath::String(string_path_components) => {
                let number_of_components: usize = string_path_components.len();
                if number_of_components == 1 && string_path_components[0].is_empty() {
                    VfsPath::COMPONENT_SEPARATOR.to_string()
                } else {
                    string_path_components.join(VfsPath::COMPONENT_SEPARATOR)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_directory_name() {
        assert_eq!(get_directory_name("/", "/"), "/");
        assert_eq!(get_directory_name("/gpt1", "/"), "/");

        assert_eq!(get_directory_name("/gpt1", "\\"), "/gpt1");
    }

    #[test]
    fn test_get_file_name() {
        assert_eq!(get_file_name("/", "/"), "/");
        assert_eq!(get_file_name("/gpt1", "/"), "gpt1");

        assert_eq!(get_file_name("/gpt1", "\\"), "/gpt1");
    }

    #[test]
    fn test_new() {
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ext, "/ext1");
        assert!(matches!(vfs_path, VfsPath::Ext(_)));

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ntfs, "\\ntfs1");
        assert!(matches!(vfs_path, VfsPath::Ntfs(_)));

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Os, "/os1");
        assert!(matches!(vfs_path, VfsPath::Os(_)));

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/apm1");
        assert!(matches!(vfs_path, VfsPath::String(_)));
    }

    #[test]
    fn test_new_with_join() {
        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ext, "/");

        let test_path: VfsPath = vfs_path.new_with_join(&mut vec!["ext1"]);
        assert_eq!(test_path.to_string(), "/ext1");

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Ntfs, "\\");

        let test_path: VfsPath = vfs_path.new_with_join(&mut vec!["ntfs1"]);
        assert_eq!(test_path.to_string(), "\\ntfs1");

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Os, "/");

        let test_path: VfsPath = vfs_path.new_with_join(&mut vec!["os1"]);
        // TODO: change // to /
        assert_eq!(test_path.to_string(), "//os1");

        let vfs_path: VfsPath = VfsPath::new(&VfsType::Apm, "/");

        let test_path: VfsPath = vfs_path.new_with_join(&mut vec!["apm1"]);
        assert_eq!(test_path.to_string(), "/apm1");
    }

    // TODO: add tests for new_with_parent_directory
    // TODO: add tests for get_file_name
    // TODO: add tests for to_string
}
