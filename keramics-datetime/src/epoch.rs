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

/// A fixed date and time used as a reference (epoch).
pub struct Epoch {
    /// The year.
    pub year: i16,

    /// The month.
    pub month: u8,

    /// The day of month.
    pub day_of_month: u8,
}

impl Epoch {
    pub fn new(year: i16, month: u8, day_of_month: u8) -> Self {
        Self {
            year: year,
            month: month,
            day_of_month: day_of_month,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let test_struct: Epoch = Epoch::new(1970, 1, 1);
        assert_eq!(test_struct.year, 1970);
        assert_eq!(test_struct.month, 1);
        assert_eq!(test_struct.day_of_month, 1);
    }
}
