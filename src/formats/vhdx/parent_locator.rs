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

use crate::mediator::{Mediator, MediatorReference};
use crate::types::Ucs2String;
use crate::vfs::VfsDataStreamReference;

use super::parent_locator_entry::VhdxParentLocatorEntry;
use super::parent_locator_header::VhdxParentLocatorHeader;

/// Virtual Hard Disk version 2 (VHDX) parent locator.
pub struct VhdxParentLocator {
    /// Mediator.
    mediator: MediatorReference,

    /// Entries.
    pub entries: HashMap<String, Ucs2String>,
}

impl VhdxParentLocator {
    /// Creates a new parent locator.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            entries: HashMap::new(),
        }
    }

    /// Reads the parent locator from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size: usize = data.len();
        let mut parent_locator_header: VhdxParentLocatorHeader = VhdxParentLocatorHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(parent_locator_header.debug_read_data(&data[0..20]));
        }
        parent_locator_header.read_data(&data[0..20])?;

        let mut data_offset: usize = 20;

        for parent_locator_entry_index in 0..parent_locator_header.number_of_entries {
            let data_end_offset: usize = data_offset + 12;

            let mut parent_locator_entry: VhdxParentLocatorEntry = VhdxParentLocatorEntry::new();

            if self.mediator.debug_output {
                self.mediator.debug_print(
                    parent_locator_entry.debug_read_data(&data[data_offset..data_end_offset]),
                );
            }
            parent_locator_entry.read_data(&data[data_offset..data_end_offset])?;
            data_offset = data_end_offset;

            if parent_locator_entry.key_data_offset < 20
                || parent_locator_entry.key_data_offset as usize >= data_size
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid parent locator entry: {} key data offset: {} value out of bounds",
                        parent_locator_entry_index, parent_locator_entry.key_data_offset,
                    ),
                ));
            }
            if parent_locator_entry.key_data_size as usize
                > data_size - (parent_locator_entry.key_data_offset as usize)
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid parent locator entry: {} key data size: {} value out of bounds",
                        parent_locator_entry_index, parent_locator_entry.key_data_size,
                    ),
                ));
            }
            if parent_locator_entry.value_data_offset < 20
                || parent_locator_entry.value_data_offset as usize >= data_size
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid parent locator entry: {} value data offset: {} value out of bounds",
                        parent_locator_entry_index, parent_locator_entry.value_data_offset,
                    ),
                ));
            }
            if parent_locator_entry.value_data_size as usize
                > data_size - (parent_locator_entry.value_data_offset as usize)
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid parent locator entry: {} value data size: {} value out of bounds",
                        parent_locator_entry_index, parent_locator_entry.value_data_size,
                    ),
                ));
            }
            let key_data_offset: usize = parent_locator_entry.key_data_offset as usize;
            let key_data_end_offset: usize =
                key_data_offset + parent_locator_entry.key_data_size as usize;

            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "VhdxParentLocatorKey data of size: {} at offset: {} (0x{:08x})\n",
                    parent_locator_entry.key_data_size,
                    parent_locator_entry.key_data_offset,
                    parent_locator_entry.key_data_offset
                ));
                self.mediator
                    .debug_print_data(&data[key_data_offset..key_data_end_offset], true);
            }
            let key: Ucs2String =
                Ucs2String::from_le_bytes(&data[key_data_offset..key_data_end_offset]);

            let value_data_offset: usize = parent_locator_entry.value_data_offset as usize;
            let value_data_end_offset: usize =
                value_data_offset + parent_locator_entry.value_data_size as usize;

            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "VhdxParentLocatorValue data of size: {} at offset: {} (0x{:08x})\n",
                    parent_locator_entry.value_data_size,
                    parent_locator_entry.value_data_offset,
                    parent_locator_entry.value_data_offset
                ));
                self.mediator
                    .debug_print_data(&data[value_data_offset..value_data_end_offset], true);
            }
            let value: Ucs2String =
                Ucs2String::from_le_bytes(&data[value_data_offset..value_data_end_offset]);

            self.entries.insert(key.to_string(), value);
        }
        Ok(())
    }

    /// Reads the parent locator from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &VfsDataStreamReference,
        data_size: u32,
        position: io::SeekFrom,
    ) -> io::Result<()> {
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "VhdxParentLocator data of size: {} at offset: {} (0x{:08x})\n",
                data_size, offset, offset
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
    // TODO: add test_read_data with invalid header type indicator
    // TODO: add test_read_data with invalid key data offset
    // TODO: add test_read_data with invalid value data offset
    // TODO: add test_read_data with invalid key data size
    // TODO: add test_read_data with invalid value data size
    // TODO: add test_read_at_position
}
