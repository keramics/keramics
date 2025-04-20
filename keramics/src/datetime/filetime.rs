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

use std::fmt;

use crate::bytes_to_u32_le;

use super::epoch::Epoch;
use super::util::{get_date_values, get_time_values};

const FILETIME_EPOCH: Epoch = Epoch {
    year: 1601,
    month: 1,
    day_of_month: 1,
};

/// Windows FILETIME timestamp.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filetime {
    /// Number of 100th nanoseconds since January 1, 1601 (UTC).
    pub timestamp: u64,
}

impl Filetime {
    /// Creates a new timestamp.
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp: timestamp,
        }
    }

    /// Reads a timestamp from a byte sequence.
    ///
    /// A FILETIME timestamp is stored in a structure that consists of
    /// 2 x 32-bit integers, both presumed to be unsigned.
    pub fn from_bytes(data: &[u8]) -> Self {
        let lower_32bit: u32 = bytes_to_u32_le!(data, 0);
        let upper_32bit: u32 = bytes_to_u32_le!(data, 4);
        Self {
            timestamp: (upper_32bit as u64) << 32 | (lower_32bit as u64),
        }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let fraction: u64 = self.timestamp % 10000000;
        let number_of_seconds: u64 = self.timestamp / 10000000;
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) =
            get_time_values(number_of_seconds as i64);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &FILETIME_EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:07}",
            year, month, day_of_month, hours, minutes, seconds, fraction
        )
    }
}

impl fmt::Display for Filetime {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} (0x{:08x}:0x{:08x})",
            self.to_iso8601_string(),
            self.timestamp >> 32,
            self.timestamp & 0xffffffff,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filetime_from_bytes() {
        let test_data: [u8; 8] = [0xce, 0x17, 0x0a, 0x3d, 0x62, 0x3a, 0xcb, 0x01];

        let test_struct: Filetime = Filetime::from_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 0x01cb3a623d0a17ce);
    }

    #[test]
    fn test_filetime_to_iso8601_string() {
        let test_struct: Filetime = Filetime::new(0x01cb3a623d0a17ce);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:31.5468750");
    }
}
