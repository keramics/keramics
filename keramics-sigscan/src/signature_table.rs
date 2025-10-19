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

use keramics_core::mediator::{Mediator, MediatorReference};

use super::constants::*;
use super::enums::PatternType;
use super::groups::ByteValueGroup;
use super::pattern_weights::PatternWeights;
use super::types::SignatureReference;

/// Signature table.
pub(super) struct SignatureTable {
    /// Mediator.
    mediator: MediatorReference,

    /// Pattern type.
    pub pattern_type: PatternType,

    /// Byte value groups.
    pub byte_value_groups: BTreeMap<usize, ByteValueGroup>,

    /// Smallest pattern offset.
    pub smallest_pattern_offset: usize,

    /// Signatures.
    pub signatures: Vec<SignatureReference>,

    /// Byte value (pattern) weights.
    byte_value_weights: PatternWeights,

    /// Occurrence (pattern) weights.
    occurrence_weights: PatternWeights,

    /// Similarity (pattern) weights.
    similarity_weights: PatternWeights,
}

impl SignatureTable {
    /// Creates a new signature table.
    pub fn new(pattern_type: &PatternType) -> Self {
        Self {
            mediator: Mediator::current(),
            pattern_type: pattern_type.clone(),
            byte_value_groups: BTreeMap::new(),
            smallest_pattern_offset: 0,
            signatures: Vec::new(),
            byte_value_weights: PatternWeights::new(),
            occurrence_weights: PatternWeights::new(),
            similarity_weights: PatternWeights::new(),
        }
    }

    /// Calculates the pattern weights.
    pub fn calculate_pattern_weights(&mut self) {
        for (_, byte_value_group) in self.byte_value_groups.iter() {
            let number_of_signature_groups: usize = byte_value_group.signature_groups.len();
            if number_of_signature_groups > 1 {
                self.occurrence_weights.append_weight(
                    byte_value_group.pattern_offset,
                    number_of_signature_groups as isize,
                );
            }
            for (_, signature_group) in byte_value_group.signature_groups.iter() {
                let number_of_signatures: usize = signature_group.signatures.len();
                if number_of_signatures > 1 {
                    self.similarity_weights.append_weight(
                        byte_value_group.pattern_offset,
                        number_of_signatures as isize,
                    );
                }
                if SIGSCAN_COMMON_BYTE_VALUES[signature_group.byte_value as usize] {
                    self.byte_value_weights
                        .append_weight(byte_value_group.pattern_offset, 1);
                }
            }
        }
    }

    /// Fills the signature table.
    pub fn fill(
        &mut self,
        signatures: &Vec<SignatureReference>,
        offsets_to_ignore: &Vec<usize>,
        largest_pattern_offset: usize,
    ) {
        for signature in signatures.iter() {
            if signature.pattern_type != self.pattern_type {
                continue;
            }
            let mut pattern_offset: usize = match self.pattern_type {
                PatternType::BoundToEnd => largest_pattern_offset - signature.pattern_offset,
                PatternType::BoundToStart => signature.pattern_offset,
                PatternType::Unbound => 0,
            };
            self.signatures.push(Arc::clone(&signature));

            for pattern_index in 0..signature.pattern_size {
                if !offsets_to_ignore.contains(&pattern_offset) {
                    self.insert_signature(
                        pattern_offset,
                        signature.pattern[pattern_index],
                        signature,
                    );
                }
                pattern_offset += 1;
            }
        }
    }

    /// Retrieve the most significant pattern offset.
    pub fn get_most_significant_pattern_offset(&self) -> Option<usize> {
        let mut result: Option<usize> = match self.signatures.len() {
            0 => None,
            1 => self.get_pattern_offset_by_byte_value_weights(),
            2 => self.get_pattern_offset_by_occurrence_weights(),
            _ => self.get_pattern_offset_by_similarity_weights(),
        };
        if result.is_none() && self.byte_value_groups.len() > 0 {
            result = Some(self.smallest_pattern_offset);
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "SignatureTable::get_most_significant_pattern_offset {{\n"
            ));
            if result.is_none() {
                self.mediator
                    .debug_print(format!("    most_significant_pattern_offset: N/A\n"));
            } else {
                self.mediator.debug_print(format!(
                    "    most_significant_pattern_offset: {}\n",
                    result.unwrap(),
                ));
            }
            self.mediator.debug_print(format!("}}\n\n"));
        }
        result
    }

    /// Retrieves the pattern offset for specific byte value weights.
    fn get_pattern_offset_by_byte_value_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "SignatureTable::get_pattern_offset_by_byte_value_weights {{\n"
            ));
            if self.byte_value_weights.largest_weight == 0 {
                self.mediator
                    .debug_print(format!("    largest_byte_value_weight: N/A\n"));
            } else {
                self.mediator.debug_print(format!(
                    "    largest_byte_value_weight: {}\n",
                    self.byte_value_weights.largest_weight
                ));
            }
            let number_of_offsets: usize = match self
                .byte_value_weights
                .offset_groups
                .get(&self.byte_value_weights.largest_weight)
            {
                Some(offset_group) => offset_group.offsets.len(),
                None => 0,
            };
            self.mediator
                .debug_print(format!("    number_of_offsets: {}\n", number_of_offsets,));
            self.mediator.debug_print(format!("}}\n\n"));
        }
        match self
            .byte_value_weights
            .offset_groups
            .get(&self.byte_value_weights.largest_weight)
        {
            Some(offset_group) => Some(offset_group.offsets[0]),
            None => None,
        }
    }

    /// Retrieves the pattern offset for specific occurrence weights.
    fn get_pattern_offset_by_occurrence_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "SignatureTable::get_pattern_offset_by_occurrence_weights {{\n"
            ));
            if self.occurrence_weights.largest_weight == 0 {
                self.mediator
                    .debug_print(format!("    largest_occurrence_weight: N/A\n"));
            } else {
                self.mediator.debug_print(format!(
                    "    largest_occurrence_weight: {}\n",
                    self.occurrence_weights.largest_weight
                ));
            }
            let number_of_offsets: usize = match self
                .occurrence_weights
                .offset_groups
                .get(&self.occurrence_weights.largest_weight)
            {
                Some(offset_group) => offset_group.offsets.len(),
                None => 0,
            };
            self.mediator
                .debug_print(format!("    number_of_offsets: {}\n", number_of_offsets));
        }
        match self
            .occurrence_weights
            .offset_groups
            .get(&self.occurrence_weights.largest_weight)
        {
            Some(offset_group) => {
                let mut largest_byte_value_weight: isize = 0;
                let mut pattern_offset: usize = 0;
                for (offset_index, occurrence_offset) in offset_group.offsets.iter().enumerate() {
                    let byte_value_weight: isize =
                        self.byte_value_weights.get_weight(occurrence_offset);

                    if offset_index == 0 || byte_value_weight > largest_byte_value_weight {
                        largest_byte_value_weight = byte_value_weight;
                        pattern_offset = *occurrence_offset;
                    }
                    if self.mediator.debug_output {
                        self.mediator
                            .debug_print(format!("    offset: {} {{\n", *occurrence_offset));
                        self.mediator.debug_print(format!(
                            "        byte_value_weight: {},\n",
                            byte_value_weight
                        ));
                        self.mediator.debug_print(format!("    }},\n"));
                    }
                }
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    pattern_offset: {},\n", pattern_offset));
                    self.mediator.debug_print(format!(
                        "    largest_byte_value_weight: {},\n",
                        largest_byte_value_weight
                    ));
                    self.mediator.debug_print(format!("}}\n\n"));
                }
                Some(pattern_offset)
            }
            None => {
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!("}}\n\n"));
                }
                self.get_pattern_offset_by_byte_value_weights()
            }
        }
    }

    /// Retrieves the pattern offset for specific similarity weights.
    fn get_pattern_offset_by_similarity_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "SignatureTable::get_pattern_offset_by_similarity_weights {{\n"
            ));
            if self.similarity_weights.largest_weight == 0 {
                self.mediator
                    .debug_print(format!("    largest_similarity_weight: N/A\n"));
            } else {
                self.mediator.debug_print(format!(
                    "    largest_similarity_weight: {}\n",
                    self.similarity_weights.largest_weight
                ));
            }
            let number_of_offsets: usize = match self
                .similarity_weights
                .offset_groups
                .get(&self.similarity_weights.largest_weight)
            {
                Some(offset_group) => offset_group.offsets.len(),
                None => 0,
            };
            self.mediator
                .debug_print(format!("    number_of_offsets: {}\n", number_of_offsets));
        }
        match self
            .similarity_weights
            .offset_groups
            .get(&self.similarity_weights.largest_weight)
        {
            Some(offset_group) => {
                let mut largest_byte_value_weight: isize = 0;
                let mut largest_occurrence_weight: isize = 0;
                let mut pattern_offset: usize = 0;

                for (offset_index, similarity_offset) in offset_group.offsets.iter().enumerate() {
                    let occurrence_weight: isize =
                        self.occurrence_weights.get_weight(similarity_offset);
                    let byte_value_weight: isize =
                        self.byte_value_weights.get_weight(similarity_offset);

                    if largest_occurrence_weight > 0
                        && occurrence_weight == largest_occurrence_weight
                    {
                        if byte_value_weight > largest_byte_value_weight {
                            largest_occurrence_weight = 0;
                        }
                    }
                    if offset_index == 0 || occurrence_weight > largest_occurrence_weight {
                        largest_byte_value_weight = byte_value_weight;
                        largest_occurrence_weight = occurrence_weight;
                        pattern_offset = *similarity_offset;
                    }
                    if self.mediator.debug_output {
                        self.mediator
                            .debug_print(format!("    offset: {} {{\n", *similarity_offset));
                        self.mediator.debug_print(format!(
                            "        occurrence_weight: {},\n",
                            occurrence_weight
                        ));
                        self.mediator.debug_print(format!(
                            "        byte_value_weight: {},\n",
                            byte_value_weight
                        ));
                        self.mediator.debug_print(format!("    }},\n"));
                    }
                }
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    pattern_offset: {},\n", pattern_offset));
                    self.mediator.debug_print(format!(
                        "    largest_occurrence_weight: {},\n",
                        largest_occurrence_weight
                    ));
                    self.mediator.debug_print(format!(
                        "    largest_byte_value_weight: {},\n",
                        largest_byte_value_weight
                    ));
                    self.mediator.debug_print(format!("}}\n\n"));
                }
                Some(pattern_offset)
            }
            None => {
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!("}}\n\n"));
                }
                self.get_pattern_offset_by_occurrence_weights()
            }
        }
    }

    /// Retrieves the signatures for a specific pattern offset.
    pub fn get_signatures_by_pattern_offset(
        &self,
        pattern_offset: usize,
    ) -> Vec<SignatureReference> {
        let mut signatures: Vec<SignatureReference> = Vec::new();

        match self.byte_value_groups.get(&pattern_offset) {
            Some(byte_value_group) => {
                for (_, signature_group) in byte_value_group.signature_groups.iter() {
                    for signature in signature_group.signatures.iter() {
                        if !signatures.contains(signature) {
                            signatures.push(Arc::clone(signature));
                        }
                    }
                }
            }
            None => {}
        }
        signatures
    }

    /// Inserts a signature for a specific offset and byte value.
    fn insert_signature(
        &mut self,
        pattern_offset: usize,
        byte_value: u8,
        signature: &SignatureReference,
    ) {
        match self.byte_value_groups.get_mut(&pattern_offset) {
            Some(byte_value_group) => byte_value_group.insert_signature(byte_value, signature),
            None => {
                self.smallest_pattern_offset = min(pattern_offset, self.smallest_pattern_offset);

                let mut byte_value_group: ByteValueGroup = ByteValueGroup::new(pattern_offset);
                byte_value_group.insert_signature(byte_value, signature);

                self.byte_value_groups
                    .insert(pattern_offset, byte_value_group);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signature::Signature;

    #[test]
    fn test_calculate_pattern_weights() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_pattern_weights();

        // TODO: check pattern weights.
    }

    #[test]
    fn test_fill() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(signature_table.byte_value_groups.len(), 0);
        assert_eq!(signature_table.signatures.len(), 0);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);

        assert_eq!(signature_table.byte_value_groups.len(), 8);
        assert_eq!(signature_table.signatures.len(), 1);
    }

    #[test]
    fn test_fill_with_offsets_to_ignore() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(signature_table.byte_value_groups.len(), 0);
        assert_eq!(signature_table.signatures.len(), 0);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = vec![1, 2, 3];
        signature_table.fill(&signatures, &offsets_to_ignore, 8);

        assert_eq!(signature_table.byte_value_groups.len(), 5);
        assert_eq!(signature_table.signatures.len(), 1);
    }

    // TODO: add tests for get_most_significant_pattern_offset
    // TODO: add tests for get_pattern_offset_by_byte_value_weights
    // TODO: add tests for get_pattern_offset_by_occurrence_weights
    // TODO: add tests for get_pattern_offset_by_similarity_weights
    // TODO: add tests for get_signatures_by_pattern_offset
    // TODO: add tests for insert_signature
}
