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

use keramics_types::bytes_to_u16_le;

use super::util::{get_days_in_month, get_days_in_year};

/// Retrieves date values.
#[inline(always)]
fn fat_get_date_values(date: u16) -> (i16, u8, u8) {
    // The year is stored in bits 9 - 15 of the date (7 bits)
    // and value of 0 represents 1980
    let year: u16 = 1980 + ((date >> 9) & 0x007f);

    // The month is stored in bits 5 - 8 of the date (4 bits)
    // and a value of 1 represents January
    let month: u16 = (date >> 5) & 0x000f;

    // The day of month is stored in bits 0 - 4 of the date (5 bits)
    let day_of_month: u16 = date & 0x001f;

    (year as i16, month as u8, day_of_month as u8)
}

/// Retrieves the number of seconds since January 1, 1980.
#[inline(always)]
fn fat_get_number_of_seconds(date: u16, time: u16) -> u32 {
    let (mut year, mut month, day_of_month): (i16, u8, u8) = fat_get_date_values(date);
    let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(time);

    let mut number_of_seconds: u32 = day_of_month as u32;

    while month > 0 {
        number_of_seconds += get_days_in_month(year, month) as u32;
        month -= 1;
    }
    while year > 1980 {
        number_of_seconds += get_days_in_year(year) as u32;
        year -= 1;
    }
    number_of_seconds = (number_of_seconds * 24) + (hours as u32);
    number_of_seconds = (number_of_seconds * 60) + (minutes as u32);

    (number_of_seconds * 60) + (seconds as u32)
}

/// Retrieves the number of seconds since January 1, 1980 with a fraction of a second.
#[inline(always)]
fn fat_get_number_of_seconds_with_fraction(date: u16, time: u16, fraction: u8) -> (u32, u32) {
    let seconds: u32 = fat_get_number_of_seconds(date, time);
    let milliseconds: u32 = (seconds * 100) + (fraction as u32);

    (milliseconds / 100, milliseconds % 100)
}

/// Retrieves time values.
#[inline(always)]
fn fat_get_time_values(time: u16) -> (u8, u8, u8) {
    // The hours are stored in bits 11 - 15 of the time (5 bits)
    let hours: u16 = (time >> 11) & 0x001f;

    // The minutes are stored in bits 5 - 10 of the time (6 bits)
    let minutes: u16 = (time >> 5) & 0x003f;

    // The seconds are stored in bits 0 - 4 of the time (5 bits)
    // The seconds are stored as 2 second intervals
    let seconds: u16 = (time & 0x001f) * 2;

    (hours as u8, minutes as u8, seconds as u8)
}

/// Retrieves time values with a fraction of second.
#[inline(always)]
fn fat_get_time_values_with_fraction(time: u16, fraction: u8) -> (u8, u8, u8, u8) {
    let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(time);

    let milliseconds: u16 = ((seconds as u16) * 100) + (fraction as u16);

    (
        hours as u8,
        minutes as u8,
        (milliseconds / 100) as u8,
        (milliseconds % 100) as u8,
    )
}

/// FAT date.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatDate {
    /// Date.
    pub date: u16,
}

impl FatDate {
    /// Creates a new timestamp.
    pub fn new(date: u16) -> Self {
        Self { date }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let date: u16 = bytes_to_u16_le!(data, 0);
        Self { date }
    }

    /// Retrieves the timestamp as number of seconds since January 1, 1980.
    pub fn get_number_of_seconds(&self) -> u32 {
        fat_get_number_of_seconds(self.date, 0)
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(self.date);

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
        Self { date, time }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let time: u16 = bytes_to_u16_le!(data, 0);
        let date: u16 = bytes_to_u16_le!(data, 2);
        Self { date, time }
    }

    /// Retrieves the timestamp as number of seconds since January 1, 1980.
    pub fn get_number_of_seconds(&self) -> u32 {
        fat_get_number_of_seconds(self.date, self.time)
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(self.date);
        let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(self.time);

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
            date,
            time,
            fraction,
        }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let time: u16 = bytes_to_u16_le!(data, 1);
        let date: u16 = bytes_to_u16_le!(data, 3);
        Self {
            date,
            time,
            fraction: data[0],
        }
    }

    /// Retrieves the timestamp as number of seconds since January 1, 1980.
    pub fn get_number_of_seconds(&self) -> (u32, u32) {
        fat_get_number_of_seconds_with_fraction(self.date, self.time, self.fraction)
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(self.date);
        let (hours, minutes, seconds, fraction): (u8, u8, u8, u8) =
            fat_get_time_values_with_fraction(self.time, self.fraction);

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:02}",
            year, month, day_of_month, hours, minutes, seconds, fraction
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

    // TODO: add tests for fat_get_date_values
    // TODO: add tests for fat_get_number_of_seconds
    // TODO: add tests for fat_get_time_values
    // TODO: add tests for fat_get_time_values_with_fraction

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
