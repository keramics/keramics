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

/// Retrieves a hierarchy prefix.
pub fn get_hierarchy_prefix(levels: &[bool]) -> String {
    let number_of_levels: usize = levels.len();
    let mut prefix: String = String::new();

    for (level, is_last) in levels[0..number_of_levels].iter().enumerate() {
        if level + 1 < number_of_levels {
            if *is_last {
                prefix.push_str("    ");
            } else {
                prefix.push_str("│   ");
            }
        } else {
            if *is_last {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }
    }
    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hierarchy_prefix() {
        let prefix: String = get_hierarchy_prefix(&[]);
        assert_eq!(prefix, "");

        let prefix: String = get_hierarchy_prefix(&[true]);
        assert_eq!(prefix, "└── ");

        let prefix: String = get_hierarchy_prefix(&[false]);
        assert_eq!(prefix, "├── ");

        let prefix: String = get_hierarchy_prefix(&[true, true]);
        assert_eq!(prefix, "    └── ");

        let prefix: String = get_hierarchy_prefix(&[false, true]);
        assert_eq!(prefix, "│   └── ");

        let prefix: String = get_hierarchy_prefix(&[false, false]);
        assert_eq!(prefix, "│   ├── ");
    }
}
