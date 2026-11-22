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

use keramics_types::Uuid;

/// Parallels Disk Image (PDI) snapshot descriptor.
#[derive(Debug)]
pub struct PdiSnapshotDescriptor {
    /// Identifier.
    pub(super) identifier: Uuid,

    /// Parent identifier.
    pub(super) parent_identifier: Option<Uuid>,
}

impl PdiSnapshotDescriptor {
    /// Creates a new snapshot descriptor.
    pub fn new(identifier: Uuid, parent_identifier: Option<Uuid>) -> Self {
        Self {
            identifier,
            parent_identifier,
        }
    }

    /// Retrieves the identifier.
    pub fn get_identifier(&self) -> &Uuid {
        &self.identifier
    }

    /// Retrieves the parent identifier.
    pub fn get_parent_identifier(&self) -> Option<&Uuid> {
        self.parent_identifier.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getters() {
        let identifier: Uuid = Uuid::from_be_bytes(&[1; 16]);
        let parent_identifier: Uuid = Uuid::from_be_bytes(&[2; 16]);
        let descriptor: PdiSnapshotDescriptor =
            PdiSnapshotDescriptor::new(identifier.clone(), Some(parent_identifier.clone()));

        assert_eq!(descriptor.get_identifier(), &identifier);
        assert_eq!(descriptor.get_parent_identifier(), Some(&parent_identifier));

        let root_descriptor: PdiSnapshotDescriptor =
            PdiSnapshotDescriptor::new(identifier.clone(), None);
        assert_eq!(root_descriptor.get_parent_identifier(), None);
    }
}
