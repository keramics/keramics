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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::{DataStreamReference, ErrorTrace};

use super::group_descriptor::ExtGroupDescriptor;

/// Extended File System (ext) group descriptor table.
pub struct ExtGroupDescriptorTable {
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
    pub fn new(
        format_version: u8,
        metadata_checksum_seed: Option<u32>,
        group_descriptor_size: usize,
        first_group_number: u32,
        number_of_group_descriptors: u32,
    ) -> Self {
        Self {
            format_version,
            metadata_checksum_seed,
            group_descriptor_size,
            first_group_number,
            number_of_group_descriptors,
            entries: Vec::new(),
        }
    }

    /// Reads the group descriptor table from a buffer.
    fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let empty_group_descriptor: Vec<u8> = vec![0; self.group_descriptor_size];
        let mut data_offset: usize = 0;

        for group_number in 0..self.number_of_group_descriptors {
            let mut group_descriptor: ExtGroupDescriptor = ExtGroupDescriptor::new();

            let data_end_offset: usize = data_offset + self.group_descriptor_size;

            if &data[data_offset..data_end_offset] == &empty_group_descriptor {
                break;
            }
            // Note that the ExtGroupDescriptor read functions rely on the size of the group descriptor.
            keramics_core::debug_trace_structure!(ExtGroupDescriptor::debug_read_data(
                self.format_version,
                &data[data_offset..data_end_offset]
            ));
            match group_descriptor
                .read_data(self.format_version, &data[data_offset..data_end_offset])
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read group descriptor");
                    return Err(error);
                }
            }
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
                        return Err(keramics_core::error_trace_new!(format!(
                            "Mismatch between stored: 0x{:04x} and calculated: 0x{:04x} checksums",
                            group_descriptor.checksum, calculated_checksum
                        )));
                    }
                }
                None => {}
            };
            data_offset = data_end_offset;

            self.entries.push(group_descriptor);
        }
        Ok(())
    }

    /// Reads the group descriptor table from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize =
            (self.number_of_group_descriptors as usize) * self.group_descriptor_size;

        // Note that 16777216 is an arbitrary chosen limit.
        if data_size == 0 || data_size > 16777216 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported parent locator data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("ExtGroupDescriptorTable", offset, &data, data_size);

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
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct: ExtGroupDescriptorTable =
            ExtGroupDescriptorTable::new(2, None, 32, 0, 1);
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entries.len(), 1);

        Ok(())
    }

    // TODO: add test_read_at_position
}
