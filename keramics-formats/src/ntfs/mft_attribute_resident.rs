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
use keramics_types::{bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "data_size", data_type = "u32")),
        member(field(name = "data_offset", data_type = "u16")),
        member(field(name = "indexed_flag", data_type = "u8")),
        member(field(name = "unknown1", data_type = "u8")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) Master File Table (MFT) resident attribute.
pub struct NtfsMftAttributeResident {
    /// Data size.
    pub data_size: u32,

    /// Data offset.
    pub data_offset: u16,
}

impl NtfsMftAttributeResident {
    /// Creates a new MFT resident attribute.
    pub fn new() -> Self {
        Self {
            data_size: 0,
            data_offset: 0,
        }
    }

    /// Reads the MFT resident attribute from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported MFT resident attribute data size"),
            ));
        }
        self.data_size = bytes_to_u32_le!(data, 0);
        self.data_offset = bytes_to_u16_le!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x48, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsMftAttributeResident::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.data_size, 72);
        assert_eq!(test_struct.data_offset, 24);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftAttributeResident::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
