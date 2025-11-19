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
use std::path::PathBuf;

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
}

impl From<&ByteString> for Path {
    /// Converts a [`&ByteString`] into a [`Path`]
    fn from(byte_string: &ByteString) -> Self {
        let components: Vec<PathComponent> = if byte_string.is_empty() {
            // Splitting "" results in [""]
            vec![]
        } else if byte_string == "/" {
            // Splitting "/" results in ["", ""]
            vec![PathComponent::ByteString(ByteString::from(""))]
        } else {
            let mut components: Vec<PathComponent> = Vec::new();

            for path_component in byte_string.elements.split(|value| *value == 0x2f) {
                if path_component == [0x2e, 0x2e] {
                    if components.is_empty() {
                        components
                            .push(PathComponent::ByteString(ByteString::from(path_component)));
                    } else {
                        components.pop();
                    }
                } else if path_component != [0x02e] {
                    components.push(PathComponent::ByteString(ByteString::from(path_component)));
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
        let components: Vec<PathComponent> = path_buf
            .iter()
            .map(|component| PathComponent::OsString(component.to_os_string()))
            .collect();

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
            vec![PathComponent::from("")]
        } else {
            let mut components: Vec<PathComponent> = Vec::new();

            for path_component in string.split("/") {
                if path_component == ".." {
                    if components.is_empty() {
                        components.push(PathComponent::from(path_component));
                    } else {
                        components.pop();
                    }
                } else if path_component != "." {
                    components.push(PathComponent::from(path_component));
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
    fn from(path_components: &[&str]) -> Self {
        let mut components: Vec<PathComponent> = Vec::new();

        for path_component in path_components {
            if *path_component == ".." {
                if components.is_empty() {
                    components.push(PathComponent::from(*path_component));
                } else {
                    components.pop();
                }
            } else if *path_component != "." {
                components.push(PathComponent::from(*path_component));
            }
        }
        Self { components }
    }
}

impl From<&[String]> for Path {
    /// Converts a [`&[String]`] into a [`Path`]
    #[inline]
    fn from(path_components: &[String]) -> Self {
        let mut components: Vec<PathComponent> = Vec::new();

        for path_component in path_components {
            if path_component == ".." {
                if components.is_empty() {
                    components.push(PathComponent::from(path_component));
                } else {
                    components.pop();
                }
            } else if path_component != "." {
                components.push(PathComponent::from(path_component));
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
        let string: String = if self.is_root() {
            String::from("/")
        } else {
            let string_parts: Vec<String> = self
                .components
                .iter()
                .map(|component| component.to_string())
                .collect();

            string_parts.join("/")
        };
        write!(formatter, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_join() {
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
        assert_eq!(result, Some(&PathComponent::from("")));
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
    fn test_from_str() {
        let test_struct: Path = Path::from("/");
        assert_eq!(test_struct.components.len(), 1);

        let test_struct: Path = Path::from("/directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: Path = Path::from("/directory/./filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: Path = Path::from("/directory/");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: Path = Path::from("./directory");
        assert_eq!(test_struct.components.len(), 1);

        let test_struct: Path = Path::from("../directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: Path = Path::from("../directory/../filename.txt");
        assert_eq!(test_struct.components.len(), 2);
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
        assert_eq!(test_struct.components.len(), 3);

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
        assert_eq!(test_struct.components.len(), 3);

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
        assert_eq!(test_struct.components.len(), 3);

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

    #[test]
    fn test_to_string() {
        let test_struct: Path = Path::from("/");
        assert_eq!(test_struct.to_string(), "/");

        let test_struct: Path = Path::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: Path = Path::from("/directory/filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: Path = Path::from("/directory/");
        assert_eq!(test_struct.to_string(), "/directory/");
    }
}
