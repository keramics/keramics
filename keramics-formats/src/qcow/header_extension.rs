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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u32_be;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "[u8; 4]", format = "hex"),
        field(name = "data_size", data_type = "u32"),
    ),
    methods("debug_read_data")
)]
/// QEMU Copy-On-Write (QCOW) (file) header extension.
pub struct QcowHeaderExtension {
    /// Signature.
    pub signature: [u8; 4],

    /// Data size.
    pub data_size: u32,
}

impl QcowHeaderExtension {
    /// Creates a new header extension.
    pub fn new() -> Self {
        Self {
            signature: [0; 4],
            data_size: 0,
        }
    }

    /// Reads the header extension from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.signature.copy_from_slice(&data[0..4]);
        self.data_size = bytes_to_u32_be!(data, 4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x44, 0x41, 0x54, 0x41, 0x00, 0x00, 0x00, 0x20, 0x73, 0x70, 0x65, 0x63, 0x69, 0x6d,
            0x65, 0x6e, 0x73, 0x2f, 0x71, 0x65, 0x6d, 0x75, 0x2d, 0x69, 0x6d, 0x67, 0x2f, 0x64,
            0x61, 0x74, 0x61, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x72, 0x61, 0x77,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = QcowHeaderExtension::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(&test_struct.signature, &test_data[0..4]);
        assert_eq!(test_struct.data_size, 32);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = QcowHeaderExtension::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
