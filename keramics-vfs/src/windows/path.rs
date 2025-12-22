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

use keramics_formats::{Path, PathComponent};
use keramics_types::Ucs2String;

/// Windows path
pub struct WindowsPath {}

impl WindowsPath {
    /// Retrieves the data fork name from a [`&Ucs2String`] if available.
    pub fn data_fork_name(ucs2_string: &Ucs2String) -> Option<Ucs2String> {
        if ucs2_string.is_empty() {
            return None;
        }
        match ucs2_string
            .elements
            .iter()
            .rposition(|value| *value == 0x003a)
        {
            Some(value_index) => {
                let data_fork_name_string: Ucs2String =
                    Ucs2String::from(&ucs2_string.elements[value_index + 1..]);

                Some(data_fork_name_string)
            }
            None => None,
        }
    }

    /// Converts a [`&str`] containing a Windows path into a [`Path`]
    pub fn from_str(string: &str) -> Path {
        let components: Vec<PathComponent> = if string.is_empty() {
            vec![]
        } else if string == "\\" {
            vec![PathComponent::Root]
        } else {
            let mut string_slice: &str = string.as_str();

            if string_slice.starts_with("\\\\.\\")
                || string_slice.starts_with("\\\\?\\")
                || string_slice.starts_with("\\??\\")
            {
                string_slice = &string_slice[4..];
            }

            let mut components: Vec<PathComponent> = Vec::new();

            for string_segment in string_slice.split("\\") {
                if string_segment.is_empty() {
                    if components.is_empty() {
                        components.push(PathComponent::Root);
                    }
                } else if string_segment == "." {
                    if components.is_empty() {
                        components.push(PathComponent::Current);
                    }
                } else if string_segment == ".." {
                    match components.last() {
                        None | Some(PathComponent::Parent) => {
                            components.push(PathComponent::Parent);
                        }
                        Some(PathComponent::Root) => {}
                        _ => _ = components.pop(),
                    }
                } else {
                    if let Some(PathComponent::Current) = components.last() {
                        _ = components.pop();
                    }
                    let path_component: PathComponent =
                        PathComponent::Ucs2String(Ucs2String::from(string_segment));

                    components.push(path_component);
                }
            }
            components
        };
        Path { components }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_fork_name() {
        let ucs2_string: Ucs2String = Ucs2String::from("");
        let result: Option<Ucs2String> = WindowsPath::data_fork_name(&ucs2_string);
        assert_eq!(result, None);

        let ucs2_string: Ucs2String = Ucs2String::from("file");
        let result: Option<Ucs2String> = WindowsPath::data_fork_name(&ucs2_string);
        assert_eq!(result, None);

        let ucs2_string: Ucs2String = Ucs2String::from(":fork");
        let result: Option<Ucs2String> = WindowsPath::data_fork_name(&ucs2_string);
        assert_eq!(result, Some(Ucs2String::from("fork")));

        let ucs2_string: Ucs2String = Ucs2String::from("file:fork");
        let result: Option<Ucs2String> = WindowsPath::data_fork_name(&ucs2_string);
        assert_eq!(result, Some(Ucs2String::from("fork")));
    }

    #[test]
    fn test_from_str() {
        let test_struct: Path = WindowsPath::from_str("\\");
        assert_eq!(test_struct.components, vec![PathComponent::Root]);

        let test_struct: Path = WindowsPath::from_str("\\directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("directory"))
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("\\directory\\filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("directory")),
                    PathComponent::Ucs2String(Ucs2String::from("filename.txt")),
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("\\directory\\.\\filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("directory")),
                    PathComponent::Ucs2String(Ucs2String::from("filename.txt")),
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("\\directory\\");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("directory"))
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str(".\\directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![PathComponent::Ucs2String(Ucs2String::from("directory"))]
            }
        );

        let test_struct: Path = WindowsPath::from_str("..\\directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Parent,
                    PathComponent::Ucs2String(Ucs2String::from("directory"))
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("..\\directory\\..\\filename.txt");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Parent,
                    PathComponent::Ucs2String(Ucs2String::from("filename.txt"))
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("..\\..\\directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Parent,
                    PathComponent::Parent,
                    PathComponent::Ucs2String(Ucs2String::from("directory")),
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str("\\..\\directory");
        assert_eq!(
            test_struct,
            Path {
                components: vec![
                    PathComponent::Root,
                    PathComponent::Ucs2String(Ucs2String::from("directory"))
                ]
            }
        );

        let test_struct: Path = WindowsPath::from_str(".");
        assert_eq!(
            test_struct,
            Path {
                components: vec![PathComponent::Current]
            }
        );
    }
}
