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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{Uuid, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "unknown_identifier", data_type = "Uuid"),
        field(name = "copy_identifier", data_type = "Uuid"),
        field(name = "copy_set_identifier", data_type = "Uuid"),
        field(name = "snapshot_context", data_type = "u32"),
        field(name = "unknown2", data_type = "u32"),
        field(name = "attribute_flags", data_type = "u32", format = "hex"),
        field(name = "unknown3", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) store metadata.
pub struct VolsnapStoreMetadata {
    /// Copy identifier.
    pub copy_identifier: Uuid,

    /// Copy set identifier.
    pub copy_set_identifier: Uuid,

    /// Attribute flags.
    pub attribute_flags: u32,
}

impl VolsnapStoreMetadata {
    /// Creates a new store metadata.
    pub fn new() -> Self {
        Self {
            copy_identifier: Uuid::new(),
            copy_set_identifier: Uuid::new(),
            attribute_flags: 0,
        }
    }

    /// Reads the store metadata from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 64 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.copy_identifier = Uuid::from_le_bytes(&data[16..32]);
        self.copy_set_identifier = Uuid::from_le_bytes(&data[32..48]);
        self.attribute_flags = bytes_to_u32_le!(data, 56);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x37, 0x93, 0x9a, 0xae, 0x48, 0x00, 0xd6, 0x4e, 0x98, 0x74, 0x71, 0x50, 0x06, 0x54,
            0xb7, 0xb3, 0xc9, 0x4f, 0xab, 0x54, 0xae, 0x3e, 0xed, 0x4d, 0x8e, 0x85, 0x6f, 0x1f,
            0x86, 0x42, 0x51, 0xdc, 0xa0, 0xc5, 0x55, 0x67, 0x62, 0xeb, 0xe9, 0x41, 0x91, 0xd6,
            0x49, 0x02, 0x46, 0xc6, 0x13, 0x75, 0x09, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x09, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapStoreMetadata::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.copy_identifier.to_string(),
            "54ab4fc9-3eae-4ded-8e85-6f1f864251dc"
        );
        assert_eq!(
            test_struct.copy_set_identifier.to_string(),
            "6755c5a0-eb62-41e9-91d6-490246c61375"
        );
        assert_eq!(test_struct.attribute_flags, 0x00420009);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapStoreMetadata::new();
        let result = test_struct.read_data(&test_data[0..63]);
        assert!(result.is_err());
    }
}
