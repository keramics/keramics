/* Copyright 2024-2026 Joachim Metz <joachim.metz@gmail.com>
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

use keramics_formats::PathComponent;

/// Virtual File System (VFS) path.
pub struct VfsPath {}

impl VfsPath {
    /// Retrieves a numeric path component suffix.
    pub fn get_numeric_suffix(path_component: &PathComponent, prefix: &str) -> Option<usize> {
        let string: String = path_component.to_string();

        if !string.starts_with(prefix) {
            return None;
        }
        string[prefix.len()..].parse::<usize>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_numeric_suffix() {
        let file_name: PathComponent = PathComponent::from("apm1");
        let numeric_suffix: Option<usize> = VfsPath::get_numeric_suffix(&file_name, "apm");
        assert_eq!(numeric_suffix, Some(1));

        let file_name: PathComponent = PathComponent::from("apm99");
        let numeric_suffix: Option<usize> = VfsPath::get_numeric_suffix(&file_name, "apm");
        assert_eq!(numeric_suffix, Some(99));

        let file_name: PathComponent = PathComponent::from("apm1s");
        let numeric_suffix: Option<usize> = VfsPath::get_numeric_suffix(&file_name, "apm");
        assert_eq!(numeric_suffix, None);

        let file_name: PathComponent = PathComponent::from("bogus1");
        let numeric_suffix: Option<usize> = VfsPath::get_numeric_suffix(&file_name, "apm");
        assert_eq!(numeric_suffix, None);
    }
}
