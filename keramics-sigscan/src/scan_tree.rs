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

use std::cmp::{max, min};
use std::sync::Arc;

use keramics_core::ErrorTrace;

use super::enums::PatternType;
use super::scan_tree_node::ScanTreeNode;
use super::signature::Signature;
use super::signature_table::SignatureTable;
use super::skip_table::SkipTable;
use crate::scan_result::ScanResult;

/// Signature scan tree.
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
            pattern_type,
            range_start_offset: 0,
            range_end_offset: 0,
            root_node: ScanTreeNode::new(),
            skip_table: SkipTable::new(),
        }
    }

    /// Builds the scan tree.
    pub fn build(&mut self, signatures: &[Arc<Signature>]) -> Result<(), ErrorTrace> {
        for signature in signatures.iter() {
            if signature.pattern_type != self.pattern_type {
                continue;
            }
            self.range_start_offset = min(signature.pattern_offset, self.range_start_offset);
            self.range_end_offset = max(
                signature.pattern_offset + signature.pattern_size,
                self.range_end_offset,
            );
        }
        let mut signature_table: SignatureTable = SignatureTable::new(&self.pattern_type);

        let offsets_to_ignore: Vec<usize> = Vec::new();
        signature_table.fill(signatures, &offsets_to_ignore, self.range_end_offset);

        if !signature_table.is_empty() {
            signature_table.calculate_weights();

            match self
                .root_node
                .build(&signature_table, &offsets_to_ignore, self.range_end_offset)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to build root node");
                    return Err(error);
                }
            }
            self.skip_table.fill(signatures);
        }
        Ok(())
    }

    /// Retrieves the spanning range.
    pub fn get_spanning_range(&self) -> (usize, usize) {
        (self.range_start_offset, self.range_end_offset)
    }

    /// Scans a buffer for a matching scan object.
    #[allow(dead_code)]
    pub(super) fn scan_buffer(
        &self,
        data_offset: u64,
        data_size: u64,
        buffer: &[u8],
        buffer_offset: usize,
        buffer_size: usize,
    ) -> ScanResult<'_> {
        self.root_node
            .scan_buffer(data_offset, data_size, buffer, buffer_offset, buffer_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use crate::signature::Signature;

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToStart);

        assert_eq!(scan_tree.root_node.scan_objects.len(), 0);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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

    #[test]
    fn test_build_without_signatures() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToStart);

        assert_eq!(scan_tree.root_node.scan_objects.len(), 0);

        let signatures: Vec<Arc<Signature>> = Vec::new();
        scan_tree.build(&signatures)?;

        assert_eq!(scan_tree.root_node.scan_objects.len(), 0);

        Ok(())
    }

    // TODO: add tests for scan_tree get_spanning_range

    #[test]
    fn test_scan_buffer_with_bound_to_start_signature() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToStart);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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
        let scan_result: ScanResult = scan_tree.scan_buffer(0, 128, &data, 0, 128);
        match scan_result {
            ScanResult::Signature(signature) => {
                assert_eq!(signature.identifier.as_str(), "msiecf1")
            }
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }

    #[test]
    fn test_scan_buffer_with_bound_to_end_signature() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::BoundToEnd);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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
        let scan_result: ScanResult = scan_tree.scan_buffer(0, 128, &data, 48, 128);
        match scan_result {
            ScanResult::Signature(signature) => assert_eq!(signature.identifier.as_str(), "vhd1"),
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }

    #[test]
    fn test_scan_buffer_with_unbound_signature() -> Result<(), ErrorTrace> {
        let mut scan_tree: ScanTree = ScanTree::new(PatternType::Unbound);

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
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
        let scan_result: ScanResult = scan_tree.scan_buffer(0, 128, &data, 15, 128);
        match scan_result {
            ScanResult::Signature(signature) => assert_eq!(signature.identifier.as_str(), "test1"),
            _ => panic!("Incorrect scan result type"),
        }
        Ok(())
    }
}
