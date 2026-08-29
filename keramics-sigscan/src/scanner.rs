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

use std::sync::Arc;

use keramics_core::ErrorTrace;

use super::enums::PatternType;
use super::scan_tree::ScanTree;
use super::signature::Signature;

/// Signature scanner.
pub struct Scanner {
    /// Signatures.
    pub(super) signatures: Vec<Arc<Signature>>,

    /// Header (offset relative from start) scan tree.
    pub(super) header_scan_tree: ScanTree,

    /// Footer (offset relative from end) scan tree.
    pub(super) footer_scan_tree: ScanTree,

    /// Unbound scan tree.
    pub(super) unbound_scan_tree: ScanTree,
}

impl Scanner {
    /// Creates a new scanner.
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            header_scan_tree: ScanTree::new(PatternType::BoundToStart),
            footer_scan_tree: ScanTree::new(PatternType::BoundToEnd),
            unbound_scan_tree: ScanTree::new(PatternType::Unbound),
        }
    }

    /// Adds a new signature.
    pub fn add_signature(&mut self, signature: Signature) {
        self.signatures.push(Arc::new(signature));
    }

    /// Builds the scan trees.
    pub fn build(&mut self) -> Result<(), ErrorTrace> {
        match self.header_scan_tree.build(&self.signatures) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build header scan tree");
                return Err(error);
            }
        }
        match self.footer_scan_tree.build(&self.signatures) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build footer scan tree");
                return Err(error);
            }
        }
        match self.unbound_scan_tree.build(&self.signatures) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to build unbound scan tree");
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::scan_context::ScanContext;

    #[test]
    fn test_add_signature() {
        let mut scanner: Scanner = Scanner::new();

        assert_eq!(scanner.signatures.len(), 0);

        scanner.add_signature(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        assert_eq!(scanner.signatures.len(), 1);
    }

    #[test]
    fn test_build() -> Result<(), ErrorTrace> {
        let mut scanner: Scanner = Scanner::new();

        scanner.add_signature(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        scanner.build()?;

        assert_eq!(scanner.signatures.len(), 1);

        Ok(())
    }

    #[test]
    fn test_build_with_all_pattern_types() -> Result<(), ErrorTrace> {
        let mut scanner: Scanner = Scanner::new();

        scanner.add_signature(Signature::new(
            "vhd_header",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        scanner.add_signature(Signature::new(
            "vhd_footer",
            PatternType::BoundToEnd,
            72,
            "conectix".as_bytes(),
        ));
        scanner.add_signature(Signature::new(
            "vhd_unbound",
            PatternType::Unbound,
            0,
            "conectix".as_bytes(),
        ));
        scanner.build()?;

        assert_eq!(scanner.signatures.len(), 3);

        let result: bool = scanner
            .header_scan_tree
            .root_node
            .scan_objects
            .contains_key(&0x63);
        assert_eq!(result, true);

        let result: bool = scanner
            .footer_scan_tree
            .root_node
            .scan_objects
            .contains_key(&0x63);
        assert_eq!(result, true);

        let result: bool = scanner
            .unbound_scan_tree
            .root_node
            .scan_objects
            .contains_key(&0x63);
        assert_eq!(result, true);

        Ok(())
    }

    #[test]
    fn test_build_with_no_signatures() -> Result<(), ErrorTrace> {
        let mut scanner: Scanner = Scanner::new();

        scanner.build()?;

        assert_eq!(scanner.signatures.len(), 0);
        assert!(scanner.header_scan_tree.root_node.scan_objects.is_empty());
        assert!(scanner.footer_scan_tree.root_node.scan_objects.is_empty());
        assert!(scanner.unbound_scan_tree.root_node.scan_objects.is_empty());

        Ok(())
    }

    #[test]
    fn test_scan_buffer_with_all_pattern_types() {
        let mut scanner: Scanner = Scanner::new();

        scanner.add_signature(Signature::new(
            "msiecf1",
            PatternType::BoundToStart,
            0,
            "Client UrlCache MMF Ver ".as_bytes(),
        ));
        scanner.add_signature(Signature::new(
            "vhd1",
            PatternType::BoundToEnd,
            72,
            "conectix".as_bytes(),
        ));
        scanner.add_signature(Signature::new(
            "test1",
            PatternType::Unbound,
            0,
            "example of unbounded pattern".as_bytes(),
        ));
        scanner.build().unwrap();

        let data: [u8; 252] = [
            0x43, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x20, 0x55, 0x72, 0x6c, 0x43, 0x61, 0x63, 0x68,
            0x65, 0x20, 0x4d, 0x4d, 0x46, 0x20, 0x56, 0x65, 0x72, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x65, 0x78, 0x61, 0x6d,
            0x70, 0x6c, 0x65, 0x20, 0x6f, 0x66, 0x20, 0x75, 0x6e, 0x62, 0x6f, 0x75, 0x6e, 0x64,
            0x65, 0x64, 0x20, 0x70, 0x61, 0x74, 0x74, 0x65, 0x72, 0x6e, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x63, 0x6f,
            0x6e, 0x65, 0x63, 0x74, 0x69, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 252);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 2);

        let signature: &Arc<Signature> = scan_context.results.get(&180).unwrap();
        assert_eq!(signature.identifier.as_str(), "vhd1");

        let result: bool = scan_context
            .results
            .values()
            .any(|signature| signature.identifier.as_str() == "test1");
        assert_eq!(result, true);
    }
}
