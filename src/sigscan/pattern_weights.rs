/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use super::groups::{OffsetGroup, WeightGroup};

pub struct PatternWeights {
    /// Offset (per weight) groups.
    pub offset_groups: HashMap<isize, OffsetGroup>,

    /// Weight (per offset) groups.
    pub weight_groups: HashMap<usize, WeightGroup>,

    /// Largest weight.
    pub largest_weight: isize,
}

impl PatternWeights {
    /// Creates new pattern weights.
    pub fn new() -> Self {
        Self {
            offset_groups: HashMap::new(),
            weight_groups: HashMap::new(),
            largest_weight: 0,
        }
    }

    /// Appends a weight for a specific offset.
    pub fn append_weight(&mut self, pattern_offset: usize, weight: isize) {
        match self.offset_groups.get_mut(&weight) {
            Some(offset_group) => offset_group.append_offset(pattern_offset),
            None => {
                let mut offset_group: OffsetGroup = OffsetGroup::new(weight);
                offset_group.append_offset(pattern_offset);

                self.offset_groups.insert(weight, offset_group);
            }
        };
        match self.weight_groups.get_mut(&pattern_offset) {
            Some(weight_group) => weight_group.weight = weight,
            None => {
                let mut weight_group: WeightGroup = WeightGroup::new(pattern_offset);
                weight_group.weight = weight;

                self.weight_groups.insert(pattern_offset, weight_group);
            }
        };
        self.largest_weight = max(weight, self.largest_weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_weight() {
        let mut pattern_weights: PatternWeights = PatternWeights::new();

        assert_eq!(pattern_weights.offset_groups.len(), 0);
        assert_eq!(pattern_weights.weight_groups.len(), 0);
        assert_eq!(pattern_weights.largest_weight, 0);

        pattern_weights.append_weight(3, 5);

        assert_eq!(pattern_weights.offset_groups.len(), 1);
        // TODO: test if offset_groups contains an offset group for weight 5
        assert_eq!(pattern_weights.weight_groups.len(), 1);
        // TODO: test if weight_groups contains a weight group for pattern offset 3
        assert_eq!(pattern_weights.largest_weight, 5);
    }
}
