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

use std::cmp::PartialEq;

use core::mediator::{Mediator, MediatorReference};

use super::enums::PatternType;

/// Signature.
#[derive(Debug)]
pub struct Signature {
    /// Identifier.
    pub identifier: String,

    /// Pattern type.
    pub pattern_type: PatternType,

    /// Pattern offset.
    pub pattern_offset: usize,

    /// Pattern.
    pub pattern: Vec<u8>,

    /// Pattern size.
    pub pattern_size: usize,
}

impl Signature {
    /// Creates a new signature.
    pub fn new(
        identifier: &str,
        pattern_type: PatternType,
        pattern_offset: usize,
        pattern: &[u8],
    ) -> Self {
        let pattern_size: usize = pattern.len();
        Self {
            identifier: identifier.to_string(),
            pattern_type: pattern_type,
            pattern_offset: pattern_offset,
            pattern: Vec::from(pattern),
            pattern_size: pattern_size,
        }
    }

    /// Scans a buffer for a matching signature.
    pub(super) fn scan_buffer(
        &self,
        data_offset: u64,
        data_size: u64,
        buffer: &[u8],
        buffer_offset: usize,
        buffer_size: usize,
    ) -> bool {
        let pattern_offset: u64 = match self.pattern_type {
            PatternType::BoundToEnd => data_size - self.pattern_offset as u64,
            PatternType::BoundToStart => self.pattern_offset as u64,
            PatternType::Unbound => data_offset,
        };
        let mediator: MediatorReference = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print(format!("Signature::scan_buffer {{\n"));
            mediator.debug_print(format!(
                "    scanning at offset: {} (0x{:08x}) for signature: {} of size: {}\n",
                pattern_offset, pattern_offset, self.identifier, self.pattern_size,
            ));
            mediator.debug_print(format!("}}\n\n"));
        }
        if pattern_offset < data_offset {
            return false;
        }
        let scan_offset: usize = match self.pattern_type {
            PatternType::Unbound => buffer_offset,
            _ => (pattern_offset - data_offset) as usize,
        };
        let scan_end_offset: usize = scan_offset + self.pattern_size;

        if scan_end_offset > buffer_size || (scan_end_offset as u64) > data_size {
            return false;
        }
        if buffer[scan_offset..scan_end_offset] != self.pattern {
            return false;
        }
        match self.pattern_type {
            PatternType::Unbound => true,
            _ => (data_offset + scan_offset as u64) == pattern_offset,
        }
    }
}

impl PartialEq for Signature {
    /// Determines if the signature is equivalent to another signature.
    fn eq(&self, other: &Signature) -> bool {
        self.pattern == other.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_buffer() {
        let signature: Signature = Signature::new(
            "qcow3",
            PatternType::BoundToStart,
            0,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03],
        );
        let test_data: [u8; 8] = [0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03];

        // Test match at data offset 0.
        let result: bool = signature.scan_buffer(0, 64, &test_data, 0, 8);
        assert_eq!(result, true);

        // Test match at data offset 8.
        let result: bool = signature.scan_buffer(8, 64, &test_data, 0, 8);
        assert_eq!(result, false);

        // Test buffer too small for pattern.
        let result: bool = signature.scan_buffer(0, 64, &test_data, 0, 7);
        assert_eq!(result, false);

        // Test data size too small for pattern.
        let result: bool = signature.scan_buffer(0, 7, &test_data, 0, 8);
        assert_eq!(result, false);

        let test_data: [u8; 8] = [0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78];

        // Test no match.
        let result: bool = signature.scan_buffer(0, 64, &test_data, 0, 8);
        assert_eq!(result, false);
    }
}
