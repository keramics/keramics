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

use std::cmp::min;
use std::collections::BTreeMap;
use std::sync::Arc;

use keramics_formats::PathComponent;

use super::component_weights::ComponentWeights;
use super::enums::ScanTreeType;
use super::groups::ComponentGroup;
use super::signature::PathFilterSignature;

/// Compontents table.
pub(super) struct ComponentTable {
    /// Scan tree type.
    pub scan_tree_type: ScanTreeType,

    /// Component groups.
    pub component_groups: BTreeMap<usize, ComponentGroup>,

    /// Smallest component index.
    pub smallest_component_index: usize,

    /// Signatures.
    pub signatures: Vec<Arc<PathFilterSignature>>,

    /// Value weights.
    value_weights: ComponentWeights,

    /// Occurrence weights.
    occurrence_weights: ComponentWeights,

    /// Similarity weights.
    similarity_weights: ComponentWeights,
}

impl ComponentTable {
    /// Creates a new component table.
    pub fn new(scan_tree_type: &ScanTreeType) -> Self {
        Self {
            scan_tree_type: scan_tree_type.clone(),
            component_groups: BTreeMap::new(),
            smallest_component_index: 0,
            signatures: Vec::new(),
            value_weights: ComponentWeights::new(),
            occurrence_weights: ComponentWeights::new(),
            similarity_weights: ComponentWeights::new(),
        }
    }

    /// Calculates the weights.
    pub fn calculate_weights(&mut self) {
        for (_, component_group) in self.component_groups.iter() {
            let number_of_path_groups: usize = component_group.path_groups.len();
            if number_of_path_groups > 1 {
                self.occurrence_weights.append_weight(
                    component_group.component_index,
                    number_of_path_groups as isize,
                );
            }
            for (_, path_group) in component_group.path_groups.iter() {
                let number_of_signatures: usize = path_group.signatures.len();
                if number_of_signatures > 1 {
                    self.similarity_weights.append_weight(
                        component_group.component_index,
                        number_of_signatures as isize,
                    );
                }
            }
        }
    }

    /// Fills the component table.
    pub fn fill(
        &mut self,
        signatures: &[Arc<PathFilterSignature>],
        component_indexes_to_ignore: &[usize],
    ) {
        match &self.scan_tree_type {
            ScanTreeType::Prefix => self.fill_prefix(signatures, component_indexes_to_ignore),
            ScanTreeType::Suffix => self.fill_suffix(signatures, component_indexes_to_ignore),
        }
    }

    /// Fills the component table for a prefix tree.
    pub fn fill_prefix(
        &mut self,
        signatures: &[Arc<PathFilterSignature>],
        component_indexes_to_ignore: &[usize],
    ) {
        for signature in signatures.iter() {
            if signature.path.is_relative() {
                continue;
            }
            self.signatures.push(Arc::clone(signature));

            for (component_index, path_component) in signature.path.components.iter().enumerate() {
                if !component_indexes_to_ignore.contains(&component_index) {
                    self.insert_component(component_index, path_component, signature);
                }
            }
        }
    }

    /// Fills the component table for a suffix tree.
    pub fn fill_suffix(
        &mut self,
        signatures: &[Arc<PathFilterSignature>],
        component_indexes_to_ignore: &[usize],
    ) {
        for signature in signatures.iter() {
            if !signature.path.is_relative() {
                continue;
            }
            self.signatures.push(Arc::clone(signature));

            for (component_index, path_component) in
                signature.path.components.iter().rev().enumerate()
            {
                if !component_indexes_to_ignore.contains(&component_index) {
                    self.insert_component(component_index, path_component, signature);
                }
            }
        }
    }

    /// Retrieves the component index based on the occurrence weights.
    fn get_component_index_by_occurrence_weights(&self) -> Option<usize> {
        match self
            .occurrence_weights
            .component_index_groups
            .get(&self.occurrence_weights.largest_weight)
        {
            Some(index_group) => {
                let mut largest_value_weight: isize = 0;
                let mut component_index: usize = 0;
                for (group_index, occurence_index) in index_group.indexes.iter().enumerate() {
                    let value_weight: isize = self.value_weights.get_weight(occurence_index);

                    if group_index == 0 || value_weight > largest_value_weight {
                        largest_value_weight = value_weight;
                        component_index = *occurence_index;
                    }
                }
                Some(component_index)
            }
            None => self.get_component_index_by_value_weights(),
        }
    }

    /// Retrieves the component index based on the similarity weights.
    fn get_component_index_by_similarity_weights(&self) -> Option<usize> {
        match self
            .similarity_weights
            .component_index_groups
            .get(&self.similarity_weights.largest_weight)
        {
            Some(index_group) => {
                let mut largest_value_weight: isize = 0;
                let mut largest_occurrence_weight: isize = 0;
                let mut component_index: usize = 0;

                for (group_index, similarity_index) in index_group.indexes.iter().enumerate() {
                    let occurrence_weight: isize =
                        self.occurrence_weights.get_weight(similarity_index);
                    let value_weight: isize = self.value_weights.get_weight(similarity_index);

                    if largest_occurrence_weight > 0
                        && occurrence_weight == largest_occurrence_weight
                        && value_weight > largest_value_weight
                    {
                        largest_occurrence_weight = 0;
                    }
                    if group_index == 0 || occurrence_weight > largest_occurrence_weight {
                        largest_value_weight = value_weight;
                        largest_occurrence_weight = occurrence_weight;
                        component_index = *similarity_index;
                    }
                }
                Some(component_index)
            }
            None => self.get_component_index_by_occurrence_weights(),
        }
    }

    /// Retrieves the component index based on the value weights.
    fn get_component_index_by_value_weights(&self) -> Option<usize> {
        self.value_weights
            .component_index_groups
            .get(&self.value_weights.largest_weight)
            .map(|index_group| index_group.indexes[0])
    }

    /// Retrieve the most significant component index.
    pub fn get_most_significant_component_index(&self) -> Option<usize> {
        let mut result: Option<usize> = match self.signatures.len() {
            0 => None,
            1 => self.get_component_index_by_value_weights(),
            2 => self.get_component_index_by_occurrence_weights(),
            _ => self.get_component_index_by_similarity_weights(),
        };
        if result.is_none() && !self.component_groups.is_empty() {
            result = Some(self.smallest_component_index);
        }
        result
    }

    /// Retrieves the signatures for a specific component index.
    pub fn get_signatures_by_component_index(
        &self,
        component_index: usize,
    ) -> Vec<Arc<PathFilterSignature>> {
        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();

        if let Some(component_group) = self.component_groups.get(&component_index) {
            for (_, path_group) in component_group.path_groups.iter() {
                for signature in path_group.signatures.iter() {
                    if !signatures.contains(signature) {
                        signatures.push(Arc::clone(signature));
                    }
                }
            }
        }
        signatures
    }

    /// Inserts a component for a specific index.
    fn insert_component(
        &mut self,
        component_index: usize,
        path_component: &PathComponent,
        path: &Arc<PathFilterSignature>,
    ) {
        match self.component_groups.get_mut(&component_index) {
            Some(components_group) => components_group.insert_component(path_component, path),
            None => {
                self.smallest_component_index = min(component_index, self.smallest_component_index);

                let mut components_group: ComponentGroup = ComponentGroup::new(component_index);
                components_group.insert_component(path_component, path);

                self.component_groups
                    .insert(component_index, components_group);
            }
        }
    }

    /// Determines if the component table is empty.
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_formats::Path;

    fn get_test_component_table(
        scan_tree_type: &ScanTreeType,
        path_strings: &[&str],
        component_indexes_to_ignore: &[usize],
    ) -> ComponentTable {
        let mut component_table: ComponentTable = ComponentTable::new(scan_tree_type);

        let signatures: Vec<Arc<PathFilterSignature>> = path_strings
            .iter()
            .map(|path_string| Arc::new(PathFilterSignature::new(Path::from(*path_string), None)))
            .collect();

        component_table.fill(&signatures, component_indexes_to_ignore);

        component_table
    }

    #[test]
    fn test_calculate_weights() {
        let mut component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[]);
        component_table.calculate_weights();

        assert_eq!(component_table.occurrence_weights.largest_weight, 0);
        assert_eq!(component_table.occurrence_weights.get_weight(&0), 0);
        assert_eq!(component_table.occurrence_weights.get_weight(&1), 0);
        assert_eq!(component_table.occurrence_weights.get_weight(&2), 0);

        assert_eq!(component_table.similarity_weights.largest_weight, 0);
        assert_eq!(component_table.similarity_weights.get_weight(&0), 0);
        assert_eq!(component_table.similarity_weights.get_weight(&1), 0);
        assert_eq!(component_table.similarity_weights.get_weight(&2), 0);

        assert_eq!(component_table.value_weights.largest_weight, 0);
        assert_eq!(component_table.value_weights.get_weight(&0), 0);
        assert_eq!(component_table.value_weights.get_weight(&1), 0);
        assert_eq!(component_table.value_weights.get_weight(&2), 0);
    }

    #[test]
    fn test_calculate_weights_with_identical_path_groups() {
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["/testdir1/testfile1", "/testdir1/testfile2"],
            &[],
        );
        component_table.calculate_weights();

        // Component index 2 (the file name) has two different path groups.
        assert_eq!(component_table.occurrence_weights.largest_weight, 2);
        assert_eq!(component_table.occurrence_weights.get_weight(&2), 2);
        assert_eq!(component_table.occurrence_weights.get_weight(&0), 0);
        assert_eq!(component_table.occurrence_weights.get_weight(&1), 0);

        // Component indexes 0 (root) and 1 (testdir1) each have two similar signatures.
        assert_eq!(component_table.similarity_weights.largest_weight, 2);
        assert_eq!(component_table.similarity_weights.get_weight(&0), 2);
        assert_eq!(component_table.similarity_weights.get_weight(&1), 2);
        assert_eq!(component_table.similarity_weights.get_weight(&2), 0);
    }

    #[test]
    fn test_calculate_weights_with_different_path_groups() {
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile3",
            ],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(component_table.occurrence_weights.largest_weight, 3);
        assert_eq!(component_table.occurrence_weights.get_weight(&1), 2);
        assert_eq!(component_table.occurrence_weights.get_weight(&2), 3);

        assert_eq!(component_table.similarity_weights.largest_weight, 3);
        assert_eq!(component_table.similarity_weights.get_weight(&0), 3);
        assert_eq!(component_table.similarity_weights.get_weight(&1), 2);
        assert_eq!(component_table.similarity_weights.get_weight(&2), 0);

        assert_eq!(component_table.value_weights.largest_weight, 0);
        assert_eq!(component_table.value_weights.get_weight(&0), 0);
        assert_eq!(component_table.value_weights.get_weight(&1), 0);
        assert_eq!(component_table.value_weights.get_weight(&2), 0);
    }

    #[test]
    fn test_fill_with_prefix_tree() {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);

        assert_eq!(component_table.component_groups.len(), 0);
        assert_eq!(component_table.signatures.len(), 0);

        let component_indexes_to_ignore: Vec<usize> = Vec::new();
        component_table.fill(
            &[Arc::new(PathFilterSignature::new(
                Path::from("/testdir1/testfile1"),
                None,
            ))],
            &component_indexes_to_ignore,
        );
        assert_eq!(component_table.component_groups.len(), 3);
        assert_eq!(component_table.signatures.len(), 1);
        assert_eq!(component_table.smallest_component_index, 0);

        let component_group: &ComponentGroup = &component_table.component_groups[&0];
        assert_eq!(component_group.component_index, 0);
        assert_eq!(component_group.path_groups.len(), 1);

        let component_group: &ComponentGroup = &component_table.component_groups[&1];
        assert_eq!(component_group.component_index, 1);
        assert_eq!(component_group.path_groups.len(), 1);

        let component_group: &ComponentGroup = &component_table.component_groups[&2];
        assert_eq!(component_group.component_index, 2);
        assert_eq!(component_group.path_groups.len(), 1);
    }

    #[test]
    fn test_fill_with_suffix_tree() {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Suffix);

        let component_indexes_to_ignore: Vec<usize> = Vec::new();
        component_table.fill(
            &[Arc::new(PathFilterSignature::new(
                Path::from("testdir1/testfile1"),
                None,
            ))],
            &component_indexes_to_ignore,
        );
        assert_eq!(component_table.component_groups.len(), 2);
        assert_eq!(component_table.signatures.len(), 1);
        assert_eq!(component_table.smallest_component_index, 0);

        // The path components are indexed in reverse order.
        let component_group: &ComponentGroup = &component_table.component_groups[&0];
        assert_eq!(
            component_group.path_groups.keys().next().unwrap(),
            &PathComponent::from("testfile1")
        );

        let component_group: &ComponentGroup = &component_table.component_groups[&1];
        assert_eq!(
            component_group.path_groups.keys().next().unwrap(),
            &PathComponent::from("testdir1")
        );
    }

    #[test]
    fn test_fill_with_mixed_paths() {
        // Relative paths are ignored for a prefix tree.
        let component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["testdir1/testfile1", "/testdir1/testfile1"],
            &[],
        );
        assert_eq!(component_table.component_groups.len(), 3);
        assert_eq!(component_table.signatures.len(), 1);

        // Absolute paths are ignored for a suffix tree.
        let component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Suffix,
            &["testdir1/testfile1", "/testdir1/testfile1"],
            &[],
        );
        assert_eq!(component_table.component_groups.len(), 2);
        assert_eq!(component_table.signatures.len(), 1);
    }

    #[test]
    fn test_fill_without_signatures() {
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &[], &[]);

        assert_eq!(component_table.component_groups.len(), 0);
        assert_eq!(component_table.signatures.len(), 0);

        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Suffix, &[], &[]);

        assert_eq!(component_table.component_groups.len(), 0);
        assert_eq!(component_table.signatures.len(), 0);
    }

    #[test]
    fn test_fill_prefix_with_ignored_component_indexes() {
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[1]);

        assert_eq!(component_table.component_groups.len(), 2);
        assert_eq!(component_table.signatures.len(), 1);
        assert_eq!(component_table.smallest_component_index, 0);

        let component_group: &ComponentGroup = &component_table.component_groups[&0];
        assert_eq!(
            component_group.path_groups.keys().next().unwrap(),
            &PathComponent::Root
        );

        let component_group: &ComponentGroup = &component_table.component_groups[&2];
        assert_eq!(
            component_group.path_groups.keys().next().unwrap(),
            &PathComponent::from("testfile1")
        );
    }

    #[test]
    fn test_fill_suffix_with_ignored_component_indexes() {
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Suffix, &["testdir1/testfile1"], &[0]);

        assert_eq!(component_table.component_groups.len(), 1);
        assert_eq!(component_table.signatures.len(), 1);
        assert_eq!(component_table.smallest_component_index, 0);

        let component_group: &ComponentGroup = &component_table.component_groups[&1];
        assert_eq!(
            component_group.path_groups.keys().next().unwrap(),
            &PathComponent::from("testdir1")
        );
    }

    #[test]
    fn test_get_component_index_by_occurrence_weights() {
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["/testdir1/testfile1", "/testdir1/testfile2"],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_component_index_by_occurrence_weights(),
            Some(2)
        );

        // Without occurrence weights the method returns None.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["/testdir1/testfile1", "/testdir1/testfile1"],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_component_index_by_occurrence_weights(),
            None
        );
    }

    #[test]
    fn test_get_component_index_by_similarity_weights() {
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile1",
            ],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_component_index_by_similarity_weights(),
            Some(0)
        );

        // With the root component index ignored, component index 1
        // has the largest similarity weight of 2.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile1",
            ],
            &[0],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_component_index_by_similarity_weights(),
            Some(1)
        );

        // Without similarity weights the method returns None.
        let mut component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Suffix, &[], &[]);
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_component_index_by_similarity_weights(),
            None
        );
    }

    #[test]
    fn test_get_component_index_by_value_weights() {
        let mut component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[]);
        component_table.calculate_weights();

        // Value weights are never calculated, so the method always returns None.
        assert_eq!(component_table.get_component_index_by_value_weights(), None);

        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile3",
            ],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(component_table.get_component_index_by_value_weights(), None);
    }

    #[test]
    fn test_get_most_significant_component_index() {
        // Without signatures the method returns None.
        let component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);
        assert_eq!(component_table.get_most_significant_component_index(), None);

        // With a single signature the value weights are empty and the method
        // falls back to the smallest component index.
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[]);

        assert_eq!(
            component_table.get_most_significant_component_index(),
            Some(0)
        );

        // With two signatures and calculated weights the occurrence weights
        // select component index 2, which has two different path groups.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["/testdir1/testfile1", "/testdir1/testfile2"],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_most_significant_component_index(),
            Some(2)
        );

        // With multiple similar signatures the similarity weights select the
        // root component index (0), which has the largest similarity weight.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile3",
            ],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_most_significant_component_index(),
            Some(0)
        );

        // With the root component index ignored, component index 1 has the
        // largest similarity weight of 2.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &[
                "/testdir1/testfile1",
                "/testdir1/testfile2",
                "/testdir2/testfile3",
            ],
            &[0],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_most_significant_component_index(),
            Some(1)
        );

        // With a suffix tree and two signatures the occurrence weights select
        // component index 0 (the file names), which has two different path groups.
        let mut component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Suffix,
            &["testdir1/testfile1", "testdir1/testfile2"],
            &[],
        );
        component_table.calculate_weights();

        assert_eq!(
            component_table.get_most_significant_component_index(),
            Some(0)
        );

        // With a single signature and all component indexes ignored, no component
        // groups exist and the method returns None.
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[0, 1, 2]);

        assert_eq!(component_table.get_most_significant_component_index(), None);
    }

    #[test]
    fn test_get_signatures_by_component_index() {
        let component_table: ComponentTable = get_test_component_table(
            &ScanTreeType::Prefix,
            &["/testdir1/testfile1", "/testdir1/testfile2"],
            &[],
        );

        let expected_paths: [Path; 2] = [
            Path::from("/testdir1/testfile1"),
            Path::from("/testdir1/testfile2"),
        ];

        let signatures: Vec<Arc<PathFilterSignature>> =
            component_table.get_signatures_by_component_index(0);
        assert_eq!(signatures.len(), 2);
        for signature in signatures.iter() {
            assert!(expected_paths.contains(&signature.path));
        }

        let signatures: Vec<Arc<PathFilterSignature>> =
            component_table.get_signatures_by_component_index(1);
        assert_eq!(signatures.len(), 2);
        let expected_path_component: PathComponent = PathComponent::from("testdir1");
        for signature in signatures.iter() {
            assert_eq!(
                signature.path.get_component_by_index(1),
                Some(&expected_path_component)
            );
        }

        // Component index 2 has two path groups, one for each signature.
        let signatures: Vec<Arc<PathFilterSignature>> =
            component_table.get_signatures_by_component_index(2);
        assert_eq!(signatures.len(), 2);
        for signature in signatures.iter() {
            assert!(expected_paths.contains(&signature.path));
        }

        // Without a component group an empty list is returned.
        let signatures: Vec<Arc<PathFilterSignature>> =
            component_table.get_signatures_by_component_index(3);
        assert_eq!(signatures.len(), 0);
    }

    #[test]
    fn test_insert_component() {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);

        let signature: Arc<PathFilterSignature> = Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        ));

        component_table.insert_component(3, &PathComponent::from("testfile1"), &signature);

        assert_eq!(component_table.component_groups.len(), 1);
        assert_eq!(component_table.smallest_component_index, 0);

        let component_group: &ComponentGroup = &component_table.component_groups[&3];
        assert_eq!(component_group.path_groups.len(), 1);
        assert_eq!(
            component_group.path_groups[&PathComponent::from("testfile1")]
                .signatures
                .len(),
            1
        );

        // Inserting a different path component in the same component group.
        component_table.insert_component(3, &PathComponent::from("testfile2"), &signature);

        let component_group: &ComponentGroup = &component_table.component_groups[&3];
        assert_eq!(component_group.path_groups.len(), 2);

        // Inserting into an existing component group.
        component_table.insert_component(1, &PathComponent::from("testdir1"), &signature);

        assert_eq!(component_table.component_groups.len(), 2);
        assert_eq!(component_table.smallest_component_index, 0);
    }

    #[test]
    fn test_is_empty() {
        let component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);
        assert!(component_table.is_empty());

        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["/testdir1/testfile1"], &[]);

        assert!(!component_table.is_empty());

        // Relative paths are ignored for a prefix tree.
        let component_table: ComponentTable =
            get_test_component_table(&ScanTreeType::Prefix, &["testdir1/testfile1"], &[]);

        assert!(component_table.is_empty());
    }
}
