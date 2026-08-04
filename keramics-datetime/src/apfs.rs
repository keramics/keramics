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

use keramics_types::{bytes_to_i64_be, bytes_to_i64_le};

use super::posix::POSIX_EPOCH;
use super::util::{get_date_values, get_time_values};

/// Apple File System (APFS) timestamp.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApfsTime {
    /// Number of nanoseconds since January 1, 1970 (POSIX epoch).
    /// Negative values represent date and times predating the epoch.
    pub timestamp: i64,
}

impl ApfsTime {
    /// Creates a new timestamp.
    pub fn new(timestamp: i64) -> Self {
        Self { timestamp }
    }

    /// Reads a big-endian timestamp from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let timestamp: i64 = bytes_to_i64_be!(data, 0);
        Self { timestamp }
    }

    /// Reads a little-endian timestamp from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let timestamp: i64 = bytes_to_i64_le!(data, 0);
        Self { timestamp }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let timestamp: i64 = self.timestamp / 1000000000;
        let fraction: i64 = self.timestamp % 1000000000;
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) = get_time_values(timestamp);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &POSIX_EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}",
            year, month, day_of_month, hours, minutes, seconds, fraction
        )
    }
}

impl fmt::Display for ApfsTime {
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
        let test_data: [u8; 8] = [0x11, 0xc9, 0x50, 0x0e, 0x75, 0x14, 0x6e, 0xb1];

        let test_struct: ApfsTime = ApfsTime::from_be_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 1281643591987654321);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 8] = [0xb1, 0x6e, 0x14, 0x75, 0x0e, 0x50, 0xc9, 0x11];

        let test_struct: ApfsTime = ApfsTime::from_le_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 1281643591987654321);
    }

    #[test]
    fn test_to_iso8601_string() {
        let test_struct: ApfsTime = ApfsTime::new(1281643591987654321);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31.987654321");
    }

    #[test]
    fn test_to_string() {
        let test_struct: ApfsTime = ApfsTime::new(1281643591987654321);

        let string: String = test_struct.to_string();
        assert_eq!(
            string.as_str(),
            "2010-08-12T20:06:31.987654321 (1281643591987654321)"
        );
    }
}
