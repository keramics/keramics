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

use std::cmp::max;
use std::collections::HashMap;

use super::groups::IndexGroup;

pub struct ComponentWeights {
    /// Component index groups per weight.
    pub component_index_groups: HashMap<isize, IndexGroup>,

    /// Weights per component index.
    pub weights: HashMap<usize, isize>,

    /// Largest weight.
    pub largest_weight: isize,
}

impl ComponentWeights {
    /// Creates new component weights.
    pub fn new() -> Self {
        Self {
            component_index_groups: HashMap::new(),
            weights: HashMap::new(),
            largest_weight: 0,
        }
    }

    /// Appends a weight for a specific component index.
    pub fn append_weight(&mut self, component_index: usize, weight: isize) {
        match self.component_index_groups.get_mut(&weight) {
            Some(index_group) => index_group.append_index(component_index),
            None => {
                let mut index_group: IndexGroup = IndexGroup::new();
                index_group.append_index(component_index);

                self.component_index_groups.insert(weight, index_group);
            }
        }
        self.weights.insert(component_index, weight);

        self.largest_weight = max(weight, self.largest_weight);
    }

    /// Retrieves a weight for a specific component index.
    pub fn get_weight(&self, component_index: &usize) -> isize {
        match self.weights.get(component_index) {
            Some(weight) => *weight,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_weight() {
        let mut component_weights: ComponentWeights = ComponentWeights::new();

        assert_eq!(component_weights.component_index_groups.len(), 0);
        assert_eq!(component_weights.weights.len(), 0);
        assert_eq!(component_weights.largest_weight, 0);

        component_weights.append_weight(3, 5);

        assert_eq!(component_weights.component_index_groups.len(), 1);
        // TODO: test if component_index_groups contains a component index for weight 5
        assert_eq!(component_weights.weights.len(), 1);
        // TODO: test if weights contains a weight for component index 3
        assert_eq!(component_weights.largest_weight, 5);
    }
}
