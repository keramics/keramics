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

use super::epoch::Epoch;

/// Determines if the year is a leap-year.
#[inline(always)]
pub(crate) fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Retrieves the number of days in a specific month.
#[inline(always)]
pub(crate) fn get_days_in_month(year: i64, month: u8) -> u8 {
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

static DAYS_IN_YEAR: [i16; 10000] = calculate_days_in_year_lookup_table();

/// Retrieves the number of days in a specific year.
#[inline(always)]
pub(crate) fn get_days_in_year(year: i64) -> i16 {
    if (0..10000).contains(&year) {
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
fn get_days_in_century(year: i64) -> i32 {
    if (0..10000).contains(&year) {
        DAYS_IN_CENTURY[(year / 100) as usize]
    } else if is_leap_year(year) {
        36525
    } else {
        36524
    }
}

/// Number of days in a 400 year cycle of the Gregorian calendar.
const DAYS_IN_400_YEARS: i64 = 146097;

/// Retrieves date values.
///
/// The returned year is a `i32` value, since for some timestamp formats (such as the Windows
/// FILETIME) the number of days can exceed the range of years representable by an `i16` value.
pub fn get_date_values(number_of_days: i64, epoch: &Epoch) -> (i32, u8, u8) {
    let mut number_of_cycles: i64 = 0;

    // Day counts predating the start of the epoch month are shifted forward by whole 400 year
    // cycles, since the Gregorian calendar repeats every 400 years, and the difference in years
    // is corrected at the end.
    let mut number_of_days: i64 = number_of_days + (epoch.day_of_month as i64) - 1;
    if number_of_days < 0 {
        number_of_cycles = (-number_of_days + DAYS_IN_400_YEARS - 1) / DAYS_IN_400_YEARS;
        number_of_days += number_of_cycles * DAYS_IN_400_YEARS;
    }

    // `number_of_days` contains the 0-based number of days since the first day of the month,
    // where day 0 represents the first day of the month.
    let mut year: i64 = epoch.year as i64;
    let mut month: u8 = epoch.month;

    // Advance to the month containing the date, which also advances to the next year when the
    // date predates the first day of the epoch month.
    while month > 1 {
        let days_in_month: i64 = get_days_in_month(year, month) as i64;
        if number_of_days < days_in_month {
            break;
        }
        month += 1;

        if month > 12 {
            month = 1;
            year += 1;
        }
        number_of_days -= days_in_month;
    }
    // Advance to the year containing the date, by first advancing to the next century boundary.
    for _ in (year % 100)..100 {
        let days_in_year: i64 = get_days_in_year(year) as i64;
        if number_of_days < days_in_year {
            break;
        }
        year += 1;
        number_of_days -= days_in_year;
    }
    // Advance to the century containing the date.
    let mut days_in_century: i64 = get_days_in_century(year) as i64;

    while number_of_days >= days_in_century {
        year += 100;
        number_of_days -= days_in_century;

        days_in_century = get_days_in_century(year) as i64;
    }
    // Advance to the year containing the date.
    let mut days_in_year: i64 = get_days_in_year(year) as i64;

    while number_of_days >= days_in_year {
        year += 1;
        number_of_days -= days_in_year;

        days_in_year = get_days_in_year(year) as i64;
    }
    // Advance to the month containing the date.
    let mut days_in_month: i64 = get_days_in_month(year, month) as i64;

    while number_of_days >= days_in_month {
        month += 1;

        if month > 12 {
            month = 1;
            year += 1;
        }
        number_of_days -= days_in_month;

        days_in_month = get_days_in_month(year, month) as i64;
    }
    (
        (year - (number_of_cycles * 400)) as i32,
        month,
        (number_of_days + 1) as u8,
    )
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
    #[should_panic]
    fn test_get_days_in_month_with_invalid_month() {
        get_days_in_month(2000, 0);
    }

    #[test]
    fn test_calculate_days_in_year_lookup_table() {
        let days_in_year: [i16; 10000] = calculate_days_in_year_lookup_table();
        assert_eq!(days_in_year[1999], 365);
        assert_eq!(days_in_year[2000], 366);
    }

    #[test]
    fn test_get_days_in_year() {
        assert_eq!(get_days_in_year(1999), 365);
        assert_eq!(get_days_in_year(2000), 366);
        assert_eq!(get_days_in_year(10001), 365);
        assert_eq!(get_days_in_year(10004), 366);
    }

    #[test]
    fn test_calculate_days_in_century_lookup_table() {
        let days_in_century: [i32; 100] = calculate_days_in_century_lookup_table();
        assert_eq!(days_in_century[1700 / 100], 36524);
        assert_eq!(days_in_century[2000 / 100], 36525);
    }

    #[test]
    fn test_get_days_in_century() {
        assert_eq!(get_days_in_century(1700), 36524);
        assert_eq!(get_days_in_century(2000), 36525);
        assert_eq!(get_days_in_century(20000), 36525);
        assert_eq!(get_days_in_century(21000), 36524);
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
        assert_eq!(get_date_values(364, &test_epoch), (1970, 12, 31));
        assert_eq!(get_date_values(1460, &test_epoch), (1973, 12, 31));
        assert_eq!(get_date_values(-1, &test_epoch), (1969, 12, 31));
        assert_eq!(get_date_values(-1460, &test_epoch), (1966, 1, 2));
        assert_eq!(get_date_values(-62091, &test_epoch), (1800, 1, 1));
    }

    #[test]
    fn test_get_date_values_before_epoch_in_epoch_month() {
        let test_epoch: Epoch = Epoch::new(2000, 1, 9);
        assert_eq!(get_date_values(-5, &test_epoch), (2000, 1, 4));
        assert_eq!(get_date_values(-8, &test_epoch), (2000, 1, 1));
        assert_eq!(get_date_values(-9, &test_epoch), (1999, 12, 31));
        assert_eq!(get_date_values(-10, &test_epoch), (1999, 12, 30));

        let test_epoch: Epoch = Epoch::new(1970, 2, 15);
        assert_eq!(get_date_values(-15, &test_epoch), (1970, 1, 31));
        assert_eq!(get_date_values(-31, &test_epoch), (1970, 1, 15));
        assert_eq!(get_date_values(-46, &test_epoch), (1969, 12, 31));
        assert_eq!(get_date_values(-50, &test_epoch), (1969, 12, 27));
    }

    #[test]
    fn test_get_date_values_before_epoch_across_month_and_year_boundaries() {
        let test_epoch: Epoch = Epoch::new(1970, 1, 1);
        assert_eq!(get_date_values(-25566, &test_epoch), (1900, 1, 2));
        assert_eq!(get_date_values(-25567, &test_epoch), (1900, 1, 1));
        assert_eq!(get_date_values(-25568, &test_epoch), (1899, 12, 31));
        assert_eq!(get_date_values(-25569, &test_epoch), (1899, 12, 30));

        let test_epoch: Epoch = Epoch::new(2000, 2, 1);
        assert_eq!(get_date_values(-31, &test_epoch), (2000, 1, 1));
        assert_eq!(get_date_values(-32, &test_epoch), (1999, 12, 31));
        assert_eq!(get_date_values(-359, &test_epoch), (1999, 2, 7));
    }

    #[test]
    fn test_get_date_values_last_day_of_month_after_non_january_epoch() {
        let test_epoch: Epoch = Epoch::new(1582, 10, 15);
        assert_eq!(get_date_values(16, &test_epoch), (1582, 10, 31));
        assert_eq!(get_date_values(46, &test_epoch), (1582, 11, 30));
        assert_eq!(get_date_values(77, &test_epoch), (1582, 12, 31));
        assert_eq!(get_date_values(78, &test_epoch), (1583, 1, 1));
        assert_eq!(get_date_values(-15, &test_epoch), (1582, 9, 30));
        assert_eq!(get_date_values(-46, &test_epoch), (1582, 8, 30));

        let test_epoch: Epoch = Epoch::new(1904, 1, 1);
        assert_eq!(get_date_values(59, &test_epoch), (1904, 2, 29));
        assert_eq!(get_date_values(424, &test_epoch), (1905, 2, 28));
        assert_eq!(get_date_values(425, &test_epoch), (1905, 3, 1));

        let test_epoch: Epoch = Epoch::new(1901, 12, 13);
        assert_eq!(get_date_values(365, &test_epoch), (1902, 12, 13));
        assert_eq!(get_date_values(807, &test_epoch), (1904, 2, 28));
        assert_eq!(get_date_values(808, &test_epoch), (1904, 2, 29));
    }

    // Number of days between January 1, year 0 and a civil date, calculated with the
    // days-difference algorithm. Used as an oracle for get_date_values.
    fn days_from_civil(year: i64, month: u8, day_of_month: u8) -> i64 {
        let year: i64 = if (month as u8) <= 2 { year - 1 } else { year };
        let era: i64 = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era: i64 = year - (era * 400);
        let month_prime: i64 = (month as i64 + 9) % 12;
        let day_of_year: i64 = ((153 * month_prime) + 2) / 5 + (day_of_month as i64) - 1;
        let day_of_era: i64 =
            (year_of_era * 365) + (year_of_era / 4) - (year_of_era / 100) + day_of_year;

        (era * 146097) + day_of_era
    }

    #[test]
    fn test_get_date_values_against_oracle() {
        let test_epochs: [Epoch; 4] = [
            Epoch::new(1970, 1, 1),
            Epoch::new(1582, 10, 15),
            Epoch::new(1899, 12, 30),
            Epoch::new(1901, 12, 13),
        ];

        for test_epoch in test_epochs.iter() {
            let epoch_days: i64 = days_from_civil(
                test_epoch.year as i64,
                test_epoch.month,
                test_epoch.day_of_month,
            );

            // Dense coverage around the epoch, covering month, year and century boundaries.
            for number_of_days in -500..501 {
                let (year, month, day_of_month): (i32, u8, u8) =
                    get_date_values(number_of_days, test_epoch);
                let date_days: i64 = days_from_civil(year as i64, month, day_of_month);
                assert_eq!(
                    date_days - epoch_days,
                    number_of_days,
                    "epoch: {}-{:02}-{:02}",
                    test_epoch.year,
                    test_epoch.month,
                    test_epoch.day_of_month
                );
            }
            // Sparse coverage across multiple centuries in both directions.
            for number_of_days in -400000..400001 {
                if number_of_days % 251 != 0 {
                    continue;
                }
                let (year, month, day_of_month): (i32, u8, u8) =
                    get_date_values(number_of_days, test_epoch);
                let date_days: i64 = days_from_civil(year as i64, month, day_of_month);
                assert_eq!(
                    date_days - epoch_days,
                    number_of_days,
                    "epoch: {}-{:02}-{:02}",
                    test_epoch.year,
                    test_epoch.month,
                    test_epoch.day_of_month
                );
            }
        }
    }

    #[test]
    fn test_get_date_values_400_year_periodicity() {
        let test_epochs: [Epoch; 6] = [
            Epoch::new(1970, 1, 1),
            Epoch::new(1582, 10, 15),
            Epoch::new(1899, 12, 30),
            Epoch::new(1901, 12, 13),
            Epoch::new(2000, 1, 9),
            Epoch::new(2000, 2, 1),
        ];
        let test_offsets: [i64; 28] = [
            0, 1, 2, 3, 7, 15, 30, 31, 46, 47, 60, 77, 78, 100, 364, 365, 366, 367, 730, 731,
            36523, 36524, 36525, 36526, 109572, 109573, 146096, 146097,
        ];

        for test_epoch in test_epochs.iter() {
            for offset in test_offsets.iter() {
                for number_of_days in [*offset, -*offset].iter() {
                    let (year, month, day_of_month): (i32, u8, u8) =
                        get_date_values(*number_of_days, test_epoch);
                    let (year_400_years, month_400_years, day_of_month_400_years): (i32, u8, u8) =
                        get_date_values(*number_of_days + 146097, test_epoch);

                    // A Gregorian calendar 400 year cycle contains exactly 146097 days, such that
                    // the date 146097 days after the original date has the same month and day
                    // of month, but is 400 years later.
                    assert_eq!(
                        year_400_years - year,
                        400,
                        "epoch: {}-{:02}-{:02}, offset: {}",
                        test_epoch.year,
                        test_epoch.month,
                        test_epoch.day_of_month,
                        number_of_days
                    );
                    assert_eq!(month_400_years, month, "offset: {}", number_of_days);
                    assert_eq!(
                        day_of_month_400_years, day_of_month,
                        "offset: {}",
                        number_of_days
                    );
                }
            }
        }
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
