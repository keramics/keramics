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

use crate::types::Ucs2String;

/// New Technologies File System (NTFS) path.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct NtfsPath {
    /// Components.
    pub components: Vec<Ucs2String>,
}

impl NtfsPath {
    /// Creates a new path.
    pub fn new() -> Self {
        Self {
            components: vec![Ucs2String::new()],
        }
    }

    /// Determines if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Creates a new path of the parent directory.
    pub fn parent_directory(&self) -> Self {
        let mut number_of_components: usize = self.components.len();
        if number_of_components > 1 {
            number_of_components -= 1;
        }
        let mut parent_components: Vec<Ucs2String> = Vec::with_capacity(number_of_components);
        parent_components.clone_from_slice(&self.components[0..number_of_components]);

        Self {
            components: parent_components,
        }
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        if self.components.len() > 1 {
            self.components
                .iter()
                .map(|component| component.to_string())
                .collect::<Vec<String>>()
                .join("/")
        } else {
            "/".to_string()
        }
    }
}

impl From<&str> for NtfsPath {
    /// Converts a [`&str`] into a [`NtfsPath`]
    fn from(string: &str) -> NtfsPath {
        let components: Vec<Ucs2String> = if string == "/" {
            vec![Ucs2String::new()]
        } else {
            string
                .split("/")
                .map(|component| Ucs2String::from_string(component))
                .collect::<Vec<Ucs2String>>()
        };
        NtfsPath {
            components: components,
        }
    }
}

impl From<&String> for NtfsPath {
    /// Converts a [`&String`] into a [`NtfsPath`]
    #[inline]
    fn from(string: &String) -> NtfsPath {
        NtfsPath::from(string.as_str())
    }
}

impl From<&Vec<Ucs2String>> for NtfsPath {
    /// Converts a [`&Vec<Ucs2String>`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &Vec<Ucs2String>) -> NtfsPath {
        NtfsPath {
            components: path_components.clone(),
        }
    }
}

impl From<&Vec<&str>> for NtfsPath {
    /// Converts a [`&Vec<&str>`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &Vec<&str>) -> NtfsPath {
        let mut components: Vec<Ucs2String> = vec![Ucs2String::new()];
        components.append(
            &mut path_components
                .iter()
                .map(|component| Ucs2String::from_string(component))
                .collect::<Vec<Ucs2String>>(),
        );
        NtfsPath {
            components: components,
        }
    }
}

impl From<&Vec<String>> for NtfsPath {
    /// Converts a [`&Vec<String>`] into a [`NtfsPath`]
    #[inline]
    fn from(path_components: &Vec<String>) -> NtfsPath {
        let mut components: Vec<Ucs2String> = vec![Ucs2String::new()];
        components.append(
            &mut path_components
                .iter()
                .map(|component| Ucs2String::from_string(component))
                .collect::<Vec<Ucs2String>>(),
        );
        NtfsPath {
            components: components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add tests for is_empty
    // TODO: add tests for parent_directory

    #[test]
    fn test_to_string() {
        let test_struct: NtfsPath = NtfsPath::from("/");
        assert_eq!(test_struct.to_string(), "/");

        let test_struct: NtfsPath = NtfsPath::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: NtfsPath = NtfsPath::from("/directory/filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: NtfsPath = NtfsPath::from("/directory/");
        assert_eq!(test_struct.to_string(), "/directory/");
    }

    #[test]
    fn test_from_str() {
        let test_struct: NtfsPath = NtfsPath::from("/");
        assert_eq!(test_struct.components.len(), 1);

        let test_struct: NtfsPath = NtfsPath::from("/directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: NtfsPath = NtfsPath::from("/directory/filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: NtfsPath = NtfsPath::from("/directory/");
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string() {
        let string: String = "/".to_string();
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = "/directory".to_string();
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = "/directory/filename.txt".to_string();
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = "/directory/".to_string();
        let test_struct: NtfsPath = NtfsPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_str_vector() {
        let str_vector: Vec<&str> = vec![];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 1);

        let str_vector: Vec<&str> = vec!["directory"];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 2);

        let str_vector: Vec<&str> = vec!["directory", "filename.txt"];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);

        let str_vector: Vec<&str> = vec!["directory", ""];
        let test_struct: NtfsPath = NtfsPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_vector() {
        let string_vector: Vec<String> = vec![];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 1);

        let string_vector: Vec<String> = vec!["directory".to_string()];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 2);

        let string_vector: Vec<String> = vec!["directory".to_string(), "filename.txt".to_string()];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);

        let string_vector: Vec<String> = vec!["directory".to_string(), "".to_string()];
        let test_struct: NtfsPath = NtfsPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);
    }
}
