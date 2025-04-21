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

use std::io;
use std::io::Read;

use core::formatters::format_as_string;
use core::DataStream;
use datetime::DateTime;
use hashes::{DigestHashContext, Md5Context};

pub const BODYFILE_HEADER: &'static str = "# extended bodyfile 3 format";

/// Calculates the MD5 of a data stream.
pub fn calculate_md5(data_stream: &mut Box<dyn DataStream>) -> io::Result<String> {
    let mut data: Vec<u8> = vec![0; 65536];
    let mut md5_context: Md5Context = Md5Context::new();

    loop {
        let read_count: usize = data_stream.read(&mut data)?;
        if read_count == 0 {
            break;
        }
        md5_context.update(&data[..read_count]);
    }
    let hash_value: Vec<u8> = md5_context.finalize();

    Ok(format_as_string(&hash_value))
}

/// Formats a date and time value as a bodyfile timestamp.
pub fn format_as_timestamp(date_time_value: Option<&DateTime>) -> io::Result<String> {
    let string: String = match date_time_value {
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
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported date time"),
                ))
            }
        },
        // Note that a timestamp value of 0 can represent that the timestamp is not present or
        // that timestamp was 0 (not set).
        None => String::from("0"),
    };
    Ok(string)
}

// TODO: add tests.
