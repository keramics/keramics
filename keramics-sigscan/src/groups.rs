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
use std::rc::Rc;

use super::types::SignatureReference;

/// Byte value group.
#[derive(Debug)]
pub(super) struct ByteValueGroup {
    /// Pattern offset.
    pub pattern_offset: usize,

    /// Signature groups.
    pub signature_groups: HashMap<u8, SignatureGroup>,
}

impl ByteValueGroup {
    /// Creates a new byte value group.
    pub fn new(pattern_offset: usize) -> Self {
        Self {
            pattern_offset: pattern_offset,
            signature_groups: HashMap::new(),
        }
    }

    /// Inserts a signature related to a specific byte value.
    pub fn insert_signature(&mut self, byte_value: u8, signature: &SignatureReference) {
        match self.signature_groups.get_mut(&byte_value) {
            Some(signature_group) => signature_group.append_signature(signature),
            None => {
                let mut signature_group: SignatureGroup = SignatureGroup::new(byte_value);
                signature_group.append_signature(signature);

                self.signature_groups.insert(byte_value, signature_group);
            }
        };
    }
}

/// Offset group.
#[derive(Debug)]
pub(super) struct OffsetGroup {
    /// Offsets.
    pub offsets: Vec<usize>,
}

impl OffsetGroup {
    /// Creates a new offset group.
    pub fn new() -> Self {
        Self {
            offsets: Vec::new(),
        }
    }

    /// Appends an offset.
    pub fn append_offset(&mut self, pattern_offset: usize) {
        self.offsets.push(pattern_offset);
    }
}

/// Signature group.
#[derive(Debug)]
pub(super) struct SignatureGroup {
    /// Byte value.
    pub byte_value: u8,

    /// Signatures.
    pub signatures: Vec<SignatureReference>,
}

impl SignatureGroup {
    /// Creates a new signature group.
    pub fn new(byte_value: u8) -> Self {
        Self {
            byte_value: byte_value,
            signatures: Vec::new(),
        }
    }

    /// Appends a signature.
    pub fn append_signature(&mut self, signature: &SignatureReference) {
        self.signatures.push(Rc::clone(signature));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::rc::Rc;

    use crate::enums::PatternType;
    use crate::signature::Signature;

    #[test]
    fn test_byte_value_group_insert_signature() {
        let mut byte_value_group: ByteValueGroup = ByteValueGroup::new(0);

        assert_eq!(byte_value_group.signature_groups.len(), 0);

        let signature: SignatureReference = Rc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        byte_value_group.insert_signature(0x63, &signature);

        assert_eq!(byte_value_group.signature_groups.len(), 1);
        // TODO: test if signature_groups contains 0x63
    }

    #[test]
    fn test_offset_group_append_offset() {
        let mut offset_group: OffsetGroup = OffsetGroup::new();

        assert_eq!(offset_group.offsets.len(), 0);

        offset_group.append_offset(5);

        assert_eq!(offset_group.offsets.len(), 1);
        // TODO: test if offsets contains 5
    }

    #[test]
    fn test_signature_group_append_signature() {
        let mut signature_group: SignatureGroup = SignatureGroup::new(0x63);

        assert_eq!(signature_group.signatures.len(), 0);

        let signature: SignatureReference = Rc::new(Signature::new(
            "vdh",
            PatternType::BoundToStart,
            0,
            "conectix".as_bytes(),
        ));
        signature_group.append_signature(&signature);

        assert_eq!(signature_group.signatures.len(), 1);
        // TODO: test if offsets contains signature
    }
}
