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
use keramics_types::{Uuid, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "entry_type", data_type = "u64"),
        field(name = "store_block_list_offset", data_type = "u64", format = "hex"),
        field(name = "store_identifier", data_type = "Uuid"),
        field(name = "store_metadata_offset", data_type = "u64", format = "hex"),
        field(
            name = "store_block_range_list_offset",
            data_type = "u64",
            format = "hex"
        ),
        field(name = "store_bitmap_offset", data_type = "u64", format = "hex"),
        field(name = "ntfs_file_reference", data_type = "u64", format = "hex"),
        field(name = "allocated_size", data_type = "u64"),
        field(
            name = "store_previous_bitmap_offset",
            data_type = "u64",
            format = "hex"
        ),
        field(name = "store_index", data_type = "u64"),
        field(name = "unknown1", data_type = "[u8; 40]"),
    ),
    methods("debug_read_data")
)]
/// Volume Shadow Snapshot (volsnap) catalog entry type 3.
pub struct VolsnapCatalogEntryType3 {
    /// Store block list offset.
    pub store_block_list_offset: u64,

    /// Store metadata offset.
    pub store_metadata_offset: u64,

    /// Store block range list offset.
    pub store_block_range_list_offset: u64,

    /// Store identifier.
    pub store_identifier: Uuid,

    /// Store bitmap offset.
    pub store_bitmap_offset: u64,

    /// Store previous bitmap offset.
    pub store_previous_bitmap_offset: u64,
}

impl VolsnapCatalogEntryType3 {
    /// Creates a new catalog entry type 3.
    pub fn new() -> Self {
        Self {
            store_block_list_offset: 0,
            store_metadata_offset: 0,
            store_block_range_list_offset: 0,
            store_identifier: Uuid::new(),
            store_bitmap_offset: 0,
            store_previous_bitmap_offset: 0,
        }
    }

    /// Reads the catalog entry type 3 from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 128 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        let entry_type: u64 = bytes_to_u64_le!(data, 0);

        if entry_type != 3 {
            return Err(keramics_core::error_trace_new!("Unsupported entry type"));
        }
        self.store_block_list_offset = bytes_to_u64_le!(data, 8);
        self.store_identifier = Uuid::from_le_bytes(&data[16..32]);
        self.store_metadata_offset = bytes_to_u64_le!(data, 32);
        self.store_block_range_list_offset = bytes_to_u64_le!(data, 40);
        self.store_bitmap_offset = bytes_to_u64_le!(data, 48);
        self.store_previous_bitmap_offset = bytes_to_u64_le!(data, 72);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0xa9, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x9b, 0x81, 0x17, 0x9f, 0xf9, 0xb0, 0xf1, 0x11, 0x90, 0xdc, 0x7c, 0xed,
            0x8d, 0x4e, 0x4e, 0x79, 0x00, 0x00, 0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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

        let mut test_struct = VolsnapCatalogEntryType3::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.store_block_list_offset, 0x02a94000);
        assert_eq!(
            test_struct.store_identifier.to_string(),
            "9f17819b-b0f9-11f1-90dc-7ced8d4e4e79"
        );
        assert_eq!(test_struct.store_metadata_offset, 0x02a90000);
        assert_eq!(test_struct.store_block_range_list_offset, 0x02a98000);
        assert_eq!(test_struct.store_bitmap_offset, 0x02aa0000);
        assert_eq!(test_struct.store_block_range_list_offset, 0x02a98000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapCatalogEntryType3::new();
        let result = test_struct.read_data(&test_data[0..127]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_entry_type() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VolsnapCatalogEntryType3::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
