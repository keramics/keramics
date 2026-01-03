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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "entry", data_type = "u32", format = "hex")
    ),
    methods("debug_read_data")
)]
/// Parallels Disk Image (PDI) block allocation table entry.
pub struct PdiBlockAllocationTableEntry {
    // Sector number.
    pub sector_number: u32,
}

impl PdiBlockAllocationTableEntry {
    /// Creates a block allocation table entry.
    pub fn new() -> Self {
        Self { sector_number: 0 }
    }

    /// Reads the block allocation table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.sector_number = bytes_to_u32_le!(data, 0);

        Ok(())
    }
}

/// Parallels Disk Image (PDI) block allocation table.
pub struct PdiBlockAllocationTable {
    /// Offset.
    offset: u64,

    /// Number of entries.
    number_of_entries: u32,
}

impl PdiBlockAllocationTable {
    /// Creates a new block allocation table.
    pub fn new() -> Self {
        Self {
            offset: 0,
            number_of_entries: 0,
        }
    }

    /// Creates a new block allocation table.
    pub fn set_range(&mut self, offset: u64, number_of_entries: u32) {
        self.offset = offset;
        self.number_of_entries = number_of_entries;
    }

    /// Reads a block allocation table entry.
    pub fn read_entry(
        &self,
        data_stream: &DataStreamReference,
        entry_index: u32,
    ) -> Result<PdiBlockAllocationTableEntry, ErrorTrace> {
        if entry_index >= self.number_of_entries {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported entry index: {} value out of bounds",
                entry_index
            )));
        }
        let entry_offset: u64 = self.offset + (entry_index as u64 * 4);
        let mut data: [u8; 4] = [0; 4];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(entry_offset)
        );
        keramics_core::debug_trace_data_and_structure!(
            format!("PdiBlockAllocationTableEntry: {}", entry_index),
            entry_offset,
            &data,
            data.len(),
            PdiBlockAllocationTableEntry::debug_read_data(&data)
        );
        let mut entry: PdiBlockAllocationTableEntry = PdiBlockAllocationTableEntry::new();

        match entry.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block allocation table"
                );
                return Err(error);
            }
        }
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = PdiBlockAllocationTableEntry::new();

        test_struct.read_data(&test_data[0..4])?;
        assert_eq!(test_struct.sector_number, 22);

        test_struct.read_data(&test_data[4..8])?;
        assert_eq!(test_struct.sector_number, 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = PdiBlockAllocationTableEntry::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = PdiBlockAllocationTable::new();
        test_struct.set_range(0, 4);

        let test_entry: PdiBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 0)?;
        assert_eq!(test_entry.sector_number, 22);

        let test_entry: PdiBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 1)?;
        assert_eq!(test_entry.sector_number, 0);

        Ok(())
    }
}
