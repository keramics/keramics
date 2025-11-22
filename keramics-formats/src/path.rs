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

use std::fmt;
use std::path::{Component, MAIN_SEPARATOR_STR, PathBuf};

use keramics_core::ErrorTrace;
use keramics_types::ByteString;

use super::path_component::PathComponent;

/// Generic path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Path {
    /// Components.
    pub components: Vec<PathComponent>,
}

impl Path {
    /// Creates a new path of the current path joined with the path.
    pub fn new_with_join(&self, path: &Path) -> Self {
        self.new_with_join_path_components(&path.components)
    }

    /// Creates a new path of the current path joined with the path components.
    pub fn new_with_join_path_components(&self, path_components: &[PathComponent]) -> Self {
        let mut components: Vec<PathComponent> = self.components.clone();

        for path_component in path_components.iter() {
            if path_component == ".." {
                if components.is_empty() {
                    components.push(path_component.clone());
                } else {
                    components.pop();
                }
            } else if path_component != "." {
                components.push(path_component.clone());
            }
        }
        Self { components }
    }

    /// Creates a new path of the parent directory.
    pub fn new_with_parent_directory(&self) -> Self {
        let mut number_of_components: usize = self.components.len();

        if number_of_components > 1 {
            number_of_components -= 1;
        }
        let parent_components: Vec<PathComponent> =
            self.components[0..number_of_components].to_vec();

        Self {
            components: parent_components,
        }
    }

    /// Retrieves the file name.
    pub fn file_name(&self) -> Option<&PathComponent> {
        let number_of_components: usize = self.components.len();

        if number_of_components > 1 {
            Some(&self.components[number_of_components - 1])
        } else {
            None
        }
    }

    /// Retrieves the number of components.
    pub fn get_number_of_components(&self) -> usize {
        self.components.len()
    }

    /// Retrieves a specific component.
    pub fn get_component_by_index(&self, component_index: usize) -> Option<&PathComponent> {
        self.components.get(component_index)
    }

    /// Determines if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Determines if the path is relative.
    pub fn is_relative(&self) -> bool {
        self.components.len() >= 1 && !self.components[0].is_empty()
    }

    /// Determines if the path represents the root.
    pub fn is_root(&self) -> bool {
        self.components.len() == 1 && self.components[0].is_empty()
    }

    /// Appends a component to the path.
    pub fn push(&mut self, component: PathComponent) {
        self.components.push(component);
    }

    /// Converts the path to a `PathBuf`.
    pub fn to_path_buf(&self) -> Result<PathBuf, ErrorTrace> {
        let mut path_buf: PathBuf = PathBuf::new();

        for path_component in self.components.iter() {
            match path_component {
                PathComponent::ByteString(byte_string) => {
                    let string: String = byte_string.to_string();

                    path_buf.push(string);
                }
                PathComponent::OsString(os_string) => path_buf.push(os_string),
                PathComponent::String(string) => path_buf.push(string),
                PathComponent::Root => path_buf.push(MAIN_SEPARATOR_STR),
                PathComponent::Ucs2String(ucs2_string) => {
                    let string: String = ucs2_string.to_string();

                    path_buf.push(string);
                }
            }
        }
        Ok(path_buf)
    }
}

impl From<&ByteString> for Path {
    /// Converts a [`&ByteString`] into a [`Path`]
    fn from(byte_string: &ByteString) -> Self {
        let components: Vec<PathComponent> = if byte_string.is_empty() {
            // Splitting "" results in [""]
            vec![]
        } else if byte_string == "/" {
            // Splitting "/" results in ["", ""]
            vec![PathComponent::Root]
        } else {
            let mut components: Vec<PathComponent> = Vec::new();

            for byte_string_segment in byte_string.elements.split(|value| *value == 0x2f) {
                if byte_string_segment.is_empty() {
                    if components.is_empty() {
                        components.push(PathComponent::Root);
                    }
                } else if byte_string_segment == [0x2e, 0x2e] && !components.is_empty() {
                    components.pop();
                } else if byte_string_segment != [0x02e] {
                    let path_component: PathComponent =
                        PathComponent::ByteString(ByteString::from(byte_string_segment));

                    components.push(path_component);
                }
            }
            components
        };
        Self { components }
    }
}

impl From<&PathBuf> for Path {
    /// Converts a [`&PathBuf`] into a [`Path`]
    #[inline(always)]
    fn from(path_buf: &PathBuf) -> Self {
        let mut components: Vec<PathComponent> = Vec::new();

        for component in path_buf.components() {
            match component {
                Component::CurDir | Component::Prefix(_) => {}
                Component::Normal(os_str) => {
                    if !os_str.is_empty() {
                        components.push(PathComponent::from(os_str));
                    }
                }
                Component::ParentDir => {
                    if components.is_empty() {
                        components.push(PathComponent::from(".."));
                    } else {
                        components.pop();
                    }
                }
                Component::RootDir => {
                    components.push(PathComponent::Root);
                }
            }
        }
        Self { components }
    }
}

impl From<&str> for Path {
    /// Converts a [`&str`] into a [`Path`]
    fn from(string: &str) -> Self {
        let components: Vec<PathComponent> = if string.is_empty() {
            // Splitting "" results in [""]
            vec![]
        } else if string == "/" {
            // Splitting "/" results in ["", ""]
            vec![PathComponent::Root]
        } else {
            let mut components: Vec<PathComponent> = Vec::new();

            for string_segment in string.split("/") {
                if string_segment.is_empty() {
                    if components.is_empty() {
                        components.push(PathComponent::Root);
                    }
                } else if string_segment == ".." && !components.is_empty() {
                    components.pop();
                } else if string_segment != "." {
                    let path_component: PathComponent = PathComponent::from(string_segment);

                    components.push(path_component);
                }
            }
            components
        };
        Self { components }
    }
}

impl From<&String> for Path {
    /// Converts a [`&String`] into a [`Path`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&[&str]> for Path {
    /// Converts a [`&[&str]`] into a [`Path`]
    #[inline]
    fn from(strings: &[&str]) -> Self {
        let mut components: Vec<PathComponent> = Vec::new();

        for string in strings {
            if string.is_empty() {
                if components.is_empty() {
                    components.push(PathComponent::Root);
                }
            } else if *string == ".." && !components.is_empty() {
                components.pop();
            } else if *string != "." {
                let path_component: PathComponent = PathComponent::from(*string);

                components.push(path_component);
            }
        }
        Self { components }
    }
}

impl From<&[String]> for Path {
    /// Converts a [`&[String]`] into a [`Path`]
    #[inline]
    fn from(strings: &[String]) -> Self {
        let mut components: Vec<PathComponent> = Vec::new();

        for string in strings {
            if string.is_empty() {
                if components.is_empty() {
                    components.push(PathComponent::Root);
                }
            } else if string == ".." && !components.is_empty() {
                components.pop();
            } else if string != "." {
                let path_component: PathComponent = PathComponent::from(string);

                components.push(path_component);
            }
        }
        Self { components }
    }
}

impl From<&[PathComponent]> for Path {
    /// Converts a [`Vec<PathComponent>`] into a [`Path`]
    #[inline]
    fn from(path_components: &[PathComponent]) -> Self {
        Self::from(path_components.to_vec())
    }
}

impl From<Vec<PathComponent>> for Path {
    /// Converts a [`Vec<PathComponent>`] into a [`Path`]
    #[inline]
    fn from(path_components: Vec<PathComponent>) -> Self {
        Self {
            components: path_components,
        }
    }
}

impl fmt::Display for Path {
    /// Formats the path for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.is_root() {
            write!(formatter, "/")
        } else {
            let path_string: String = self
                .components
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join("/");

            write!(formatter, "{}", path_string)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;

    use keramics_types::Ucs2String;

    #[test]
    fn test_new_with_join() {
        let string_path: Path = Path::from("/");
        let additional_string_path: Path = Path::from("directory");

        let test_struct: Path = string_path.new_with_join(&additional_string_path);
        assert_eq!(test_struct.to_string(), "/directory");

        let string_path: Path = Path::from("/directory");
        let additional_string_path: Path = Path::from("filename.txt");

        let test_struct: Path = string_path.new_with_join(&additional_string_path);
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let string_path: Path = Path::from("/directory");
        let additional_string_path: Path = Path::from("./filename.txt");

        let test_struct: Path = string_path.new_with_join(&additional_string_path);
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let string_path: Path = Path::from("/directory");
        let additional_string_path: Path = Path::from("../filename.txt");

        let test_struct: Path = string_path.new_with_join(&additional_string_path);
        assert_eq!(test_struct.to_string(), "/filename.txt");

        let string_path: Path = Path::from("../directory");
        let additional_string_path: Path = Path::from("../filename.txt");

        let test_struct: Path = string_path.new_with_join(&additional_string_path);
        assert_eq!(test_struct.to_string(), "../filename.txt");
    }

    #[test]
    fn test_new_with_join_path_components() {
        let string_path: Path = Path::from("/");
        let additional_path_components: [PathComponent; 1] = [PathComponent::from("directory")];

        let test_struct: Path =
            string_path.new_with_join_path_components(&additional_path_components);
        assert_eq!(test_struct.to_string(), "/directory");

        let string_path: Path = Path::from("/directory");
        let additional_path_components: [PathComponent; 1] = [PathComponent::from("filename.txt")];

        let test_struct: Path =
            string_path.new_with_join_path_components(&additional_path_components);
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let string_path: Path = Path::from("/directory");
        let additional_path_components: [PathComponent; 2] = [
            PathComponent::from("."),
            PathComponent::from("filename.txt"),
        ];

        let test_struct: Path =
            string_path.new_with_join_path_components(&additional_path_components);
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let string_path: Path = Path::from("/directory");
        let additional_path_components: [PathComponent; 2] = [
            PathComponent::from(".."),
            PathComponent::from("filename.txt"),
        ];

        let test_struct: Path =
            string_path.new_with_join_path_components(&additional_path_components);
        assert_eq!(test_struct.to_string(), "/filename.txt");

        let string_path: Path = Path::from("../directory");
        let additional_path_components: [PathComponent; 2] = [
            PathComponent::from(".."),
            PathComponent::from("filename.txt"),
        ];

        let test_struct: Path =
            string_path.new_with_join_path_components(&additional_path_components);
        assert_eq!(test_struct.to_string(), "../filename.txt");
    }

    #[test]
    fn test_new_with_parent_directory() {
        let string_path: Path = Path::from("/directory/filename.txt");

        let test_struct: Path = string_path.new_with_parent_directory();
        assert_eq!(test_struct.to_string(), "/directory");
    }

    #[test]
    fn test_file_name() {
        let test_struct: Path = Path::from("/");
        let result: Option<&PathComponent> = test_struct.file_name();
        assert_eq!(result, None);

        let test_struct: Path = Path::from("/directory");
        let result: Option<&PathComponent> = test_struct.file_name();
        assert_eq!(result, Some(&PathComponent::from("directory")));

        let test_struct: Path = Path::from("/directory/filename.txt");
        let result: Option<&PathComponent> = test_struct.file_name();
        assert_eq!(result, Some(&PathComponent::from("filename.txt")));

        let test_struct: Path = Path::from("/directory/");
        let result: Option<&PathComponent> = test_struct.file_name();
        assert_eq!(result, Some(&PathComponent::from("directory")));
    }

    #[test]
    fn test_get_number_of_components() {
        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.get_number_of_components(), 3);

        let test_struct: Path = Path::from("");
        assert_eq!(test_struct.get_number_of_components(), 0);
    }

    #[test]
    fn test_get_component_by_index() {
        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(
            test_struct.get_component_by_index(1),
            Some(&PathComponent::from("directory"))
        );

        let test_struct: Path = Path::from("");
        assert_eq!(test_struct.get_component_by_index(1), None);
    }

    #[test]
    fn test_is_empty() {
        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.is_empty(), false);

        let test_struct: Path = Path::from("");
        assert_eq!(test_struct.is_empty(), true);
    }

    #[test]
    fn test_is_relative() {
        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.is_relative(), false);

        let test_struct: Path = Path::from("../filename.txt");
        assert_eq!(test_struct.is_relative(), true);
    }

    #[test]
    fn test_is_root() {
        let test_struct: Path = Path::from("/");
        assert_eq!(test_struct.is_root(), true);

        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.is_root(), false);

        let test_struct: Path = Path::from("");
        assert_eq!(test_struct.is_root(), false);
    }

    #[test]
    fn test_to_path_buf() -> Result<(), ErrorTrace> {
        let path: Path = Path {
            components: vec![
                PathComponent::Root,
                PathComponent::ByteString(ByteString::from("directory")),
                PathComponent::ByteString(ByteString::from("filename.txt")),
            ],
        };

        let path_buf: PathBuf = path.to_path_buf()?;
        assert_eq!(path_buf, PathBuf::from("/directory/filename.txt"));

        let path: Path = Path::from("/");

        let path_buf: PathBuf = path.to_path_buf()?;
        assert_eq!(path_buf, PathBuf::from("/"));

        let path: Path = Path {
            components: vec![
                PathComponent::Root,
                PathComponent::OsString(OsString::from("directory")),
                PathComponent::OsString(OsString::from("filename.txt")),
            ],
        };

        let path_buf: PathBuf = path.to_path_buf()?;
        assert_eq!(path_buf, PathBuf::from("/directory/filename.txt"));

        let path: Path = Path::from("/directory/filename.txt");

        let path_buf: PathBuf = path.to_path_buf()?;
        assert_eq!(path_buf, PathBuf::from("/directory/filename.txt"));

        let path: Path = Path {
            components: vec![
                PathComponent::Root,
                PathComponent::Ucs2String(Ucs2String::from("directory")),
                PathComponent::Ucs2String(Ucs2String::from("filename.txt")),
            ],
        };

        let path_buf: PathBuf = path.to_path_buf()?;
        assert_eq!(path_buf, PathBuf::from("/directory/filename.txt"));

        Ok(())
    }

    #[test]
    fn test_from_path_buf() {
        let path_buf: PathBuf = PathBuf::from("/");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(test_struct.components, vec![PathComponent::Root,]);

        let path_buf: PathBuf = PathBuf::from("/directory");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::OsString(OsString::from("directory")),
                ]
            }
        );

        let path_buf: PathBuf = PathBuf::from("/directory/filename.txt");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::OsString(OsString::from("directory")),
                    PathComponent::OsString(OsString::from("filename.txt")),
                ]
            }
        );

        let path_buf: PathBuf = PathBuf::from("/directory/./filename.txt");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::OsString(OsString::from("directory")),
                    PathComponent::OsString(OsString::from("filename.txt")),
                ]
            }
        );

        let path_buf: PathBuf = PathBuf::from("/directory/");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::OsString(OsString::from("directory")),
                ]
            }
        );

        let path_buf: PathBuf = PathBuf::from("./directory");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![PathComponent::OsString(OsString::from("directory"))]
            }
        );

        let path_buf: PathBuf = PathBuf::from("../directory");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::String(String::from("..")),
                    PathComponent::OsString(OsString::from("directory")),
                ]
            }
        );

        let path_buf: PathBuf = PathBuf::from("../directory/../filename.txt");
        let test_struct: Path = Path::from(&path_buf);
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::String(String::from("..")),
                    PathComponent::OsString(OsString::from("filename.txt")),
                ]
            }
        );
    }

    #[test]
    fn test_from_str() {
        let test_struct: Path = Path::from("/");
        assert_eq!(test_struct.components, vec![PathComponent::Root,]);

        let test_struct: Path = Path::from("/directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::String(String::from("directory")),
                ]
            }
        );

        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::String(String::from("directory")),
                    PathComponent::String(String::from("filename.txt")),
                ]
            }
        );

        let test_struct: Path = Path::from("/directory/./filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::String(String::from("directory")),
                    PathComponent::String(String::from("filename.txt")),
                ]
            }
        );

        let test_struct: Path = Path::from("/directory/");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::String(String::from("directory")),
                ]
            }
        );

        let test_struct: Path = Path::from("./directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![PathComponent::String(String::from("directory")),]
            }
        );

        let test_struct: Path = Path::from("../directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::String(String::from("..")),
                    PathComponent::String(String::from("directory")),
                ]
            }
        );

        let test_struct: Path = Path::from("../directory/../filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::String(String::from("..")),
                    PathComponent::String(String::from("filename.txt")),
                ]
            }
        );
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("/");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = String::from("/directory");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = String::from("/directory/filename.txt");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = String::from("/directory/./filename.txt");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = String::from("/directory/");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = String::from("./directory");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = String::from("../directory");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = String::from("../directory/../filename.txt");
        let test_struct: Path = Path::from(&string);
        assert_eq!(test_struct.components.len(), 2);
    }

    #[test]
    fn test_from_str_slice() {
        let str_array: [&str; 1] = [""];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let str_array: [&str; 2] = ["", "directory"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 3] = ["", "directory", "filename.txt"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let str_array: [&str; 4] = ["", "directory", ".", "filename.txt"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let str_array: [&str; 3] = ["", "directory", ""];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 2] = [".", "directory"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let str_array: [&str; 2] = ["..", "directory"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 4] = ["..", "directory", "..", "filename.txt"];
        let test_struct: Path = Path::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);
    }

    #[test]
    fn test_from_string_slice() {
        let string_array: [String; 1] = [String::from("")];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let string_array: [String; 2] = [String::from(""), String::from("directory")];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let string_array: [String; 4] = [
            String::from(""),
            String::from("directory"),
            String::from("."),
            String::from("filename.txt"),
        ];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 2] = [String::from("."), String::from("directory")];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let string_array: [String; 2] = [String::from(".."), String::from("directory")];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 4] = [
            String::from(".."),
            String::from("directory"),
            String::from(".."),
            String::from("filename.txt"),
        ];
        let test_struct: Path = Path::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);
    }

    // TODO: add tests for from path components slice
    // TODO: add tests for from path components vector

    #[test]
    fn test_to_string() {
        let test_struct: Path = Path::from("/");
        assert_eq!(test_struct.to_string(), "/");

        let test_struct: Path = Path::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: Path = Path::from("/directory/./filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: Path = Path::from("/directory/");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: Path = Path::from("./directory");
        assert_eq!(test_struct.to_string(), "directory");

        let test_struct: Path = Path::from("../directory");
        assert_eq!(test_struct.to_string(), "../directory");

        let test_struct: Path = Path::from("../directory/../filename.txt");
        assert_eq!(test_struct.to_string(), "../filename.txt");
    }
}
