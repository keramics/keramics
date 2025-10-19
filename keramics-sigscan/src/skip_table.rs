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
use std::collections::HashMap;

use super::types::SignatureReference;

/// Skip table.
#[derive(Debug)]
pub(super) struct SkipTable {
    /// Smallest pattern size.
    pub smallest_pattern_size: usize,

    /// Skip values.
    pub skip_values: HashMap<u8, usize>,

    /// Smallest skip value.
    pub smallest_skip_value: usize,
}

impl SkipTable {
    /// Creates a new skip table.
    pub fn new() -> Self {
        Self {
            smallest_pattern_size: 0,
            skip_values: HashMap::new(),
            smallest_skip_value: 0,
        }
    }

    /// Fills the skip table.
    pub fn fill(&mut self, signatures: &Vec<SignatureReference>) {
        for signature in signatures.iter() {
            self.smallest_pattern_size = if self.smallest_pattern_size == 0 {
                signature.pattern_size
            } else {
                min(signature.pattern_size, self.smallest_pattern_size)
            };
        }
        self.smallest_skip_value = self.smallest_pattern_size;

        for signature in signatures.iter() {
            let mut skip_value: usize = self.smallest_pattern_size;

            for byte_value in signature.pattern[0..self.smallest_pattern_size].iter() {
                skip_value -= 1;

                let insert_skip_value: bool = match self.skip_values.get(byte_value) {
                    Some(skip_table_value) => skip_value < *skip_table_value,
                    None => true,
                };
                if insert_skip_value {
                    self.skip_values.insert(*byte_value, skip_value);

                    if skip_value > 0 {
                        self.smallest_skip_value = min(skip_value, self.smallest_skip_value);
                    }
                }
            }
        }
    }

    /// Retrieves a skip value.
    pub fn get_skip_value(&self, byte_value: &u8) -> usize {
        match self.skip_values.get(byte_value) {
            Some(value) => *value,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use crate::enums::PatternType;
    use crate::signature::Signature;

    #[test]
    fn test_fill() {
        let mut skip_table: SkipTable = SkipTable::new();

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        skip_table.fill(&signatures);

        assert_eq!(skip_table.smallest_pattern_size, 8);
        assert_eq!(skip_table.skip_values.get(&0x63), Some(3).as_ref());
        assert_eq!(skip_table.smallest_skip_value, 1);
    }

    // TODO: add test for get_skip_value
}
