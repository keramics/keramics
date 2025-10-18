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

/// String path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StringPath {
    /// Components.
    pub components: Vec<String>,
}

impl StringPath {
    const COMPONENT_SEPARATOR: &'static str = "/";

    /// Creates a new path of the parent directory.
    pub fn new_with_parent_directory(&self) -> Self {
        let mut number_of_components: usize = self.components.len();
        if number_of_components > 1 {
            number_of_components -= 1;
        }
        let parent_components: Vec<String> = self.components[0..number_of_components].to_vec();

        Self {
            components: parent_components,
        }
    }

    /// Retrieves the file name.
    pub fn file_name(&self) -> Option<&String> {
        let number_of_components: usize = self.components.len();
        if number_of_components > 1 {
            Some(&self.components[number_of_components - 1])
        } else {
            None
        }
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        let number_of_components: usize = self.components.len();
        if number_of_components == 1 && self.components[0].is_empty() {
            Self::COMPONENT_SEPARATOR.to_string()
        } else {
            self.components.join(Self::COMPONENT_SEPARATOR)
        }
    }
}

impl From<&str> for StringPath {
    /// Converts a [`&str`] into a [`StringPath`]
    fn from(string: &str) -> Self {
        let components: Vec<String> = if string == Self::COMPONENT_SEPARATOR {
            // Splitting "/" results in ["", ""]
            vec![String::new()]
        } else {
            string
                .split(Self::COMPONENT_SEPARATOR)
                .map(|component| String::from(component))
                .collect::<Vec<String>>()
        };
        Self {
            components: components,
        }
    }
}

impl From<&String> for StringPath {
    /// Converts a [`&String`] into a [`StringPath`]
    #[inline(always)]
    fn from(string: &String) -> Self {
        Self::from(string.as_str())
    }
}

impl From<&[&str]> for StringPath {
    /// Converts a [`&[&str]`] into a [`StringPath`]
    #[inline]
    fn from(path_components: &[&str]) -> Self {
        let components: Vec<String> = path_components
            .iter()
            .map(|component| String::from(*component))
            .collect::<Vec<String>>();

        Self {
            components: components,
        }
    }
}

impl From<&[String]> for StringPath {
    /// Converts a [`&[String]`] into a [`StringPath`]
    #[inline]
    fn from(path_components: &[String]) -> Self {
        Self {
            components: path_components.to_vec(),
        }
    }
}

impl From<&Vec<&str>> for StringPath {
    /// Converts a [`&Vec<&str>`] into a [`StringPath`]
    #[inline(always)]
    fn from(path_components: &Vec<&str>) -> Self {
        Self::from(path_components.as_slice())
    }
}

impl From<&Vec<String>> for StringPath {
    /// Converts a [`&Vec<String>`] into a [`StringPath`]
    #[inline]
    fn from(path_components: &Vec<String>) -> Self {
        Self {
            components: path_components.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_parent_directory() {
        let string_path: StringPath = StringPath::from("/directory/filename.txt");

        let test_struct: StringPath = string_path.new_with_parent_directory();
        assert_eq!(test_struct.to_string(), "/directory");
    }

    #[test]
    fn test_file_name() {
        let test_struct: StringPath = StringPath::from("/");
        let result: Option<&String> = test_struct.file_name();
        assert_eq!(result, None);

        let test_struct: StringPath = StringPath::from("/directory");
        let result: Option<&String> = test_struct.file_name();
        assert_eq!(result, Some(&String::from("directory")));

        let test_struct: StringPath = StringPath::from("/directory/filename.txt");
        let result: Option<&String> = test_struct.file_name();
        assert_eq!(result, Some(&String::from("filename.txt")));

        let test_struct: StringPath = StringPath::from("/directory/");
        let result: Option<&String> = test_struct.file_name();
        assert_eq!(result, Some(&String::from("")));
    }

    #[test]
    fn test_to_string() {
        let test_struct: StringPath = StringPath::from("/");
        assert_eq!(test_struct.to_string(), "/");

        let test_struct: StringPath = StringPath::from("/directory");
        assert_eq!(test_struct.to_string(), "/directory");

        let test_struct: StringPath = StringPath::from("/directory/filename.txt");
        assert_eq!(test_struct.to_string(), "/directory/filename.txt");

        let test_struct: StringPath = StringPath::from("/directory/");
        assert_eq!(test_struct.to_string(), "/directory/");
    }

    #[test]
    fn test_from_str() {
        let test_struct: StringPath = StringPath::from("/");
        assert_eq!(test_struct.components.len(), 1);

        let test_struct: StringPath = StringPath::from("/directory");
        assert_eq!(test_struct.components.len(), 2);

        let test_struct: StringPath = StringPath::from("/directory/filename.txt");
        assert_eq!(test_struct.components.len(), 3);

        let test_struct: StringPath = StringPath::from("/directory/");
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("/");
        let test_struct: StringPath = StringPath::from(&string);
        assert_eq!(test_struct.components.len(), 1);

        let string: String = String::from("/directory");
        let test_struct: StringPath = StringPath::from(&string);
        assert_eq!(test_struct.components.len(), 2);

        let string: String = String::from("/directory/filename.txt");
        let test_struct: StringPath = StringPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);

        let string: String = String::from("/directory/");
        let test_struct: StringPath = StringPath::from(&string);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_str_slice() {
        let str_array: [&str; 1] = [""];
        let test_struct: StringPath = StringPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let str_array: [&str; 2] = ["", "directory"];
        let test_struct: StringPath = StringPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let str_array: [&str; 3] = ["", "directory", "filename.txt"];
        let test_struct: StringPath = StringPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let str_array: [&str; 3] = ["", "directory", ""];
        let test_struct: StringPath = StringPath::from(str_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_slice() {
        let string_array: [String; 1] = [String::from("")];
        let test_struct: StringPath = StringPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 1);

        let string_array: [String; 2] = [String::from(""), String::from("directory")];
        let test_struct: StringPath = StringPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 2);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: StringPath = StringPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);

        let string_array: [String; 3] = [
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: StringPath = StringPath::from(string_array.as_slice());
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_str_vector() {
        let str_vector: Vec<&str> = vec![""];
        let test_struct: StringPath = StringPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 1);

        let str_vector: Vec<&str> = vec!["", "directory"];
        let test_struct: StringPath = StringPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 2);

        let str_vector: Vec<&str> = vec!["", "directory", "filename.txt"];
        let test_struct: StringPath = StringPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);

        let str_vector: Vec<&str> = vec!["", "directory", ""];
        let test_struct: StringPath = StringPath::from(&str_vector);
        assert_eq!(test_struct.components.len(), 3);
    }

    #[test]
    fn test_from_string_vector() {
        let string_vector: Vec<String> = vec![String::from("")];
        let test_struct: StringPath = StringPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 1);

        let string_vector: Vec<String> = vec![String::from(""), String::from("directory")];
        let test_struct: StringPath = StringPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 2);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from("filename.txt"),
        ];
        let test_struct: StringPath = StringPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);

        let string_vector: Vec<String> = vec![
            String::from(""),
            String::from("directory"),
            String::from(""),
        ];
        let test_struct: StringPath = StringPath::from(&string_vector);
        assert_eq!(test_struct.components.len(), 3);
    }
}
