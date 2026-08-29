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

use keramics_core::mediator::{Mediator, MediatorReference};

use super::constants::*;
use super::enums::PatternType;
use super::groups::ByteValueGroup;
use super::pattern_weights::PatternWeights;
use super::signature::Signature;

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
    pub signatures: Vec<Arc<Signature>>,

    /// Byte value weights.
    byte_value_weights: PatternWeights,

    /// Occurrence weights.
    occurrence_weights: PatternWeights,

    /// Similarity weights.
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

    /// Calculates the weights.
    pub fn calculate_weights(&mut self) {
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
        signatures: &[Arc<Signature>],
        offsets_to_ignore: &[usize],
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
            self.signatures.push(Arc::clone(signature));

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
        if result.is_none() && !self.byte_value_groups.is_empty() {
            result = Some(self.smallest_pattern_offset);
        }
        if self.mediator.debug_output {
            self.mediator
                .debug_print("SignatureTable::get_most_significant_pattern_offset {\n");
            if result.is_none() {
                self.mediator
                    .debug_print("    most_significant_pattern_offset: N/A\n");
            } else {
                self.mediator.debug_print(format!(
                    "    most_significant_pattern_offset: {}\n",
                    result.unwrap(),
                ));
            }
            self.mediator.debug_print("}\n\n");
        }
        result
    }

    /// Retrieves the pattern offset based on the byte value weights.
    fn get_pattern_offset_by_byte_value_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator
                .debug_print("SignatureTable::get_pattern_offset_by_byte_value_weights {\n");
            if self.byte_value_weights.largest_weight == 0 {
                self.mediator
                    .debug_print("    largest_byte_value_weight: N/A\n");
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
                .debug_print(format!("    number_of_offsets: {}\n", number_of_offsets));
            self.mediator.debug_print("}\n\n");
        }
        self.byte_value_weights
            .offset_groups
            .get(&self.byte_value_weights.largest_weight)
            .map(|offset_group| offset_group.offsets[0])
    }

    /// Retrieves the pattern offset based on the occurrence weights.
    fn get_pattern_offset_by_occurrence_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator
                .debug_print("SignatureTable::get_pattern_offset_by_occurrence_weights {\n");
            if self.occurrence_weights.largest_weight == 0 {
                self.mediator
                    .debug_print("    largest_occurrence_weight: N/A\n");
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
                for (group_index, occurrence_offset) in offset_group.offsets.iter().enumerate() {
                    let byte_value_weight: isize =
                        self.byte_value_weights.get_weight(occurrence_offset);

                    if group_index == 0 || byte_value_weight > largest_byte_value_weight {
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
                        self.mediator.debug_print("    },\n");
                    }
                }
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(format!("    pattern_offset: {},\n", pattern_offset));
                    self.mediator.debug_print(format!(
                        "    largest_byte_value_weight: {},\n",
                        largest_byte_value_weight
                    ));
                    self.mediator.debug_print("}\n\n");
                }
                Some(pattern_offset)
            }
            None => {
                if self.mediator.debug_output {
                    self.mediator.debug_print("}\n\n");
                }
                self.get_pattern_offset_by_byte_value_weights()
            }
        }
    }

    /// Retrieves the pattern offset based on the similarity weights.
    fn get_pattern_offset_by_similarity_weights(&self) -> Option<usize> {
        if self.mediator.debug_output {
            self.mediator
                .debug_print("SignatureTable::get_pattern_offset_by_similarity_weights {\n");
            if self.similarity_weights.largest_weight == 0 {
                self.mediator
                    .debug_print("    largest_similarity_weight: N/A\n");
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

                for (group_index, similarity_offset) in offset_group.offsets.iter().enumerate() {
                    let occurrence_weight: isize =
                        self.occurrence_weights.get_weight(similarity_offset);
                    let byte_value_weight: isize =
                        self.byte_value_weights.get_weight(similarity_offset);

                    if largest_occurrence_weight > 0
                        && occurrence_weight == largest_occurrence_weight
                        && byte_value_weight > largest_byte_value_weight
                    {
                        largest_occurrence_weight = 0;
                    }
                    if group_index == 0 || occurrence_weight > largest_occurrence_weight {
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
                        self.mediator.debug_print("    },\n");
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
                    self.mediator.debug_print("}\n\n");
                }
                Some(pattern_offset)
            }
            None => {
                if self.mediator.debug_output {
                    self.mediator.debug_print("}\n\n");
                }
                self.get_pattern_offset_by_occurrence_weights()
            }
        }
    }

    /// Retrieves the signatures for a specific pattern offset.
    pub fn get_signatures_by_pattern_offset(&self, pattern_offset: usize) -> Vec<Arc<Signature>> {
        let mut signatures: Vec<Arc<Signature>> = Vec::new();

        if let Some(byte_value_group) = self.byte_value_groups.get(&pattern_offset) {
            for (_, signature_group) in byte_value_group.signature_groups.iter() {
                for signature in signature_group.signatures.iter() {
                    if !signatures.contains(signature) {
                        signatures.push(Arc::clone(signature));
                    }
                }
            }
        }
        signatures
    }

    /// Inserts a signature for a specific offset and byte value.
    fn insert_signature(
        &mut self,
        pattern_offset: usize,
        byte_value: u8,
        signature: &Arc<Signature>,
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
        }
    }

    /// Determines if the signature table is empty.
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signature::Signature;

    #[test]
    fn test_calculate_weights() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);

        assert!(signature_table.byte_value_weights.offset_groups.is_empty());
        assert!(signature_table.occurrence_weights.offset_groups.is_empty());
        assert!(signature_table.similarity_weights.offset_groups.is_empty());

        signature_table.calculate_weights();

        // "conectix" contains only common byte values, no offsets share a signature or signature group.
        for pattern_offset in 0..8 {
            assert_eq!(
                signature_table
                    .byte_value_weights
                    .get_weight(&pattern_offset),
                1
            );
        }
        assert!(signature_table.occurrence_weights.offset_groups.is_empty());
        assert!(signature_table.similarity_weights.offset_groups.is_empty());
    }

    #[test]
    fn test_calculate_weights_with_two_signatures() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        signatures.push(Arc::new(Signature::new(
            "vhd2",
            PatternType::BoundToStart,
            0,
            "connectx".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        // Offsets 3 through 6 contain different byte values in both signatures.
        assert_eq!(signature_table.occurrence_weights.get_weight(&3), 2);
        assert_eq!(signature_table.occurrence_weights.get_weight(&6), 2);
        assert_eq!(signature_table.occurrence_weights.largest_weight, 2);
        assert_eq!(signature_table.similarity_weights.get_weight(&0), 2);
        assert_eq!(signature_table.similarity_weights.get_weight(&7), 2);
        assert_eq!(signature_table.similarity_weights.largest_weight, 2);
        for pattern_offset in 0..8 {
            assert_eq!(
                signature_table
                    .byte_value_weights
                    .get_weight(&pattern_offset),
                1
            );
        }
    }

    #[test]
    fn test_calculate_weights_with_three_signatures() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        signatures.push(Arc::new(Signature::new(
            "vhd2",
            PatternType::BoundToStart,
            0,
            "connectx".as_bytes(),
        )));
        signatures.push(Arc::new(Signature::new(
            "vhd3",
            PatternType::BoundToStart,
            0,
            "conectiz".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        // Offsets 0, 1 and 2 have the same byte value in all three signatures.
        assert_eq!(signature_table.similarity_weights.get_weight(&0), 3);
        assert_eq!(signature_table.similarity_weights.get_weight(&1), 3);
        assert_eq!(signature_table.similarity_weights.largest_weight, 3);
        assert_eq!(signature_table.occurrence_weights.get_weight(&3), 2);
        assert_eq!(signature_table.occurrence_weights.get_weight(&7), 2);
        assert_eq!(signature_table.occurrence_weights.largest_weight, 2);
    }

    #[test]
    fn test_fill() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(signature_table.byte_value_groups.len(), 0);
        assert_eq!(signature_table.signatures.len(), 0);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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

    #[test]
    fn test_get_most_significant_pattern_offset() {
        let offsets_to_ignore: Vec<usize> = Vec::new();

        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        assert_eq!(signature_table.get_most_significant_pattern_offset(), None);

        let empty_signatures: Vec<Arc<Signature>> = Vec::new();
        signature_table.fill(&empty_signatures, &offsets_to_ignore, 8);
        assert_eq!(signature_table.get_most_significant_pattern_offset(), None);

        let signatures1: Vec<Arc<Signature>> = vec![Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ))];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        signature_table.fill(&signatures1, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        // A single signature uses the byte value weights, offset 0 is added first.
        assert_eq!(
            signature_table.get_most_significant_pattern_offset(),
            Some(0)
        );

        let signatures2: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "vhd1",
                PatternType::BoundToStart,
                0,
                "conectix".as_bytes(),
            )),
            Arc::new(Signature::new(
                "vhd2",
                PatternType::BoundToStart,
                0,
                "connectx".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        signature_table.fill(&signatures2, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        // Two signatures use the occurrence weights, offset 3 is added first.
        assert_eq!(
            signature_table.get_most_significant_pattern_offset(),
            Some(3)
        );

        let signatures3: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "vhd1",
                PatternType::BoundToStart,
                0,
                "conectix".as_bytes(),
            )),
            Arc::new(Signature::new(
                "vhd2",
                PatternType::BoundToStart,
                0,
                "connectx".as_bytes(),
            )),
            Arc::new(Signature::new(
                "vhd3",
                PatternType::BoundToStart,
                0,
                "conectiz".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        signature_table.fill(&signatures3, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        // Three or more signatures use the similarity weights, offset 0 is added first.
        assert_eq!(
            signature_table.get_most_significant_pattern_offset(),
            Some(0)
        );
    }

    #[test]
    fn test_get_pattern_offset_by_byte_value_weights() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(
            signature_table.get_pattern_offset_by_byte_value_weights(),
            None
        );

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        assert_eq!(signature_table.byte_value_weights.largest_weight, 1);
        assert_eq!(
            signature_table.get_pattern_offset_by_byte_value_weights(),
            Some(0)
        );
    }

    #[test]
    fn test_get_pattern_offset_by_occurrence_weights() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(
            signature_table.get_pattern_offset_by_occurrence_weights(),
            None
        );

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        signatures.push(Arc::new(Signature::new(
            "vhd2",
            PatternType::BoundToStart,
            0,
            "connectx".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        assert_eq!(signature_table.occurrence_weights.largest_weight, 2);
        assert_eq!(
            signature_table.get_pattern_offset_by_occurrence_weights(),
            Some(3)
        );
    }

    #[test]
    fn test_get_pattern_offset_by_similarity_weights() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(
            signature_table.get_pattern_offset_by_similarity_weights(),
            None
        );

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        signatures.push(Arc::new(Signature::new(
            "vhd2",
            PatternType::BoundToStart,
            0,
            "connectx".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        assert_eq!(signature_table.similarity_weights.largest_weight, 2);
        assert_eq!(
            signature_table.get_pattern_offset_by_similarity_weights(),
            Some(0)
        );
    }

    #[test]
    fn test_get_signatures_by_pattern_offset() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        signature_table.insert_signature(0, 0x63, &signatures[0]);

        assert_eq!(signature_table.get_signatures_by_pattern_offset(1).len(), 0);

        let table_signatures: Vec<Arc<Signature>> =
            signature_table.get_signatures_by_pattern_offset(0);
        assert_eq!(table_signatures.len(), 1);
        assert_eq!(table_signatures[0].identifier.as_str(), "vdh");
    }

    #[test]
    fn test_insert_signature() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert_eq!(signature_table.byte_value_groups.len(), 0);
        assert!(signature_table.byte_value_groups.get(&0).is_none());

        let signature: Arc<Signature> = Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        signature_table.insert_signature(0, 0x63, &signature);

        let byte_value_group: &ByteValueGroup = signature_table.byte_value_groups.get(&0).unwrap();
        assert_eq!(byte_value_group.pattern_offset, 0);
        assert_eq!(byte_value_group.signature_groups.len(), 1);
        assert_eq!(
            byte_value_group
                .signature_groups
                .get(&0x63)
                .unwrap()
                .signatures
                .len(),
            1
        );

        let signature_other: Arc<Signature> = Arc::new(Signature::new(
            "vdh2",
            PatternType::BoundToStart,
            0,
            "connectx".as_bytes(),
        ));
        signature_table.insert_signature(0, 0x63, &signature_other);

        let byte_value_group: &ByteValueGroup = signature_table.byte_value_groups.get(&0).unwrap();
        assert_eq!(byte_value_group.signature_groups.len(), 1);
        assert_eq!(
            byte_value_group
                .signature_groups
                .get(&0x63)
                .unwrap()
                .signatures
                .len(),
            2
        );

        signature_table.insert_signature(1, 0x6f, &signature);

        assert_eq!(signature_table.byte_value_groups.len(), 2);
        let byte_value_group: &ByteValueGroup = signature_table.byte_value_groups.get(&1).unwrap();
        assert_eq!(byte_value_group.pattern_offset, 1);
        assert_eq!(
            byte_value_group
                .signature_groups
                .get(&0x6f)
                .unwrap()
                .signatures
                .len(),
            1
        );
    }

    #[test]
    fn test_is_empty() {
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        assert!(signature_table.is_empty());

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);

        assert!(!signature_table.is_empty());
    }
}
