/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use layout_map::LayoutMap;

use super::constants::*;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "[u8; 4]", format = "hex"),
        field(name = "format_version", data_type = "u32")
    ),
    method(name = "debug_read_data")
)]
/// QEMU Copy-On-Write (QCOW) file header.
pub struct QcowFileHeaderCommon {
    pub format_version: u32,
}

impl QcowFileHeaderCommon {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self { format_version: 0 }
    }

    /// Reads the file header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported data size"),
            ));
        }
        if data[0..4] != QCOW_FILE_HEADER_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported signature"),
            ));
        }
        self.format_version = crate::bytes_to_u32_be!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data = get_test_data();

        let mut test_struct = QcowFileHeaderCommon::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format_version, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data = get_test_data();

        let mut test_struct = QcowFileHeaderCommon::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = QcowFileHeaderCommon::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
