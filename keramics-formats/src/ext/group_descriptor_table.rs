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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::DataStreamReference;
use keramics_core::mediator::{Mediator, MediatorReference};

use super::features::ExtFeatures;
use super::group_descriptor::ExtGroupDescriptor;

/// Extended File System (ext) group descriptor table.
pub struct ExtGroupDescriptorTable {
    /// Mediator.
    mediator: MediatorReference,

    /// Format version.
    format_version: u8,

    /// Metadata checksum seed.
    metadata_checksum_seed: Option<u32>,

    /// Group descriptor size.
    group_descriptor_size: usize,

    /// First group number.
    first_group_number: u32,

    /// Number of group descriptors.
    number_of_group_descriptors: u32,

    /// Entries.
    pub entries: Vec<ExtGroupDescriptor>,
}

impl ExtGroupDescriptorTable {
    /// Creates a new group descriptor table.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            format_version: 2,
            metadata_checksum_seed: None,
            group_descriptor_size: 32,
            first_group_number: 0,
            number_of_group_descriptors: 0,
            entries: Vec::new(),
        }
    }

    /// Initializes the group descriptor table.
    pub fn initialize(
        &mut self,
        features: &ExtFeatures,
        first_group_number: u32,
        number_of_group_descriptors: u32,
    ) {
        self.format_version = features.get_format_version();
        self.metadata_checksum_seed = features.get_metadata_checksum_seed();
        self.group_descriptor_size = features.get_group_descriptor_size() as usize;
        self.first_group_number = first_group_number;
        self.number_of_group_descriptors = number_of_group_descriptors;
    }

    /// Reads the group descriptor table from a buffer.
    fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let empty_group_descriptor: Vec<u8> = vec![0; self.group_descriptor_size];
        let mut data_offset: usize = 0;

        for group_number in 0..self.number_of_group_descriptors {
            let mut group_descriptor: ExtGroupDescriptor = ExtGroupDescriptor::new();

            let data_end_offset: usize = data_offset + self.group_descriptor_size;

            if &data[data_offset..data_end_offset] == &empty_group_descriptor {
                break;
            }
            if self.mediator.debug_output {
                self.mediator.debug_print(
                    group_descriptor
                        .debug_read_data(self.format_version, &data[data_offset..data_end_offset]),
                );
            }
            group_descriptor.read_data(self.format_version, &data[data_offset..data_end_offset])?;

            match self.metadata_checksum_seed {
                Some(checksum_seed) => {
                    // TODO: add support for crc16 used by EXT4_FEATURE_RO_COMPAT_GDT_CSUM
                    let mut crc32_context: ReversedCrc32Context =
                        ReversedCrc32Context::new(0x82f63b78, checksum_seed);

                    let group_number_data: [u8; 4] =
                        (self.first_group_number + group_number).to_le_bytes();
                    crc32_context.update(&group_number_data);
                    crc32_context.update(&data[data_offset..data_offset + 30]);
                    crc32_context.update(&[0; 2]);
                    crc32_context.update(&data[data_offset + 32..data_end_offset]);

                    let mut calculated_checksum: u32 = crc32_context.finalize();
                    calculated_checksum = (0xffffffff - calculated_checksum) & 0x0000ffff;

                    if group_descriptor.checksum != 0
                        && (group_descriptor.checksum as u32) != calculated_checksum
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "Mismatch between stored: 0x{:04x} and calculated: 0x{:04x} ext group descriptor table checksums",
                                group_descriptor.checksum, calculated_checksum
                            ),
                        ));
                    }
                }
                None => {}
            };
            data_offset = data_end_offset;

            self.entries.push(group_descriptor);
        }
        Ok(())
    }

    /// Reads the metadata table from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: io::SeekFrom,
    ) -> io::Result<()> {
        let data_size: usize =
            (self.number_of_group_descriptors as usize) * self.group_descriptor_size;
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 = match data_stream.write() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "ExtGroupDescriptorTable data of size: {} at offset: {} (0x{:08x})\n",
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

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x12, 0x00, 0x00, 0x00, 0x13, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x58, 0x0f,
            0xf0, 0x03, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let features: ExtFeatures = ExtFeatures::new();
        let mut test_struct: ExtGroupDescriptorTable = ExtGroupDescriptorTable::new();
        test_struct.initialize(&features, 0, 1);
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entries.len(), 1);

        Ok(())
    }

    // TODO: add test_read_at_position
}
