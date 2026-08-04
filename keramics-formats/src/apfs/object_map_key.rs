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
use keramics_types::bytes_to_u64_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "object_identifier", data_type = "u64"),
        field(name = "object_transaction_identifier", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) object map key.
pub struct ApfsObjectMapKey {
    /// Object identifier.
    pub object_identifier: u64,

    /// Object transaction identifier.
    pub object_transaction_identifier: u64,
}

impl ApfsObjectMapKey {
    /// Creates a new key.
    pub fn new() -> Self {
        Self {
            object_identifier: 0,
            object_transaction_identifier: 0,
        }
    }

    /// Reads the key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 16 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.object_identifier = bytes_to_u64_le!(data, 0);
        self.object_transaction_identifier = bytes_to_u64_le!(data, 8);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x02, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsObjectMapKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.object_identifier, 1026);
        assert_eq!(test_struct.object_transaction_identifier, 6);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsObjectMapKey::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..15]);
        assert!(result.is_err());
    }
}
