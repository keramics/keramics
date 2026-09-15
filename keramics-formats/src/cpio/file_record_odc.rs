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

use std::str::from_utf8;

use keramics_core::ErrorTrace;
use keramics_datetime::{DateTime, PosixTime32};

use super::file_record::CpioFileRecord;

/// Copy in and out (CPIO) portable ASCII file record
pub struct CpioPortableAsciiFileRecord {}

impl CpioPortableAsciiFileRecord {
    /// Reads the record from a buffer.
    pub fn read_data(file_record: &mut CpioFileRecord, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 76 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..6] != b"070707" {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let octal_string: &str = match from_utf8(&data[0..76]) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to data to octal string",
                    error
                ));
            }
        };
        file_record.inode_number = match u16::from_str_radix(&octal_string[12..18], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert inode number octal string to integer",
                    error
                ));
            }
        };
        file_record.file_mode = match u16::from_str_radix(&octal_string[18..24], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert file mode octal string to integer",
                    error
                ));
            }
        };
        file_record.owner_identifier = match u16::from_str_radix(&octal_string[24..30], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert owner identifier octal string to integer",
                    error
                ));
            }
        };
        file_record.group_identifier = match u16::from_str_radix(&octal_string[30..36], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert group identifier octal string to integer",
                    error
                ));
            }
        };
        file_record.number_of_links = match u16::from_str_radix(&octal_string[36..42], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert number of links octal string to integer",
                    error
                ));
            }
        };
        file_record.device_identifier = match u16::from_str_radix(&octal_string[42..48], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert device identifier octal string to integer",
                    error
                ));
            }
        };
        let timestamp: i32 = match i32::from_str_radix(&octal_string[48..59], 8) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert modification time octal string to integer",
                    error
                ));
            }
        };
        file_record.modification_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::PosixTime32(PosixTime32::new(timestamp as i32))
        };
        file_record.path_size = match u16::from_str_radix(&octal_string[59..65], 8) {
            Ok(value) => value as u32,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert path size octal string to integer",
                    error
                ));
            }
        };
        file_record.data_size = match u32::from_str_radix(&octal_string[65..76], 8) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert data size octal string to integer",
                    error
                ));
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x30, 0x37, 0x30, 0x37, 0x30, 0x37, 0x30, 0x30, 0x33, 0x34, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x32, 0x32, 0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x30, 0x30, 0x31, 0x37,
            0x35, 0x30, 0x30, 0x30, 0x31, 0x37, 0x35, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x34, 0x37, 0x33, 0x36, 0x31, 0x36, 0x35,
            0x30, 0x31, 0x31, 0x30, 0x30, 0x30, 0x30, 0x33, 0x35, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2f, 0x6d, 0x6e, 0x74, 0x2f, 0x6b, 0x65, 0x72,
            0x61, 0x6d, 0x69, 0x63, 0x73, 0x2f, 0x6e, 0x66, 0x63, 0x5f, 0x74, 0xc3, 0xa9, 0x73,
            0x74, 0x66, 0x69, 0x6c, 0xc3, 0xa8, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        CpioPortableAsciiFileRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.inode_number, 18);
        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(test_struct.device_identifier, 0);
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            })
        );
        assert_eq!(test_struct.path_size, 29);
        assert_eq!(test_struct.data_size, 0);
        assert_eq!(test_struct.checksum, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        let result: Result<(), ErrorTrace> =
            CpioPortableAsciiFileRecord::read_data(&mut test_struct, &test_data[0..75]);
        assert!(result.is_err());
    }
}
