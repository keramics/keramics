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

use crate::types::ByteString;

/// Extended File System (ext) path.
pub struct ExtPath {
    /// Components.
    pub(super) components: Vec<ByteString>,
}

impl ExtPath {
    /// Creates a new path.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Determines if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.components.len() == 0
    }

    /// Retrieves a string representation of the path.
    pub fn to_string(&self) -> String {
        self.components
            .iter()
            .map(|component| component.to_string())
            .collect::<Vec<String>>()
            .join("")
    }
}

impl From<&str> for ExtPath {
    /// Converts a [`&String`] into a [`ExtPath`]
    fn from(string: &str) -> ExtPath {
        let components: Vec<ByteString> = if string == "/" {
            vec![ByteString::new()]
        } else {
            string
                .split("/")
                .map(|component| ByteString::from_bytes(component.as_bytes()))
                .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_test_from_string() {
        let test_struct: ExtPath = ExtPath::from("/");

        assert_eq!(test_struct.components.len(), 1);

        let test_struct: ExtPath = ExtPath::from("/directory");

        assert_eq!(test_struct.components.len(), 2);

        let test_struct: ExtPath = ExtPath::from("/directory/filename.txt");

        assert_eq!(test_struct.components.len(), 3);

        let test_struct: ExtPath = ExtPath::from("/directory/");

        assert_eq!(test_struct.components.len(), 3);
    }
}
