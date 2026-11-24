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

/// Linux Unified Key Setup (LUKS) version 2 metadata configuration object.
#[derive(Clone, Debug, PartialEq)]
pub struct LuksMetadataConfiguration {
    /// JSON area size in bytes.
    pub json_size: u64,

    /// Keyslots area size in bytes.
    pub keyslots_size: u64,

    /// Configuration flags.
    pub flags: Vec<String>,
}

impl LuksMetadataConfiguration {
    /// Creates a new metadata configuration object.
    pub fn new() -> Self {
        Self {
            json_size: 0,
            keyslots_size: 0,
            flags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let configuration: LuksMetadataConfiguration = LuksMetadataConfiguration::new();

        assert_eq!(configuration.json_size, 0);
        assert_eq!(configuration.keyslots_size, 0);
        assert!(configuration.flags.is_empty());
    }
}
