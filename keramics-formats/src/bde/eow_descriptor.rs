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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le, bytes_to_u64_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 8]", format = "hex"),
        field(name = "header_size", data_type = "u16"),
        field(name = "data_size", data_type = "u16"),
        field(name = "logical_sector_size", data_type = "u32"),
        field(name = "physical_sector_size", data_type = "u32"),
        field(name = "relocation_block_size", data_type = "u32"),
        field(name = "relocation_log_area_size", data_type = "u32"),
        field(name = "relocation_log_entry_size", data_type = "u32"),
        field(name = "number_of_offsets", data_type = "u32"),
        field(name = "checksum", data_type = "u32", format = "hex"),
        field(name = "eow_descriptor_offset1", data_type = "u64", format = "hex"),
        field(name = "eow_descriptor_offset2", data_type = "u64", format = "hex"),
        field(
            name = "block_map_area_offsets",
            data_type = "[u64; 8]",
            format = "hex"
        ),
    ),
    methods("debug_read_data")
)]
/// BitLocker Drive Encryption (BDE) Encrypt-on-Write (EOW) descriptor.
pub struct BdeEowDescriptor {
    /// Physical sector size.
    pub physical_sector_size: u32,

    /// Encrypt-on-Write relocation block size.
    pub relocation_block_size: u32,

    /// Encrypt-on-Write relocation log area size.
    pub relocation_log_area_size: u32,

    /// Block map area offsets.
    pub block_map_area_offsets: Vec<u64>,
}

impl BdeEowDescriptor {
    /// Creates a new Encrypt-on-Write (EOW) descriptor.
    pub fn new() -> Self {
        Self {
            physical_sector_size: 0,
            relocation_block_size: 0,
            relocation_log_area_size: 0,
            block_map_area_offsets: Vec::new(),
        }
    }

    /// Reads the Encrypt-on-Write (EOW) descriptor from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 56 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..8] != b"FVE-EOW\x00" {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let header_size: u16 = bytes_to_u16_le!(data, 8);

        if header_size != 56 {
            return Err(keramics_core::error_trace_new!("Unsupported header size"));
        }
        let data_end_offset: usize = bytes_to_u16_le!(data, 10) as usize;

        if data_end_offset < 56 || data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid data size value out of bounds"
            ));
        }
        let checksum: u32 = bytes_to_u32_le!(data, 36);

        if checksum != 0 {
            let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

            crc32_context.update(&data[0..36]);
            crc32_context.update(&[0; 4]);
            crc32_context.update(&data[40..data_end_offset]);

            let calculated_checksum: u32 = crc32_context.finalize();

            if checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    checksum, calculated_checksum
                )));
            }
        }
        self.physical_sector_size = bytes_to_u32_le!(data, 16);
        self.relocation_block_size = bytes_to_u32_le!(data, 20);
        self.relocation_log_area_size = bytes_to_u32_le!(data, 24);

        let number_of_offsets: u32 = bytes_to_u32_le!(data, 32);

        if (number_of_offsets as usize) > (data_size - 56) / 8 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of offsets value out of bounds"
            ));
        }
        let offsets_end_offset: usize = 56 + ((number_of_offsets as usize) * 8);

        for chunk in data[56..offsets_end_offset].chunks_exact(8) {
            let offset: u64 = bytes_to_u64_le!(chunk, 0);
            self.block_map_area_offsets.push(offset);
        }
        Ok(())
    }

    /// Reads the Encrypt-on-Write (EOW) block record from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: usize,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data_and_structure!(
            "BdeEowDescriptor",
            offset,
            &data,
            data_size,
            BdeEowDescriptor::debug_read_data(&data)
        );
        match self.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read Encrypt-on-Write (EOW) block record at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x46, 0x56, 0x45, 0x2d, 0x45, 0x4f, 0x57, 0x00, 0x38, 0x00, 0x68, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x0c, 0x02, 0x00,
            0x00, 0x04, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, 0x25, 0x95, 0x3a, 0x6e, 0x00, 0x20,
            0x21, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcf, 0x02, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x30, 0x21, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x59, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x60, 0x95, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0xcf, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x0a, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60,
            0x44, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowDescriptor::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.physical_sector_size, 512);
        assert_eq!(test_struct.relocation_block_size, 2097152);
        assert_eq!(test_struct.relocation_log_area_size, 134144);
        assert_eq!(test_struct.block_map_area_offsets.len(), 6);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowDescriptor::new();
        let result = test_struct.read_data(&test_data[0..55]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = BdeEowDescriptor::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = BdeEowDescriptor::new();
        test_struct.read_at_position(&data_stream, 512, SeekFrom::Start(0))?;

        Ok(())
    }
}
