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

use std::io;

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::DataStreamReference;
use keramics_types::bytes_to_u64_le;
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "block_state", data_type = "BitField64<3>"),
        field(name = "unknown1", data_type = "BitField64<17>", format = "hex"),
        field(name = "block_offset", data_type = "BitField64<44>", format = "hex"),
    ),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk version 2 (VHDX) block allocation table entry.
pub struct VhdxBlockAllocationTableEntry {
    /// Block state.
    pub block_state: u8,

    /// Block offset.
    pub block_offset: u64,
}

impl VhdxBlockAllocationTableEntry {
    /// Creates a block allocation table entry.
    pub fn new() -> Self {
        Self {
            block_state: 0,
            block_offset: 0,
        }
    }

    /// Reads the block allocation table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        let entry: u64 = bytes_to_u64_le!(data, 0);

        self.block_state = (entry & 0x0000000000000007) as u8;
        self.block_offset = entry & 0xfffffffffff00000;

        Ok(())
    }
}

/// Virtual Hard Disk version 2 (VHDX) block allocation table.
pub struct VhdxBlockAllocationTable {
    /// Mediator.
    mediator: MediatorReference,

    /// Offset.
    offset: u64,

    /// Number of entries.
    number_of_entries: u32,
}

impl VhdxBlockAllocationTable {
    /// Creates a new block allocation table.
    pub fn new(offset: u64, number_of_entries: u32) -> Self {
        Self {
            mediator: Mediator::current(),
            offset: offset,
            number_of_entries: number_of_entries,
        }
    }

    /// Reads a block allocation table entry.
    pub fn read_entry(
        &self,
        data_stream: &DataStreamReference,
        entry_index: u32,
    ) -> io::Result<VhdxBlockAllocationTableEntry> {
        if entry_index >= self.number_of_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported entry index: {} value out of bounds",
                    entry_index
                ),
            ));
        }
        let entry_offset: u64 = self.offset + (entry_index as u64 * 8);
        let mut data: [u8; 8] = [0; 8];

        match data_stream.write() {
            Ok(mut data_stream) => {
                data_stream.read_exact_at_position(&mut data, io::SeekFrom::Start(entry_offset))?
            }
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        let mut entry: VhdxBlockAllocationTableEntry = VhdxBlockAllocationTableEntry::new();

        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "VhdxBlockAllocationTableEntry: {} data of size: {} at offset: {} (0x{:08x})\n",
                entry_index,
                data.len(),
                entry_offset,
                entry_offset
            ));
            self.mediator.debug_print_data(&data, true);
            self.mediator
                .debug_print(VhdxBlockAllocationTableEntry::debug_read_data(&data));
        }
        entry.read_data(&data)?;

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x06, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxBlockAllocationTableEntry::new();

        test_struct.read_data(&test_data[0..8])?;
        assert_eq!(test_struct.block_state, 6);
        assert_eq!(test_struct.block_offset, 4 * 1024 * 1024);

        test_struct.read_data(&test_data[8..16])?;
        assert_eq!(test_struct.block_state, 7);
        assert_eq!(test_struct.block_offset, 0xfffffffffff00000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxBlockAllocationTableEntry::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let test_struct = VhdxBlockAllocationTable::new(0, 2);

        let test_entry: VhdxBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 0)?;
        assert_eq!(test_entry.block_state, 6);
        assert_eq!(test_entry.block_offset, 4 * 1024 * 1024);

        let test_entry: VhdxBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 1)?;
        assert_eq!(test_entry.block_state, 7);
        assert_eq!(test_entry.block_offset, 0xfffffffffff00000);

        Ok(())
    }
}
