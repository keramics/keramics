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

use std::rc::Rc;

use super::enums::PatternType;
use super::scan_tree::ScanTree;
use super::signature::Signature;
use super::types::SignatureReference;

/// Signature scanner.
pub struct Scanner {
    /// Signatures.
    pub(super) signatures: Vec<SignatureReference>,

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
        self.signatures.push(Rc::new(signature));
    }

    /// Builds the scan trees.
    pub fn build(&mut self) {
        self.header_scan_tree.build(&self.signatures);
        self.footer_scan_tree.build(&self.signatures);
        self.unbound_scan_tree.build(&self.signatures);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner() {
        let mut scanner: Scanner = Scanner::new();

        scanner.add_signature(Signature::new(
            "test1",
            PatternType::Unbound,
            0,
            "example of unbounded pattern".as_bytes(),
        ));
        scanner.build();
    }
}
