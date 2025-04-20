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

use crate::mediator::Mediator;
use crate::vfs::VfsDataStreamReference;

/// Virtual Hard Disk (VHD) sector bitmap range.
pub struct VhdSectorBitmapRange {
    /// Size.
    pub size: u64,

    /// Value to indicate the bit was set.
    pub is_set: bool,
}

impl VhdSectorBitmapRange {
    /// Creates a new sector bitmap range.
    pub fn new(start_offset: u64, end_offset: u64, is_set: bool) -> Self {
        Self {
            size: end_offset - start_offset,
            is_set: is_set,
        }
    }
}

/// Virtual Hard Disk (VHD) sector bitmap.
pub struct VhdSectorBitmap {
    /// Size.
    size: usize,

    /// Number bytes a single bit represents.
    bytes_per_bit: u16,

    /// The ranges.
    pub ranges: Vec<VhdSectorBitmapRange>,
}

impl VhdSectorBitmap {
    /// Creates a new sector bitmap.
    pub fn new(size: usize, bytes_per_bit: u16) -> Self {
        Self {
            size: size,
            bytes_per_bit: bytes_per_bit,
            ranges: Vec::new(),
        }
    }

    /// Reads the sector bitmap from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let mut offset: u64 = 0;
        let mut range_offset: u64 = 0;
        let mut range_bit_value: u8 = data[0] >> 7;

        for data_offset in 0..data.len() {
            let mut byte_value: u8 = data[data_offset];

            for _ in (0..8).rev() {
                let bit_value: u8 = byte_value & 0x01;
                byte_value >>= 1;

                if bit_value != range_bit_value {
                    self.ranges.push(VhdSectorBitmapRange::new(
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
        self.ranges.push(VhdSectorBitmapRange::new(
            range_offset,
            offset,
            range_bit_value != 0,
        ));

        Ok(())
    }

    /// Reads the sector bitmap from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &VfsDataStreamReference,
        position: io::SeekFrom,
    ) -> io::Result<()> {
        let mut data: Vec<u8> = vec![0; self.size];

        let offset: u64 = match data_stream.with_write_lock() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(crate::error_to_io_error!(error)),
        };
        let mediator = Mediator::current();
        if mediator.debug_output {
            mediator.debug_print(format!(
                "VhdSectorBitmap data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            mediator.debug_print_data(&data, true);
            // TODO: print ranges.
        }
        self.read_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::new_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: VfsDataStreamReference = new_fake_data_stream(test_data);

        let mut test_struct = VhdSectorBitmap::new(32, 512);
        test_struct.read_at_position(&data_stream, io::SeekFrom::Start(0))?;

        assert_eq!(test_struct.ranges.len(), 4);
        assert_eq!(test_struct.ranges[2].size, 32768);
        assert_eq!(test_struct.ranges[2].is_set, true);

        Ok(())
    }
}
