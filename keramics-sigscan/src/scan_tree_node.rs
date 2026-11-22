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

use keramics_core::ErrorTrace;
use keramics_core::mediator::{Mediator, MediatorReference};

use super::scan_object::ScanObject;
use super::scan_result::ScanResult;
use super::signature::Signature;
use super::signature_table::SignatureTable;

/// Signature scan tree node.
#[derive(Debug)]
pub(super) struct ScanTreeNode {
    /// Pattern offset.
    pub pattern_offset: usize,

    /// Scan objects.
    pub scan_objects: HashMap<i16, ScanObject>,
}

impl ScanTreeNode {
    const DEFAULT_SCAN_OBJECT: i16 = -1;

    /// Creates a new scan tree node.
    pub fn new() -> Self {
        Self {
            pattern_offset: 0,
            scan_objects: HashMap::new(),
        }
    }

    /// Builds a scan tree node.
    pub(super) fn build(
        &mut self,
        signature_table: &SignatureTable,
        offsets_to_ignore: &[usize],
        largest_pattern_offset: usize,
    ) -> Result<(), ErrorTrace> {
        self.pattern_offset = match signature_table.get_most_significant_pattern_offset() {
            Some(pattern_offset) => pattern_offset,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to determine most significant pattern offset"
                ));
            }
        };
        let signatures_in_node: Vec<Arc<Signature>> =
            signature_table.get_signatures_by_pattern_offset(self.pattern_offset);

        let mut remaining_signatures: Vec<Arc<Signature>> = Vec::new();
        for signature in signature_table.signatures.iter() {
            if !signatures_in_node.contains(signature) {
                remaining_signatures.push(Arc::clone(signature));
            }
        }
        let mut sub_offsets_to_ignore: Vec<usize> = offsets_to_ignore.to_vec();
        sub_offsets_to_ignore.push(self.pattern_offset);

        if let Some(byte_value_group) = signature_table.byte_value_groups.get(&self.pattern_offset)
        {
            for (group_index, (_, signature_group)) in
                byte_value_group.signature_groups.iter().enumerate()
            {
                let number_of_signatures: usize = signature_group.signatures.len();

                if number_of_signatures == 0 {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid byte value group for pattern offset: {} invalid signature group: {} missing signatures",
                        self.pattern_offset, group_index
                    )));
                }
                if number_of_signatures == 1 {
                    self.scan_objects.insert(
                        signature_group.byte_value as i16,
                        ScanObject::Signature(Arc::clone(&signature_group.signatures[0])),
                    );
                } else {
                    let mut sub_signature_table: SignatureTable =
                        SignatureTable::new(&signature_table.pattern_type);

                    sub_signature_table.fill(
                        &signature_group.signatures,
                        &sub_offsets_to_ignore,
                        largest_pattern_offset,
                    );
                    sub_signature_table.calculate_weights();

                    let mut sub_node: ScanTreeNode = ScanTreeNode::new();

                    match sub_node.build(
                        &sub_signature_table,
                        &sub_offsets_to_ignore,
                        largest_pattern_offset,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            error.add_frame(format!(
                                "Unable to build sub scan tree node for signature group: {} and pattern offset: {}",
                                group_index, self.pattern_offset
                            ));
                            return Err(error);
                        }
                    }
                    self.scan_objects.insert(
                        signature_group.byte_value as i16,
                        ScanObject::ScanTreeNode(sub_node),
                    );
                }
            }
        }
        let number_of_remaining_signatures: usize = remaining_signatures.len();

        if number_of_remaining_signatures == 1 {
            self.scan_objects.insert(
                Self::DEFAULT_SCAN_OBJECT,
                ScanObject::Signature(Arc::clone(&remaining_signatures[0])),
            );
        } else if number_of_remaining_signatures > 1 {
            let mut sub_signature_table: SignatureTable =
                SignatureTable::new(&signature_table.pattern_type);

            sub_signature_table.fill(
                &remaining_signatures,
                &sub_offsets_to_ignore,
                largest_pattern_offset,
            );
            sub_signature_table.calculate_weights();

            let mut sub_node: ScanTreeNode = ScanTreeNode::new();

            match sub_node.build(
                &sub_signature_table,
                &sub_offsets_to_ignore,
                largest_pattern_offset,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    error.add_frame(format!(
                        "Unable to build sub scan tree node for remaining signatures and pattern offset: {}",
                        largest_pattern_offset,
                    ));
                    return Err(error);
                }
            }
            self.scan_objects.insert(
                Self::DEFAULT_SCAN_OBJECT,
                ScanObject::ScanTreeNode(sub_node),
            );
        }
        Ok(())
    }

    /// Retrieves the default scan object.
    pub(super) fn get_default_scan_object(&self) -> Option<&ScanObject> {
        self.scan_objects.get(&Self::DEFAULT_SCAN_OBJECT)
    }

    /// Recursively retrieves the signatures.
    // Used for testing.
    #[cfg(test)]
    pub(super) fn get_signatures(&self) -> Vec<Arc<Signature>> {
        let mut signatures: Vec<Arc<Signature>> = Vec::new();

        for scan_object in self.scan_objects.values() {
            match scan_object {
                ScanObject::Signature(signature) => {
                    if !signatures.contains(signature) {
                        signatures.push(Arc::clone(signature));
                    }
                }
                ScanObject::ScanTreeNode(scan_tree_node) => {
                    for sub_signature in scan_tree_node.get_signatures() {
                        if !signatures.contains(&sub_signature) {
                            signatures.push(sub_signature);
                        }
                    }
                }
            }
        }
        signatures
    }

    /// Scans a buffer for a matching scan object.
    pub(super) fn scan_buffer(
        &self,
        data_offset: u64,
        data_size: u64,
        buffer: &[u8],
        buffer_offset: usize,
        buffer_size: usize,
    ) -> ScanResult<'_> {
        if data_offset >= data_size {
            return ScanResult::None;
        }
        let mediator: MediatorReference = Mediator::current();

        let scan_offset: usize = buffer_offset + self.pattern_offset;

        let mut scan_object_key: i16 = Self::DEFAULT_SCAN_OBJECT;

        // Note that if the pattern offset exceeds the (total) data size the scan continues with the default scan object.
        let mut scan_object: Option<&ScanObject> = None;

        if scan_offset < buffer_size && (scan_offset as u64) < data_size - data_offset {
            scan_object_key = buffer[scan_offset] as i16;
            scan_object = self.scan_objects.get(&scan_object_key);
        }
        if scan_object.is_none() {
            scan_object_key = Self::DEFAULT_SCAN_OBJECT;
            scan_object = self.scan_objects.get(&scan_object_key);
        }
        if mediator.debug_output {
            mediator.debug_print("ScanTreeNode::scan_buffer {\n");

            let pattern_offset: u64 = data_offset + scan_offset as u64;
            mediator.debug_print(format!(
                "    scanning at offset: {} (0x{:08x}) for scan object: ",
                pattern_offset, pattern_offset
            ));
            match scan_object {
                Some(_) => {
                    if scan_object_key == Self::DEFAULT_SCAN_OBJECT {
                        mediator.debug_print("default\n");
                    } else {
                        mediator.debug_print(format!("byte value: 0x{:02x}\n", scan_object_key));
                    }
                }
                None => mediator.debug_print("N/A\n"),
            };
            mediator.debug_print("}\n\n");
        }
        if let Some(ScanObject::Signature(signature)) = scan_object {
            if signature.scan_buffer(data_offset, data_size, buffer, buffer_offset, buffer_size) {
                return ScanResult::Signature(Arc::clone(signature));
            } else if scan_object_key == Self::DEFAULT_SCAN_OBJECT {
                return ScanResult::None;
            }
            scan_object = self.scan_objects.get(&Self::DEFAULT_SCAN_OBJECT);
        }
        match scan_object {
            Some(ScanObject::ScanTreeNode(scan_tree_node)) => {
                ScanResult::ScanTreeNode(scan_tree_node)
            }
            Some(ScanObject::Signature(signature)) => {
                if signature.scan_buffer(data_offset, data_size, buffer, buffer_offset, buffer_size)
                {
                    ScanResult::Signature(Arc::clone(signature))
                } else {
                    ScanResult::None
                }
            }
            None => ScanResult::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::PatternType;
    use crate::groups::SignatureGroup;
    use crate::signature::Signature;

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        assert_eq!(scan_tree_node.scan_objects.len(), 0);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        scan_tree_node.build(&signature_table, &offsets_to_ignore, 8)?;

        assert_eq!(scan_tree_node.scan_objects.len(), 1);

        Ok(())
    }

    #[test]
    fn test_build_with_empty_signature_table() {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let signatures: Vec<Arc<Signature>> = Vec::new();
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        let result: Result<(), ErrorTrace> =
            scan_tree_node.build(&signature_table, &offsets_to_ignore, 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_multiple_signatures() -> Result<(), ErrorTrace> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

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
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        scan_tree_node.build(&signature_table, &offsets_to_ignore, 8)?;

        // The signatures share the first 3 bytes, therefore the root node branches on the 4th
        // byte value and both branches resolve to a single signature.
        assert_eq!(scan_tree_node.pattern_offset, 3);
        assert_eq!(scan_tree_node.scan_objects.len(), 2);
        assert!(scan_tree_node.scan_objects.contains_key(&0x65));
        assert!(scan_tree_node.scan_objects.contains_key(&0x6e));

        for object in scan_tree_node.scan_objects.values() {
            match object {
                ScanObject::Signature(signature) => {
                    assert_eq!(signature.pattern_size, 8)
                }
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported scan object type"
                    ));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_scan_buffer() -> Result<(), ErrorTrace> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);

        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();
        scan_tree_node
            .build(&signature_table, &offsets_to_ignore, 8)
            .unwrap();

        let data: [u8; 8] = [0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78];

        let scan_result: ScanResult = scan_tree_node.scan_buffer(0, 64, &data, 0, 8);
        match scan_result {
            ScanResult::Signature(signature) => {
                assert_eq!(signature.identifier.as_str(), "vdh")
            }
            _ => {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported scan result type"
                ));
            }
        };

        // Test no match.
        let data: [u8; 8] = [0x78, 0x69, 0x74, 0x63, 0x65, 0x6e, 0x6f, 0x63];
        let scan_result: ScanResult = scan_tree_node.scan_buffer(0, 64, &data, 0, 8);
        assert!(matches!(scan_result, ScanResult::None));

        // Test data offset beyond data size.
        let scan_result: ScanResult = scan_tree_node.scan_buffer(64, 64, &data, 0, 8);
        assert!(matches!(scan_result, ScanResult::None));

        Ok(())
    }

    #[test]
    fn test_build_with_sub_nodes_and_get_signatures() -> Result<(), ErrorTrace> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let signatures: Vec<Arc<Signature>> = vec![
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
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        scan_tree_node.build(&signature_table, &offsets_to_ignore, 8)?;

        assert_eq!(scan_tree_node.pattern_offset, 0);
        assert!(matches!(
            scan_tree_node.scan_objects.get(&0x63),
            Some(ScanObject::ScanTreeNode(_))
        ));

        let collected_signatures: Vec<Arc<Signature>> = scan_tree_node.get_signatures();
        assert_eq!(collected_signatures.len(), 3);

        Ok(())
    }

    #[test]
    fn test_build_with_remaining_signatures() -> Result<(), ErrorTrace> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let signatures: Vec<Arc<Signature>> = vec![
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
                "sig3",
                PatternType::BoundToStart,
                16,
                "12345678".as_bytes(),
            )),
            Arc::new(Signature::new(
                "sig4",
                PatternType::BoundToStart,
                32,
                "abcdefgh".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 40);
        signature_table.calculate_weights();

        scan_tree_node.build(&signature_table, &offsets_to_ignore, 40)?;

        assert!(matches!(
            scan_tree_node
                .scan_objects
                .get(&ScanTreeNode::DEFAULT_SCAN_OBJECT),
            Some(ScanObject::ScanTreeNode(_))
        ));

        let collected_signatures: Vec<Arc<Signature>> = scan_tree_node.get_signatures();
        assert_eq!(collected_signatures.len(), 4);

        Ok(())
    }

    #[test]
    fn test_build_with_empty_signature_group() {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let signatures: Vec<Arc<Signature>> = vec![Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ))];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();

        signature_table
            .byte_value_groups
            .get_mut(&0)
            .unwrap()
            .signature_groups
            .insert(0x00, SignatureGroup::new(0x00));

        let result: Result<(), ErrorTrace> =
            scan_tree_node.build(&signature_table, &offsets_to_ignore, 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_duplicate_signatures() {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let signatures: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "dup1",
                PatternType::BoundToStart,
                0,
                "ab".as_bytes(),
            )),
            Arc::new(Signature::new(
                "dup2",
                PatternType::BoundToStart,
                0,
                "ab".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 2);
        signature_table.calculate_weights();

        let result: Result<(), ErrorTrace> =
            scan_tree_node.build(&signature_table, &offsets_to_ignore, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_unbuildable_remaining_signatures() {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        let signatures: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "sig1",
                PatternType::BoundToStart,
                0,
                "a".as_bytes(),
            )),
            Arc::new(Signature::new(
                "sig2",
                PatternType::BoundToStart,
                16,
                "b".as_bytes(),
            )),
            Arc::new(Signature::new(
                "sig3",
                PatternType::BoundToStart,
                16,
                "c".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = vec![16];
        signature_table.fill(&signatures, &offsets_to_ignore, 17);
        signature_table.calculate_weights();

        let result: Result<(), ErrorTrace> =
            scan_tree_node.build(&signature_table, &offsets_to_ignore, 17);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_buffer_with_sub_nodes_and_default_fallback() -> Result<(), ErrorTrace> {
        // Test returning a child ScanTreeNode.
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();
        let signatures: Vec<Arc<Signature>> = vec![
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
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_weights();
        scan_tree_node.build(&signature_table, &offsets_to_ignore, 8)?;

        let data: [u8; 8] = *b"conectix";
        let scan_result: ScanResult = scan_tree_node.scan_buffer(0, 64, &data, 0, 8);
        assert!(matches!(scan_result, ScanResult::ScanTreeNode(_)));

        // Test fallback to DEFAULT_SCAN_OBJECT signature when byte-value signature mismatches
        // and when DEFAULT_SCAN_OBJECT signature itself matches or mismatches.
        let mut fallback_node: ScanTreeNode = ScanTreeNode::new();
        let signatures: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "vhd0",
                PatternType::BoundToStart,
                0,
                "conectix".as_bytes(),
            )),
            Arc::new(Signature::new(
                "sig16",
                PatternType::BoundToStart,
                16,
                "12345678".as_bytes(),
            )),
        ];
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        signature_table.fill(&signatures, &offsets_to_ignore, 24);
        signature_table.calculate_weights();
        fallback_node.build(&signature_table, &offsets_to_ignore, 24)?;

        // Byte at offset 0 does not match 'c', falls back to DEFAULT_SCAN_OBJECT (sig16),
        // which does not match either.
        let data_no_match: [u8; 32] = [0x7a; 32];
        assert!(matches!(
            fallback_node.scan_buffer(0, 32, &data_no_match, 0, 32),
            ScanResult::None
        ));

        // Byte at offset 0 matches 'c', vhd0 fails full pattern match, falls back to
        // DEFAULT_SCAN_OBJECT (sig16), which matches at offset 16.
        let mut data_fallback_match: [u8; 32] = [0; 32];
        data_fallback_match[0] = b'c';
        data_fallback_match[16..24].copy_from_slice(b"12345678");
        assert!(matches!(
            fallback_node.scan_buffer(0, 32, &data_fallback_match, 0, 32),
            ScanResult::Signature(signature) if signature.identifier.as_str() == "sig16"
        ));

        // Byte at offset 0 matches 'c', vhd0 fails full pattern match, falls back to
        // DEFAULT_SCAN_OBJECT (sig16), which also fails.
        let mut data_fallback_mismatch: [u8; 32] = [0; 32];
        data_fallback_mismatch[0] = b'c';
        assert!(matches!(
            fallback_node.scan_buffer(0, 32, &data_fallback_mismatch, 0, 32),
            ScanResult::None
        ));

        // Uncommon-byte signature at a non-zero offset falls back to DEFAULT_SCAN_OBJECT.
        let mut uncommon_node: ScanTreeNode = ScanTreeNode::new();
        let uncommon_signatures: Vec<Arc<Signature>> = vec![Arc::new(Signature::new(
            "uncommon16",
            PatternType::BoundToStart,
            16,
            &[0x02, 0x03, 0x04, 0x05],
        ))];
        let mut uncommon_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        uncommon_table.fill(&uncommon_signatures, &offsets_to_ignore, 20);
        uncommon_table.calculate_weights();
        uncommon_node.build(&uncommon_table, &offsets_to_ignore, 20)?;

        let mut data_uncommon: [u8; 32] = [0; 32];
        data_uncommon[16..20].copy_from_slice(&[0x02, 0x03, 0x04, 0x05]);
        assert!(matches!(
            uncommon_node.scan_buffer(0, 32, &data_uncommon, 0, 32),
            ScanResult::Signature(signature) if signature.identifier.as_str() == "uncommon16"
        ));

        Ok(())
    }

    #[test]
    fn test_scan_buffer_with_debug_output() -> Result<(), ErrorTrace> {
        let original_debug_output: bool = Mediator::current().debug_output;
        Mediator::new(true).make_current();

        // Node without DEFAULT_SCAN_OBJECT (exercises byte value match and N/A).
        let mut node_without_default: ScanTreeNode = ScanTreeNode::new();
        let signatures1: Vec<Arc<Signature>> = vec![Arc::new(Signature::new(
            "vhd",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ))];
        let mut table1: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = Vec::new();
        table1.fill(&signatures1, &offsets_to_ignore, 8);
        table1.calculate_weights();
        node_without_default.build(&table1, &offsets_to_ignore, 8)?;

        let matching_data: [u8; 8] = *b"conectix";
        assert!(matches!(
            node_without_default.scan_buffer(0, 64, &matching_data, 0, 8),
            ScanResult::Signature(_)
        ));
        let non_matching_data: [u8; 8] = [0x00; 8];
        assert!(matches!(
            node_without_default.scan_buffer(0, 64, &non_matching_data, 0, 8),
            ScanResult::None
        ));

        // Node with DEFAULT_SCAN_OBJECT (exercises default scan object debug output).
        let mut node_with_default: ScanTreeNode = ScanTreeNode::new();
        let signatures2: Vec<Arc<Signature>> = vec![
            Arc::new(Signature::new(
                "vhd",
                PatternType::BoundToStart,
                0,
                "conectix".as_bytes(),
            )),
            Arc::new(Signature::new(
                "sig16",
                PatternType::BoundToStart,
                16,
                "12345678".as_bytes(),
            )),
        ];
        let mut table2: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        table2.fill(&signatures2, &offsets_to_ignore, 24);
        table2.calculate_weights();
        node_with_default.build(&table2, &offsets_to_ignore, 24)?;

        let mut data_default: [u8; 32] = [0; 32];
        data_default[16..24].copy_from_slice(b"12345678");
        assert!(matches!(
            node_with_default.scan_buffer(0, 32, &data_default, 0, 32),
            ScanResult::Signature(_)
        ));

        Mediator::new(original_debug_output).make_current();

        Ok(())
    }
}
