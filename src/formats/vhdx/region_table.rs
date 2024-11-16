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

use std::collections::HashMap;
use std::io;

use crate::checksums::ReversedCrc32Context;
use crate::mediator::{Mediator, MediatorReference};
use crate::types::Uuid;
use crate::vfs::VfsDataStreamReference;

use super::region_table_entry::VhdxRegionTableEntry;
use super::region_table_header::VhdxRegionTableHeader;

/// Virtual Hard Disk version 2 (VHDX) region table.
pub struct VhdxRegionTable {
    /// Mediator.
    mediator: MediatorReference,

    /// Entries.
    pub entries: HashMap<Uuid, VhdxRegionTableEntry>,
}

impl VhdxRegionTable {
    /// Creates a new region table.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            entries: HashMap::new(),
        }
    }

    /// Reads the region table from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let mut region_table_header: VhdxRegionTableHeader = VhdxRegionTableHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(region_table_header.debug_read_data(&data[0..16]));
        }
        region_table_header.read_data(&data[0..16])?;

        let mut data_offset: usize = 16;

        let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0x82f63b78, 0);

        crc32_context.update(&data[0..4]);

        let empty_data: [u8; 4] = [0; 4];
        crc32_context.update(&empty_data);

        crc32_context.update(&data[8..65536]);

        let calculated_checksum: u32 = crc32_context.finalize();

        if region_table_header.checksum != 0 && region_table_header.checksum != calculated_checksum
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    region_table_header.checksum, calculated_checksum
                ),
            ));
        }
        for _ in 0..region_table_header.number_of_entries {
            let data_end_offset: usize = data_offset + 32;

            let mut region_table_entry: VhdxRegionTableEntry = VhdxRegionTableEntry::new();

            if self.mediator.debug_output {
                self.mediator.debug_print(
                    region_table_entry.debug_read_data(&data[data_offset..data_end_offset]),
                );
            }
            region_table_entry.read_data(&data[data_offset..data_end_offset])?;
            data_offset = data_end_offset;

            self.entries.insert(
                region_table_entry.type_identifier.clone(),
                region_table_entry,
            );
        }
        Ok(())
    }

    /// Reads the region table from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &VfsDataStreamReference,
        position: io::SeekFrom,
    ) -> io::Result<()> {
        let mut data: Vec<u8> = vec![0; 65536];

        let offset: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "VhdxRegionTable data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.read_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: add test_read_data
    // TODO: add test_read_data with invalid header sigature
    // TODO: add test_read_data with checksum mismatch
    // TODO: add test_read_at_position
}
