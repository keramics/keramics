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
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExtPath {
    /// Components.
    pub components: Vec<ByteString>,
}

impl ExtPath {
    /// Creates a new path.
    pub fn new() -> Self {
        Self {
            components: vec![ByteString::new()],
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
        let mut parent_components: Vec<ByteString> = Vec::with_capacity(number_of_components);
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

impl From<&str> for ExtPath {
    /// Converts a [`&str`] into a [`ExtPath`]
    fn from(string: &str) -> ExtPath {
        let components: Vec<ByteString> = if string == "/" {
            vec![ByteString::new()]
        } else {
            string
                .split("/")
                .map(|component| ByteString::from_string(component))
                .collect::<Vec<ByteString>>()
        };
        ExtPath {
            components: components,
        }
    }
}

impl From<&String> for ExtPath {
    /// Converts a [`&String`] into a [`ExtPath`]
    #[inline]
    fn from(string: &String) -> ExtPath {
        ExtPath::from(string.as_str())
    }
}

impl From<&Vec<ByteString>> for ExtPath {
    /// Converts a [`&Vec<ByteString>`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &Vec<ByteString>) -> ExtPath {
        ExtPath {
            components: path_components.clone(),
        }
    }
}

impl From<&Vec<&str>> for ExtPath {
    /// Converts a [`&Vec<&str>`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &Vec<&str>) -> ExtPath {
        let mut components: Vec<ByteString> = vec![ByteString::new()];
        components.append(
            &mut path_components
                .iter()
                .map(|component| ByteString::from_string(component))
                .collect::<Vec<ByteString>>(),
        );
        ExtPath {
            components: components,
        }
    }
}

impl From<&Vec<String>> for ExtPath {
    /// Converts a [`&Vec<String>`] into a [`ExtPath`]
    #[inline]
    fn from(path_components: &Vec<String>) -> ExtPath {
        let mut components: Vec<ByteString> = vec![ByteString::new()];
        components.append(
            &mut path_components
                .iter()
                .map(|component| ByteString::from_string(component))
                .collect::<Vec<ByteString>>(),
        );
        ExtPath {
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

        let test_struct: ExtPath = ExtPath::from("/directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: ExtPath = ExtPath::from("/directory/");
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string() {
        let string: String = "/".to_string();
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = "/directory".to_string();
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = "/directory/filename.txt".to_string();
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = "/directory/".to_string();
        let test_struct: ExtPath = ExtPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_str_vector() {
        let str_vector: Vec<&str> = vec![];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 1);

        let str_vector: Vec<&str> = vec!["directory"];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 2);

        let str_vector: Vec<&str> = vec!["directory", "filename.txt"];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);

        let str_vector: Vec<&str> = vec!["directory", ""];
        let test_struct: ExtPath = ExtPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_vector() {
        let string_vector: Vec<String> = vec![];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 1);

        let string_vector: Vec<String> = vec!["directory".to_string()];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 2);

        let string_vector: Vec<String> = vec!["directory".to_string(), "filename.txt".to_string()];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);

        let string_vector: Vec<String> = vec!["directory".to_string(), "".to_string()];
        let test_struct: ExtPath = ExtPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);
    }
}
