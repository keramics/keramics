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

    if month == 0 || month > 12 || day_of_month == 0 {
        return 0;
    }
    // Number of days since January 1, 1980, where day 0 represents January 1, 1980.
    let mut number_of_days: u32 = (day_of_month as u32) - 1;

    month -= 1;

    while month > 0 {
        number_of_days += get_days_in_month(year as i64, month) as u32;
        month -= 1;
    }
    year -= 1;

    while year >= 1980 {
        number_of_days += get_days_in_year(year as i64) as u32;
        year -= 1;
    }
    let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(time);

    let mut number_of_seconds: u32 = (number_of_days * 24) + (hours as u32);
    number_of_seconds = (number_of_seconds * 60) + (minutes as u32);
    (number_of_seconds * 60) + (seconds as u32)
}

/// Retrieves the number of seconds since January 1, 1980 with a fraction of a second.
#[inline(always)]
fn fat_get_number_of_seconds_with_fraction(date: u16, time: u16, fraction: u8) -> (u32, u32) {
    let seconds: u32 = fat_get_number_of_seconds(date, time);
    let milliseconds: u64 = ((seconds as u64) * 100) + (fraction as u64);

    ((milliseconds / 100) as u32, (milliseconds % 100) as u32)
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
        hours,
        minutes,
        (milliseconds / 100) as u8,
        (milliseconds % 100) as u8,
    )
}

/// File Allocation Table (FAT) date.
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

/// File Allocation Table (FAT) time and date.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatTimeDate {
    /// Date.
    pub date: u16,

    /// Time.
    pub time: u16,

    /// UTC offset.
    pub utc_offset: u8,
}

impl FatTimeDate {
    /// Creates a new timestamp.
    pub fn new(date: u16, time: u16) -> Self {
        Self {
            date,
            time,
            utc_offset: 0,
        }
    }

    /// Reads a timestamp from a byte sequence.
    pub fn from_bytes(data: &[u8]) -> Self {
        let time: u16 = bytes_to_u16_le!(data, 0);
        let date: u16 = bytes_to_u16_le!(data, 2);
        Self {
            date,
            time,
            utc_offset: 0,
        }
    }

    /// Retrieves the timestamp as number of seconds since January 1, 1980.
    pub fn get_number_of_seconds(&self) -> u32 {
        let mut number_of_seconds: u32 = fat_get_number_of_seconds(self.date, self.time);

        // UTC offset contains the number of 15 minute intervals of the time relative to UTC, where
        // an MSB of 1 indicates the offset is valid (and 0 invalid).
        if self.utc_offset & 0x80 != 0 {
            let utc_offset: u8 = self.utc_offset & 0x7f;
            if utc_offset & 0x40 != 0 {
                number_of_seconds =
                    number_of_seconds.saturating_add(((0x80 - utc_offset) as u32) * 15 * 60);
            } else {
                // Note that a timestamp in with an UTC offset can predate January 1, 1980, UTC.
                // Such timestamps are currently represented as 0.
                number_of_seconds = number_of_seconds.saturating_sub((utc_offset as u32) * 15 * 60);
            }
        }
        number_of_seconds
    }

    /// Sets the UTC offset.
    pub fn set_utc_offset(&mut self, utc_offset: u8) {
        self.utc_offset = utc_offset
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(self.date);
        let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(self.time);

        if self.utc_offset & 0x80 != 0 {
            let (time_zone_sign, time_zone_hours, time_zone_minutes): (char, u8, u8) =
                if self.utc_offset & 0x40 != 0 {
                    let utc_offset: u8 = 0x80 - (self.utc_offset & 0x7f);
                    ('-', (utc_offset / 4), (utc_offset % 4) * 60)
                } else {
                    let utc_offset: u8 = self.utc_offset & 0x7f;
                    ('+', (utc_offset / 4), (utc_offset % 4) * 60)
                };
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
                year,
                month,
                day_of_month,
                hours,
                minutes,
                seconds,
                time_zone_sign,
                time_zone_hours,
                time_zone_minutes
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                year, month, day_of_month, hours, minutes, seconds
            )
        }
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

/// File Allocation Table (FAT) time and date in 10 millisecond intervals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FatTimeDate10Ms {
    /// Date.
    pub date: u16,

    /// Time.
    pub time: u16,

    /// Fraction of second.
    pub fraction: u8,

    /// UTC offset.
    pub utc_offset: u8,
}

impl FatTimeDate10Ms {
    /// Creates a new timestamp.
    pub fn new(date: u16, time: u16, fraction: u8) -> Self {
        Self {
            date,
            time,
            fraction,
            utc_offset: 0,
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
            utc_offset: 0,
        }
    }

    /// Retrieves the timestamp as number of seconds since January 1, 1980.
    pub fn get_number_of_seconds(&self) -> (u32, u32) {
        let (mut number_of_seconds, fraction) =
            fat_get_number_of_seconds_with_fraction(self.date, self.time, self.fraction);

        // UTC offset contains the number of 15 minute intervals of the time relative to UTC, where
        // an MSB of 1 indicates the offset is valid (and 0 invalid).
        if self.utc_offset & 0x80 != 0 {
            let utc_offset: u8 = self.utc_offset & 0x7f;
            if utc_offset & 0x40 != 0 {
                number_of_seconds =
                    number_of_seconds.saturating_add(((0x80 - utc_offset) as u32) * 15 * 60);
            } else {
                // Note that a timestamp in with an UTC offset can predate January 1, 1980, UTC.
                // Such timestamps are currently represented as 0.
                number_of_seconds = number_of_seconds.saturating_sub((utc_offset as u32) * 15 * 60);
            }
        }
        (number_of_seconds, fraction)
    }

    /// Sets the UTC offset.
    pub fn set_utc_offset(&mut self, utc_offset: u8) {
        self.utc_offset = utc_offset
    }

    /// Retrieves an ISO 8601 string representation of the timestamp.
    pub fn to_iso8601_string(&self) -> String {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(self.date);
        let (hours, minutes, seconds, fraction): (u8, u8, u8, u8) =
            fat_get_time_values_with_fraction(self.time, self.fraction);

        if self.utc_offset & 0x80 != 0 {
            let (time_zone_sign, time_zone_hours, time_zone_minutes): (char, u8, u8) =
                if self.utc_offset & 0x40 != 0 {
                    let utc_offset: u8 = 0x80 - (self.utc_offset & 0x7f);
                    ('-', (utc_offset / 4), (utc_offset % 4) * 60)
                } else {
                    let utc_offset: u8 = self.utc_offset & 0x7f;
                    ('+', (utc_offset / 4), (utc_offset % 4) * 60)
                };
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:02}{}{:02}:{:02}",
                year,
                month,
                day_of_month,
                hours,
                minutes,
                seconds,
                fraction,
                time_zone_sign,
                time_zone_hours,
                time_zone_minutes
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:02}",
                year, month, day_of_month, hours, minutes, seconds, fraction
            )
        }
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

    // Calculates the number of days between January 1, year 0 and a civil date, with the
    // days-difference algorithm.
    fn get_days_from_civil(year: i64, month: u8, day_of_month: u8) -> i64 {
        let year: i64 = if month <= 2 { year - 1 } else { year };
        let era: i64 = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era: i64 = year - (era * 400);
        let month_prime: i64 = (month as i64 + 9) % 12;
        let day_of_year: i64 = ((153 * month_prime) + 2) / 5 + (day_of_month as i64) - 1;
        let day_of_era: i64 =
            (year_of_era * 365) + (year_of_era / 4) - (year_of_era / 100) + day_of_year;

        (era * 146097) + day_of_era
    }

    #[test]
    fn test_fat_get_date_values() {
        let (year, month, day_of_month): (i16, u8, u8) = fat_get_date_values(0x3d0c);
        assert_eq!(year, 2010);
        assert_eq!(month, 8);
        assert_eq!(day_of_month, 12);
    }

    #[test]
    fn test_fat_get_number_of_seconds() {
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x3d0c, 0xa8d0);
        assert_eq!(number_of_seconds, 966114392);

        // The first valid date.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0021, 0x0000);
        assert_eq!(number_of_seconds, 0);

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0022, 0x0000);
        assert_eq!(number_of_seconds, 86400);

        // The leap day of the leap year 2000.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x285d, 0x0000);
        assert_eq!(number_of_seconds, 636249600);

        // The last day of February of the regular year 2010, and the day after.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x3c5c, 0x0000);
        assert_eq!(number_of_seconds, 951782400);

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x3c61, 0x0000);
        assert_eq!(number_of_seconds, 951868800);

        // Other leap days and their surroundings.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x085d, 0x0000);
        assert_eq!(number_of_seconds, 131328000); // 1984-02-29

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x305d, 0x0000);
        assert_eq!(number_of_seconds, 762480000); // 2004-02-29

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x385c, 0x0000);
        assert_eq!(number_of_seconds, 888624000); // 2008-02-28

        let number_of_seconds: u32 = fat_get_number_of_seconds(0xf85d, 0x0000);
        assert_eq!(number_of_seconds, 3918153600); // 2104-02-29

        // Year and month boundaries across the valid date range up to the last valid date.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0061, 0x0000);
        assert_eq!(number_of_seconds, 5184000); // 1980-03-01

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x019f, 0x0000);
        assert_eq!(number_of_seconds, 31536000); // 1980-12-31

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0221, 0x0000);
        assert_eq!(number_of_seconds, 31622400); // 1981-01-01

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x279f, 0x0000);
        assert_eq!(number_of_seconds, 631065600); // 1999-12-31

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x3a21, 0x0000);
        assert_eq!(number_of_seconds, 915235200); // 2009-01-01

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x8821, 0x0000);
        assert_eq!(number_of_seconds, 2145916800); // 2048-01-01

        let number_of_seconds: u32 = fat_get_number_of_seconds(0xef9f, 0x0000);
        assert_eq!(number_of_seconds, 3786825600); // 2099-12-31

        let number_of_seconds: u32 = fat_get_number_of_seconds(0xfe5c, 0x0000);
        assert_eq!(number_of_seconds, 4012761600); // 2107-02-28

        let number_of_seconds: u32 = fat_get_number_of_seconds(0xf061, 0x0000);
        assert_eq!(number_of_seconds, 3792009600); // 2100-03-01

        let number_of_seconds: u32 = fat_get_number_of_seconds(0xff9f, 0x0000);
        assert_eq!(number_of_seconds, 4039200000); // 2107-12-31

        // A zero date is invalid, such as an unspecified timestamp, and results in 0 seconds.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0000, 0x0000);
        assert_eq!(number_of_seconds, 0);

        let number_of_seconds: u32 = fat_get_number_of_seconds(0x0000, 0xa8d0);
        assert_eq!(number_of_seconds, 0);

        // A zero month with a non-zero day of month is invalid and results in 0 seconds.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0xfe0f, 0x0000);
        assert_eq!(number_of_seconds, 0);

        // A month larger than 12 is invalid and results in 0 seconds.
        let number_of_seconds: u32 = fat_get_number_of_seconds(0xffff, 0x0000);
        assert_eq!(number_of_seconds, 0);
    }

    #[test]
    fn test_fat_get_number_of_seconds_full_range() {
        let epoch_days: i64 = get_days_from_civil(1980, 1, 1);
        let days_in_month: [u8; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        for year in 1980..2108i16 {
            let leap_year: bool = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            for month in 1..13u8 {
                let last_day: u8 = if month == 2 && leap_year {
                    29
                } else {
                    days_in_month[month as usize]
                };
                for day_of_month in 1..=last_day {
                    let date: u16 =
                        (((year - 1980) << 9) | (month as i16) << 5 | day_of_month as i16) as u16;
                    let expected_days: i64 =
                        get_days_from_civil(year as i64, month, day_of_month) - epoch_days;
                    assert_eq!(
                        fat_get_number_of_seconds(date, 0) as i64,
                        expected_days * 86400,
                        "year: {}, month: {}, day_of_month: {}",
                        year,
                        month,
                        day_of_month
                    );
                }
            }
        }
    }

    #[test]
    fn test_fat_get_number_of_seconds_with_time_full_range() {
        for hours in 0..24u8 {
            for minutes in 0..60u8 {
                for seconds_ticks in 0..30u8 {
                    let time: u16 =
                        ((hours as u16) << 11) | ((minutes as u16) << 5) | (seconds_ticks as u16);
                    let oracle_seconds: u32 = ((hours as u32 * 60) + (minutes as u32)) * 60
                        + ((seconds_ticks as u32) * 2);
                    assert_eq!(
                        fat_get_number_of_seconds(0x0021, time),
                        oracle_seconds,
                        "hours: {}, minutes: {}, seconds_ticks: {}",
                        hours,
                        minutes,
                        seconds_ticks
                    );
                }
            }
        }
    }

    #[test]
    fn test_fat_get_number_of_seconds_with_fraction() {
        let (number_of_seconds, fraction): (u32, u32) =
            fat_get_number_of_seconds_with_fraction(0x3d0c, 0xa8d0, 0x7d);
        assert_eq!(number_of_seconds, 966114393);
        assert_eq!(fraction, 25);
    }

    #[test]
    fn test_fat_get_time_values() {
        let (hours, minutes, seconds): (u8, u8, u8) = fat_get_time_values(0xa8d0);
        assert_eq!(hours, 21);
        assert_eq!(minutes, 6);
        assert_eq!(seconds, 32);
    }

    #[test]
    fn test_fat_get_time_values_with_fraction() {
        let (hours, minutes, seconds, fraction): (u8, u8, u8, u8) =
            fat_get_time_values_with_fraction(0xa8d0, 0x7d);
        assert_eq!(hours, 21);
        assert_eq!(minutes, 6);
        assert_eq!(seconds, 33);
        assert_eq!(fraction, 25);
    }

    #[test]
    fn test_fat_date_from_bytes() {
        let test_data: [u8; 2] = [0x0c, 0x3d];

        let test_struct: FatDate = FatDate::from_bytes(&test_data);
        assert_eq!(test_struct.date, 0x3d0c);
    }

    #[test]
    fn test_fat_date_get_number_of_seconds() {
        let test_struct: FatDate = FatDate::new(0x3d0c);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966038400);
    }

    #[test]
    fn test_fat_date_to_iso8601_string() {
        let test_struct: FatDate = FatDate::new(0x3d0c);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12");
    }

    #[test]
    fn test_fat_date_to_string() {
        let test_struct: FatDate = FatDate::new(0x3d0c);

        let string: String = test_struct.to_string();
        assert_eq!(string.as_str(), "2010-08-12 (0x3d0c)");
    }

    #[test]
    fn test_fat_time_date_from_bytes() {
        let test_data: [u8; 4] = [0xd0, 0xa8, 0x0c, 0x3d];

        let test_struct: FatTimeDate = FatTimeDate::from_bytes(&test_data);
        assert_eq!(test_struct.date, 0x3d0c);
        assert_eq!(test_struct.time, 0xa8d0);
    }

    #[test]
    fn test_fat_time_date_get_number_of_seconds() {
        let mut test_struct: FatTimeDate = FatTimeDate::new(0x3d0c, 0xa8d0);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966114392);

        test_struct.set_utc_offset(0x88);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966107192);

        test_struct.set_utc_offset(0xf8);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966121592);

        // Test with a timestamp in with an UTC offset that predates January 1, 1980, UTC.
        let mut test_struct: FatTimeDate = FatTimeDate::new(0x0021, 0x0000);
        test_struct.set_utc_offset(0x81);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 0);

        let mut test_struct: FatTimeDate = FatTimeDate::new(0x0021, 0x03c0);
        test_struct.set_utc_offset(0x81);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 900);

        let mut test_struct: FatTimeDate = FatTimeDate::new(0x0021, 0x0000);
        test_struct.set_utc_offset(0xf8);

        let number_of_seconds: u32 = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 7200);
    }

    #[test]
    fn test_fat_time_date_to_iso8601_string() {
        let mut test_struct: FatTimeDate = FatTimeDate::new(0x3d0c, 0xa8d0);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:32");

        test_struct.set_utc_offset(0x88);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:32+02:00");

        test_struct.set_utc_offset(0xf8);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:32-02:00");
    }

    #[test]
    fn test_fat_time_date_to_string() {
        let test_struct: FatTimeDate = FatTimeDate::new(0x3d0c, 0xa8d0);

        let string: String = test_struct.to_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:32 (0x3d0c:0xa8d0)");
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
    fn test_fat_time_date_10ms_get_number_of_seconds() {
        let mut test_struct: FatTimeDate10Ms = FatTimeDate10Ms::new(0x3d0c, 0xa8d0, 0x7d);

        let (number_of_seconds, fraction): (u32, u32) = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966114393);
        assert_eq!(fraction, 25);

        test_struct.set_utc_offset(0x88);

        let (number_of_seconds, fraction): (u32, u32) = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966107193);
        assert_eq!(fraction, 25);

        test_struct.set_utc_offset(0xf8);

        let (number_of_seconds, fraction): (u32, u32) = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 966121593);
        assert_eq!(fraction, 25);

        // Test with a timestamp in with an UTC offset that predates January 1, 1980, UTC.
        let mut test_struct: FatTimeDate10Ms = FatTimeDate10Ms::new(0x0021, 0x0000, 0x7d);
        test_struct.set_utc_offset(0x81);

        let (number_of_seconds, fraction): (u32, u32) = test_struct.get_number_of_seconds();
        assert_eq!(number_of_seconds, 0);
        assert_eq!(fraction, 25);
    }

    #[test]
    fn test_fat_time_date_10ms_to_iso8601_string() {
        let mut test_struct: FatTimeDate10Ms = FatTimeDate10Ms::new(0x3d0c, 0xa8d0, 0x7d);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:33.25");

        test_struct.set_utc_offset(0x88);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:33.25+02:00");

        test_struct.set_utc_offset(0xf8);

        let string: String = test_struct.to_iso8601_string();
        assert_eq!(string.as_str(), "2010-08-12T21:06:33.25-02:00");
    }

    #[test]
    fn test_fat_time_date_10ms_to_string() {
        let test_struct: FatTimeDate10Ms = FatTimeDate10Ms::new(0x3d0c, 0xa8d0, 0x7d);

        let string: String = test_struct.to_string();
        assert_eq!(
            string.as_str(),
            "2010-08-12T21:06:33.25 (0x3d0c:0xa8d0:0x7d)"
        );
    }
}
