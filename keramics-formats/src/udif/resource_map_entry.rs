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
use keramics_types::{ByteString, bytes_to_u16_be};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "name", data_type = "ByteString<4>"),
        field(
            name = "number_of_resource_descriptors",
            data_type = "u16",
            modifier = "+ 1"
        ),
        field(name = "resource_descriptors_offset", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Universal Disk Image Format (UDIF) resource map entry.
pub struct UdifResourceMapEntry {
    /// Name.
    pub name: ByteString,

    /// Number of resource descriptors.
    pub number_of_resource_descriptors: u16,

    /// Resource descriptors offset.
    pub resource_descriptors_offset: u16,
}

impl UdifResourceMapEntry {
    /// Creates a new resource map entry.
    pub fn new() -> Self {
        Self {
            name: ByteString::new(),
            number_of_resource_descriptors: 0,
            resource_descriptors_offset: 0,
        }
    }

    /// Reads the resource map entry from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 8 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.name.read_data(&data[0..4]);
        self.number_of_resource_descriptors = bytes_to_u16_be!(data, 4) + 1;
        self.resource_descriptors_offset = bytes_to_u16_be!(data, 6);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![0x70, 0x6c, 0x73, 0x74, 0x00, 0x00, 0x00, 0x12];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceMapEntry::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.name, ByteString::from("plst"));
        assert_eq!(test_struct.number_of_resource_descriptors, 1);
        assert_eq!(test_struct.resource_descriptors_offset, 18);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifResourceMapEntry::new();
        let result = test_struct.read_data(&test_data[0..7]);
        assert!(result.is_err());
    }
}
