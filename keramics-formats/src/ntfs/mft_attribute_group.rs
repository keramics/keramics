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

use std::collections::HashMap;

/// New Technologies File System (NTFS) Master File Table (MFT) attribute group.
pub struct NtfsMftAttributeGroup {
    /// Attribute index per type.
    pub(super) attributes: HashMap<u32, usize>,
}

impl NtfsMftAttributeGroup {
    /// Creates a new MFT attribute group.
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    /// Adds an attribute index to the group.
    pub fn add_attribute_index(&mut self, attribute_type: u32, attribute_index: usize) {
        self.attributes.insert(attribute_type, attribute_index);
    }

    /// Retrieves a specific attribute index.
    pub fn get_attribute_index(&self, attribute_type: u32) -> Option<&usize> {
        self.attributes.get(&attribute_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_attribute_index() {
        let mut attribute_group: NtfsMftAttributeGroup = NtfsMftAttributeGroup::new();

        assert_eq!(attribute_group.attributes.len(), 0);

        attribute_group.add_attribute_index(0x00000010, 0);
        assert_eq!(attribute_group.attributes.len(), 1);
    }

    #[test]
    fn test_get_attribute_index() {
        let mut attribute_group: NtfsMftAttributeGroup = NtfsMftAttributeGroup::new();
        attribute_group.add_attribute_index(0x00000010, 0);

        assert_eq!(attribute_group.get_attribute_index(0x00000010), Some(&0));
        assert_eq!(attribute_group.get_attribute_index(0x00000030), None);
    }
}
