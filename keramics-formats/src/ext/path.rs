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

use keramics_types::ByteString;

/// Extended File System (ext) path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExtPath {
    /// Components.
    pub components: Vec<ByteString>,
}

impl ExtPath {
    const COMPONENT_SEPARATOR: &'static str = "/";

    /// Creates a new path of the current path and additional path components.
    pub fn new_with_join(&self, path_components: &[ByteString]) -> Self {
        let mut components: Vec<ByteString> = self.components.clone();
        components.extend_from_slice(path_components);

        Self {
            components: components,
        }
    }

    /// Creates a new path of the parent directory.
    pub fn new_with_parent_directory(&self) -> Self {
        let mut number_of_components: usize = self.components.len();
        if number_of_components > 1 {
            number_of_components -= 1;
        }
        let parent_components: Vec<ByteString> = self.components[0..number_of_components].to_vec();

        Self {
            components: parent_components,
        }
    }

    /// Retrieves the extension if available.
    pub fn extension(&self) -> Option<ByteString> {
        match self.file_name() {
            Some(byte_string) => match byte_string
                .elements
                .iter()
                .skip(1)
                .rev()
                .position(|value| *value == 0x2e)
            {
                Some(value_index) => {
                    // Note that value_index is relative to the end of the string.
                    let string_index: usize = byte_string.elements.len() - value_index;
                    Some(ByteString::from(&byte_string.elements[string_index..]))
                }
                None => None,
            },
            None => None,
        }
    }

    /// Retrieves the file stem if available.
    pub fn file_stem(&self) -> Option<ByteString> {
        match self.file_name() {
            Some(byte_string) => match byte_string
                .elements
                .iter()
                .skip(1)
                .rev()
                .position(|value| *value == 0x2e)
            {
                Some(value_index) => {
                    // Note that value_index is relative to the end of the string.
                    let string_size: usize = byte_string.elements.len() - value_index - 1;
                    Some(ByteString::from(&byte_string.elements[0..string_size]))
                }
                None => Some(byte_string.clone()),
            },
            None => None,
        }
    }

    /// Retrieves the file name.
    pub fn file_name(&self) -> Option<&ByteString> {
        let number_of_components: usize = self.components.len();
        if number_of_components > 1 {
            Some(&self.components[number_of_components - 1])
        } else {
            None
        }
    }

    /// Determines if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Appends a component to the path.
    pub fn push(&mut self, component: ByteString) {
        self.components.push(component);
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        let number_of_components: usize = self.components.len();
        if number_of_components == 1 && self.components[0].is_empty() {
            Self::COMPONENT_SEPARATOR.to_string()
        } else {
            self.components
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join(Self::COMPONENT_SEPARATOR)
        }
    }
}

impl From<&str> for ExtPath {
    /// Converts a [`&str`] into a [`ExtPath`]
    fn from(string: &str) -> Self {
        let components: Vec<ByteString> = if string.is_empty() {
            // Splitting "" results in [""]
            vec![]
        } else if string == Self::COMPONENT_SEPARATOR {
            // Splitting "/" results in ["", ""]
            vec![ByteString::new()]
        } else {
            string
                .split(Self::COMPONENT_SEPARATOR)
                .map(|component| ByteString::from(component))
                .collect::<Vec<ByteString>>()
        };
        Self {
            components: components,
        }
    }
}

impl From<&String> for ExtPath {
    /// Converts a [`&String`] into a [`ExtPath`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&[&str]> for ExtPath {
    /// Converts a [`&[&str]`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &[&str]) -> Self {
        let components: Vec<ByteString> = path_components
            .iter()
            .map(|component| ByteString::from(*component))
            .collect::<Vec<ByteString>>();

        Self {
            components: components,
        }
    }
}

impl From<&[ByteString]> for ExtPath {
    /// Converts a [`&[ByteString]`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &[ByteString]) -> Self {
        Self {
            components: path_components.to_vec(),
        }
    }
}

impl From<&[String]> for ExtPath {
    /// Converts a [`&[String]`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &[String]) -> Self {
        let components: Vec<ByteString> = path_components
            .iter()
            .map(|component| ByteString::from(component))
            .collect::<Vec<ByteString>>();

        Self {
            components: components,
        }
    }
}

impl From<&Vec<&str>> for ExtPath {
    /// Converts a [`&Vec<&str>`] into a [`ExtPath`]
    #[inline(always)]
    fn from(path_components: &Vec<&str>) -> Self {
        Self::from(path_components.as_slice())
    }
}

impl From<&Vec<ByteString>> for ExtPath {
    /// Converts a [`&Vec<ByteString>`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &Vec<ByteString>) -> Self {
        Self {
            components: path_components.clone(),
        }
    }
}

impl From<&Vec<String>> for ExtPath {
    /// Converts a [`&Vec<String>`] into a [`ExtPath`]
    #[inline(always)]
    fn from(path_components: &Vec<String>) -> Self {
        Self::from(path_components.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_join() {
        let ext_path: ExtPath = ExtPath::from("/directory");

        let test_struct: ExtPath = ext_path.new_with_join(&[ByteString::from("filename.txt")]);
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");
    }

    #[test]
    fn test_new_with_parent_directory() {
        let ext_path: ExtPath = ExtPath::from("/directory/filename.txt");

        let test_struct: ExtPath = ext_path.new_with_parent_directory();
        assert_eq!(test_struct.to_string(), "/directory");
    }

    #[test]
    fn test_extension() {
        let test_struct: ExtPath = ExtPath::from("/");
        let result: Option<ByteString> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: ExtPath = ExtPath::from("/file");
        let result: Option<ByteString> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: ExtPath = ExtPath::from("/.file");
        let result: Option<ByteString> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: ExtPath = ExtPath::from("/file.txt");
        let result: Option<ByteString> = test_struct.extension();
        assert_eq!(result, Some(ByteString::from("txt")));
    }

    #[test]
    fn test_file_stem() {
        let test_struct: ExtPath = ExtPath::from("/");
        let result: Option<ByteString> = test_struct.file_stem();
        assert_eq!(result, None);

        let test_struct: ExtPath = ExtPath::from("/file");
        let result: Option<ByteString> = test_struct.file_stem();
        assert_eq!(result, Some(ByteString::from("file")));

        let test_struct: ExtPath = ExtPath::from("/.file");
        let result: Option<ByteString> = test_struct.file_stem();
        assert_eq!(result, Some(ByteString::from(".file")));

        let test_struct: ExtPath = ExtPath::from("/file.txt");
        let result: Option<ByteString> = test_struct.file_stem();
        assert_eq!(result, Some(ByteString::from("file")));
    }

    #[test]
    fn test_file_name() {
        let test_struct: ExtPath = ExtPath::from("/");
        let result: Option<&ByteString> = test_struct.file_name();
        assert_eq!(result, None);

        let test_struct: ExtPath = ExtPath::from("/directory");
        let result: Option<&ByteString> = test_struct.file_name();
        assert_eq!(result, Some(&ByteString::from("directory")));

        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");
        let result: Option<&ByteString> = test_struct.file_name();
        assert_eq!(result, Some(&ByteString::from("filename.txt")));

        let test_struct: ExtPath = ExtPath::from("/directory/");
        let result: Option<&ByteString> = test_struct.file_name();
        assert_eq!(result, Some(&ByteString::from("")));
    }

    #[test]
    fn test_is_empty() {
        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");
        assert_eq!(test_struct.is_empty(), false);

        let test_struct: ExtPath = ExtPath::from("");
        assert_eq!(test_struct.is_empty(), true);
    }

    #[test]
    fn test_push() {
        let mut test_struct: ExtPath = ExtPath::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![ByteString::from(""), ByteString::from("directory")]
            }
        );

        test_struct.push(ByteString::from("filename.txt"));
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![
                    ByteString::from(""),
                    ByteString::from("directory"),
                    ByteString::from("filename.txt")
                ]
            }
        );
    }

    #[test]
    fn test_to_string() {
        let test_struct: ExtPath = ExtPath::from("/");
        assert_eq!(test_struct.to_string(), "/");

        let test_struct: ExtPath = ExtPath::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: ExtPath = ExtPath::from("/directory/");
        assert_eq!(test_struct.to_string(), "/directory/");
    }

    #[test]
    fn test_from_str() {
        let test_struct: ExtPath = ExtPath::from("/");
        assert_eq!(test_struct.components.len(), 1);
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![ByteString::from("")]
            }
        );

        let test_struct: ExtPath = ExtPath::from("/directory");
        assert_eq!(test_struct.components.len(), 2);
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![ByteString::from(""), ByteString::from("directory")]
            }
        );

        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");
        assert_eq!(test_struct.components.len(), 3);
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![
                    ByteString::from(""),
                    ByteString::from("directory"),
                    ByteString::from("filename.txt")
                ]
            }
        );

        let test_struct: ExtPath = ExtPath::from("/directory/");
        assert_eq!(test_struct.components.len(), 3);
        assert_eq!(
            test_struct,
            ExtPath {
                components: vec![
                    ByteString::from(""),
                    ByteString::from("directory"),
                    ByteString::from("")
                ]
            }
        );

        let test_struct: ExtPath = ExtPath::from("");
        assert_eq!(test_struct.components.len(), 0);
        assert_eq!(test_struct, ExtPath { components: vec![] });
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("/");
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = String::from("/directory");
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = String::from("/directory/filename.txt");
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = String::from("/directory/");
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = String::from("");
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 0);
    }

    #[test]
    fn test_from_str_slice() {
        let str_array: [&str; 1] = [""];
        let test_struct: ExtPath = ExtPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let str_array: [&str; 2] = ["", "directory"];
        let test_struct: ExtPath = ExtPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 3] = ["", "directory", "filename.txt"];
        let test_struct: ExtPath = ExtPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let str_array: [&str; 3] = ["", "directory", ""];
        let test_struct: ExtPath = ExtPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    // TODO: add tests for from_byte_string_slice

    #[test]
    fn test_from_string_slice() {
        let string_array: [String; 1] = [String::from("")];
        let test_struct: ExtPath = ExtPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let string_array: [String; 2] = [String::from(""), String::from("directory")];
        let test_struct: ExtPath = ExtPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: ExtPath = ExtPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: ExtPath = ExtPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_str_vector() {
        let str_vector: Vec<&str> = vec![""];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 1);

        let str_vector: Vec<&str> = vec!["", "directory"];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 2);

        let str_vector: Vec<&str> = vec!["", "directory", "filename.txt"];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);

        let str_vector: Vec<&str> = vec!["", "directory", ""];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    // TODO: add tests for from_byte_string_vector

    #[test]
    fn test_from_string_vector() {
        let string_vector: Vec<String> = vec![String::from("")];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 1);

        let string_vector: Vec<String> = vec![String::from(""), String::from("directory")];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 2);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);
    }
}
