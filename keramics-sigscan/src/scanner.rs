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
        assert!(
            scanner
                .header_scan_tree
                .root_node
                .scan_objects
                .contains_key(&0x63_i16)
        );
        assert!(
            scanner
                .footer_scan_tree
                .root_node
                .scan_objects
                .contains_key(&0x63_i16)
        );
        assert!(
            scanner
                .unbound_scan_tree
                .root_node
                .scan_objects
                .contains_key(&0x63_i16)
        );

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

        let mut data: Vec<u8> = "Client UrlCache MMF Ver ".as_bytes().to_vec();
        for _ in 24..80 {
            data.push(0x20);
        }
        data.extend_from_slice(b"example of unbounded pattern");
        for _ in 0..72 {
            data.push(0x20);
        }
        data.extend_from_slice(b"conectix");
        for _ in 0..64 {
            data.push(0x00);
        }

        let data_size: u64 = data.len() as u64;
        let mut scan_context: ScanContext = ScanContext::new(&scanner, data_size);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 2);
        assert_eq!(
            scan_context
                .results
                .get(&(data_size - 72))
                .unwrap()
                .identifier
                .as_str(),
            "vhd1"
        );
        assert!(
            scan_context
                .results
                .values()
                .any(|signature| signature.identifier.as_str() == "test1")
        );
    }
}
