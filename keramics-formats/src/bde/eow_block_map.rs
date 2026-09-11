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
        field(name = "signature", data_type = "[u8; 10]", format = "hex"),
        field(name = "header_size", data_type = "u16"),
        field(name = "block_map_size", data_type = "u32"),
        field(name = "block_map_index", data_type = "u32"),
        field(name = "volume_region_offset", data_type = "u64", format = "hex"),
        field(name = "volume_region_size", data_type = "u64"),
        field(name = "relocation_log_area_offset", data_type = "u64", format = "hex"),
        field(name = "block_record_offset1", data_type = "u32", format = "hex"),
        field(name = "block_record_offset2", data_type = "u32", format = "hex"),
        field(name = "block_record_size", data_type = "u32"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker Drive Encryption (BDE) Encrypt-on-Write (EOW) block map.
pub struct BdeEowBlockMap {
    /// Block map size.
    pub block_map_size: u32,

    /// Volume region offset.
    pub volume_region_offset: u64,

    /// Volume region size.
    pub volume_region_size: u64,

    /// Encrypt-on-Write relocation log area offset.
    pub relocation_log_area_offset: u64,

    /// Block record offset 1.
    pub block_record_offset1: u32,

    /// Block record offset 2.
    pub block_record_offset2: u32,

    /// Block record size.
    pub block_record_size: u32,
}

impl BdeEowBlockMap {
    /// Creates a new Encrypt-on-Write (EOW) block map.
    pub fn new() -> Self {
        Self {
            block_map_size: 0,
            volume_region_offset: 0,
            volume_region_size: 0,
            relocation_log_area_offset: 0,
            block_record_offset1: 0,
            block_record_offset2: 0,
            block_record_size: 0,
        }
    }

    /// Reads the Encrypt-on-Write (EOW) block map from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 60 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..10] != b"FVE-EOWBM\x00" {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let header_size: u16 = bytes_to_u16_le!(data, 10);

        if header_size != 60 {
            return Err(keramics_core::error_trace_new!("Unsupported header size"));
        }
        let checksum: u32 = bytes_to_u32_le!(data, 56);

        if checksum != 0 {
            let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

            crc32_context.update(&data[0..56]);
            crc32_context.update(&[0; 4]);
            crc32_context.update(&data[60..data_size]);

            let calculated_checksum: u32 = crc32_context.finalize();

            if checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    checksum, calculated_checksum
                )));
            }
        }
        self.block_map_size = bytes_to_u32_le!(data, 12);
        self.volume_region_offset = bytes_to_u64_le!(data, 20);
        self.volume_region_size = bytes_to_u64_le!(data, 28);
        self.relocation_log_area_offset = bytes_to_u64_le!(data, 36);
        self.block_record_offset1 = bytes_to_u32_le!(data, 44);
        self.block_record_offset2 = bytes_to_u32_le!(data, 48);
        self.block_record_size = bytes_to_u32_le!(data, 52);

        Ok(())
    }

    /// Reads the Encrypt-on-Write (EOW) block map from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: usize,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        if data_size != 512 && data_size != 4096 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported Encrypt-on-Write (EOW) block map size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data_and_structure!(
            "BdeEowBlockMap",
            offset,
            &data,
            data_size,
            BdeEowBlockMap::debug_read_data(&data)
        );
        match self.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read Encrypt-on-Write (EOW) block map at offset: {} (0x{:08x})",
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

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x46, 0x56, 0x45, 0x2d, 0x45, 0x4f, 0x57, 0x42, 0x4d, 0x00, 0x3c, 0x00, 0x00, 0x06,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xe0, 0x1e, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x21, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x33, 0xd6, 0xf1, 0x6a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowBlockMap::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.block_map_size, 1536);
        assert_eq!(test_struct.volume_region_offset, 0x00002000);
        assert_eq!(test_struct.volume_region_size, 18800640);
        assert_eq!(test_struct.relocation_log_area_offset, 0x02214000);
        assert_eq!(test_struct.block_record_offset1, 0x00000200);
        assert_eq!(test_struct.block_record_offset2, 0x00000400);
        assert_eq!(test_struct.block_record_size, 512);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowBlockMap::new();
        let result = test_struct.read_data(&test_data[0..59]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = BdeEowBlockMap::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = BdeEowBlockMap::new();
        test_struct.read_at_position(&data_stream, 512, SeekFrom::Start(0))?;

        Ok(())
    }
}
