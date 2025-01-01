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

use std::rc::Rc;

use super::enums::PatternType;
use super::errors::BuildError;
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
    pub fn build(&mut self) -> Result<(), BuildError> {
        self.header_scan_tree.build(&self.signatures)?;
        self.footer_scan_tree.build(&self.signatures)?;
        self.unbound_scan_tree.build(&self.signatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_build() -> Result<(), BuildError> {
        let mut scanner: Scanner = Scanner::new();

        scanner.add_signature(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        scanner.build()
    }
}
