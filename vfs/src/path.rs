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
use std::sync::Arc;

use formats::ext::ExtPath;
use formats::ntfs::NtfsPath;
use types::{ByteString, Ucs2String};

use super::enums::VfsPathType;

/// Retrieves the directory name of the path.
fn get_directory_name<'a>(path: &'a str, separator: &'a str) -> &'a str {
    let directory_name: &str = match path.rsplit_once(separator) {
        Some(path_components) => path_components.0,
        None => "",
    };
    if directory_name == "" {
        separator
    } else {
        directory_name
    }
}

/// Retrieves the filename of the path.
fn get_filename<'a>(path: &'a str, separator: &'a str) -> &'a str {
    let filename: &str = match path.rsplit_once(separator) {
        Some(path_components) => path_components.1,
        None => "",
    };
    if filename == "" {
        separator
    } else {
        filename
    }
}

/// Virtual File System (VFS) path.
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum VfsPath {
    Apm {
        location: String,
        parent: Arc<VfsPath>,
    },
    Ext {
        ext_path: ExtPath,
        parent: Arc<VfsPath>,
    },
    Fake {
        location: String,
    },
    Gpt {
        location: String,
        parent: Arc<VfsPath>,
    },
    Mbr {
        location: String,
        parent: Arc<VfsPath>,
    },
    Ntfs {
        ntfs_path: NtfsPath,
        parent: Arc<VfsPath>,
    },
    Os {
        location: String,
    },
    Qcow {
        location: String,
        parent: Arc<VfsPath>,
    },
    Vhd {
        location: String,
        parent: Arc<VfsPath>,
    },
    Vhdx {
        location: String,
        parent: Arc<VfsPath>,
    },
}

impl VfsPath {
    /// Creates a new child path.
    pub fn new_child(&self, path_type: VfsPathType, location: &str) -> VfsPath {
        let parent: Arc<VfsPath> = Arc::new(self.clone());
        match &path_type {
            VfsPathType::Apm => VfsPath::Apm {
                location: location.to_string(),
                parent: parent,
            },
            VfsPathType::Ext => VfsPath::Ext {
                ext_path: ExtPath::from(location),
                parent: parent,
            },
            VfsPathType::Fake => {
                panic!("Unsupported path_type: VfsPathType::Fake")
            }
            VfsPathType::Gpt => VfsPath::Gpt {
                location: location.to_string(),
                parent: parent,
            },
            VfsPathType::Mbr => VfsPath::Mbr {
                location: location.to_string(),
                parent: parent,
            },
            VfsPathType::Ntfs => VfsPath::Ntfs {
                ntfs_path: NtfsPath::from(location),
                parent: parent,
            },
            VfsPathType::Os => panic!("Unsupported path_type: VfsPathType::Os"),
            VfsPathType::Qcow => VfsPath::Qcow {
                location: location.to_string(),
                parent: parent,
            },
            VfsPathType::Vhd => VfsPath::Vhd {
                location: location.to_string(),
                parent: parent,
            },
            VfsPathType::Vhdx => VfsPath::Vhdx {
                location: location.to_string(),
                parent: parent,
            },
        }
    }

    /// Creates a new path from the location with the same parent.
    pub fn new_with_parent(&self, location: &str) -> VfsPath {
        match self {
            VfsPath::Apm { parent, .. } => VfsPath::Apm {
                location: location.to_string(),
                parent: parent.clone(),
            },
            VfsPath::Ext { parent, .. } => VfsPath::Ext {
                ext_path: ExtPath::from(location),
                parent: parent.clone(),
            },
            VfsPath::Fake { .. } => VfsPath::Fake {
                location: location.to_string(),
            },
            VfsPath::Gpt { parent, .. } => VfsPath::Gpt {
                location: location.to_string(),
                parent: parent.clone(),
            },
            VfsPath::Mbr { parent, .. } => VfsPath::Mbr {
                location: location.to_string(),
                parent: parent.clone(),
            },
            VfsPath::Ntfs { parent, .. } => VfsPath::Ntfs {
                ntfs_path: NtfsPath::from(location),
                parent: parent.clone(),
            },
            VfsPath::Os { .. } => VfsPath::Os {
                location: location.to_string(),
            },
            VfsPath::Qcow { parent, .. } => VfsPath::Qcow {
                location: location.to_string(),
                parent: parent.clone(),
            },
            VfsPath::Vhd { parent, .. } => VfsPath::Vhd {
                location: location.to_string(),
                parent: parent.clone(),
            },
            VfsPath::Vhdx { parent, .. } => VfsPath::Vhdx {
                location: location.to_string(),
                parent: parent.clone(),
            },
        }
    }

    /// Creates a new path of the path and additional path components.
    pub fn append_components<'a>(&'a self, path_components: &mut Vec<&'a str>) -> VfsPath {
        match self {
            VfsPath::Apm { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Apm {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Ext { ext_path, parent } => {
                let mut location_components: Vec<ByteString> = ext_path.components.clone();
                location_components.append(
                    &mut path_components
                        .iter()
                        .map(|component| ByteString::from_string(component))
                        .collect::<Vec<ByteString>>(),
                );
                VfsPath::Ext {
                    ext_path: ExtPath::from(&location_components),
                    parent: parent.clone(),
                }
            }
            VfsPath::Fake { location } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Fake {
                    location: location_components.join("/"),
                }
            }
            VfsPath::Gpt { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Gpt {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Mbr { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Mbr {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Ntfs { ntfs_path, parent } => {
                let mut location_components: Vec<Ucs2String> = ntfs_path.components.clone();
                location_components.append(
                    &mut path_components
                        .iter()
                        .map(|component| Ucs2String::from_string(component))
                        .collect::<Vec<Ucs2String>>(),
                );
                VfsPath::Ntfs {
                    ntfs_path: NtfsPath::from(&location_components),
                    parent: parent.clone(),
                }
            }
            VfsPath::Os { location } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Os {
                    location: location_components.join(MAIN_SEPARATOR_STR),
                }
            }
            VfsPath::Qcow { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Qcow {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Vhd { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Vhd {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Vhdx { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Vhdx {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
        }
    }

    /// Retrieves the location.
    pub fn get_location(&self) -> String {
        match self {
            VfsPath::Apm { location, .. } => location.clone(),
            VfsPath::Ext { ext_path, .. } => ext_path.to_string(),
            VfsPath::Fake { location } => location.clone(),
            VfsPath::Gpt { location, .. } => location.clone(),
            VfsPath::Mbr { location, .. } => location.clone(),
            VfsPath::Ntfs { ntfs_path, .. } => ntfs_path.to_string(),
            VfsPath::Os { location } => location.clone(),
            VfsPath::Qcow { location, .. } => location.clone(),
            VfsPath::Vhd { location, .. } => location.clone(),
            VfsPath::Vhdx { location, .. } => location.clone(),
        }
    }

    /// Retrieves the parent path.
    pub fn get_parent(&self) -> Option<&VfsPath> {
        match self {
            VfsPath::Apm { parent, .. } => Some(parent.as_ref()),
            VfsPath::Ext { parent, .. } => Some(parent.as_ref()),
            VfsPath::Fake { .. } => None,
            VfsPath::Gpt { parent, .. } => Some(parent.as_ref()),
            VfsPath::Mbr { parent, .. } => Some(parent.as_ref()),
            VfsPath::Ntfs { parent, .. } => Some(parent.as_ref()),
            VfsPath::Os { .. } => None,
            VfsPath::Qcow { parent, .. } => Some(parent.as_ref()),
            VfsPath::Vhd { parent, .. } => Some(parent.as_ref()),
            VfsPath::Vhdx { parent, .. } => Some(parent.as_ref()),
        }
    }

    /// Retrieves the path type.
    pub fn get_path_type(&self) -> VfsPathType {
        match self {
            VfsPath::Apm { .. } => VfsPathType::Apm,
            VfsPath::Ext { .. } => VfsPathType::Ext,
            VfsPath::Fake { .. } => VfsPathType::Fake,
            VfsPath::Gpt { .. } => VfsPathType::Gpt,
            VfsPath::Mbr { .. } => VfsPathType::Mbr,
            VfsPath::Ntfs { .. } => VfsPathType::Ntfs,
            VfsPath::Os { .. } => VfsPathType::Os,
            VfsPath::Qcow { .. } => VfsPathType::Qcow,
            VfsPath::Vhd { .. } => VfsPathType::Vhd,
            VfsPath::Vhdx { .. } => VfsPathType::Vhdx,
        }
    }

    /// Retrieves the filename.
    pub fn get_file_name(&self) -> &str {
        match self {
            VfsPath::Apm { location, parent } => get_filename(location.as_str(), "/"),
            VfsPath::Ext { ext_path, parent } => todo!(),
            VfsPath::Fake { location } => get_filename(location.as_str(), "/"),
            VfsPath::Gpt { location, parent } => get_filename(location.as_str(), "/"),
            VfsPath::Mbr { location, parent } => get_filename(location.as_str(), "/"),
            VfsPath::Ntfs { ntfs_path, parent } => todo!(),
            VfsPath::Os { location } => get_filename(location.as_str(), MAIN_SEPARATOR_STR),
            VfsPath::Qcow { location, parent } => get_filename(location.as_str(), "/"),
            VfsPath::Vhd { location, parent } => get_filename(location.as_str(), "/"),
            VfsPath::Vhdx { location, parent } => get_filename(location.as_str(), "/"),
        }
    }

    /// Creates a new path of the parent directory.
    pub fn parent_directory(&self) -> VfsPath {
        match self {
            VfsPath::Apm { location, parent } => VfsPath::Apm {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Ext { ext_path, parent } => VfsPath::Ext {
                ext_path: ext_path.parent_directory(),
                parent: parent.clone(),
            },
            VfsPath::Fake { location } => VfsPath::Fake {
                location: get_directory_name(location.as_str(), "/").to_string(),
            },
            VfsPath::Gpt { location, parent } => VfsPath::Gpt {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Mbr { location, parent } => VfsPath::Mbr {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Ntfs { ntfs_path, parent } => VfsPath::Ntfs {
                ntfs_path: ntfs_path.parent_directory(),
                parent: parent.clone(),
            },
            VfsPath::Os { location } => VfsPath::Os {
                location: get_directory_name(location.as_str(), MAIN_SEPARATOR_STR).to_string(),
            },
            VfsPath::Qcow { location, parent } => VfsPath::Qcow {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Vhd { location, parent } => VfsPath::Vhd {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Vhdx { location, parent } => VfsPath::Vhdx {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
        }
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        match self {
            VfsPath::Apm { location, parent } => format!(
                "{}\ntype: APM: location: {}\n",
                parent.to_string(),
                location
            ),
            VfsPath::Ext { ext_path, parent } => format!(
                "{}\ntype: EXT: location: {}\n",
                parent.to_string(),
                ext_path.to_string()
            ),
            VfsPath::Fake { location } => format!("type: FAKE: location: {}\n", location),
            VfsPath::Gpt { location, parent } => format!(
                "{}\ntype: GPT: location: {}\n",
                parent.to_string(),
                location
            ),
            VfsPath::Mbr { location, parent } => format!(
                "{}\ntype: MBR: location: {}\n",
                parent.to_string(),
                location
            ),
            VfsPath::Ntfs { ntfs_path, parent } => format!(
                "{}\ntype: NTFS: location: {}\n",
                parent.to_string(),
                ntfs_path.to_string()
            ),
            VfsPath::Os { location } => format!("type: OS: location: {}\n", location),
            VfsPath::Qcow { location, parent } => format!(
                "{}\ntype: QCOW: location: {}\n",
                parent.to_string(),
                location
            ),
            VfsPath::Vhd { location, parent } => format!(
                "{}\ntype: VHD: location: {}\n",
                parent.to_string(),
                location
            ),
            VfsPath::Vhdx { location, parent } => format!(
                "{}\ntype: VHDX: location: {}\n",
                parent.to_string(),
                location
            ),
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
    }

    #[test]
    fn test_get_filename() {
        assert_eq!(get_filename("/", "/"), "/");
        assert_eq!(get_filename("/gpt1", "/"), "gpt1");
    }

    #[test]
    fn test_new_child() {
        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let test_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");

        assert!(test_path.get_path_type() == VfsPathType::Qcow);
        assert_eq!(test_path.get_location(), "/");
    }

    // TODO: test with new_child panicking

    #[test]
    fn test_new_with_parent() {
        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/file.txt".to_string(),
        };

        let test_path: VfsPath = vfs_path.new_with_parent("../test_data/bogus.txt");

        assert_eq!(test_path.get_location(), "../test_data/bogus.txt");

        let vfs_path_type: VfsPathType = test_path.get_path_type();
        assert!(vfs_path_type == VfsPathType::Os);

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let vfs_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");

        let test_path: VfsPath = vfs_path.new_with_parent("/qcow1");

        assert_eq!(test_path.get_location(), "/qcow1");

        let vfs_path_type: VfsPathType = test_path.get_path_type();
        assert!(vfs_path_type == VfsPathType::Qcow);
    }

    #[test]
    fn test_get_parent() {
        let test_path: VfsPath = VfsPath::Os {
            location: "../test_data/file.txt".to_string(),
        };

        let parent: Option<&VfsPath> = test_path.get_parent();
        assert!(parent.is_none());

        let os_vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/qcow/ext2.qcow2".to_string(),
        };
        let test_path: VfsPath = os_vfs_path.new_child(VfsPathType::Qcow, "/");

        let parent: Option<&VfsPath> = test_path.get_parent();
        assert!(parent.is_some());
    }
}
