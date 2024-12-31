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

use std::path::MAIN_SEPARATOR_STR;
use std::rc::Rc;

use crate::formats::ext::ExtPath;

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

/// Virtual File System (VFS) path.
pub enum VfsPath {
    Apm {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Ext {
        /// Location.
        location: ExtPath,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Fake {
        /// Location.
        location: String,
    },
    Gpt {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Mbr {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Os {
        /// Location.
        location: String,
    },
    Qcow {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Vhd {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
    Vhdx {
        /// Location.
        location: String,

        /// Parent.
        parent: Rc<VfsPath>,
    },
}

impl VfsPath {
    /// Creates a new path.
    pub fn new(path_type: VfsPathType, location: &str, parent: Option<&Rc<VfsPath>>) -> VfsPath {
        match &path_type {
            VfsPathType::Apm => VfsPath::Apm {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Ext => VfsPath::Ext {
                location: ExtPath::from(location),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Fake => VfsPath::Fake {
                location: location.to_string(),
            },
            VfsPathType::Gpt => VfsPath::Gpt {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Mbr => VfsPath::Mbr {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Os => VfsPath::Os {
                location: location.to_string(),
            },
            VfsPathType::Qcow => VfsPath::Qcow {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Vhd => VfsPath::Vhd {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPathType::Vhdx => VfsPath::Vhdx {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
        }
    }

    /// Creates a new path from the parent directory of the current location.
    pub fn new_of_parent_directory(&self) -> VfsPath {
        match self {
            VfsPath::Apm { location, parent } => VfsPath::Apm {
                location: get_directory_name(location.as_str(), "/").to_string(),
                parent: parent.clone(),
            },
            VfsPath::Ext { .. } => todo!(),
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

    /// Creates a new path from the location with the same parent.
    pub fn new_with_parent(&self, location: &str) -> VfsPath {
        let parent: Option<&Rc<VfsPath>> = self.get_parent();
        match self {
            VfsPath::Apm { .. } => VfsPath::Apm {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Ext { .. } => VfsPath::Ext {
                location: ExtPath::from(location),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Fake { .. } => VfsPath::Fake {
                location: location.to_string(),
            },
            VfsPath::Gpt { .. } => VfsPath::Gpt {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Mbr { .. } => VfsPath::Mbr {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Os { .. } => VfsPath::Os {
                location: location.to_string(),
            },
            VfsPath::Qcow { .. } => VfsPath::Qcow {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Vhd { .. } => VfsPath::Vhd {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
            VfsPath::Vhdx { .. } => VfsPath::Vhdx {
                location: location.to_string(),
                parent: parent.cloned().unwrap(),
            },
        }
    }

    /// Creates a new path from the current location and additional path components.
    pub fn new_with_suffix<'a>(&'a self, path_components: &mut Vec<&'a str>) -> VfsPath {
        match self {
            VfsPath::Apm { location, parent } => {
                let mut location_components: Vec<&str> = vec![location.as_str()];
                location_components.append(path_components);
                VfsPath::Apm {
                    location: location_components.join("/"),
                    parent: parent.clone(),
                }
            }
            VfsPath::Ext { .. } => todo!(),
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
    pub fn get_location(&self) -> &str {
        match self {
            VfsPath::Apm { location, .. } => location.as_str(),
            VfsPath::Ext { .. } => todo!(),
            VfsPath::Fake { location } => location.as_str(),
            VfsPath::Gpt { location, .. } => location.as_str(),
            VfsPath::Mbr { location, .. } => location.as_str(),
            VfsPath::Os { location } => location.as_str(),
            VfsPath::Qcow { location, .. } => location.as_str(),
            VfsPath::Vhd { location, .. } => location.as_str(),
            VfsPath::Vhdx { location, .. } => location.as_str(),
        }
    }

    /// Retrieves the parent path.
    pub fn get_parent(&self) -> Option<&Rc<VfsPath>> {
        match self {
            VfsPath::Apm { parent, .. } => Some(&parent),
            VfsPath::Ext { parent, .. } => Some(&parent),
            VfsPath::Fake { .. } => None,
            VfsPath::Gpt { parent, .. } => Some(&parent),
            VfsPath::Mbr { parent, .. } => Some(&parent),
            VfsPath::Os { .. } => None,
            VfsPath::Qcow { parent, .. } => Some(&parent),
            VfsPath::Vhd { parent, .. } => Some(&parent),
            VfsPath::Vhdx { parent, .. } => Some(&parent),
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
            VfsPath::Os { .. } => VfsPathType::Os,
            VfsPath::Qcow { .. } => VfsPathType::Qcow,
            VfsPath::Vhd { .. } => VfsPathType::Vhd,
            VfsPath::Vhdx { .. } => VfsPathType::Vhdx,
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
            VfsPath::Ext { location, parent } => format!(
                "{}\ntype: EXT: location: {}\n",
                parent.to_string(),
                location.to_string()
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
    fn test_new() {
        let test_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        assert!(test_path.get_path_type() == VfsPathType::Os);
        assert_eq!(test_path.get_location(), "./test_data/file.txt");

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let test_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));

        assert!(test_path.get_path_type() == VfsPathType::Qcow);
        assert_eq!(test_path.get_location(), "/");
    }

    #[test]
    fn test_new_with_parent() {
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        let test_path: VfsPath = vfs_path.new_with_parent("./test_data/bogus.txt");

        assert_eq!(test_path.get_location(), "./test_data/bogus.txt");

        let vfs_path_type: VfsPathType = test_path.get_path_type();
        assert!(vfs_path_type == VfsPathType::Os);

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let vfs_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));

        let test_path: VfsPath = vfs_path.new_with_parent("/qcow1");

        assert_eq!(test_path.get_location(), "/qcow1");

        let vfs_path_type: VfsPathType = test_path.get_path_type();
        assert!(vfs_path_type == VfsPathType::Qcow);
    }

    #[test]
    fn test_get_parent() {
        let test_path: VfsPath = VfsPath::new(VfsPathType::Os, "./test_data/file.txt", None);

        let parent: Option<&Rc<VfsPath>> = test_path.get_parent();
        assert!(parent.is_none());

        let os_vfs_path: Rc<VfsPath> = Rc::new(VfsPath::new(
            VfsPathType::Os,
            "./test_data/qcow/ext2.qcow2",
            None,
        ));
        let test_path: VfsPath = VfsPath::new(VfsPathType::Qcow, "/", Some(&os_vfs_path));

        let parent: Option<&Rc<VfsPath>> = test_path.get_parent();
        assert!(parent.is_some());
    }
}
