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

use keramics_core::ErrorTrace;
use keramics_layout_map::LayoutMap;
use keramics_types::{Ucs2String, bytes_to_u16_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "substitute_name_offset", data_type = "u16")),
        member(field(name = "substitute_name_size", data_type = "u16")),
        member(field(name = "display_name_offset", data_type = "u16")),
        member(field(name = "display_name_size", data_type = "u16")),
        member(field(name = "symbolic_link_flags", data_type = "u32")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) symbolic link reparse data.
pub struct NtfsSymbolicLinkReparseData {
    /// Substitute name.
    pub substitute_name: Ucs2String,

    /// Display name.
    pub display_name: Ucs2String,
}

impl NtfsSymbolicLinkReparseData {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            substitute_name: Ucs2String::new(),
            display_name: Ucs2String::new(),
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size = data.len();

        if data_size < 12 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported symbolic link reparse data size"
            ));
        }
        let substitute_name_offset: u16 = bytes_to_u16_le!(data, 0);
        let substitute_name_size: u16 = bytes_to_u16_le!(data, 2);

        let data_offset: usize = (substitute_name_offset as usize) + 12;

        if data_offset >= data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid substitute name offset value out of bounds"
            ));
        }
        let data_end_offset: usize = data_offset + (substitute_name_size as usize);

        if data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid substitute name size value out of bounds"
            ));
        }
        self.substitute_name
            .read_data_le(&data[data_offset..data_end_offset]);

        let display_name_offset: u16 = bytes_to_u16_le!(data, 4);
        let display_name_size: u16 = bytes_to_u16_le!(data, 6);

        let data_offset: usize = (display_name_offset as usize) + 12;

        if data_offset >= data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid display name offset value out of bounds"
            ));
        }
        let data_end_offset: usize = data_offset + (display_name_size as usize);

        if data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid display name size value out of bounds"
            ));
        }
        self.display_name
            .read_data_le(&data[data_offset..data_end_offset]);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x0c, 0x00, 0x00, 0xa0, 0x68, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x32, 0x00, 0x00, 0x00,
            0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x00, 0x3a, 0x00, 0x5c, 0x00, 0x74, 0x00,
            0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x69, 0x00, 0x72, 0x00, 0x31, 0x00,
            0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x66, 0x00, 0x69, 0x00,
            0x6c, 0x00, 0x65, 0x00, 0x31, 0x00, 0x5c, 0x00, 0x3f, 0x00, 0x3f, 0x00, 0x5c, 0x00,
            0x78, 0x00, 0x3a, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00,
            0x64, 0x00, 0x69, 0x00, 0x72, 0x00, 0x31, 0x00, 0x5c, 0x00, 0x74, 0x00, 0x65, 0x00,
            0x73, 0x00, 0x74, 0x00, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x31, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct: NtfsSymbolicLinkReparseData = NtfsSymbolicLinkReparseData::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data[8..])?;

        assert_eq!(
            test_struct.substitute_name,
            Ucs2String::from("\\??\\x:\\testdir1\\testfile1")
        );
        assert_eq!(
            test_struct.display_name,
            Ucs2String::from("x:\\testdir1\\testfile1")
        );

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct: NtfsSymbolicLinkReparseData = NtfsSymbolicLinkReparseData::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[8..19]);
        assert!(result.is_err());
    }
}
