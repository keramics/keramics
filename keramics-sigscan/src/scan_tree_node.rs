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
            mediator.debug_print(String::from("ScanTreeNode::scan_buffer {\n"));

            let pattern_offset: u64 = data_offset + scan_offset as u64;
            mediator.debug_print(format!(
                "    scanning at offset: {} (0x{:08x}) for scan object: ",
                pattern_offset, pattern_offset
            ));
            match scan_object {
                Some(_) => {
                    if scan_object_key == Self::DEFAULT_SCAN_OBJECT {
                        mediator.debug_print(String::from("default\n"));
                    } else {
                        mediator.debug_print(format!("byte value: 0x{:02x}\n", scan_object_key));
                    }
                }
                None => mediator.debug_print(String::from("N/A\n")),
            };
            mediator.debug_print(String::from("}\n\n"));
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
}
