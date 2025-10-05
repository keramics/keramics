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

use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u32_le;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "chunk_data_offset", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data")
)]
/// Expert Witness Compression Format (EWF) table entry.
pub struct EwfTableEntry {
    /// Chunk data offset.
    pub chunk_data_offset: u32,
}

impl EwfTableEntry {
    /// Creates a new table entry.
    pub fn new() -> Self {
        Self {
            chunk_data_offset: 0,
        }
    }

    /// Determines if the table entry is compressed.
    pub fn is_compressed(&self) -> bool {
        (self.chunk_data_offset & 0x80000000) != 0
    }

    /// Reads the table entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF table entry data size"),
            ));
        }
        self.chunk_data_offset = bytes_to_u32_le!(data, 0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x00, 0x08, 0x00, 0x80];
    }

    #[test]
    fn test_is_compressed() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.is_compressed(), true);

        Ok(())
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfTableEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.chunk_data_offset, 0x80000800);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfTableEntry::new();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
