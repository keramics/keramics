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

use std::ascii::escape_default;
use std::cmp::PartialEq;
use std::fmt;

use keramics_core::mediator::{Mediator, MediatorReference};

use super::enums::PatternType;

/// Signature.
pub struct Signature {
    /// Identifier.
    pub identifier: String,

    /// Pattern type.
    pub pattern_type: PatternType,

    /// Pattern offset.
    pub pattern_offset: usize,

    /// Pattern size.
    pub pattern_size: usize,

    /// Pattern.
    pub pattern: Vec<u8>,
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
            pattern_type,
            pattern_offset,
            pattern_size,
            pattern: pattern.to_vec(),
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
            PatternType::BoundToEnd => data_size.saturating_sub(self.pattern_offset as u64),
            PatternType::BoundToStart => self.pattern_offset as u64,
            PatternType::Unbound => data_offset,
        };
        if self.pattern_type == PatternType::BoundToEnd && self.pattern_offset as u64 > data_size {
            return false;
        }
        let mediator: MediatorReference = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print("Signature::scan_buffer {\n");
            mediator.debug_print(format!(
                "    scanning at offset: {} (0x{:08x}) for signature: {} of size: {}\n",
                pattern_offset, pattern_offset, self.identifier, self.pattern_size,
            ));
            mediator.debug_print("}\n\n");
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
        self.pattern_offset == other.pattern_offset
            && self.pattern_size == other.pattern_size
            && self.pattern_type == other.pattern_type
            && self.pattern == other.pattern
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pattern_string: String = self
            .pattern
            .iter()
            .flat_map(|byte_value| escape_default(*byte_value).map(|byte_value| byte_value as char))
            .collect();

        formatter
            .debug_struct("Signature")
            .field("identifier", &self.identifier)
            .field("pattern_type", &self.pattern_type)
            .field("pattern_offset", &self.pattern_offset)
            .field("pattern", &pattern_string)
            .field("pattern_size", &self.pattern_size)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let signature: Signature =
            Signature::new("vdh", PatternType::BoundToStart, 56, b"conectix");

        assert_eq!(signature.identifier.as_str(), "vdh");
        assert_eq!(signature.pattern_type, PatternType::BoundToStart);
        assert_eq!(signature.pattern_offset, 56);
        assert_eq!(
            signature.pattern,
            vec![0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78]
        );
        assert_eq!(signature.pattern_size, 8);
    }

    #[test]
    fn test_debug() {
        let signature: Signature = Signature::new(
            "qcow3",
            PatternType::BoundToStart,
            56,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03],
        );

        assert_eq!(
            format!("{:?}", signature),
            "Signature { identifier: \"qcow3\", pattern_type: BoundToStart, pattern_offset: 56, pattern: \"QFI\\\\xfb\\\\x00\\\\x00\\\\x00\\\\x03\", pattern_size: 8 }"
        );
    }

    #[test]
    fn test_eq() {
        let signature1: Signature =
            Signature::new("vdh1", PatternType::BoundToStart, 0, b"conectix");
        let signature2: Signature =
            Signature::new("vdh2", PatternType::BoundToStart, 0, b"conectix");
        let signature3: Signature =
            Signature::new("vdh3", PatternType::BoundToStart, 56, b"conectix");
        let signature4: Signature = Signature::new("vdh4", PatternType::Unbound, 0, b"conectix");
        let signature5: Signature =
            Signature::new("vdh5", PatternType::BoundToStart, 0, b"connectx");
        let signature6: Signature =
            Signature::new("vdh6", PatternType::BoundToStart, 0, b"conecti");

        assert_eq!(signature1, signature2);
        assert_ne!(signature1, signature3);
        assert_ne!(signature1, signature4);
        assert_ne!(signature1, signature5);
        assert_ne!(signature1, signature6);
    }

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

    #[test]
    fn test_scan_buffer_with_bound_to_end() {
        let signature: Signature = Signature::new("vhd1", PatternType::BoundToEnd, 16, b"conectix");
        let mut test_data: [u8; 32] = [0; 32];
        test_data[16..24].copy_from_slice(b"conectix");

        // Test match when data_offset is 32 and data_size is 64 (pattern at offset 48).
        assert!(signature.scan_buffer(32, 64, &test_data, 0, 32));

        // Test pattern_offset larger than data_size.
        assert!(!signature.scan_buffer(0, 8, &test_data, 0, 32));

        // Test pattern_offset before data_offset.
        assert!(!signature.scan_buffer(56, 64, &test_data, 0, 32));

        // Test buffer too small for pattern.
        assert!(!signature.scan_buffer(32, 64, &test_data, 0, 20));

        // Test no match.
        let empty_data: [u8; 32] = [0; 32];
        assert!(!signature.scan_buffer(32, 64, &empty_data, 0, 32));
    }

    #[test]
    fn test_scan_buffer_with_unbound() {
        let signature: Signature = Signature::new("test1", PatternType::Unbound, 0, b"conectix");
        let mut test_data: [u8; 32] = [0; 32];
        test_data[8..16].copy_from_slice(b"conectix");

        // Test match at buffer offset 8.
        assert!(signature.scan_buffer(0, 64, &test_data, 8, 32));

        // Test no match at buffer offset 0.
        assert!(!signature.scan_buffer(0, 64, &test_data, 0, 32));

        // Test buffer too small for pattern from buffer offset 28.
        assert!(!signature.scan_buffer(0, 64, &test_data, 28, 32));
    }

    #[test]
    fn test_scan_buffer_with_debug_output() {
        let original_debug_output: bool = Mediator::current().debug_output;
        Mediator::new(true).make_current();

        let signature: Signature = Signature::new(
            "qcow3",
            PatternType::BoundToStart,
            0,
            &[0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03],
        );
        let test_data: [u8; 8] = [0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03];

        let result: bool = signature.scan_buffer(0, 64, &test_data, 0, 8);
        assert!(result);

        Mediator::new(original_debug_output).make_current();
    }
}
