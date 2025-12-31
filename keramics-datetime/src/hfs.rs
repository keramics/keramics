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

use keramics_types::{bytes_to_u32_be, bytes_to_u32_le};

use super::epoch::Epoch;
use super::util::{get_date_values, get_time_values};

/// 32-bit MFS or HFS timestamp.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HfsTime {
    /// Number of seconds since January 1, 1904.
    pub timestamp: u32,
}

impl HfsTime {
    const EPOCH: Epoch = Epoch {
        year: 1904,
        month: 1,
        day_of_month: 1,
    };

    /// Creates a new timestamp.
    pub fn new(timestamp: u32) -> Self {
        Self { timestamp }
    }

    /// Reads a big-endian timestamp from a byte sequence.
    pub fn from_be_bytes(data: &[u8]) -> Self {
        let timestamp: u32 = bytes_to_u32_be!(data, 0);
        Self { timestamp }
    }

    /// Reads a little-endian timestamp from a byte sequence.
    pub fn from_le_bytes(data: &[u8]) -> Self {
        let timestamp: u32 = bytes_to_u32_le!(data, 0);
        Self { timestamp }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (days, hours, minutes, seconds): (i64, u8, u8, u8) =
            get_time_values(self.timestamp as i64);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &Self::EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day_of_month, hours, minutes, seconds
        )
    }
}

impl fmt::Display for HfsTime {
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
        let test_data: [u8; 4] = [0xce, 0x20, 0x2e, 0x68];

        let test_struct: HfsTime = HfsTime::from_be_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 3458215528);
    }

    #[test]
    fn test_from_le_bytes() {
        let test_data: [u8; 4] = [0x68, 0x2e, 0x20, 0xce];

        let test_struct: HfsTime = HfsTime::from_le_bytes(&test_data);
        assert_eq!(test_struct.timestamp, 3458215528);
    }

    #[test]
    fn test_to_iso8601_string() {
        let test_struct: HfsTime = HfsTime::new(3458215528);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2013-08-01T15:25:28");
    }
}
