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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

/// BitLocker Drive Encryption (BDE) Encrypt-on-Write (EOW) block bitmap range.
pub struct BdeEowBlockBitmapRange {
    /// Start offset.
    pub start_offset: u64,

    /// End offset.
    pub end_offset: u64,

    /// Value to indicate the bit was set.
    pub is_set: bool,
}

impl BdeEowBlockBitmapRange {
    /// Creates a new bitmap range.
    pub fn new(start_offset: u64, end_offset: u64, is_set: bool) -> Self {
        Self {
            start_offset,
            end_offset,
            is_set,
        }
    }
}

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "[u8; 10]", format = "hex"),
        field(name = "header_size", data_type = "u16"),
        field(name = "physical_sector_size", data_type = "u32"),
        field(name = "number_of_bits", data_type = "u32"),
        field(name = "sequence_number", data_type = "u32"),
        field(name = "unknown1", data_type = "u32"),
        field(name = "flags", data_type = "u32", format = "hex"),
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker Drive Encryption (BDE) Encrypt-on-Write (EOW) block record.
pub struct BdeEowBlockRecord {
    /// Number of bytes a single bit represents.
    bytes_per_bit: u32,

    /// Sequence number.
    pub sequence_number: u32,

    /// Bitmap ranges.
    pub ranges: Vec<BdeEowBlockBitmapRange>,
}

impl BdeEowBlockRecord {
    /// Creates a new Encrypt-on-Write (EOW) block record.
    pub fn new(bytes_per_bit: u32) -> Self {
        Self {
            bytes_per_bit,
            sequence_number: 0,
            ranges: Vec::new(),
        }
    }

    /// Reads the bitmap from a buffer.
    fn read_bitmap(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let mut offset: u64 = 0;
        let mut range_offset: u64 = 0;
        let mut range_bit_value: u8 = data[0] & 0x01;

        for byte_value in data.iter() {
            let mut bit_values: u8 = *byte_value;
            for _ in 0..8 {
                let bit_value: u8 = bit_values & 0x01;
                bit_values >>= 1;

                if bit_value != range_bit_value {
                    self.ranges.push(BdeEowBlockBitmapRange::new(
                        range_offset,
                        offset,
                        range_bit_value != 0,
                    ));
                    range_offset = offset;
                    range_bit_value = bit_value;
                }
                offset += self.bytes_per_bit as u64;
            }
        }
        self.ranges.push(BdeEowBlockBitmapRange::new(
            range_offset,
            offset,
            range_bit_value != 0,
        ));
        Ok(())
    }

    /// Reads the Encrypt-on-Write (EOW) block record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 36 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..10] != b"FVE-EOWBR\x00" {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let header_size: u16 = bytes_to_u16_le!(data, 10);

        if header_size != 36 {
            return Err(keramics_core::error_trace_new!("Unsupported header size"));
        }
        let checksum: u32 = bytes_to_u32_le!(data, 32);

        if checksum != 0 {
            let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);

            crc32_context.update(&data[0..32]);
            crc32_context.update(&[0; 4]);
            crc32_context.update(&data[36..data_size]);

            let calculated_checksum: u32 = crc32_context.finalize();

            if checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    checksum, calculated_checksum
                )));
            }
        }
        let number_of_bits: u32 = bytes_to_u32_le!(data, 16);

        self.sequence_number = bytes_to_u32_le!(data, 20);

        let bitmap_end_offset: usize = 36 + (number_of_bits.next_multiple_of(8) as usize);

        if bitmap_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid bitmap size value out of bounds"
            ));
        }
        match self.read_bitmap(&data[36..bitmap_end_offset]) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read bitmap");
                Err(error)
            }
        }
    }

    /// Reads the Encrypt-on-Write (EOW) block record from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: usize,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        // Note that 65536 is an arbitrary chosen limit.
        if data_size < 60 || data_size > 65536 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported Encrypt-on-Write (EOW) block record size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data_and_structure!(
            "BdeEowBlockRecord",
            offset,
            &data,
            data_size,
            BdeEowBlockRecord::debug_read_data(&data)
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

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        vec![
            0x46, 0x56, 0x45, 0x2d, 0x45, 0x4f, 0x57, 0x42, 0x52, 0x00, 0x24, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x8b, 0x52, 0x85, 0x70, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowBlockRecord::new(2097152);
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.sequence_number, 5);
        assert_eq!(test_struct.ranges.len(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeEowBlockRecord::new(2097152);
        let result = test_struct.read_data(&test_data[0..35]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = BdeEowBlockRecord::new(2097152);
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = BdeEowBlockRecord::new(2097152);
        test_struct.read_at_position(&data_stream, 512, SeekFrom::Start(0))?;

        Ok(())
    }
}
