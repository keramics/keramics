/* Copyright 2024 Joachim Metz <joachim.metz@gmail.com>
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

use crate::{bytes_to_i32_be, bytes_to_i32_le};

use super::epoch::Epoch;
use super::util::{get_date_values, get_time_values};

const POSIX_EPOCH: Epoch = Epoch {
    year: 1970,
    month: 1,
    day_of_month: 1,
};

/// 32-bit POSIX timestamp (time_t).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PosixTime32 {
    /// Number of seconds since January 1, 1970 (UTC) (POSIX epoch).
    /// Negative values represent date and times predating the epoch.
    pub timestamp: i32,
}

impl PosixTime32 {
    /// Creates a new timestamp.
    pub fn new(timestamp: i32) -> Self {
        Self {
            timestamp: timestamp,
        }
    }

    /// Reads a big-endian timestamp from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let timestamp: i32 = bytes_to_i32_be!(data, 0);
        Self {
            timestamp: timestamp,
        }
    }

    /// Reads a little-endian timestamp from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let timestamp: i32 = bytes_to_i32_le!(data, 0);
        Self {
            timestamp: timestamp,
        }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) =
            get_time_values(self.timestamp as i64);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &POSIX_EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day_of_month, hours, minutes, seconds
        )
    }
}

impl fmt::Display for PosixTime32 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({})",
            self.to_iso8601_string(),
            self.timestamp
        )
    }
}

/// 64-bit POSIX timestamp in nanoseconds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PosixTime64Ns {
    /// Number of seconds since January 1, 1970 (UTC) (POSIX epoch).
    /// Negative values represent date and times predating the epoch.
    pub timestamp: i64,

    /// Fraction of second in nanoseconds.
    pub fraction: u32,
}

impl PosixTime64Ns {
    /// Creates a new timestamp.
    pub fn new(timestamp: i64, fraction: u32) -> Self {
        Self {
            timestamp: timestamp,
            fraction: fraction,
        }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) = get_time_values(self.timestamp);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &POSIX_EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}",
            year, month, day_of_month, hours, minutes, seconds, self.fraction
        )
    }
}

impl fmt::Display for PosixTime64Ns {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({}.{})",
            self.to_iso8601_string(),
            self.timestamp,
            self.fraction
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posix_time32_from_be_bytes() {
        let test_data: [u8; 4] = [0x67, 0x54, 0x0d, 0xd9];

        let posix_time: PosixTime32 = PosixTime32::from_be_bytes(&test_data);
        assert_eq!(posix_time.timestamp, 1733561817);
    }

    #[test]
    fn test_posix_time32_from_le_bytes() {
        let test_data: [u8; 4] = [0xd9, 0x0d, 0x54, 0x67];

        let posix_time: PosixTime32 = PosixTime32::from_le_bytes(&test_data);
        assert_eq!(posix_time.timestamp, 1733561817);
    }

    #[test]
    fn test_posix_time32_to_iso8601_string() {
        let test_struct: PosixTime32 = PosixTime32::new(1281643591);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31");

        let test_struct: PosixTime32 = PosixTime32::new(-1281643591);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "1929-05-22T03:53:29");
    }

    #[test]
    fn test_posix_time64ns_to_iso8601_string() {
        let test_struct: PosixTime64Ns = PosixTime64Ns::new(1281643591, 987654321);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31.987654321");

        let test_struct: PosixTime64Ns = PosixTime64Ns::new(-1281643592, 12345679);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "1929-05-22T03:53:28.012345679");
    }
}
