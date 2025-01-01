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

use super::epoch::Epoch;

/// Determines if the year is a leap-year.
#[inline(always)]
fn is_leap_year(year: i16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Retrieves the number of days in a specific month.
#[inline(always)]
fn get_days_in_month(year: i16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => panic!("Invalid month: {} value out of bounds", month),
    }
}

/// Calculate the days in year lookup table.
const fn calculate_days_in_year_lookup_table() -> [i16; 10000] {
    let mut lookup_table: [i16; 10000] = [0; 10000];
    let mut table_index: usize = 0;
    let mut year: i16 = 0;

    while year < 10000 {
        if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            lookup_table[table_index] = 366;
        } else {
            lookup_table[table_index] = 365;
        }
        table_index += 1;
        year += 1;
    }
    lookup_table
}

const DAYS_IN_YEAR: [i16; 10000] = calculate_days_in_year_lookup_table();

/// Retrieves the number of days in a specific year.
#[inline(always)]
fn get_days_in_year(year: i16) -> i16 {
    if year >= 0 && year <= 10000 {
        DAYS_IN_YEAR[year as usize]
    } else if is_leap_year(year) {
        366
    } else {
        365
    }
}

/// Calculate the days in century lookup table.
const fn calculate_days_in_century_lookup_table() -> [i32; 100] {
    let mut lookup_table: [i32; 100] = [0; 100];
    let mut table_index: usize = 0;
    let mut year: i16 = 0;

    while year < 10000 {
        if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            lookup_table[table_index] = 36525;
        } else {
            lookup_table[table_index] = 36524;
        }
        table_index += 1;
        year += 100;
    }
    lookup_table
}

const DAYS_IN_CENTURY: [i32; 100] = calculate_days_in_century_lookup_table();

/// Retrieves the number of days in a specific century.
#[inline(always)]
fn get_days_in_century(year: i16) -> i32 {
    if year >= 0 && year <= 10000 {
        DAYS_IN_CENTURY[(year / 100) as usize]
    } else if is_leap_year(year) {
        36525
    } else {
        36524
    }
}

/// Retrieves date values.
pub fn get_date_values(mut number_of_days: i64, epoch: &Epoch) -> (i16, u8, u8) {
    let before_epoch: bool = number_of_days < 0;

    let mut year: i16 = epoch.year;
    let mut month: u8 = epoch.month;
    number_of_days += epoch.day_of_month as i64;

    if before_epoch {
        month -= 1;

        if month <= 0 {
            month = 12;
            year -= 1;
        }
        number_of_days *= -1;
    }
    // Align with the start of the year.
    while month > 1 {
        let days_in_month: i64 = get_days_in_month(year, month) as i64;
        if number_of_days < days_in_month {
            break;
        }
        if before_epoch {
            month -= 1;
        } else {
            month += 1;
        }
        if month > 12 {
            month = 1;
            year += 1;
        }
        number_of_days -= days_in_month;
    }
    // Align with the start of the next century.
    let remaining_years: usize = (year as usize) % 100;
    for _ in remaining_years..100 {
        let days_in_year: i64 = get_days_in_year(year) as i64;
        if number_of_days < days_in_year {
            break;
        }
        if before_epoch {
            year -= 1;
        } else {
            year += 1;
        }
        number_of_days -= days_in_year;
    }
    let mut days_in_century: i64 = get_days_in_century(year) as i64;
    while number_of_days > days_in_century {
        if before_epoch {
            year -= 100;
        } else {
            year += 100;
        }
        number_of_days -= days_in_century;

        days_in_century = get_days_in_century(year) as i64;
    }
    let mut days_in_year: i64 = get_days_in_year(year) as i64;
    while number_of_days > days_in_year {
        if before_epoch {
            year -= 1;
        } else {
            year += 1;
        }
        number_of_days -= days_in_year;

        days_in_year = get_days_in_year(year) as i64;
    }
    let mut days_in_month: i64 = get_days_in_month(year, month) as i64;
    while number_of_days > days_in_month {
        if before_epoch {
            month -= 1;
        } else {
            month += 1;
        }
        if month <= 0 {
            month = 12;
            year -= 1;
        } else if month > 12 {
            month = 1;
            year += 1;
        }
        number_of_days -= days_in_month;

        days_in_month = get_days_in_month(year, month) as i64;
    }
    if before_epoch {
        let days_in_month: i64 = get_days_in_month(year, month) as i64;
        number_of_days = days_in_month - number_of_days;
    } else if number_of_days == 0 {
        number_of_days = 31;
        month = 12;
        year -= 1;
    }
    (year, month, number_of_days as u8)
}

/// Retrieves time values.
pub fn get_time_values(mut number_of_seconds: i64) -> (i64, u8, u8, u8) {
    let seconds: u8 = number_of_seconds.rem_euclid(60) as u8;
    number_of_seconds = number_of_seconds.div_euclid(60);

    let minutes: u8 = number_of_seconds.rem_euclid(60) as u8;
    number_of_seconds = number_of_seconds.div_euclid(60);

    let hours: u8 = number_of_seconds.rem_euclid(24) as u8;
    number_of_seconds = number_of_seconds.div_euclid(24);

    (number_of_seconds, hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert_eq!(is_leap_year(1996), true);
        assert_eq!(is_leap_year(1999), false);
        assert_eq!(is_leap_year(2000), true);
    }

    #[test]
    fn test_get_days_in_month() {
        assert_eq!(get_days_in_month(1999, 1), 31);
        assert_eq!(get_days_in_month(1999, 2), 28);
        assert_eq!(get_days_in_month(1999, 3), 31);
        assert_eq!(get_days_in_month(1999, 4), 30);
        assert_eq!(get_days_in_month(1999, 5), 31);
        assert_eq!(get_days_in_month(1999, 6), 30);
        assert_eq!(get_days_in_month(1999, 7), 31);
        assert_eq!(get_days_in_month(1999, 8), 31);
        assert_eq!(get_days_in_month(1999, 9), 30);
        assert_eq!(get_days_in_month(1999, 10), 31);
        assert_eq!(get_days_in_month(1999, 11), 30);
        assert_eq!(get_days_in_month(1999, 12), 31);

        assert_eq!(get_days_in_month(2000, 2), 29);
    }

    #[test]
    fn test_get_days_in_year() {
        assert_eq!(get_days_in_year(1999), 365);
        assert_eq!(get_days_in_year(2000), 366);
    }

    #[test]
    fn test_get_days_in_century() {
        assert_eq!(get_days_in_century(1700), 36524);
        assert_eq!(get_days_in_century(2000), 36525);
    }

    #[test]
    fn test_get_date_values() {
        let test_epoch: Epoch = Epoch::new(2000, 1, 1);
        assert_eq!(get_date_values(0, &test_epoch), (2000, 1, 1));
        assert_eq!(get_date_values(10, &test_epoch), (2000, 1, 11));
        assert_eq!(get_date_values(100, &test_epoch), (2000, 4, 10));

        assert_eq!(get_date_values(-10, &test_epoch), (1999, 12, 22));
        assert_eq!(get_date_values(-100, &test_epoch), (1999, 9, 23));

        let test_epoch: Epoch = Epoch::new(1999, 1, 1);
        assert_eq!(get_date_values(100, &test_epoch), (1999, 4, 11));

        let test_epoch: Epoch = Epoch::new(1999, 12, 30);
        assert_eq!(get_date_values(0, &test_epoch), (1999, 12, 30));
        assert_eq!(get_date_values(5, &test_epoch), (2000, 1, 4));

        let test_epoch: Epoch = Epoch::new(2000, 1, 9);
        assert_eq!(get_date_values(-10, &test_epoch), (1999, 12, 30));

        let test_epoch: Epoch = Epoch::new(1899, 12, 30);
        assert_eq!(get_date_values(0, &test_epoch), (1899, 12, 30));
        assert_eq!(get_date_values(25569, &test_epoch), (1970, 1, 1));
        assert_eq!(get_date_values(36526, &test_epoch), (2000, 1, 1));
        assert_eq!(get_date_values(41275, &test_epoch), (2013, 1, 1));
        assert_eq!(get_date_values(41443, &test_epoch), (2013, 6, 18));
        assert_eq!(get_date_values(-25569, &test_epoch), (1829, 12, 28));

        let test_epoch: Epoch = Epoch::new(1970, 1, 1);
        assert_eq!(get_date_values(0, &test_epoch), (1970, 1, 1));
        assert_eq!(get_date_values(-1, &test_epoch), (1969, 12, 31));
        assert_eq!(get_date_values(364, &test_epoch), (1970, 12, 31));
        assert_eq!(get_date_values(1460, &test_epoch), (1973, 12, 31));
    }

    #[test]
    fn test_get_time_values() {
        assert_eq!(get_time_values(10), (0, 0, 0, 10));
        assert_eq!(get_time_values(190), (0, 0, 3, 10));
        assert_eq!(get_time_values(18190), (0, 5, 3, 10));
        assert_eq!(get_time_values(190990), (2, 5, 3, 10));

        assert_eq!(get_time_values(-10), (-1, 23, 59, 50));
        assert_eq!(get_time_values(-190), (-1, 23, 56, 50));
        assert_eq!(get_time_values(-18190), (-1, 18, 56, 50));
        assert_eq!(get_time_values(-190990), (-3, 18, 56, 50));
    }
}
