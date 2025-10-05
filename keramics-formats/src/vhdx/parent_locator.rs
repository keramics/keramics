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

use std::collections::HashMap;
use std::io;
use std::io::SeekFrom;

use keramics_core::DataStreamReference;
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_types::Ucs2String;

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
    fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let mut parent_locator_header: VhdxParentLocatorHeader = VhdxParentLocatorHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(VhdxParentLocatorHeader::debug_read_data(data));
        }
        parent_locator_header.read_data(data)?;

        let mut data_offset: usize = 20;
        let data_size: usize = data.len();

        for parent_locator_entry_index in 0..parent_locator_header.number_of_entries {
            let data_end_offset: usize = data_offset + 12;
            if data_end_offset > data_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid number of entries: {} value out of bounds",
                        parent_locator_header.number_of_entries
                    ),
                ));
            }
            let mut parent_locator_entry: VhdxParentLocatorEntry = VhdxParentLocatorEntry::new();

            if self.mediator.debug_output {
                self.mediator
                    .debug_print(VhdxParentLocatorEntry::debug_read_data(
                        &data[data_offset..data_end_offset],
                    ));
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
        data_stream: &DataStreamReference,
        data_size: u32,
        position: SeekFrom,
    ) -> io::Result<()> {
        // Note that 65536 is an arbitrary chosen limit.
        if data_size < 20 || data_size > 65536 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported parent locator data size: {} value out of bounds",
                    data_size
                ),
            ));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 = match data_stream.write() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
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

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xb7, 0xef, 0x4a, 0xb0, 0x9e, 0xd1, 0x81, 0x4a, 0xb7, 0x89, 0x25, 0xb8, 0xe9, 0x44,
            0x59, 0x13, 0x00, 0x00, 0x05, 0x00, 0x50, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00,
            0x1c, 0x00, 0x4c, 0x00, 0xb8, 0x00, 0x00, 0x00, 0xde, 0x00, 0x00, 0x00, 0x26, 0x00,
            0x58, 0x00, 0x36, 0x01, 0x00, 0x00, 0x50, 0x01, 0x00, 0x00, 0x1a, 0x00, 0x24, 0x00,
            0x74, 0x01, 0x00, 0x00, 0x8a, 0x01, 0x00, 0x00, 0x16, 0x00, 0xb4, 0x00, 0x3e, 0x02,
            0x00, 0x00, 0x5c, 0x02, 0x00, 0x00, 0x1e, 0x00, 0x4c, 0x00, 0x70, 0x00, 0x61, 0x00,
            0x72, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x5f, 0x00, 0x6c, 0x00, 0x69, 0x00,
            0x6e, 0x00, 0x6b, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0x7b, 0x00, 0x37, 0x00,
            0x35, 0x00, 0x38, 0x00, 0x34, 0x00, 0x66, 0x00, 0x38, 0x00, 0x66, 0x00, 0x62, 0x00,
            0x2d, 0x00, 0x33, 0x00, 0x36, 0x00, 0x64, 0x00, 0x33, 0x00, 0x2d, 0x00, 0x34, 0x00,
            0x30, 0x00, 0x39, 0x00, 0x31, 0x00, 0x2d, 0x00, 0x61, 0x00, 0x66, 0x00, 0x62, 0x00,
            0x35, 0x00, 0x2d, 0x00, 0x62, 0x00, 0x31, 0x00, 0x61, 0x00, 0x66, 0x00, 0x65, 0x00,
            0x35, 0x00, 0x38, 0x00, 0x37, 0x00, 0x62, 0x00, 0x66, 0x00, 0x61, 0x00, 0x38, 0x00,
            0x7d, 0x00, 0x61, 0x00, 0x62, 0x00, 0x73, 0x00, 0x6f, 0x00, 0x6c, 0x00, 0x75, 0x00,
            0x74, 0x00, 0x65, 0x00, 0x5f, 0x00, 0x77, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x33, 0x00,
            0x32, 0x00, 0x5f, 0x00, 0x70, 0x00, 0x61, 0x00, 0x74, 0x00, 0x68, 0x00, 0x43, 0x00,
            0x3a, 0x00, 0x5c, 0x00, 0x50, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x6a, 0x00, 0x65, 0x00,
            0x63, 0x00, 0x74, 0x00, 0x73, 0x00, 0x5c, 0x00, 0x64, 0x00, 0x66, 0x00, 0x76, 0x00,
            0x66, 0x00, 0x73, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00,
            0x5f, 0x00, 0x64, 0x00, 0x61, 0x00, 0x74, 0x00, 0x61, 0x00, 0x5c, 0x00, 0x6e, 0x00,
            0x74, 0x00, 0x66, 0x00, 0x73, 0x00, 0x2d, 0x00, 0x70, 0x00, 0x61, 0x00, 0x72, 0x00,
            0x65, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x2e, 0x00, 0x76, 0x00, 0x68, 0x00, 0x64, 0x00,
            0x78, 0x00, 0x72, 0x00, 0x65, 0x00, 0x6c, 0x00, 0x61, 0x00, 0x74, 0x00, 0x69, 0x00,
            0x76, 0x00, 0x65, 0x00, 0x5f, 0x00, 0x70, 0x00, 0x61, 0x00, 0x74, 0x00, 0x68, 0x00,
            0x2e, 0x00, 0x5c, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x66, 0x00, 0x73, 0x00, 0x2d, 0x00,
            0x70, 0x00, 0x61, 0x00, 0x72, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x2e, 0x00,
            0x76, 0x00, 0x68, 0x00, 0x64, 0x00, 0x78, 0x00, 0x76, 0x00, 0x6f, 0x00, 0x6c, 0x00,
            0x75, 0x00, 0x6d, 0x00, 0x65, 0x00, 0x5f, 0x00, 0x70, 0x00, 0x61, 0x00, 0x74, 0x00,
            0x68, 0x00, 0x5c, 0x00, 0x5c, 0x00, 0x3f, 0x00, 0x5c, 0x00, 0x56, 0x00, 0x6f, 0x00,
            0x6c, 0x00, 0x75, 0x00, 0x6d, 0x00, 0x65, 0x00, 0x7b, 0x00, 0x39, 0x00, 0x63, 0x00,
            0x64, 0x00, 0x66, 0x00, 0x38, 0x00, 0x64, 0x00, 0x66, 0x00, 0x61, 0x00, 0x2d, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x2d, 0x00, 0x35, 0x00, 0x30, 0x00, 0x31, 0x00, 0x66, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x7d, 0x00,
            0x5c, 0x00, 0x50, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x6a, 0x00, 0x65, 0x00, 0x63, 0x00,
            0x74, 0x00, 0x73, 0x00, 0x5c, 0x00, 0x64, 0x00, 0x66, 0x00, 0x76, 0x00, 0x66, 0x00,
            0x73, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x5f, 0x00,
            0x64, 0x00, 0x61, 0x00, 0x74, 0x00, 0x61, 0x00, 0x5c, 0x00, 0x6e, 0x00, 0x74, 0x00,
            0x66, 0x00, 0x73, 0x00, 0x2d, 0x00, 0x70, 0x00, 0x61, 0x00, 0x72, 0x00, 0x65, 0x00,
            0x6e, 0x00, 0x74, 0x00, 0x2e, 0x00, 0x76, 0x00, 0x68, 0x00, 0x64, 0x00, 0x78, 0x00,
            0x70, 0x00, 0x61, 0x00, 0x72, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x5f, 0x00,
            0x6c, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x6b, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00,
            0x32, 0x00, 0x7b, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x2d, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x2d, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00,
            0x30, 0x00, 0x30, 0x00, 0x30, 0x00, 0x7d, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VhdxParentLocator::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entries.len(), 5);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_type_indicator() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VhdxParentLocator::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    // TODO: add test_read_data with invalid key data offset
    // TODO: add test_read_data with invalid value data offset
    // TODO: add test_read_data with invalid key data size
    // TODO: add test_read_data with invalid value data size

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let test_data_size: u32 = test_data.len() as u32;
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = VhdxParentLocator::new();
        test_struct.read_at_position(&data_stream, test_data_size, SeekFrom::Start(0))?;

        assert_eq!(test_struct.entries.len(), 5);

        Ok(())
    }
}
