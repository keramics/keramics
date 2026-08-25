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
use keramics_encodings::CharacterEncoding;
use keramics_layout_map::LayoutMap;
use keramics_types::{ByteString, bytes_to_u64_le};

/// Linux Logical Volume Manager (LVM) physical volume header.
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "identifier", data_type = "ByteString<32>"),
        field(name = "volume_size", data_type = "u64"),
    ),
    methods("debug_read_data")
)]
pub struct LinuxLvmPhysicalVolumeHeader {
    /// Identifier.
    pub identifier: ByteString,

    /// Volume size.
    pub volume_size: u64,
}

impl LinuxLvmPhysicalVolumeHeader {
    /// Creates a new physical volume header.
    pub fn new() -> Self {
        Self {
            identifier: ByteString::new_with_encoding(&CharacterEncoding::Ascii),
            volume_size: 0,
        }
    }

    /// Reads the physical volume header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 40 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.identifier.read_data(&data[0..32]);
        self.volume_size = bytes_to_u64_le!(data, 32);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x6b, 0x36, 0x58, 0x5a, 0x5a, 0x66, 0x48, 0x63, 0x69, 0x79, 0x6b, 0x6b, 0x78, 0x66,
            0x63, 0x46, 0x7a, 0x41, 0x32, 0x36, 0x57, 0x48, 0x51, 0x61, 0x53, 0x6f, 0x58, 0x70,
            0x58, 0x63, 0x32, 0x49, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LinuxLvmPhysicalVolumeHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.identifier,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![
                    0x6b, 0x36, 0x58, 0x5a, 0x5a, 0x66, 0x48, 0x63, 0x69, 0x79, 0x6b, 0x6b, 0x78,
                    0x66, 0x63, 0x46, 0x7a, 0x41, 0x32, 0x36, 0x57, 0x48, 0x51, 0x61, 0x53, 0x6f,
                    0x58, 0x70, 0x58, 0x63, 0x32, 0x49,
                ]
            },
        );
        assert_eq!(test_struct.volume_size, 16777216);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = LinuxLvmPhysicalVolumeHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..39]);
        assert!(result.is_err());
    }
}
