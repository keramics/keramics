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

/// New Technologies File System (NTFS) data run typ.
#[derive(Clone, Debug, PartialEq)]
pub enum NtfsDataRunType {
    EndOfList,
    InFile,
    Sparse,
}

/// New Technologies File System (NTFS) data run.
pub struct NtfsDataRun {
    /// Block number.
    pub block_number: u64,

    /// Number of blocks.
    pub number_of_blocks: u64,

    /// Data run type.
    pub run_type: NtfsDataRunType,
}

impl NtfsDataRun {
    /// Creates a new data run.
    pub fn new() -> Self {
        Self {
            block_number: 0,
            number_of_blocks: 0,
            run_type: NtfsDataRunType::InFile,
        }
    }

    /// Reads the data run from a buffer.
    pub fn read_data(&mut self, data: &[u8], last_block_number: u64) -> io::Result<usize> {
        let data_size: usize = data.len();
        if data_size < 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        let sizes_tuple: usize = data[0] as usize;

        let number_of_blocks_size: usize = sizes_tuple & 0x0f;
        let block_number_size: usize = sizes_tuple >> 4;

        let data_run_size: usize = 1 + number_of_blocks_size + block_number_size;
        if data_run_size > data_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported data run size: {}", data_run_size),
            ));
        }
        if number_of_blocks_size > 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported number of blocks size: {}",
                    number_of_blocks_size
                ),
            ));
        }
        if block_number_size > 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported block number size: {}", block_number_size),
            ));
        }
        // A number of blocks size of 0 indicates the end of the data runs.
        if number_of_blocks_size == 0 {
            self.block_number = 0;
            self.number_of_blocks = 0;
            self.run_type = NtfsDataRunType::EndOfList;

            return Ok(1);
        }
        let data_end_offset: usize = 1 + number_of_blocks_size;

        let mut number_of_blocks: u64 = 0;
        for byte_value in data[1..data_end_offset].iter().rev() {
            number_of_blocks <<= 8;
            number_of_blocks |= *byte_value as u64;
        }
        self.number_of_blocks = number_of_blocks;

        let mut data_offset: usize = data_end_offset;

        if block_number_size == 0 {
            self.block_number = 0;
            self.run_type = NtfsDataRunType::Sparse;
        } else {
            let data_end_offset: usize = data_offset + block_number_size;

            let mut block_number: i64 =
                if last_block_number != 0 && data[data_end_offset - 1] & 0x80 != 0 {
                    -1
                } else {
                    0
                };
            for byte_value in data[data_offset..data_end_offset].iter().rev() {
                block_number <<= 8;
                block_number |= *byte_value as i64;
            }
            block_number += last_block_number as i64;

            self.block_number = block_number as u64;
        }
        data_offset += block_number_size;

        Ok(data_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x11, 0x03, 0x37, 0x01, 0x0d, 0x00];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsDataRun::new();

        let test_data: Vec<u8> = get_test_data();
        let read_count: usize = test_struct.read_data(&test_data, 0)?;

        assert_eq!(read_count, 3);
        assert_eq!(test_struct.block_number, 55);
        assert_eq!(test_struct.number_of_blocks, 3);
        assert_eq!(test_struct.run_type, NtfsDataRunType::InFile);

        let read_count: usize = test_struct.read_data(&test_data[3..], 55)?;

        assert_eq!(read_count, 2);
        assert_eq!(test_struct.block_number, 0);
        assert_eq!(test_struct.number_of_blocks, 13);
        assert_eq!(test_struct.run_type, NtfsDataRunType::Sparse);

        let read_count: usize = test_struct.read_data(&test_data[5..], 0)?;

        assert_eq!(read_count, 1);
        assert_eq!(test_struct.block_number, 0);
        assert_eq!(test_struct.number_of_blocks, 0);
        assert_eq!(test_struct.run_type, NtfsDataRunType::EndOfList);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsDataRun::new();
        let result = test_struct.read_data(&test_data[0..0], 0);
        assert!(result.is_err());
    }
}
