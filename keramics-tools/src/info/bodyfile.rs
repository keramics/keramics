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

use keramics_core::formatters::format_as_string;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_datetime::DateTime;
use keramics_hashes::{DigestHashContext, Md5Context};

pub const BODYFILE_HEADER: &'static str = "# extended bodyfile 3 format";

/// Calculates the MD5 of a data stream.
pub fn calculate_md5(data_stream: &DataStreamReference) -> Result<String, ErrorTrace> {
    let mut data: Vec<u8> = vec![0; 65536];
    let mut md5_context: Md5Context = Md5Context::new();

    match data_stream.write() {
        Ok(mut data_stream) => loop {
            let read_count = match data_stream.read(&mut data) {
                Ok(read_count) => read_count,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read data stream");
                    return Err(error);
                }
            };
            if read_count == 0 {
                break;
            }
            md5_context.update(&data[..read_count]);
        },
        Err(error) => {
            return Err(keramics_core::error_trace_new_with_error!(
                "Unable to obtain write lock on data stream",
                error
            ));
        }
    };
    let hash_value: Vec<u8> = md5_context.finalize();

    Ok(format_as_string(&hash_value))
}

/// Formats a date and time value as a bodyfile timestamp.
pub fn format_as_timestamp(date_time: Option<&DateTime>) -> Result<String, ErrorTrace> {
    // Note that a timestamp value of 0 can represent that the timestamp is not present or
    // that timestamp was 0 (not set).

    let string: String = match date_time {
        Some(date_time) => match date_time {
            DateTime::Filetime(filetime) => {
                let posix_time: u64 = filetime.timestamp - 116444736000000000;
                let number_of_seconds: u64 = posix_time / 10000000;
                let fraction: u64 = posix_time % 10000000;

                format!("{}.{:07}", number_of_seconds, fraction)
            }
            DateTime::NotSet => String::from("0"),
            DateTime::PosixTime32(posix_time32) => format!("{}", posix_time32.timestamp),
            DateTime::PosixTime64Ns(posix_time64ns) => format!(
                "{}.{:09}",
                posix_time64ns.timestamp, posix_time64ns.fraction
            ),
            _ => return Err(keramics_core::error_trace_new!("Unsupported date time")),
        },
        None => String::from("0"),
    };
    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::{DataStreamReference, open_fake_data_stream};
    use keramics_datetime::{Filetime, PosixTime32, PosixTime64Ns};

    #[test]
    fn test_calculate_md5() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = vec![
            0x41, 0x20, 0x63, 0x65, 0x72, 0x61, 0x6d, 0x69, 0x63, 0x20, 0x69, 0x73, 0x20, 0x61,
            0x6e, 0x79, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x76, 0x61, 0x72, 0x69,
            0x6f, 0x75, 0x73, 0x20, 0x68, 0x61, 0x72, 0x64, 0x2c, 0x20, 0x62, 0x72, 0x69, 0x74,
            0x74, 0x6c, 0x65, 0x2c, 0x20, 0x68, 0x65, 0x61, 0x74, 0x2d, 0x72, 0x65, 0x73, 0x69,
            0x73, 0x74, 0x61, 0x6e, 0x74, 0x2c, 0x20, 0x61, 0x6e, 0x64, 0x20, 0x63, 0x6f, 0x72,
            0x72, 0x6f, 0x73, 0x69, 0x6f, 0x6e, 0x2d, 0x72, 0x65, 0x73, 0x69, 0x73, 0x74, 0x61,
            0x6e, 0x74, 0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x73, 0x20, 0x6d,
            0x61, 0x64, 0x65, 0x20, 0x62, 0x79, 0x20, 0x73, 0x68, 0x61, 0x70, 0x69, 0x6e, 0x67,
            0x20, 0x61, 0x6e, 0x64, 0x20, 0x74, 0x68, 0x65, 0x6e, 0x20, 0x66, 0x69, 0x72, 0x69,
            0x6e, 0x67, 0x20, 0x61, 0x6e, 0x20, 0x69, 0x6e, 0x6f, 0x72, 0x67, 0x61, 0x6e, 0x69,
            0x63, 0x2c, 0x20, 0x6e, 0x6f, 0x6e, 0x6d, 0x65, 0x74, 0x61, 0x6c, 0x6c, 0x69, 0x63,
            0x20, 0x6d, 0x61, 0x74, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x2c, 0x20, 0x73, 0x75, 0x63,
            0x68, 0x20, 0x61, 0x73, 0x20, 0x63, 0x6c, 0x61, 0x79, 0x2c, 0x20, 0x61, 0x74, 0x20,
            0x61, 0x20, 0x68, 0x69, 0x67, 0x68, 0x20, 0x74, 0x65, 0x6d, 0x70, 0x65, 0x72, 0x61,
            0x74, 0x75, 0x72, 0x65, 0x2e, 0x0a,
        ];
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let md5: String = calculate_md5(&data_stream)?;
        assert_eq!(md5, "f19106bcf25fa9cabc1b5ac91c726001");

        Ok(())
    }

    #[test]
    fn test_format_as_timestamp() -> Result<(), ErrorTrace> {
        let date_time: DateTime = DateTime::Filetime(Filetime::new(0x01cb3a623d0a17ce));
        let timestamp: String = format_as_timestamp(Some(&date_time))?;
        assert_eq!(timestamp, "1281647191.5468750");

        let date_time: DateTime = DateTime::PosixTime32(PosixTime32::new(1281643591));
        let timestamp: String = format_as_timestamp(Some(&date_time))?;
        assert_eq!(timestamp, "1281643591");

        let date_time: DateTime =
            DateTime::PosixTime64Ns(PosixTime64Ns::new(1281643591, 987654321));
        let timestamp: String = format_as_timestamp(Some(&date_time))?;
        assert_eq!(timestamp, "1281643591.987654321");

        let date_time: DateTime = DateTime::NotSet;
        let timestamp: String = format_as_timestamp(Some(&date_time))?;
        assert_eq!(timestamp, "0");

        let timestamp: String = format_as_timestamp(None)?;
        assert_eq!(timestamp, "0");

        Ok(())
    }
}
