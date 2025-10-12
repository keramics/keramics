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

// TODO: add Utf16String support.
use keramics_types::{ByteString, Ucs2String};

/// Path component for file resolver.
#[derive(Clone, Debug, PartialEq)]
pub enum PathComponent {
    ByteString(ByteString),
    String(String),
    Ucs2String(Ucs2String),
}

impl PathComponent {
    /// Retrieves the extension if available.
    pub fn extension(&self) -> Option<PathComponent> {
        match self {
            PathComponent::String(string) => {
                if string.is_empty() {
                    None
                } else {
                    match string[1..].chars().rev().position(|value| value == '.') {
                        Some(value_index) => {
                            // Note that value_index is relative to string[1..]
                            Some(PathComponent::String(string[value_index + 2..].to_string()))
                        }
                        None => None,
                    }
                }
            }
            _ => todo!(),
        }
    }

    /// Retrieves the file stem if available.
    pub fn file_stem(&self) -> Option<PathComponent> {
        match self {
            PathComponent::String(string) => {
                if string.is_empty() {
                    None
                } else {
                    match string[1..].chars().rev().position(|value| value == '.') {
                        Some(value_index) => {
                            // Note that value_index is relative to string[1..]
                            Some(PathComponent::String(
                                string[0..value_index + 1].to_string(),
                            ))
                        }
                        None => Some(PathComponent::String(string.clone())),
                    }
                }
            }
            _ => todo!(),
        }
    }

    /// Converts the path components to a `String`.
    pub fn to_string(&self) -> String {
        match self {
            PathComponent::ByteString(byte_string) => byte_string.to_string(),
            PathComponent::String(string) => string.clone(),
            PathComponent::Ucs2String(ucs2_string) => ucs2_string.to_string(),
        }
    }
}

impl From<&str> for PathComponent {
    /// Converts a [`&str`] into a [`PathComponent`]
    fn from(string: &str) -> Self {
        Self::String(string.to_string())
    }
}

impl From<&String> for PathComponent {
    /// Converts a [`&String`] into a [`PathComponent`]
    fn from(string: &String) -> Self {
        Self::String(string.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension() {
        let path_component: PathComponent = PathComponent::from("");
        let result: Option<PathComponent> = path_component.extension();
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file");
        let result: Option<PathComponent> = path_component.extension();
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from(".file");
        let result: Option<PathComponent> = path_component.extension();
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file.txt");
        let result: Option<PathComponent> = path_component.extension();
        assert_eq!(result, Some(PathComponent::from("txt")));
    }

    #[test]
    fn test_file_stem() {
        let path_component: PathComponent = PathComponent::from("");
        let result: Option<PathComponent> = path_component.file_stem();
        assert_eq!(result, None);

        let path_component: PathComponent = PathComponent::from("file");
        let result: Option<PathComponent> = path_component.file_stem();
        assert_eq!(result, Some(PathComponent::from("file")));

        let path_component: PathComponent = PathComponent::from(".file");
        let result: Option<PathComponent> = path_component.file_stem();
        assert_eq!(result, Some(PathComponent::from(".file")));

        let path_component: PathComponent = PathComponent::from("file.txt");
        let result: Option<PathComponent> = path_component.file_stem();
        assert_eq!(result, Some(PathComponent::from("file")));
    }

    #[test]
    fn test_to_string() {
        let path_component: PathComponent = PathComponent::from("test");

        assert_eq!(path_component.to_string(), String::from("test"));
    }

    #[test]
    fn test_from_str() {
        let path_component: PathComponent = PathComponent::from("test");

        assert_eq!(path_component, PathComponent::String(String::from("test")));
    }

    #[test]
    fn test_from_string() {
        let string: String = String::from("test");
        let path_component: PathComponent = PathComponent::from(&string);

        assert_eq!(path_component, PathComponent::String(String::from("test")));
    }
}
