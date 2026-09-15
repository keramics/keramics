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

/// Copy in and out (CPIO) new ASCII file record
pub struct CpioNewAsciiFileRecord {}

impl CpioNewAsciiFileRecord {
    /// Reads the record from a buffer.
    pub fn read_data(file_record: &mut CpioFileRecord, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 110 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..5] != b"07070" && data[6] != b'1' && data[6] != b'2' {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        let hexadecimal_string: &str = match from_utf8(&data[0..110]) {
            Ok(string) => string,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to data to hexadecimal string",
                    error
                ));
            }
        };
        file_record.inode_number = match u32::from_str_radix(&hexadecimal_string[6..14], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert inode number hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.file_mode = match u32::from_str_radix(&hexadecimal_string[14..22], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert file mode hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.owner_identifier = match u32::from_str_radix(&hexadecimal_string[22..30], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert owner identifier hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.group_identifier = match u32::from_str_radix(&hexadecimal_string[30..38], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert group identifier hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.number_of_links = match u32::from_str_radix(&hexadecimal_string[38..46], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert number of links hexadecimal string to integer",
                    error
                ));
            }
        };
        let timestamp: i32 = match i32::from_str_radix(&hexadecimal_string[46..54], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert modification time hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.modification_time = if timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::PosixTime32(PosixTime32::new(timestamp as i32))
        };
        file_record.data_size = match u32::from_str_radix(&hexadecimal_string[54..62], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert data size hexadecimal string to integer",
                    error
                ));
            }
        };
        // TODO: reconstruct device identifier?

        file_record.path_size = match u32::from_str_radix(&hexadecimal_string[94..102], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert path size hexadecimal string to integer",
                    error
                ));
            }
        };
        file_record.checksum = match u32::from_str_radix(&hexadecimal_string[102..110], 16) {
            Ok(value) => value,
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to convert checksum hexadecimal string to integer",
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
            0x30, 0x37, 0x30, 0x37, 0x30, 0x32, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x32,
            0x30, 0x30, 0x30, 0x30, 0x38, 0x31, 0x41, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x33,
            0x45, 0x38, 0x30, 0x30, 0x30, 0x30, 0x30, 0x33, 0x45, 0x38, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x31, 0x36, 0x37, 0x37, 0x38, 0x45, 0x41, 0x30, 0x39, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x37,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x31, 0x44, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2f, 0x6d,
            0x6e, 0x74, 0x2f, 0x6b, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x73, 0x2f, 0x6e, 0x66,
            0x63, 0x5f, 0x74, 0xc3, 0xa9, 0x73, 0x74, 0x66, 0x69, 0x6c, 0xc3, 0xa8, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        CpioNewAsciiFileRecord::read_data(&mut test_struct, &test_data)?;

        assert_eq!(test_struct.inode_number, 18);
        assert_eq!(test_struct.file_mode, 0o100644);
        assert_eq!(test_struct.owner_identifier, 1000);
        assert_eq!(test_struct.group_identifier, 1000);
        assert_eq!(test_struct.number_of_links, 1);
        assert_eq!(
            test_struct.modification_time,
            DateTime::PosixTime32(PosixTime32 {
                timestamp: 1735977481
            })
        );
        assert_eq!(test_struct.data_size, 0);
        // TODO: assert_eq!(test_struct.device_identifier, 0);
        assert_eq!(test_struct.path_size, 29);
        assert_eq!(test_struct.checksum, 0x00000000);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = CpioFileRecord::new();
        let result: Result<(), ErrorTrace> =
            CpioNewAsciiFileRecord::read_data(&mut test_struct, &test_data[0..109]);
        assert!(result.is_err());
    }
}
