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

use std::fmt;

use keramics_types::{bytes_to_u64_be, bytes_to_u64_le};

use super::epoch::Epoch;
use super::util::{get_date_values, get_time_values};

/// X File System (XFS) bigtime timestamp.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct XfsBigtime {
    /// Number of nanoseconds since December 13, 1901 20:45:52.
    pub timestamp: u64,
}

impl XfsBigtime {
    const EPOCH: Epoch = Epoch {
        year: 1901,
        month: 12,
        day_of_month: 13,
    };

    /// Creates a new timestamp.
    pub fn new(timestamp: u64) -> Self {
        Self { timestamp }
    }

    /// Reads a big-endian timestamp from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let timestamp: u64 = bytes_to_u64_be!(data, 0);
        Self { timestamp }
    }

    /// Reads a little-endian timestamp from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let timestamp: u64 = bytes_to_u64_le!(data, 0);
        Self { timestamp }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        // 74752 is the number of seconds between 00:00:00 and 20:45:52
        let timestamp: i64 = ((self.timestamp / 1000000000) as i64) + 74752;
        let fraction: u64 = self.timestamp % 1000000000;
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) = get_time_values(timestamp);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &Self::EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}",
            year, month, day_of_month, hours, minutes, seconds, fraction
        )
    }
}

impl fmt::Display for XfsBigtime {
    /// Formats the timestamp for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} ({})",
            self.to_iso8601_string(),
            self.timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_be_bytes() {
        let test_data: [u8; 8] = [0x2f, 0x96, 0xb5, 0x0e, 0x75, 0x14, 0x6e, 0xb1];

        let test_struct: XfsBigtime = XfsBigtime::from_be_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 3429127239987654321);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 8] = [0xb1, 0x6e, 0x14, 0x75, 0x0e, 0xb5, 0x96, 0x2f];

        let test_struct: XfsBigtime = XfsBigtime::from_le_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 3429127239987654321);
    }

    #[test]
    fn test_to_iso8601_string() {
        let test_struct: XfsBigtime = XfsBigtime::new(3429127239987654321);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31.987654321");

        let test_struct: XfsBigtime = XfsBigtime::new(0);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "1901-12-13T20:45:52.000000000");

        let test_struct: XfsBigtime = XfsBigtime::new(2147483648000000000);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "1970-01-01T00:00:00.000000000");

        let test_struct: XfsBigtime = XfsBigtime::new(4294967295000000000);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2038-01-19T03:14:07.000000000");
    }

    #[test]
    fn test_to_string() {
        let test_struct: XfsBigtime = XfsBigtime::new(3429127239987654321);

        let string: String = test_struct.to_string();
        assert_eq!(
            string.as_str(),
            "2010-08-12T20:06:31.987654321 (3429127239987654321)"
        );
    }
}
