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

use std::collections::HashMap;
use std::io::SeekFrom;

use keramics_compression::ZlibContext;
use keramics_core::{ByteOrder, DataStreamReference, ErrorTrace};
use keramics_datetime::PosixTime32;

use super::enums::EwfHeaderValueType;
use super::header_value::EwfHeaderValue;
use super::object_storage::EwfUtf16ObjectStorage;

/// Expert Witness Compression Format (EWF) header2.
pub struct EwfHeader2 {}

impl EwfHeader2 {
    /// Creates a new header2.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the header2 from a compressed buffer.
    pub fn read_compressed_data(
        &mut self,
        compressed_data: &[u8],
        offset: u64,
        header_values: &mut HashMap<EwfHeaderValueType, EwfHeaderValue>,
    ) -> Result<(), ErrorTrace> {
        let compressed_data_size: usize = compressed_data.len();

        // On average the uncompressed header will be more than twice as large
        // as the compressed header. Note that the uncompressed header size
        // should be a multitude of 2 bytes.
        let mut data: Vec<u8> = vec![0; compressed_data_size * 4];

        let mut zlib_context: ZlibContext = ZlibContext::new();

        match zlib_context.decompress(compressed_data, &mut data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decompress data");
                return Err(error);
            }
        }
        keramics_core::debug_trace_data!(
            "EwfHeader2",
            offset,
            &data,
            zlib_context.uncompressed_data_size
        );
        self.read_data(&data, header_values)
    }

    /// Reads the header from a buffer.
    pub fn read_data(
        &mut self,
        data: &[u8],
        header_values: &mut HashMap<EwfHeaderValueType, EwfHeaderValue>,
    ) -> Result<(), ErrorTrace> {
        let mut data_offset: usize = 0;

        let byte_order: ByteOrder = match &data[0..2] {
            [0xfe, 0xff] => {
                data_offset += 2;

                ByteOrder::BigEndian
            }
            [0xff, 0xfe] => {
                data_offset += 2;

                ByteOrder::LittleEndian
            }
            _ => ByteOrder::LittleEndian,
        };
        let mut object_storage: EwfUtf16ObjectStorage =
            EwfUtf16ObjectStorage::new(&data[data_offset..], byte_order);

        let number_of_categories: u8 = match object_storage.next_line().as_deref() {
            // "1"
            Some([0x0031]) => 1,
            // "3"
            Some([0x0033]) => 3,
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - unsupported number of categories"
                ));
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing number of categories"
                ));
            }
        };
        // TODO: if number_of_categories == 1 then format is EnCase 4
        // TODO: if number_of_categories == 3 then format is at least EnCase 5

        match object_storage.next_line().as_deref() {
            // "main"
            Some([0x006d, 0x0061, 0x0069, 0x006e]) => {}
            Some(_) => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - unsupported category"
                ));
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing category"
                ));
            }
        }
        let value_types_line: Vec<u16> = match object_storage.next_line() {
            Some(line) => line,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing value types"
                ));
            }
        };
        let values_line: Vec<u16> = match object_storage.next_line() {
            Some(line) => line,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Invalid header data - missing values"
                ));
            }
        };
        let value_types: Vec<&[u16]> = value_types_line
            .split(|value_16bit| *value_16bit == 0x0009)
            .collect::<Vec<&[u16]>>();

        let values: Vec<&[u16]> = values_line
            .split(|value_16bit| *value_16bit == 0x0009)
            .collect::<Vec<&[u16]>>();

        let number_of_value_types: usize = value_types.len();

        if number_of_value_types > values.len() {
            return Err(keramics_core::error_trace_new!(
                "Invalid header data - number of values does not corresponding to number of value types"
            ));
        }
        for value_index in 0..number_of_value_types {
            let header_value_type: EwfHeaderValueType = match value_types[value_index] {
                // "a" => description
                [0x0061] => EwfHeaderValueType::Description,
                // "av" => acquisition software version
                [0x0061, 0x0076] => EwfHeaderValueType::Version,
                // "c" => case number
                [0x0063] => EwfHeaderValueType::CaseNumber,
                // "e" => examiner name
                [0x0065] => EwfHeaderValueType::ExaminerName,
                // "l" => label of source media device
                [0x006c] => EwfHeaderValueType::DeviceLabel,
                // "m" => acquisition date and time
                [0x006d] => EwfHeaderValueType::AcquisitionDate,
                // "md" => model of source media device
                [0x006d, 0x0064] => EwfHeaderValueType::Model,
                // "n" => evidence number
                [0x006e] => EwfHeaderValueType::EvidenceNumber,
                // "ov" => acquisition platform
                [0x006f, 0x0076] => EwfHeaderValueType::Platform,
                // "p" => password hash
                [0x0070] => EwfHeaderValueType::PasswordHash,
                // "pid" => process identifier of source process
                [0x0070, 0x0069, 0x0064] => EwfHeaderValueType::ProcessIdentifier,
                // "r" => compression level
                [0x0072] => EwfHeaderValueType::CompressionLevel,
                // "sn" => serial number of source media device
                [0x0073, 0x006e] => EwfHeaderValueType::SerialNumber,
                // "t" => notes
                [0x0077] => EwfHeaderValueType::Notes,
                // "u" => system date and time
                [0x0075] => EwfHeaderValueType::SystemDate,
                _ => EwfHeaderValueType::NotSet,
            };
            if header_value_type != EwfHeaderValueType::NotSet
                && !header_values.contains_key(&header_value_type)
            {
                let header_value: EwfHeaderValue = match &header_value_type {
                    EwfHeaderValueType::AcquisitionDate | EwfHeaderValueType::SystemDate => {
                        match EwfUtf16ObjectStorage::parse_date_value(values[value_index]) {
                            Some(timestamp) => {
                                EwfHeaderValue::PosixTime(PosixTime32::new(timestamp))
                            }
                            None => continue, // TODO: consider tracking parse error
                        }
                    }
                    _ => EwfHeaderValue::from_utf16(values[value_index]),
                };
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
                "Unsupported header2 data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        self.read_compressed_data(&data, offset, header_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_compressed_data() -> Vec<u8> {
        return vec![
            0x78, 0x9c, 0x75, 0x8f, 0x4b, 0x6e, 0xc3, 0x30, 0x0c, 0x44, 0xdf, 0x76, 0x74, 0x97,
            0x02, 0x56, 0x93, 0x7e, 0x0e, 0xd1, 0x4b, 0x28, 0xb2, 0xd0, 0x0a, 0xa8, 0xad, 0xd4,
            0x9f, 0x20, 0xa7, 0x6f, 0x03, 0x4f, 0x8d, 0x6e, 0x8a, 0x80, 0x00, 0x17, 0x43, 0x72,
            0x1e, 0xe7, 0xe7, 0xfb, 0x40, 0x60, 0x20, 0x51, 0x19, 0x09, 0x24, 0x44, 0x46, 0x8c,
            0x88, 0x82, 0x58, 0x10, 0x03, 0x3d, 0x62, 0xb6, 0x96, 0xb8, 0x20, 0x9a, 0xfb, 0x80,
            0x58, 0x11, 0x67, 0x44, 0x4f, 0x26, 0xd0, 0x53, 0x98, 0xc9, 0x4c, 0x54, 0xce, 0x2c,
            0x54, 0x9a, 0xaf, 0x32, 0x89, 0xd9, 0x7e, 0x85, 0x0b, 0xd5, 0x7b, 0x23, 0x79, 0x57,
            0xae, 0x24, 0x06, 0xf3, 0x0b, 0x93, 0xd9, 0x8d, 0xc5, 0x4e, 0x72, 0x3d, 0xd2, 0x11,
            0x39, 0xd2, 0xf1, 0x4a, 0xe4, 0x05, 0xf1, 0xe6, 0xed, 0x95, 0x2b, 0xb2, 0xf2, 0xe4,
            0xc9, 0x81, 0x48, 0xe4, 0x99, 0x78, 0x57, 0xed, 0x10, 0x81, 0xc0, 0xcc, 0x64, 0x7a,
            0xb0, 0x12, 0x09, 0xce, 0xb0, 0x7d, 0x5a, 0x9d, 0xb5, 0x38, 0xdf, 0xc2, 0x09, 0xf1,
            0x49, 0x73, 0xc6, 0xe6, 0xf4, 0x1f, 0x88, 0x77, 0xa7, 0x4e, 0x7c, 0xed, 0xf7, 0x1d,
            0x61, 0xff, 0x74, 0xab, 0x07, 0x93, 0x7e, 0xbb, 0xfe, 0x78, 0x2b, 0xa7, 0xbb, 0xb4,
            0xd1, 0x7e, 0xd9, 0x84, 0xcd, 0xfb, 0xbf, 0x6b, 0xdc, 0x7d, 0x6e, 0x29, 0x24, 0x36,
            0x19,
        ];
    }

    fn get_test_data() -> Vec<u8> {
        return vec![
            0xff, 0xfe, 0x33, 0x00, 0x0a, 0x00, 0x6d, 0x00, 0x61, 0x00, 0x69, 0x00, 0x6e, 0x00,
            0x0a, 0x00, 0x61, 0x00, 0x09, 0x00, 0x63, 0x00, 0x09, 0x00, 0x6e, 0x00, 0x09, 0x00,
            0x65, 0x00, 0x09, 0x00, 0x74, 0x00, 0x09, 0x00, 0x6d, 0x00, 0x64, 0x00, 0x09, 0x00,
            0x73, 0x00, 0x6e, 0x00, 0x09, 0x00, 0x61, 0x00, 0x76, 0x00, 0x09, 0x00, 0x6f, 0x00,
            0x76, 0x00, 0x09, 0x00, 0x6d, 0x00, 0x09, 0x00, 0x75, 0x00, 0x09, 0x00, 0x70, 0x00,
            0x09, 0x00, 0x64, 0x00, 0x63, 0x00, 0x0a, 0x00, 0x64, 0x00, 0x65, 0x00, 0x73, 0x00,
            0x63, 0x00, 0x72, 0x00, 0x69, 0x00, 0x70, 0x00, 0x74, 0x00, 0x69, 0x00, 0x6f, 0x00,
            0x6e, 0x00, 0x09, 0x00, 0x63, 0x00, 0x61, 0x00, 0x73, 0x00, 0x65, 0x00, 0x09, 0x00,
            0x65, 0x00, 0x76, 0x00, 0x69, 0x00, 0x64, 0x00, 0x65, 0x00, 0x6e, 0x00, 0x63, 0x00,
            0x65, 0x00, 0x09, 0x00, 0x65, 0x00, 0x78, 0x00, 0x61, 0x00, 0x6d, 0x00, 0x69, 0x00,
            0x6e, 0x00, 0x65, 0x00, 0x72, 0x00, 0x09, 0x00, 0x6e, 0x00, 0x6f, 0x00, 0x74, 0x00,
            0x65, 0x00, 0x73, 0x00, 0x09, 0x00, 0x09, 0x00, 0x09, 0x00, 0x32, 0x00, 0x30, 0x00,
            0x31, 0x00, 0x34, 0x00, 0x30, 0x00, 0x38, 0x00, 0x31, 0x00, 0x37, 0x00, 0x09, 0x00,
            0x4c, 0x00, 0x69, 0x00, 0x6e, 0x00, 0x75, 0x00, 0x78, 0x00, 0x09, 0x00, 0x31, 0x00,
            0x37, 0x00, 0x35, 0x00, 0x38, 0x00, 0x31, 0x00, 0x33, 0x00, 0x31, 0x00, 0x31, 0x00,
            0x36, 0x00, 0x31, 0x00, 0x09, 0x00, 0x31, 0x00, 0x37, 0x00, 0x35, 0x00, 0x38, 0x00,
            0x31, 0x00, 0x33, 0x00, 0x31, 0x00, 0x31, 0x00, 0x36, 0x00, 0x31, 0x00, 0x09, 0x00,
            0x30, 0x00, 0x09, 0x00, 0x0a, 0x00, 0x0a, 0x00, 0x73, 0x00, 0x72, 0x00, 0x63, 0x00,
            0x65, 0x00, 0x0a, 0x00, 0x30, 0x00, 0x09, 0x00, 0x31, 0x00, 0x0a, 0x00, 0x70, 0x00,
            0x09, 0x00, 0x6e, 0x00, 0x09, 0x00, 0x69, 0x00, 0x64, 0x00, 0x09, 0x00, 0x65, 0x00,
            0x76, 0x00, 0x09, 0x00, 0x74, 0x00, 0x62, 0x00, 0x09, 0x00, 0x6c, 0x00, 0x6f, 0x00,
            0x09, 0x00, 0x70, 0x00, 0x6f, 0x00, 0x09, 0x00, 0x61, 0x00, 0x68, 0x00, 0x09, 0x00,
            0x67, 0x00, 0x75, 0x00, 0x09, 0x00, 0x61, 0x00, 0x71, 0x00, 0x0a, 0x00, 0x30, 0x00,
            0x09, 0x00, 0x30, 0x00, 0x0a, 0x00, 0x09, 0x00, 0x09, 0x00, 0x09, 0x00, 0x09, 0x00,
            0x09, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x09, 0x00, 0x2d, 0x00, 0x31, 0x00, 0x09, 0x00,
            0x09, 0x00, 0x09, 0x00, 0x0a, 0x00, 0x0a, 0x00, 0x73, 0x00, 0x75, 0x00, 0x62, 0x00,
            0x0a, 0x00, 0x30, 0x00, 0x09, 0x00, 0x31, 0x00, 0x0a, 0x00, 0x70, 0x00, 0x09, 0x00,
            0x6e, 0x00, 0x09, 0x00, 0x69, 0x00, 0x64, 0x00, 0x09, 0x00, 0x6e, 0x00, 0x75, 0x00,
            0x09, 0x00, 0x63, 0x00, 0x6f, 0x00, 0x09, 0x00, 0x67, 0x00, 0x75, 0x00, 0x0a, 0x00,
            0x30, 0x00, 0x09, 0x00, 0x30, 0x00, 0x0a, 0x00, 0x09, 0x00, 0x09, 0x00, 0x09, 0x00,
            0x09, 0x00, 0x31, 0x00, 0x09, 0x00, 0x0a, 0x00, 0x0a, 0x00,
        ];
    }

    #[test]
    fn test_read_compressed_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_compressed_data();

        let mut test_struct = EwfHeader2::new();
        let mut header_values: HashMap<EwfHeaderValueType, EwfHeaderValue> = HashMap::new();
        test_struct.read_compressed_data(&test_data, 0, &mut header_values)?;

        assert_eq!(header_values.len(), 11);
        Ok(())
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = EwfHeader2::new();
        let mut header_values: HashMap<EwfHeaderValueType, EwfHeaderValue> = HashMap::new();
        test_struct.read_data(&test_data, &mut header_values)?;

        assert_eq!(header_values.len(), 11);
        Ok(())
    }

    // TODO: add test with invalid checksum.

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_compressed_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = EwfHeader2::new();
        let mut header_values: HashMap<EwfHeaderValueType, EwfHeaderValue> = HashMap::new();
        test_struct.read_at_position(&data_stream, 197, SeekFrom::Start(0), &mut header_values)?;

        assert_eq!(header_values.len(), 11);
        Ok(())
    }
}
