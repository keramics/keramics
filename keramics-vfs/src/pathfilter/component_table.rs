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

    #[test]
    fn test_calculate_weights() {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);

        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        signatures.push(Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        )));

        let component_indexes_to_ignore: Vec<usize> = Vec::new();
        component_table.fill(&signatures, &component_indexes_to_ignore);
        component_table.calculate_weights();

        // TODO: check weights.
    }

    #[test]
    fn test_fill() {
        let mut component_table: ComponentTable = ComponentTable::new(&ScanTreeType::Prefix);

        assert_eq!(component_table.component_groups.len(), 0);
        assert_eq!(component_table.signatures.len(), 0);

        let mut signatures: Vec<Arc<PathFilterSignature>> = Vec::new();
        signatures.push(Arc::new(PathFilterSignature::new(
            Path::from("/testdir1/testfile1"),
            None,
        )));

        let component_indexes_to_ignore: Vec<usize> = Vec::new();
        component_table.fill(&signatures, &component_indexes_to_ignore);

        assert_eq!(component_table.component_groups.len(), 3);
        assert_eq!(component_table.signatures.len(), 1);
    }

    // TODO: add tests for fill_prefix
    // TODO: add tests for fill_suffix
    // TODO: add tests for get_component_index_by_occurrence_weights
    // TODO: add tests for get_component_index_by_similarity_weights
    // TODO: add tests for get_component_index_by_value_weights
    // TODO: add tests for get_most_significant_component_index
    // TODO: add tests for get_signatures_by_component_index
    // TODO: add tests for insert_component
    // TODO: add tests for is_empty
}
