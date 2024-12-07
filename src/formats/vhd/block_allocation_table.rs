/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use layout_map::LayoutMap;

use crate::bytes_to_u32_be;
use crate::mediator::Mediator;
use crate::vfs::VfsDataStreamReference;

#[derive(LayoutMap)]
#[layout_map(
    structure(byte_order = "big", field(name = "sector_number", data_type = "u32")),
    method(name = "debug_read_data")
)]
/// Virtual Hard Disk (VHD) block allocation table entry.
pub struct VhdBlockAllocationTableEntry {
    /// Sector number.
    pub sector_number: u32,
}

impl VhdBlockAllocationTableEntry {
    /// Creates a block allocation table entry.
    pub fn new() -> Self {
        Self { sector_number: 0 }
    }

    /// Reads the block allocation table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.sector_number = bytes_to_u32_be!(data, 0);

        Ok(())
    }
}

/// Virtual Hard Disk (VHD) block allocation table.
pub struct VhdBlockAllocationTable {
    /// Offset.
    offset: u64,

    /// Number of entries.
    number_of_entries: u32,
}

impl VhdBlockAllocationTable {
    /// Creates a new block allocation table.
    pub fn new(offset: u64, number_of_entries: u32) -> Self {
        Self {
            offset: offset,
            number_of_entries: number_of_entries,
        }
    }

    /// Reads a block allocation table entry.
    pub fn read_entry(
        &self,
        data_stream: &VfsDataStreamReference,
        entry_index: u32,
    ) -> io::Result<VhdBlockAllocationTableEntry> {
        if entry_index >= self.number_of_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported entry index: {} value out of bounds",
                    entry_index
                ),
            ));
        }
        let entry_offset: u64 = self.offset + (entry_index as u64 * 4);
        let mut data: [u8; 4] = [0; 4];

        match data_stream.with_write_lock() {
            Ok(mut data_stream) => {
                data_stream.read_exact_at_position(&mut data, io::SeekFrom::Start(entry_offset))?
            }
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mut entry: VhdBlockAllocationTableEntry = VhdBlockAllocationTableEntry::new();

        let mediator = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print(format!(
                "VhdBlockAllocationTableEntry: {} data of size: {} at offset: {} (0x{:08x})\n",
                entry_index,
                data.len(),
                entry_offset,
                entry_offset
            ));
            mediator.debug_print_data(&data, true);
            mediator.debug_print(VhdBlockAllocationTableEntry::debug_read_data(&data));
        }
        entry.read_data(&data)?;

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{new_fake_data_stream, VfsDataStreamReference};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x04, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data = get_test_data();

        let mut test_struct = VhdBlockAllocationTableEntry::new();

        test_struct.read_data(&test_data[0..4])?;
        assert_eq!(test_struct.sector_number, 4);

        test_struct.read_data(&test_data[4..8])?;
        assert_eq!(test_struct.sector_number, 0xffffffff);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data = get_test_data();

        let mut test_struct = VhdBlockAllocationTableEntry::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry() -> io::Result<()> {
        let test_data = get_test_data();
        let data_stream: VfsDataStreamReference = new_fake_data_stream(test_data)?;

        let test_struct = VhdBlockAllocationTable::new(0, 3);

        let test_entry: VhdBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 0)?;
        assert_eq!(test_entry.sector_number, 4);

        let test_entry: VhdBlockAllocationTableEntry = test_struct.read_entry(&data_stream, 1)?;
        assert_eq!(test_entry.sector_number, 0xffffffff);

        Ok(())
    }
}
