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

use std::collections::HashMap;
use std::sync::Arc;

use keramics_formats::PathComponent;

use super::signature::PathFilterSignature;

/// Component group.
pub(super) struct ComponentGroup {
    /// Component index.
    pub component_index: usize,

    /// Signature groups.
    pub path_groups: HashMap<PathComponent, PathGroup>,
}

impl ComponentGroup {
    /// Creates a new component group.
    pub fn new(component_index: usize) -> Self {
        Self {
            component_index,
            path_groups: HashMap::new(),
        }
    }

    /// Inserts a signature related to a specific component.
    pub fn insert_component(
        &mut self,
        path_component: &PathComponent,
        signature: &Arc<PathFilterSignature>,
    ) {
        match self.path_groups.get_mut(&path_component) {
            Some(path_group) => path_group.append_signature(signature),
            None => {
                let mut path_group: PathGroup = PathGroup::new(path_component);
                path_group.append_signature(signature);

                self.path_groups.insert(path_component.clone(), path_group);
            }
        };
    }
}

/// Index group.
#[derive(Debug)]
pub(super) struct IndexGroup {
    /// Indexes.
    pub indexes: Vec<usize>,
}

impl IndexGroup {
    /// Creates a new index group.
    pub fn new() -> Self {
        Self {
            indexes: Vec::new(),
        }
    }

    /// Appends a index.
    pub fn append_index(&mut self, index: usize) {
        self.indexes.push(index);
    }
}

/// Path group.
pub(super) struct PathGroup {
    /// Path component.
    pub path_component: PathComponent,

    /// Signatures.
    pub signatures: Vec<Arc<PathFilterSignature>>,
}

impl PathGroup {
    /// Creates a new path group.
    pub fn new(path_component: &PathComponent) -> Self {
        Self {
            path_component: path_component.clone(),
            signatures: Vec::new(),
        }
    }

    /// Appends a signature.
    pub fn append_signature(&mut self, signature: &Arc<PathFilterSignature>) {
        self.signatures.push(Arc::clone(signature));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_formats::Path;

    #[test]
    fn test_component_group_insert_component() {
        let mut component_group: ComponentGroup = ComponentGroup::new(1);

        let path_component: PathComponent = PathComponent::from("testdir1");
        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        ));

        component_group.insert_component(&path_component, &signature);

        assert_eq!(component_group.path_groups.len(), 1);
        let path_group: &PathGroup = &component_group.path_groups[&path_component];
        assert_eq!(path_group.path_component, path_component);
        assert_eq!(path_group.signatures.len(), 1);
        assert_eq!(
            path_group.signatures[0].path,
            Path::from("/testdir1/testfile1")
        );

        // Inserting a different signature into the same path group.
        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile2"),
            None,
        ));
        component_group.insert_component(&path_component, &signature);

        assert_eq!(component_group.path_groups.len(), 1);
        let path_group: &PathGroup = &component_group.path_groups[&path_component];
        assert_eq!(path_group.signatures.len(), 2);
        assert_eq!(
            path_group.signatures[0].path,
            Path::from("/testdir1/testfile1")
        );
        assert_eq!(
            path_group.signatures[1].path,
            Path::from("/testdir1/testfile2")
        );

        // Inserting a signature into a different path group.
        let path_component: PathComponent = PathComponent::from("testfile1");
        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        ));
        component_group.insert_component(&path_component, &signature);

        assert_eq!(component_group.path_groups.len(), 2);
        let path_group: &PathGroup = &component_group.path_groups[&path_component];
        assert_eq!(path_group.signatures.len(), 1);
    }

    #[test]
    fn test_index_group_append_index() {
        let mut index_group: IndexGroup = IndexGroup::new();

        index_group.append_index(3);
        index_group.append_index(5);
        index_group.append_index(1);

        assert_eq!(index_group.indexes, vec![3, 5, 1]);
    }

    #[test]
    fn test_path_group_append_signature() {
        let path_component: PathComponent = PathComponent::from("testfile1");

        let mut path_group: PathGroup = PathGroup::new(&path_component);

        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        ));
        path_group.append_signature(&signature);

        assert_eq!(path_group.signatures.len(), 1);
        assert_eq!(
            path_group.signatures[0].path,
            Path::from("/testdir1/testfile1")
        );

        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir2/testfile1"),
            None,
        ));
        path_group.append_signature(&signature);

        assert_eq!(path_group.signatures.len(), 2);
        assert_eq!(
            path_group.signatures[1].path,
            Path::from("/testdir2/testfile1")
        );
    }
}
