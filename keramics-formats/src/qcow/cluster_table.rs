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

use std::io::SeekFrom;

use keramics_core::mediator::Mediator;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u64_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "reference", data_type = "u64", format = "hex")
    ),
    method(name = "debug_read_data")
)]
/// QEMU Copy-On-Write (QCOW) cluster table entry.
pub struct QcowClusterTableEntry {
    pub reference: u64,
}

impl QcowClusterTableEntry {
    /// Creates a cluster table entry.
    pub fn new() -> Self {
        Self { reference: 0 }
    }

    /// Reads the cluster table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() != 8 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported QCOW cluster table entry data size"
            ));
        }
        self.reference = bytes_to_u64_be!(data, 0);

        Ok(())
    }
}

/// QEMU Copy-On-Write (QCOW) cluster table.
pub struct QcowClusterTable {
    offset: u64,
    number_of_entries: u32,
}

impl QcowClusterTable {
    /// Creates a new cluster table.
    pub fn new() -> Self {
        Self {
            offset: 0,
            number_of_entries: 0,
        }
    }

    /// Creates a new cluster table.
    pub fn set_range(&mut self, offset: u64, number_of_entries: u32) {
        self.offset = offset;
        self.number_of_entries = number_of_entries;
    }

    /// Reads a cluster table entry.
    pub fn read_entry(
        &self,
        data_stream: &DataStreamReference,
        entry_index: u32,
    ) -> Result<QcowClusterTableEntry, ErrorTrace> {
        if entry_index >= self.number_of_entries {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported entry index: {} value out of bounds",
                entry_index
            )));
        }
        let entry_offset: u64 = self.offset + (entry_index as u64 * 8);
        let mut data: [u8; 8] = [0; 8];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(entry_offset)
        );
        let mut entry: QcowClusterTableEntry = QcowClusterTableEntry::new();

        let mediator = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print(format!(
                "QcowClusterTableEntry: {} data of size: {} at offset: {} (0x{:08x})\n",
                entry_index,
                data.len(),
                entry_offset,
                entry_offset
            ));
            mediator.debug_print_data(&data, true);
            mediator.debug_print(QcowClusterTableEntry::debug_read_data(&data));
        }
        match entry.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read cluster table");
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowClusterTableEntry::new();

        test_struct.read_data(&test_data[0..8])?;
        assert_eq!(test_struct.reference, 4);

        test_struct.read_data(&test_data[8..16])?;
        assert_eq!(test_struct.reference, 0xffffffffffffffff);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowClusterTableEntry::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = QcowClusterTable::new();
        test_struct.set_range(0, 2);

        let test_entry: QcowClusterTableEntry = test_struct.read_entry(&data_stream, 0)?;
        assert_eq!(test_entry.reference, 4);

        let test_entry: QcowClusterTableEntry = test_struct.read_entry(&data_stream, 1)?;
        assert_eq!(test_entry.reference, 0xffffffffffffffff);

        Ok(())
    }
}
