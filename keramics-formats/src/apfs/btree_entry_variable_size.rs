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
use keramics_types::bytes_to_u16_le;

use super::btree_entry::ApfsBtreeEntry;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "key_data_offset", data_type = "u16", format = "hex"),
        field(name = "key_data_size", data_type = "u16"),
        field(name = "value_data_offset", data_type = "u16", format = "hex"),
        field(name = "value_data_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) B-Tree variable size entry.
pub struct ApfsBtreeEntryVariableSize {}

impl ApfsBtreeEntryVariableSize {
    /// Reads the B-Tree variable size entry from a buffer.
    pub fn read_data(entry: &mut ApfsBtreeEntry, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        entry.key_data_offset = bytes_to_u16_le!(data, 0) as usize;
        entry.key_data_size = bytes_to_u16_le!(data, 2) as usize;
        entry.value_data_offset = bytes_to_u16_le!(data, 4) as usize;
        entry.value_data_size = bytes_to_u16_le!(data, 6) as usize;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x00, 0x00, 0x10, 0x00, 0x10, 0x00, 0x10, 0x00];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsBtreeEntry::new();
        ApfsBtreeEntryVariableSize::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.key_data_offset, 0x0000);
        assert_eq!(test_struct.key_data_size, 16);
        assert_eq!(test_struct.value_data_offset, 0x0010);
        assert_eq!(test_struct.value_data_size, 16);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsBtreeEntry::new();

        let test_data: Vec<u8> = get_test_data();
        let result = ApfsBtreeEntryVariableSize::read_data(&mut test_struct, &test_data[0..7]);
        assert!(result.is_err());
    }
}
