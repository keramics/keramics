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
use std::collections::HashMap;
use std::sync::Arc;

use super::enums::PatternType;
use super::scan_result::ScanResult;
use super::scan_tree::ScanTree;
use super::scan_tree_node::ScanTreeNode;
use super::scanner::Scanner;
use super::signature::Signature;

/// Scan context.
pub struct ScanContext<'a> {
    /// Scanner.
    scanner: &'a Scanner,

    /// Data offset.
    pub data_offset: u64,

    /// Data size.
    data_size: u64,

    /// Header (range) size.
    pub header_range_size: u64,

    /// Footer (range) size.
    pub footer_range_size: u64,

    /// Unbound range size.
    unbound_range_size: usize,

    /// Results.
    pub results: HashMap<u64, Arc<Signature>>,
}

impl<'a> ScanContext<'a> {
    /// Creates a new scan context.
    pub fn new(scanner: &'a Scanner, data_size: u64) -> Self {
        // The header range starts at 0 since the header scan tree is relative to the start of the
        // data.
        let (_, header_end_offset): (usize, usize) = scanner.header_scan_tree.get_spanning_range();

        // The footer range ends at data_size since the footer scan tree is relative to offset end
        // of the data.
        let (_, footer_end_offset): (usize, usize) = scanner.footer_scan_tree.get_spanning_range();

        let (range_start_offset, range_end_offset): (usize, usize) =
            scanner.unbound_scan_tree.get_spanning_range();
        let unbound_range_size: usize = range_end_offset - range_start_offset;

        Self {
            scanner,
            data_offset: 0,
            data_size,
            header_range_size: header_end_offset as u64,
            footer_range_size: footer_end_offset as u64,
            unbound_range_size,
            results: HashMap::new(),
        }
    }

    /// Checks for signatures in the scan tree that also match at the current position.
    fn check_additional_matching_signatures(
        &mut self,
        scan_tree: &'a ScanTree,
        matched_signature: &Arc<Signature>,
        buffer: &[u8],
        buffer_offset: usize,
        buffer_size: usize,
    ) {
        for signature in scan_tree.root_node.get_signatures().iter() {
            if Arc::ptr_eq(signature, matched_signature) {
                continue;
            }
            if !signature.scan_buffer(
                self.data_offset,
                self.data_size,
                buffer,
                buffer_offset,
                buffer_size,
            ) {
                continue;
            }
            let pattern_offset: u64 = match signature.pattern_type {
                PatternType::Unbound => self.get_unbound_match_offset(buffer_offset),
                _ => self.get_signature_match_offset(signature),
            };
            self.results.insert(pattern_offset, signature.clone());
        }
    }

    /// Retrieves the offset at which a signature matches.
    fn get_signature_match_offset(&self, signature: &Signature) -> u64 {
        match signature.pattern_type {
            PatternType::BoundToEnd => self.data_size - signature.pattern_offset as u64,
            PatternType::BoundToStart => signature.pattern_offset as u64,
            PatternType::Unbound => self.data_offset,
        }
    }

    /// Retrieves the data offset at which an unbound signature matched.
    fn get_unbound_match_offset(&self, buffer_offset: usize) -> u64 {
        self.data_offset + buffer_offset as u64
    }

    /// Scans a buffer with a specific scan tree.
    fn scan_buffer_with_scan_tree(
        &mut self,
        scan_tree: &'a ScanTree,
        buffer: &[u8],
        mut buffer_offset: usize,
        buffer_size: usize,
    ) {
        let buffer_end_offset: usize = buffer_size - 1;
        let mut skip_value: usize;
        let mut scan_tree_node: &ScanTreeNode = &scan_tree.root_node;

        while buffer_offset < buffer_size {
            let scan_result: ScanResult = scan_tree_node.scan_buffer(
                self.data_offset,
                self.data_size,
                buffer,
                buffer_offset,
                buffer_size,
            );
            match scan_result {
                ScanResult::ScanTreeNode(next_node) => {
                    scan_tree_node = next_node;

                    continue;
                }
                ScanResult::Signature(signature) => {
                    let pattern_offset: u64 = match signature.pattern_type {
                        PatternType::Unbound => self.get_unbound_match_offset(buffer_offset),
                        _ => self.get_signature_match_offset(&signature),
                    };
                    self.results.insert(pattern_offset, signature.clone());

                    self.check_additional_matching_signatures(
                        scan_tree,
                        &signature,
                        buffer,
                        buffer_offset,
                        buffer_size,
                    );
                    skip_value = signature.pattern_size;
                }
                _ => {
                    let smallest_pattern_size: usize =
                        min(buffer_size, scan_tree.skip_table.smallest_pattern_size);
                    let mut skip_value_offset: usize =
                        min(buffer_offset + smallest_pattern_size - 1, buffer_end_offset);

                    loop {
                        let byte_value: u8 = buffer[skip_value_offset];
                        skip_value = scan_tree.skip_table.get_skip_value(&byte_value);
                        if skip_value == 0 {
                            skip_value = match scan_tree.pattern_type {
                                PatternType::Unbound => scan_tree.skip_table.smallest_skip_value,
                                _ => scan_tree.skip_table.smallest_pattern_size,
                            };
                        }
                        skip_value_offset -= 1;

                        if skip_value_offset <= buffer_offset || skip_value != 0 {
                            break;
                        }
                    }
                }
            }
            if scan_tree.pattern_type != PatternType::Unbound {
                break;
            }
            buffer_offset += max(skip_value, 1);
        }
    }

    /// Scans the buffer for signatures.
    pub fn scan_buffer(&mut self, buffer: &[u8]) {
        let buffer_size: usize = buffer.len();
        if buffer_size == 0 {
            return;
        }
        if self.data_offset < self.header_range_size {
            self.scan_buffer_with_scan_tree(&self.scanner.header_scan_tree, buffer, 0, buffer_size);
        }
        let next_data_offset: u64 = self.data_offset + buffer_size as u64;
        let footer_start_offset: u64 = self.data_size.saturating_sub(self.footer_range_size);

        if next_data_offset >= footer_start_offset {
            let remaining_data_size: usize = (next_data_offset - footer_start_offset) as usize;
            let buffer_start_offset: usize = buffer_size.saturating_sub(remaining_data_size);

            self.scan_buffer_with_scan_tree(
                &self.scanner.footer_scan_tree,
                buffer,
                buffer_start_offset,
                buffer_size,
            );
        }
        if self.unbound_range_size > 0 {
            self.scan_buffer_with_scan_tree(
                &self.scanner.unbound_scan_tree,
                buffer,
                0,
                buffer_size,
            );
        }
        self.data_offset = next_data_offset;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::enums::PatternType;
    use crate::signature::Signature;

    #[test]
    fn test_scan_buffer_with_bound_to_start_signature() {
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
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "msiecf1",
            PatternType::BoundToStart,
            0,
            "Client UrlCache MMF Ver ".as_bytes(),
        ));
        scanner.build().unwrap();

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);

        // Test with buffer size smaller than signature.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data[0..15]);

        assert_eq!(scan_context.results.len(), 0);

        // Test with data size smaller than signature.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 8);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 0);

        // Test with data size smaller than header range size.
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "apm1",
            PatternType::BoundToStart,
            560,
            &[
                0x41, 0x70, 0x70, 0x6c, 0x65, 0x5f, 0x70, 0x61, 0x72, 0x74, 0x69, 0x74, 0x69, 0x6f,
                0x6e, 0x5f, 0x6d, 0x61, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
        ));
        scanner.add_signature(Signature::new(
            "msiecf1",
            PatternType::BoundToStart,
            0,
            "Client UrlCache MMF Ver ".as_bytes(),
        ));
        scanner.build().unwrap();

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
    }

    #[test]
    fn test_scan_buffer_with_bound_to_start_signature_at_different_offsets() {
        let pattern: &[u8] = "conectix".as_bytes();

        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "vhd1",
            PatternType::BoundToStart,
            0,
            pattern,
        ));
        scanner.add_signature(Signature::new(
            "vhd16",
            PatternType::BoundToStart,
            16,
            pattern,
        ));
        scanner.build().unwrap();

        // A buffer where both signatures match at their expected offsets.
        let mut data: [u8; 64] = [0; 64];
        data[0..8].copy_from_slice(pattern);
        data[16..24].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 2);
        assert_eq!(
            scan_context.results.get(&0).unwrap().identifier.as_str(),
            "vhd1"
        );
        assert_eq!(
            scan_context.results.get(&16).unwrap().identifier.as_str(),
            "vhd16"
        );

        // A buffer where only the offset 0 signature matches.
        let mut data: [u8; 64] = [0; 64];
        data[0..8].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&0).unwrap().identifier.as_str(),
            "vhd1"
        );
        assert!(scan_context.results.get(&16).is_none());

        // A buffer where only the offset 16 signature matches.
        let mut data: [u8; 64] = [0; 64];
        data[16..24].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&16).unwrap().identifier.as_str(),
            "vhd16"
        );
        assert!(scan_context.results.get(&0).is_none());
    }

    #[test]
    fn test_scan_buffer_with_bound_to_end_signature() {
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
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "vhd1",
            PatternType::BoundToEnd,
            72,
            "conectix".as_bytes(),
        ));
        scanner.build().unwrap();

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);

        // Test with buffer size smaller than signature.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data[73..]);

        assert_eq!(scan_context.results.len(), 0);

        // Test with data size smaller than signature.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 8);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 0);

        // Test with data size smaller than footer range size.
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "udif1",
            PatternType::BoundToEnd,
            512,
            &[0x6b, 0x6f, 0x6c, 0x79],
        ));
        scanner.add_signature(Signature::new(
            "vhd1",
            PatternType::BoundToEnd,
            72,
            "conectix".as_bytes(),
        ));
        scanner.build().unwrap();

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
    }

    #[test]
    fn test_scan_buffer_with_bound_to_end_signature_at_different_offsets() {
        let pattern: &[u8] = "conectix".as_bytes();

        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "vhd_e16",
            PatternType::BoundToEnd,
            16,
            pattern,
        ));
        scanner.add_signature(Signature::new(
            "vhd_e8",
            PatternType::BoundToEnd,
            8,
            pattern,
        ));
        scanner.build().unwrap();

        // A buffer where both signatures match at their expected offsets.
        let mut data: [u8; 64] = [0; 64];
        data[48..56].copy_from_slice(pattern);
        data[56..64].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert!(!scan_context.results.is_empty());
        for (offset, signature) in scan_context.results.iter() {
            assert_eq!(data[*offset as usize..*offset as usize + 8], pattern[..]);
            assert!(["vhd_e16", "vhd_e8",].contains(&signature.identifier.as_str()));
        }

        // A buffer where only the offset 8 (from end) signature matches.
        let mut data: [u8; 64] = [0; 64];
        data[56..64].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&56).unwrap().identifier.as_str(),
            "vhd_e8"
        );
        assert!(scan_context.results.get(&48).is_none());

        // A buffer where only the offset 16 (from end) signature matches.
        let mut data: [u8; 64] = [0; 64];
        data[48..56].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&48).unwrap().identifier.as_str(),
            "vhd_e16"
        );
        assert!(scan_context.results.get(&56).is_none());
    }

    #[test]
    fn test_scan_buffer_with_unbound_signature() {
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "test1",
            PatternType::Unbound,
            0,
            "example of unbounded pattern".as_bytes(),
        ));
        scanner.build().unwrap();

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
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
    }

    #[test]
    fn test_scan_buffer_with_unbound_signature_at_different_offsets() {
        let pattern: &[u8] = "conectix".as_bytes();

        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new("test1", PatternType::Unbound, 0, pattern));
        scanner.build().unwrap();

        let mut data: [u8; 64] = [0; 64];
        data[16..24].copy_from_slice(pattern);
        data[32..40].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 2);
        assert_eq!(
            scan_context.results.get(&16).unwrap().identifier.as_str(),
            "test1"
        );
        assert_eq!(
            scan_context.results.get(&32).unwrap().identifier.as_str(),
            "test1"
        );

        // A single occurrence is reported at the data offset of the occurrence.
        let mut data: [u8; 64] = [0; 64];
        data[32..40].copy_from_slice(pattern);

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&32).unwrap().identifier.as_str(),
            "test1"
        );
        assert!(scan_context.results.get(&16).is_none());

        // A non-matching buffer yields no results.
        let data: [u8; 64] = [0; 64];

        let mut scan_context: ScanContext = ScanContext::new(&scanner, 64);
        scan_context.scan_buffer(&data);

        assert_eq!(scan_context.results.len(), 0);
        assert_eq!(scan_context.data_offset, 64);
    }

    #[test]
    fn test_scan_buffer_in_chunks() {
        let mut scanner: Scanner = Scanner::new();
        scanner.add_signature(Signature::new(
            "msiecf1",
            PatternType::BoundToStart,
            0,
            "Client UrlCache MMF Ver ".as_bytes(),
        ));
        scanner.build().unwrap();

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

        // Test with an empty buffer.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&[]);
        assert_eq!(scan_context.data_offset, 0);
        assert_eq!(scan_context.results.len(), 0);

        // A bound to start signature can only match in the first scanned buffer.
        let mut scan_context: ScanContext = ScanContext::new(&scanner, 128);
        scan_context.scan_buffer(&data[0..32]);
        assert_eq!(scan_context.data_offset, 32);
        assert_eq!(scan_context.results.len(), 1);
        assert_eq!(
            scan_context.results.get(&0).unwrap().identifier.as_str(),
            "msiecf1"
        );

        // Scanning the next buffer advances the data offset and does not add a result.
        scan_context.scan_buffer(&data[32..64]);
        assert_eq!(scan_context.data_offset, 64);
        assert_eq!(scan_context.results.len(), 1);
    }
}
