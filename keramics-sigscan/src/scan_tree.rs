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

use std::cmp::{max, min};
use std::collections::HashMap;
use std::sync::Arc;

use keramics_core::mediator::{Mediator, MediatorReference};

use super::enums::PatternType;
use super::errors::BuildError;
use super::scan_result::ScanResult;
use super::signature_table::SignatureTable;
use super::skip_table::SkipTable;
use super::types::SignatureReference;

const DEFAULT_SCAN_OBJECT: i16 = -1;

/// Scan object.
#[derive(Debug)]
pub enum ScanObject {
    ScanTreeNode(ScanTreeNode),
    Signature(SignatureReference),
}

/// Scan tree node.
#[derive(Debug)]
pub(super) struct ScanTreeNode {
    /// Pattern offset.
    pub pattern_offset: usize,

    /// Scan objects.
    pub scan_objects: HashMap<i16, ScanObject>,
}

impl ScanTreeNode {
    /// Creates a new scan tree node.
    pub fn new() -> Self {
        Self {
            pattern_offset: 0,
            scan_objects: HashMap::new(),
        }
    }

    /// Builds a scan tree node.
    fn build(
        &mut self,
        signature_table: &SignatureTable,
        offsets_to_ignore: &Vec<usize>,
        largest_pattern_offset: usize,
    ) -> Result<(), BuildError> {
        self.pattern_offset = match signature_table.get_most_significant_pattern_offset() {
            Some(pattern_offset) => pattern_offset,
            None => {
                return Err(BuildError::new(format!(
                    "Unable to determine most significant pattern offset"
                )));
            }
        };
        let signatures_in_node: Vec<SignatureReference> =
            signature_table.get_signatures_by_pattern_offset(self.pattern_offset);

        let mut remaining_signatures: Vec<SignatureReference> = Vec::new();
        for signature in signature_table.signatures.iter() {
            if !signatures_in_node.contains(signature) {
                remaining_signatures.push(Arc::clone(signature));
            }
        }
        let mut sub_offsets_to_ignore: Vec<usize> = offsets_to_ignore.clone();
        sub_offsets_to_ignore.push(self.pattern_offset);

        match signature_table.byte_value_groups.get(&self.pattern_offset) {
            Some(byte_value_group) => {
                for (group_index, (_, signature_group)) in
                    byte_value_group.signature_groups.iter().enumerate()
                {
                    let number_of_signatures: usize = signature_group.signatures.len();

                    if number_of_signatures == 0 {
                        return Err(BuildError::new(format!(
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
                        sub_signature_table.fill(
                            &remaining_signatures,
                            &sub_offsets_to_ignore,
                            largest_pattern_offset,
                        );
                        sub_signature_table.calculate_pattern_weights();

                        let mut sub_node: ScanTreeNode = ScanTreeNode::new();

                        sub_node.build(
                            &sub_signature_table,
                            &sub_offsets_to_ignore,
                            largest_pattern_offset,
                        )?;
                        self.scan_objects.insert(
                            signature_group.byte_value as i16,
                            ScanObject::ScanTreeNode(sub_node),
                        );
                    }
                }
            }
            None => {}
        };
        let number_of_remaining_signatures: usize = remaining_signatures.len();

        if number_of_remaining_signatures == 1 {
            self.scan_objects.insert(
                DEFAULT_SCAN_OBJECT,
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
            sub_signature_table.calculate_pattern_weights();

            let mut sub_node: ScanTreeNode = ScanTreeNode::new();

            sub_node.build(
                &sub_signature_table,
                &sub_offsets_to_ignore,
                largest_pattern_offset,
            )?;
            self.scan_objects
                .insert(DEFAULT_SCAN_OBJECT, ScanObject::ScanTreeNode(sub_node));
        }
        Ok(())
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

        let mut scan_object_key: i16 = DEFAULT_SCAN_OBJECT;

        // Note that if the pattern offset exceeds the (total) data size the scan continues with the default scan object.
        let mut scan_object: Option<&ScanObject> = None;

        if scan_offset < buffer_size && (scan_offset as u64) < data_size - data_offset {
            scan_object_key = buffer[scan_offset] as i16;
            scan_object = self.scan_objects.get(&scan_object_key);
        }
        if scan_object.is_none() {
            scan_object_key = DEFAULT_SCAN_OBJECT;
            scan_object = self.scan_objects.get(&scan_object_key);
        }
        if mediator.debug_output {
            mediator.debug_print(format!("ScanTreeNode::scan_buffer {{\n"));
            let pattern_offset: u64 = data_offset + scan_offset as u64;
            mediator.debug_print(format!(
                "    scanning at offset: {} (0x{:08x}) for scan object: ",
                pattern_offset, pattern_offset
            ));
            match scan_object {
                Some(_) => {
                    if scan_object_key == DEFAULT_SCAN_OBJECT {
                        mediator.debug_print(format!("default\n"));
                    } else {
                        mediator.debug_print(format!("byte value: 0x{:02x}\n", scan_object_key));
                    }
                }
                None => mediator.debug_print(format!("N/A\n")),
            };
            mediator.debug_print(format!("}}\n\n"));
        }
        if let Some(ScanObject::Signature(signature)) = scan_object {
            if signature.scan_buffer(data_offset, data_size, buffer, buffer_offset, buffer_size) {
                return ScanResult::Signature(Arc::clone(signature));
            } else if scan_object_key == DEFAULT_SCAN_OBJECT {
                return ScanResult::None;
            }
            scan_object = self.scan_objects.get(&DEFAULT_SCAN_OBJECT);
        }
        match scan_object {
            Some(scan_object) => match scan_object {
                ScanObject::ScanTreeNode(scan_tree_node) => {
                    ScanResult::ScanTreeNode(&scan_tree_node)
                }
                ScanObject::Signature(signature) => {
                    if signature.scan_buffer(
                        data_offset,
                        data_size,
                        buffer,
                        buffer_offset,
                        buffer_size,
                    ) {
                        ScanResult::Signature(Arc::clone(signature))
                    } else {
                        ScanResult::None
                    }
                }
            },
            None => ScanResult::None,
        }
    }
}

/// Scan tree.
#[derive(Debug)]
pub(super) struct ScanTree {
    /// Pattern type.
    pub pattern_type: PatternType,

    /// Pattern range start offset.
    range_start_offset: usize,

    /// Pattern range end offset.
    range_end_offset: usize,

    /// Root node.
    pub root_node: ScanTreeNode,

    /// Skip table.
    pub skip_table: SkipTable,
}

impl ScanTree {
    /// Creates a new scan tree.
    pub fn new(pattern_type: PatternType) -> Self {
        Self {
            pattern_type: pattern_type,
            range_start_offset: 0,
            range_end_offset: 0,
            root_node: ScanTreeNode::new(),
            skip_table: SkipTable::new(),
        }
    }

    /// Builds the scan tree.
    pub fn build(&mut self, signatures: &Vec<SignatureReference>) -> Result<(), BuildError> {
        let mut number_of_signatures: usize = 0;
        for signature in signatures.iter() {
            if signature.pattern_type != self.pattern_type {
                continue;
            }
            number_of_signatures += 1;

            self.range_start_offset = min(signature.pattern_offset, self.range_start_offset);
            let pattern_end_offset: usize = signature.pattern_offset + signature.pattern_size;
            self.range_end_offset = max(pattern_end_offset, self.range_end_offset);
        }
        if number_of_signatures > 0 {
            let mut signature_table: SignatureTable = SignatureTable::new(&self.pattern_type);
            let offsets_to_ignore: Vec<usize> = Vec::new();
            signature_table.fill(&signatures, &offsets_to_ignore, self.range_end_offset);
            signature_table.calculate_pattern_weights();

            self.root_node
                .build(&signature_table, &offsets_to_ignore, self.range_end_offset)?;
            self.skip_table.fill(&signatures);
        }
        Ok(())
    }

    /// Retrieves the spanning range.
    pub fn get_spanning_range(&self) -> (usize, usize) {
        (self.range_start_offset, self.range_end_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signature::Signature;

    #[test]
    fn test_scan_tree_node_build() -> Result<(), BuildError> {
        let mut scan_tree_node: ScanTreeNode = ScanTreeNode::new();

        assert_eq!(scan_tree_node.scan_objects.len(), 0);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        let mut signature_table: SignatureTable = SignatureTable::new(&PatternType::BoundToStart);
        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(&signatures, &offsets_to_ignore, 8);
        signature_table.calculate_pattern_weights();

        scan_tree_node.build(&signature_table, &offsets_to_ignore, 8)?;

        assert_eq!(scan_tree_node.scan_objects.len(), 1);

        Ok(())
    }

    #[test]
    fn test_scan_tree_build() -> Result<(), BuildError> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToStart);

        assert_eq!(scan_tree.root_node.scan_objects.len(), 0);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        )));
        scan_tree.build(&signatures)?;

        assert_eq!(scan_tree.root_node.scan_objects.len(), 1);

        Ok(())
    }

    // TODO: add tests for scan_tree get_spanning_range

    #[test]
    fn test_scan_tree_node_scan_buffer_with_bound_to_start_signature() -> Result<(), BuildError> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToStart);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "msiecf1",
            PatternType::BoundToStart,
            0,
            "Client UrlCache MMF Ver ".as_bytes(),
        )));
        scan_tree.build(&signatures)?;

        let data: [u8; 128] = [
            0x43, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x20, 0x55, 0x72, 0x6c, 0x43, 0x61, 0x63, 0x68,
            0x65, 0x20, 0x4d, 0x4d, 0x46, 0x20, 0x56, 0x65, 0x72, 0x20, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let scan_result: ScanResult = scan_tree.root_node.scan_buffer(0, 128, &data, 0, 128);
        match scan_result {
            ScanResult::Signature(signature) => {
                assert_eq!(signature.identifier.as_str(), "msiecf1")
            }
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }

    #[test]
    fn test_scan_tree_node_scan_buffer_with_bound_to_end_signature() -> Result<(), BuildError> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToEnd);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "vhd1",
            PatternType::BoundToEnd,
            72,
            "conectix".as_bytes(),
        )));
        scan_tree.build(&signatures)?;

        let data: [u8; 128] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let scan_result: ScanResult = scan_tree.root_node.scan_buffer(0, 128, &data, 48, 128);
        match scan_result {
            ScanResult::Signature(signature) => assert_eq!(signature.identifier.as_str(), "vhd1"),
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }

    #[test]
    fn test_scan_tree_node_scan_buffer_with_unbound_signature() -> Result<(), BuildError> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::Unbound);

        let mut signatures: Vec<SignatureReference> = Vec::new();
        signatures.push(Arc::new(Signature::new(
            "test1",
            PatternType::Unbound,
            0,
            "example of unbounded pattern".as_bytes(),
        )));
        scan_tree.build(&signatures)?;

        let data: [u8; 128] = [
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x20, 0x6f, 0x66, 0x20, 0x75, 0x6e,
            0x62, 0x6f, 0x75, 0x6e, 0x64, 0x65, 0x64, 0x20, 0x70, 0x61, 0x74, 0x74, 0x65, 0x72,
            0x6e, 0x20, 0x20, 0x20, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20,
        ];
        let scan_result: ScanResult = scan_tree.root_node.scan_buffer(0, 128, &data, 15, 128);
        match scan_result {
            ScanResult::Signature(signature) => assert_eq!(signature.identifier.as_str(), "test1"),
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }
}
