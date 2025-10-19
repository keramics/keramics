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

use keramics_types::Ucs2String;

/// New Technologies File System (NTFS) path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NtfsPath {
    /// Components.
    pub components: Vec<Ucs2String>,
}

impl NtfsPath {
    const COMPONENT_SEPARATOR: &'static str = "\\";

    /// Creates a new path of the current path and additional path components.
    pub fn new_with_join(&self, path_components: &[Ucs2String]) -> Self {
        let mut components: Vec<Ucs2String> = self.components.clone();
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
        let parent_components: Vec<Ucs2String> = self.components[0..number_of_components].to_vec();

        Self {
            components: parent_components,
        }
    }

    /// Retrieves the extension if available.
    pub fn extension(&self) -> Option<Ucs2String> {
        match self.file_name() {
            Some(ucs2_string) => match ucs2_string
                .elements
                .iter()
                .skip(1)
                .rev()
                .position(|value| *value == 0x002e)
            {
                Some(value_index) => {
                    // Note that value_index is relative to the end of the string.
                    let string_index: usize = ucs2_string.elements.len() - value_index;
                    Some(Ucs2String::from(&ucs2_string.elements[string_index..]))
                }
                None => None,
            },
            None => None,
        }
    }

    /// Retrieves the file stem if available.
    pub fn file_stem(&self) -> Option<Ucs2String> {
        match self.file_name() {
            Some(ucs2_string) => match ucs2_string
                .elements
                .iter()
                .skip(1)
                .rev()
                .position(|value| *value == 0x002e)
            {
                Some(value_index) => {
                    // Note that value_index is relative to the end of the string.
                    let string_size: usize = ucs2_string.elements.len() - value_index - 1;
                    Some(Ucs2String::from(&ucs2_string.elements[0..string_size]))
                }
                None => Some(ucs2_string.clone()),
            },
            None => None,
        }
    }

    /// Retrieves the file name.
    pub fn file_name(&self) -> Option<&Ucs2String> {
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
    pub fn push(&mut self, component: Ucs2String) {
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

impl From<&str> for NtfsPath {
    /// Converts a [`&str`] into a [`NtfsPath`]
    fn from(string: &str) -> Self {
        let components: Vec<Ucs2String> = if string.is_empty() {
            // Splitting "" results in [""]
            vec![]
        } else if string == Self::COMPONENT_SEPARATOR {
            // Splitting "\\" results in ["", ""]
            vec![Ucs2String::new()]
        } else {
            string
                .split(Self::COMPONENT_SEPARATOR)
                .map(|component| Ucs2String::from(component))
                .collect::<Vec<Ucs2String>>()
        };
        Self {
            components: components,
        }
    }
}

impl From<&String> for NtfsPath {
    /// Converts a [`&String`] into a [`NtfsPath`]
    #[inline]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&[&str]> for NtfsPath {
    /// Converts a [`&[&str]`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &[&str]) -> Self {
        let components: Vec<Ucs2String> = path_components
            .iter()
            .map(|component| Ucs2String::from(*component))
            .collect::<Vec<Ucs2String>>();

        Self {
            components: components,
        }
    }
}

impl From<&[String]> for NtfsPath {
    /// Converts a [`&[String]`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &[String]) -> Self {
        let components: Vec<Ucs2String> = path_components
            .iter()
            .map(|component| Ucs2String::from(component.as_str()))
            .collect::<Vec<Ucs2String>>();

        Self {
            components: components,
        }
    }
}

impl From<&[Ucs2String]> for NtfsPath {
    /// Converts a [`&[Ucs2String]`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &[Ucs2String]) -> Self {
        Self {
            components: path_components.to_vec(),
        }
    }
}

impl From<&Vec<&str>> for NtfsPath {
    /// Converts a [`&Vec<&str>`] into a [`NtfsPath`]
    #[inline(always)]
    fn from(path_components: &Vec<&str>) -> Self {
        Self::from(path_components.as_slice())
    }
}

impl From<&Vec<String>> for NtfsPath {
    /// Converts a [`&Vec<String>`] into a [`NtfsPath`]
    #[inline(always)]
    fn from(path_components: &Vec<String>) -> Self {
        Self::from(path_components.as_slice())
    }
}

impl From<&Vec<Ucs2String>> for NtfsPath {
    /// Converts a [`&Vec<Ucs2String>`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &Vec<Ucs2String>) -> Self {
        Self {
            components: path_components.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_join() {
        let ntfs_path: NtfsPath = NtfsPath::from("\\directory");

        let test_struct: NtfsPath = ntfs_path.new_with_join(&[Ucs2String::from("filename.txt")]);
        assert_eq!(test_struct.to_string(), "\\directory\\filename.txt");
    }

    #[test]
    fn test_new_with_parent_directory() {
        let ntfs_path: NtfsPath = NtfsPath::from("\\directory\\filename.txt");

        let test_struct: NtfsPath = ntfs_path.new_with_parent_directory();
        assert_eq!(test_struct.to_string(), "\\directory");
    }

    #[test]
    fn test_extension() {
        let test_struct: NtfsPath = NtfsPath::from("\\");
        let result: Option<Ucs2String> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: NtfsPath = NtfsPath::from("\\file");
        let result: Option<Ucs2String> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: NtfsPath = NtfsPath::from("\\.file");
        let result: Option<Ucs2String> = test_struct.extension();
        assert_eq!(result, None);

        let test_struct: NtfsPath = NtfsPath::from("\\file.txt");
        let result: Option<Ucs2String> = test_struct.extension();
        assert_eq!(result, Some(Ucs2String::from("txt")));
    }

    #[test]
    fn test_file_stem() {
        let test_struct: NtfsPath = NtfsPath::from("\\");
        let result: Option<Ucs2String> = test_struct.file_stem();
        assert_eq!(result, None);

        let test_struct: NtfsPath = NtfsPath::from("\\file");
        let result: Option<Ucs2String> = test_struct.file_stem();
        assert_eq!(result, Some(Ucs2String::from("file")));

        let test_struct: NtfsPath = NtfsPath::from("\\.file");
        let result: Option<Ucs2String> = test_struct.file_stem();
        assert_eq!(result, Some(Ucs2String::from(".file")));

        let test_struct: NtfsPath = NtfsPath::from("\\file.txt");
        let result: Option<Ucs2String> = test_struct.file_stem();
        assert_eq!(result, Some(Ucs2String::from("file")));
    }

    #[test]
    fn test_file_name() {
        let test_struct: NtfsPath = NtfsPath::from("\\");
        let result: Option<&Ucs2String> = test_struct.file_name();
        assert_eq!(result, None);

        let test_struct: NtfsPath = NtfsPath::from("\\directory");
        let result: Option<&Ucs2String> = test_struct.file_name();
        assert_eq!(result, Some(&Ucs2String::from("directory")));

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\filename.txt");
        let result: Option<&Ucs2String> = test_struct.file_name();
        assert_eq!(result, Some(&Ucs2String::from("filename.txt")));

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\");
        let result: Option<&Ucs2String> = test_struct.file_name();
        assert_eq!(result, Some(&Ucs2String::from("")));
    }

    #[test]
    fn test_is_empty() {
        let test_struct: NtfsPath = NtfsPath::from("\\directory\\filename.txt");
        assert_eq!(test_struct.is_empty(), false);

        let test_struct: NtfsPath = NtfsPath::from("");
        assert_eq!(test_struct.is_empty(), true);
    }

    #[test]
    fn test_push() {
        let mut test_struct: NtfsPath = NtfsPath::from("\\directory");
        assert_eq!(test_struct.to_string(), "\\directory");
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![Ucs2String::from(""), Ucs2String::from("directory")]
            }
        );

        test_struct.push(Ucs2String::from("filename.txt"));
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![
                    Ucs2String::from(""),
                    Ucs2String::from("directory"),
                    Ucs2String::from("filename.txt")
                ]
            }
        );
    }

    #[test]
    fn test_to_string() {
        let test_struct: NtfsPath = NtfsPath::from("\\");
        assert_eq!(test_struct.to_string(), "\\");

        let test_struct: NtfsPath = NtfsPath::from("\\directory");
        assert_eq!(test_struct.to_string(), "\\directory");

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\filename.txt");
        assert_eq!(test_struct.to_string(), "\\directory\\filename.txt");

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\");
        assert_eq!(test_struct.to_string(), "\\directory\\");
    }

    #[test]
    fn test_from_str() {
        let test_struct: NtfsPath = NtfsPath::from("\\");
        assert_eq!(test_struct.components.len(), 1);

        let test_struct: NtfsPath = NtfsPath::from("\\directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: NtfsPath = NtfsPath::from("\\directory\\");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: NtfsPath = NtfsPath::from("");
        assert_eq!(test_struct.components.len(), 0);
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("\\");
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 1);
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![Ucs2String::from(""),]
            }
        );

        let string: String = String::from("\\directory");
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 2);
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![Ucs2String::from(""), Ucs2String::from("directory"),]
            }
        );

        let string: String = String::from("\\directory\\filename.txt");
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![
                    Ucs2String::from(""),
                    Ucs2String::from("directory"),
                    Ucs2String::from("filename.txt")
                ]
            }
        );

        let string: String = String::from("\\directory\\");
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);
        assert_eq!(
            test_struct,
            NtfsPath {
                components: vec![
                    Ucs2String::from(""),
                    Ucs2String::from("directory"),
                    Ucs2String::from(""),
                ]
            }
        );

        let string: String = String::from("");
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 0);
        assert_eq!(test_struct, NtfsPath { components: vec![] });
    }

    #[test]
    fn test_from_str_slice() {
        let str_array: [&str; 1] = [""];
        let test_struct: NtfsPath = NtfsPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let str_array: [&str; 2] = ["", "directory"];
        let test_struct: NtfsPath = NtfsPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 3] = ["", "directory", "filename.txt"];
        let test_struct: NtfsPath = NtfsPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let str_array: [&str; 3] = ["", "directory", ""];
        let test_struct: NtfsPath = NtfsPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_slice() {
        let string_array: [String; 1] = [String::from("")];
        let test_struct: NtfsPath = NtfsPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let string_array: [String; 2] = [String::from(""), String::from("directory")];
        let test_struct: NtfsPath = NtfsPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: NtfsPath = NtfsPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: NtfsPath = NtfsPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    // TODO: add tests for from_ucs2_string_slice

    #[test]
    fn test_from_str_vector() {
        let str_vector: Vec<&str> = vec![""];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 1);

        let str_vector: Vec<&str> = vec!["", "directory"];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 2);

        let str_vector: Vec<&str> = vec!["", "directory", "filename.txt"];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);

        let str_vector: Vec<&str> = vec!["", "directory", ""];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_vector() {
        let string_vector: Vec<String> = vec![String::from("")];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 1);

        let string_vector: Vec<String> = vec![String::from(""), String::from("directory")];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 2);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    // TODO: add tests for from_ucs2_string_vector
}
