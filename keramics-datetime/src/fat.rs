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

use keramics_types::bytes_to_u16_le;

/// FAT date.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatDate {
    /// Date.
    pub date: u16,
}

impl FatDate {
    /// Creates a new timestamp.
    pub fn new(date: u16) -> Self {
        Self { date: date }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let date: u16 = bytes_to_u16_le!(data, 0);
        Self { date: date }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let year: u16 = 1980 + ((self.date >> 9) & 0x7f);
        let month: u16 = (self.date >> 5) & 0x0f;
        let day_of_month: u16 = self.date & 0x1f;

        format!("{:04}-{:02}-{:02}", year, month, day_of_month)
    }
}

impl fmt::Display for FatDate {
    /// Formats the timestamp for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} (0x{:04x})",
            self.to_iso8601_string(),
            self.date,
        )
    }
}

/// FAT time and date.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatTimeDate {
    /// Date.
    pub date: u16,

    /// Time.
    pub time: u16,
}

impl FatTimeDate {
    /// Creates a new timestamp.
    pub fn new(date: u16, time: u16) -> Self {
        Self {
            date: date,
            time: time,
        }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let time: u16 = bytes_to_u16_le!(data, 0);
        let date: u16 = bytes_to_u16_le!(data, 2);
        Self {
            date: date,
            time: time,
        }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let year: u16 = 1980 + ((self.date >> 9) & 0x7f);
        let month: u16 = (self.date >> 5) & 0x0f;
        let day_of_month: u16 = self.date & 0x1f;
        let hours: u16 = (self.time >> 11) & 0x1f;
        let minutes: u16 = (self.time >> 5) & 0x3f;
        let seconds: u16 = (self.time & 0x1f) * 2;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day_of_month, hours, minutes, seconds
        )
    }
}

impl fmt::Display for FatTimeDate {
    /// Formats the timestamp for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} (0x{:04x}:0x{:04x})",
            self.to_iso8601_string(),
            self.date,
            self.time,
        )
    }
}

/// FAT time and date in 10 millisecond intervals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatTimeDate10Ms {
    /// Date.
    pub date: u16,

    /// Time.
    pub time: u16,

    /// Fraction of second.
    pub fraction: u8,
}

impl FatTimeDate10Ms {
    /// Creates a new timestamp.
    pub fn new(date: u16, time: u16, fraction: u8) -> Self {
        Self {
            date: date,
            time: time,
            fraction: fraction,
        }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let time: u16 = bytes_to_u16_le!(data, 1);
        let date: u16 = bytes_to_u16_le!(data, 3);
        Self {
            date: date,
            time: time,
            fraction: data[0],
        }
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let year: u16 = 1980 + ((self.date >> 9) & 0x7f);
        let month: u16 = (self.date >> 5) & 0x0f;
        let day_of_month: u16 = self.date & 0x1f;
        let hours: u16 = (self.time >> 11) & 0x1f;
        let minutes: u16 = (self.time >> 5) & 0x3f;
        let milliseconds: u16 = ((self.time & 0x1f) * 2000) + ((self.fraction as u16) * 10);

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:02}",
            year,
            month,
            day_of_month,
            hours,
            minutes,
            (milliseconds / 1000),
            (milliseconds % 1000) / 10
        )
    }
}

impl fmt::Display for FatTimeDate10Ms {
    /// Formats the timestamp for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{} (0x{:04x}:0x{:04x}:0x{:02x})",
            self.to_iso8601_string(),
            self.date,
            self.time,
            self.fraction,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fat_date_from_bytes() {
        let test_data: [u8; 2] = [0x0c, 0x3d];

        let test_struct: FatDate = FatDate::from_bytes(&test_data);
        assert_eq!(test_struct.date, 0x3d0c);
    }

    #[test]
    fn test_fat_date_to_iso8601_string() {
        let test_struct: FatDate = FatDate::new(0x3d0c);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12");
    }

    #[test]
    fn test_fat_time_date_from_bytes() {
        let test_data: [u8; 4] = [0xd0, 0xa8, 0x0c, 0x3d];

        let test_struct: FatTimeDate = FatTimeDate::from_bytes(&test_data);
        assert_eq!(test_struct.date, 0x3d0c);
        assert_eq!(test_struct.time, 0xa8d0);
    }

    #[test]
    fn test_fat_time_date_to_iso8601_string() {
        let test_struct: FatTimeDate = FatTimeDate::new(0x3d0c, 0xa8d0);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:32");
    }

    #[test]
    fn test_fat_time_date_10ms_from_bytes() {
        let test_data: [u8; 5] = [0x7d, 0xd0, 0xa8, 0x0c, 0x3d];

        let test_struct: FatTimeDate10Ms = FatTimeDate10Ms::from_bytes(&test_data);
        assert_eq!(test_struct.date, 0x3d0c);
        assert_eq!(test_struct.time, 0xa8d0);
        assert_eq!(test_struct.fraction, 0x7d);
    }

    #[test]
    fn test_fat_time_date_10ms_to_iso8601_string() {
        let test_struct: FatTimeDate10Ms = FatTimeDate10Ms::new(0x3d0c, 0xa8d0, 0x7d);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:33.25");
    }
}
