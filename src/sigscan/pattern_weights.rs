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

use super::groups::OffsetGroup;

pub struct PatternWeights {
    /// Offset (per weight) groups.
    pub offset_groups: HashMap<isize, OffsetGroup>,

    /// Weight (per offset) groups.
    pub weights: HashMap<usize, isize>,

    /// Largest weight.
    pub largest_weight: isize,
}

impl PatternWeights {
    /// Creates new pattern weights.
    pub fn new() -> Self {
        Self {
            offset_groups: HashMap::new(),
            weights: HashMap::new(),
            largest_weight: 0,
        }
    }

    /// Appends a weight for a specific offset.
    pub fn append_weight(&mut self, pattern_offset: usize, weight: isize) {
        match self.offset_groups.get_mut(&weight) {
            Some(offset_group) => offset_group.append_offset(pattern_offset),
            None => {
                let mut offset_group: OffsetGroup = OffsetGroup::new();
                offset_group.append_offset(pattern_offset);

                self.offset_groups.insert(weight, offset_group);
            }
        };
        self.weights.insert(pattern_offset, weight);

        self.largest_weight = max(weight, self.largest_weight);
    }

    /// Retrieves a weight for a specific offset.
    pub fn get_weight(&self, pattern_offset: &usize) -> isize {
        match self.weights.get(pattern_offset) {
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
        let mut pattern_weights: PatternWeights = PatternWeights::new();

        assert_eq!(pattern_weights.offset_groups.len(), 0);
        assert_eq!(pattern_weights.weights.len(), 0);
        assert_eq!(pattern_weights.largest_weight, 0);

        pattern_weights.append_weight(3, 5);

        assert_eq!(pattern_weights.offset_groups.len(), 1);
        // TODO: test if offset_groups contains an offset group for weight 5
        assert_eq!(pattern_weights.weights.len(), 1);
        // TODO: test if weights contains a weight for pattern offset 3
        assert_eq!(pattern_weights.largest_weight, 5);
    }
}
