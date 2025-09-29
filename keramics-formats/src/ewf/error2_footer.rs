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
    structure(
        byte_order = "little",
        field(name = "checksum", data_type = "u32", format = "hex"),
    ),
    method(name = "debug_read_data")
)]
/// Expert Witness Compression Format (EWF) error2 footer.
pub struct EwfError2Footer {
    /// Checksum.
    pub checksum: u32,
}

impl EwfError2Footer {
    /// Creates a new error2 footer.
    pub fn new() -> Self {
        Self { checksum: 0 }
    }

    /// Reads the error2 footer from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported EWF error2 footer data size"),
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
        return vec![0xaf, 0x03, 0xbb, 0x35];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfError2Footer::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.checksum, 0x35bb03af);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfError2Footer::new();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }
}
