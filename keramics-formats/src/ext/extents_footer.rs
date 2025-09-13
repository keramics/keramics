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

use keramics_types::bytes_to_u32_le;
use layout_map::LayoutMap;

#[derive(LayoutMap)]
#[layout_map(
    structure(byte_order = "little", field(name = "checksum", data_type = "u32"),),
    method(name = "debug_read_data")
)]
/// Extended File System (ext) extents footer.
pub struct ExtExtentsFooter {
    checksum: u32,
}

impl ExtExtentsFooter {
    /// Creates a new extents footer.
    pub fn new() -> Self {
        Self { checksum: 0 }
    }

    /// Reads the extents footer from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() != 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        self.checksum = bytes_to_u32_le!(data, 0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x12, 0x34, 0x56, 0x78];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtExtentsFooter::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.checksum, 0x78563412);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ExtExtentsFooter::new();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
