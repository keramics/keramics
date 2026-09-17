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
use keramics_types::{bytes_to_u32_le, bytes_to_u64_le};

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "format_identifier", data_type = "Uuid"),
        field(name = "format_version", data_type = "u32"),
        field(name = "block_type", data_type = "u32"),
        field(name = "relative_block_offset", data_type = "u64", format = "hex"),
        field(name = "current_block_offset", data_type = "u64", format = "hex"),
        field(name = "next_block_offset", data_type = "u64", format = "hex"),
        field(name = "store_metadata_size", data_type = "u64"),
        field(name = "unknown1", data_type = "[u8; 72]"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) store block header.
pub struct VolsnapStoreBlockHeader {
    /// Block type.
    pub block_type: u32,

    /// Current block offset.
    pub current_block_offset: u64,

    /// Next block offset.
    pub next_block_offset: u64,

    /// Store metadata size.
    pub store_metadata_size: u64,
}

impl VolsnapStoreBlockHeader {
    /// Creates a new store block header.
    pub fn new() -> Self {
        Self {
            block_type: 0,
            current_block_offset: 0,
            next_block_offset: 0,
            store_metadata_size: 0,
        }
    }

    /// Reads the store block header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..16] != VOLSNAP_IDENTIFIER {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.block_type = bytes_to_u32_le!(data, 20);
        self.current_block_offset = bytes_to_u64_le!(data, 32);
        self.next_block_offset = bytes_to_u64_le!(data, 40);
        self.store_metadata_size = bytes_to_u64_le!(data, 48);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x6b, 0x87, 0x08, 0x38, 0x76, 0xc1, 0x48, 0x4e, 0xb7, 0xae, 0x04, 0x04, 0x6e, 0x6c,
            0xc7, 0x52, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapStoreBlockHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_type, 4);
        assert_eq!(test_struct.current_block_offset, 0x02a90000);
        assert_eq!(test_struct.next_block_offset, 0x00000000);
        assert_eq!(test_struct.store_metadata_size, 120);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapStoreBlockHeader::new();
        let result = test_struct.read_data(&test_data[0..127]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VolsnapStoreBlockHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
