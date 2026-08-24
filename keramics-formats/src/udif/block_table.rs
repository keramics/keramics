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

use super::block_table_entry::UdifBlockTableEntry;
use super::block_table_header::UdifBlockTableHeader;

/// Universal Disk Image Format (UDIF) block table.
pub struct UdifBlockTable {
    /// Start sector.
    pub start_sector: u64,

    /// Entries.
    pub entries: Vec<UdifBlockTableEntry>,
}

impl UdifBlockTable {
    /// Creates a new block table.
    pub fn new() -> Self {
        Self {
            start_sector: 0,
            entries: Vec::new(),
        }
    }

    /// Reads the block table from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        keramics_core::debug_trace_structure!(UdifBlockTableHeader::debug_read_data(data));

        let mut block_table_header: UdifBlockTableHeader = UdifBlockTableHeader::new();

        match block_table_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read block table header");
                return Err(error);
            }
        }
        let data_end_offset: usize = 204 + ((block_table_header.number_of_entries as usize) * 40);

        if data_end_offset > data.len() {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid block table number of entries: {} value out of bounds",
                block_table_header.number_of_entries
            )));
        }
        self.start_sector = block_table_header.start_sector;

        let mut data_offset: usize = 204;

        for entry_index in 0..block_table_header.number_of_entries {
            keramics_core::debug_trace_structure!(UdifBlockTableEntry::debug_read_data(
                &data[data_offset..]
            ));
            let mut block_table_entry: UdifBlockTableEntry = UdifBlockTableEntry::new();

            match block_table_entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read block table entry: {}", entry_index)
                    );
                    return Err(error);
                }
            }
            data_offset += 40;

            block_table_entry.data_offset += block_table_header.base_data_offset;

            self.entries.push(block_table_entry);
        }
        Ok(())
    }

    /// Reads the block table from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u32,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        // Note that 65536 is an arbitrary chosen limit.
        if data_size < 204 || data_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported block table data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        keramics_core::data_stream_read_exact_at_position_with_debug_trace_data!(
            "UdifBlockTable",
            data_stream,
            &mut data,
            data_size,
            position,
        );
        self.read_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x6d, 0x69, 0x73, 0x68, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x20, 0x41, 0xf2, 0xfa, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x80, 0x00, 0x00, 0x05, 0x00, 0x00,
            0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x0d, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifBlockTable::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.start_sector, 0);
        assert_eq!(test_struct.entries.len(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = UdifBlockTable::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
