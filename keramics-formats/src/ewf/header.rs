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

use std::collections::HashMap;
use std::io::SeekFrom;

use keramics_compression::ZlibContext;
use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};

use super::enums::EwfHeaderValueType;
use super::header_value::EwfHeaderValue;
use super::object_storage::EwfByteObjectStorage;

/// Expert Witness Compression Format (EWF) header.
pub struct EwfHeader {
    /// Mediator.
    mediator: MediatorReference,
}

impl EwfHeader {
    /// Creates a new header.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
        }
    }

    /// Reads the header from a buffer.
    pub fn read_data(
        &mut self,
        data: &[u8],
        header_values: &mut HashMap<EwfHeaderValueType, EwfHeaderValue>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        // On average the uncompressed header will be more than twice as large
        // as the compressed header.
        let mut header_data: Vec<u8> = vec![0; data_size * 4];

        let mut zlib_context: ZlibContext = ZlibContext::new();

        match zlib_context.decompress(data, &mut header_data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decompress header data");
                return Err(error);
            }
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "Uncompressed header data of size: {}\n",
                zlib_context.uncompressed_data_size,
            ));
            self.mediator.debug_print_data(&header_data, true);
        }
        let mut object_storage: EwfByteObjectStorage = EwfByteObjectStorage::new(&header_data);

        let number_of_categories: u8 = match object_storage.next_line() {
            Some(line) => match line {
                // "1"
                [b'1'] => 1,
                // "3"
                [b'3'] => 3,
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid header data - unsupported number of categories"
                    ));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing number of categories"
                ));
            }
        };
        // TODO: if number_of_categories == 1 then format is at least EnCase 1
        // TODO: if number_of_categories == 3 then format is at least linen 5

        match object_storage.next_line() {
            Some(line) => match line {
                // "main"
                [b'm', b'a', b'i', b'n'] => {}
                _ => {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid header data - unsupported category"
                    ));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing category"
                ));
            }
        };
        let value_types_line: &[u8] = match object_storage.next_line() {
            Some(line) => line,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing value types"
                ));
            }
        };
        let values_line: &[u8] = match object_storage.next_line() {
            Some(line) => line,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing values"
                ));
            }
        };
        let value_types: Vec<&[u8]> = value_types_line
            .split(|byte| *byte == b'\t')
            .collect::<Vec<&[u8]>>();
        let values: Vec<&[u8]> = values_line
            .split(|byte| *byte == b'\t')
            .collect::<Vec<&[u8]>>();

        let number_of_values: usize = values.len();

        if number_of_values != value_types.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid header data - number of value types does not match number of values"
            ));
        }
        // TODO: if number_of_values == 9 then format is EnCase 1
        // TODO: if number_of_values == 11 then format is EnCase 2 - 3 or FTK Imager
        // TODO: if number_of_values == 10 then format is EnCase 4 - 7
        // TODO: if number_of_values == 16 then format is linen 5 - 7

        // TODO: store header values in hash map, use enum for value types

        for value_index in 0..number_of_values {
            let header_value_type: EwfHeaderValueType = match value_types[value_index] {
                // "a" => description
                [b'a'] => EwfHeaderValueType::Description,
                // "av" => acquisition software version
                [b'a', b'v'] => EwfHeaderValueType::Version,
                // "c" => case number
                [b'c'] => EwfHeaderValueType::CaseNumber,
                // "e" => examiner name
                [b'e'] => EwfHeaderValueType::ExaminerName,
                // "l" => label of source media device
                [b'l'] => EwfHeaderValueType::DeviceLabel,
                // "m" => acquisition date and time
                [b'm'] => EwfHeaderValueType::AcquisitionDate,
                // "md" => model of source media device
                [b'm', b'd'] => EwfHeaderValueType::Model,
                // "n" => evidence number
                [b'n'] => EwfHeaderValueType::EvidenceNumber,
                // "ov" => acquisition platform
                [b'o', b'v'] => EwfHeaderValueType::Platform,
                // "p" => password hash
                [b'p'] => EwfHeaderValueType::PasswordHash,
                // "pid" => process identifier of source process
                [b'p', b'i', b'd'] => EwfHeaderValueType::ProcessIdentifier,
                // "r" => compression level
                [b'r'] => EwfHeaderValueType::CompressionLevel,
                // "sn" => serial number of source media device
                [b's', b'n'] => EwfHeaderValueType::SerialNumber,
                // "t" => notes
                [b't'] => EwfHeaderValueType::Notes,
                // "u" => system date and time
                [b'u'] => EwfHeaderValueType::SystemDate,
                _ => EwfHeaderValueType::NotSet,
            };
            if header_value_type != EwfHeaderValueType::NotSet
                && !header_values.contains_key(&header_value_type)
            {
                let header_value: EwfHeaderValue = EwfHeaderValue::from_bytes(values[value_index]);
                header_values.insert(header_value_type, header_value);
            }
        }
        Ok(())
    }

    /// Reads the header from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u64,
        position: SeekFrom,
        header_values: &mut HashMap<EwfHeaderValueType, EwfHeaderValue>,
    ) -> Result<(), ErrorTrace> {
        // Note that 16777216 is an arbitrary chosen limit.
        if data_size < 2 || data_size > 16777216 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported header data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "EwfHeader data of size: {} at offset: {} (0x{:08x})\n",
                data_size, offset, offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.read_data(&data, header_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x78, 0x9c, 0x6d, 0xc8, 0xb1, 0x0d, 0xc4, 0x20, 0x0c, 0x05, 0xd0, 0xfa, 0x5b, 0xf2,
            0x0e, 0x8c, 0x80, 0x51, 0xee, 0x92, 0xec, 0x90, 0x25, 0x10, 0x71, 0xe1, 0x02, 0x13,
            0x05, 0x82, 0x32, 0xfe, 0x2d, 0x70, 0xaf, 0x7c, 0xc2, 0x54, 0xb3, 0x39, 0x53, 0x81,
            0x23, 0x43, 0x31, 0x90, 0x27, 0xda, 0x44, 0xc5, 0x83, 0x8b, 0xa9, 0xe4, 0xae, 0xd0,
            0x69, 0xa7, 0x7a, 0x51, 0x9c, 0xda, 0xcb, 0x6d, 0xd7, 0xb0, 0xe6, 0xd0, 0x37, 0x57,
            0x73, 0xbd, 0xe1, 0x6d, 0x68, 0x47, 0x8a, 0xb2, 0xc4, 0x4d, 0x56, 0x1c, 0xe6, 0xcf,
            0x8b, 0x14, 0xd3, 0x27, 0xec, 0x41, 0xd6, 0x20, 0x7b, 0x58, 0xbe, 0x41, 0xfe, 0x4c,
            0x64, 0x62, 0xfa, 0x01, 0x34, 0x99, 0x20, 0xff,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfHeader::new();
        let mut header_values: HashMap<EwfHeaderValueType, EwfHeaderValue> = HashMap::new();
        test_struct.read_data(&test_data, &mut header_values)?;

        assert_eq!(header_values.len(), 10);
        Ok(())
    }

    // TODO: add test with invalid checksum.

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = EwfHeader::new();
        let mut header_values: HashMap<EwfHeaderValueType, EwfHeaderValue> = HashMap::new();
        test_struct.read_at_position(&data_stream, 106, SeekFrom::Start(0), &mut header_values)?;

        assert_eq!(header_values.len(), 10);
        Ok(())
    }
}
