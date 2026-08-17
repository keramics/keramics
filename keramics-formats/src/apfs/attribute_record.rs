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
use keramics_types::{bytes_to_u16_le, bytes_to_u64_le};

use super::data_stream_descriptor::ApfsDataStreamDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "flags", data_type = "u16"),
        field(name = "data_size", data_type = "u16"),
    ),
    methods("debug_read_data")
)]
/// Apple File System (APFS) attribute record.
pub struct ApfsAttributeRecord {
    /// Flags.
    pub flags: u16,

    /// Data size.
    pub data_size: u16,

    /// Data stream (object) identifier.
    pub data_stream_identifier: u64,

    /// Inline data.
    pub inline_data: Vec<u8>,

    /// Data stream descriptor.
    pub data_stream_descriptor: Option<ApfsDataStreamDescriptor>,
}

impl ApfsAttributeRecord {
    /// Creates a new attribute record.
    pub fn new() -> Self {
        Self {
            flags: 0,
            data_size: 0,
            data_stream_identifier: 0,
            inline_data: Vec::new(),
            data_stream_descriptor: None,
        }
    }

    /// Reads the attribute record from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 4 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.flags = bytes_to_u16_le!(data, 0);
        self.data_size = bytes_to_u16_le!(data, 2);

        if self.flags & 0x0001 != 0 {
            if self.flags & 0x0002 != 0 {
                return Err(keramics_core::error_trace_new!("Unsupported flags"));
            }
            if self.data_size != 48 {
                return Err(keramics_core::error_trace_new!(
                    "Unsupported record data size"
                ));
            }
            if data_size < 52 {
                return Err(keramics_core::error_trace_new!("Unsupported data size"));
            }
            self.data_stream_identifier = bytes_to_u64_le!(data, 4);

            keramics_core::debug_trace_structure!(ApfsDataStreamDescriptor::debug_read_data(
                &data[12..]
            ));
            let mut data_stream_descriptor: ApfsDataStreamDescriptor =
                ApfsDataStreamDescriptor::new();

            match data_stream_descriptor.read_data(&data[12..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read data stream descriptor"
                    );
                    return Err(error);
                }
            }
            self.data_stream_descriptor = Some(data_stream_descriptor);
        } else if self.flags & 0x0002 != 0 {
            let data_end_offset: usize = 4 + (self.data_size as usize);

            if data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(
                    "Invalid record data size value out of bounds"
                ));
            }
            self.inline_data = data[4..data_end_offset].to_vec();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x02, 0x00, 0x19, 0x00, 0x4d, 0x79, 0x20, 0x31, 0x73, 0x74, 0x20, 0x65, 0x78, 0x74,
            0x65, 0x6e, 0x64, 0x65, 0x64, 0x20, 0x61, 0x74, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74,
            0x65,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = ApfsAttributeRecord::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.flags, 0x0002);
        assert_eq!(test_struct.data_size, 25);
        assert_eq!(test_struct.inline_data, &test_data[4..29]);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = ApfsAttributeRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_invalid_record_data_size() {
        let mut test_struct = ApfsAttributeRecord::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..28]);
        assert!(result.is_err());
    }
}
