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

use super::epoch::Epoch;
use super::util::{get_date_values, get_time_values};

const POSIX_EPOCH: Epoch = Epoch {
    year: 1970,
    month: 1,
    day_of_month: 1,
};

/// 32-bit POSIX timestamp (comparable with 32-bit time_t).
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

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (days, hours, minutes, seconds): (i32, u8, u8, u8) = get_time_values(self.timestamp);
        let (year, month, day_of_month): (i16, u8, u8) = get_date_values(days, &POSIX_EPOCH);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day_of_month, hours, minutes, seconds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_iso8601_string() {
        let test_struct: PosixTime32 = PosixTime32::new(1281643591);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T20:06:31");

        let test_struct: PosixTime32 = PosixTime32::new(-1281643591);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "1929-05-22T03:53:29");
    }
}
