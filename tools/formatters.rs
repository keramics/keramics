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

const UNITS: [&str; 9] = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];

/// Formats an integer as bytes size.
pub fn format_as_bytesize(value: u64, base: u64) -> String {
    let mut factor: u64 = 1;
    let mut next_factor: u64 = base;
    let mut units_index: usize = 0;

    while next_factor <= value {
        factor = next_factor;
        next_factor *= base;
        units_index += 1;
    }
    if units_index > 0 {
        let float_value: f64 = value as f64 / factor as f64;
        let mut base_string: &str = "B";
        if base == 1024 {
            base_string = "iB";
        }
        return format!("{:.1} {}{}", float_value, UNITS[units_index], base_string);
    }
    return format!("{} B", value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_as_bytesize() {
        let string: String = format_as_bytesize(512, 1024);
        assert_eq!(string, "512 B");

        let string: String = format_as_bytesize(1024, 1024);
        assert_eq!(string, "1.0 KiB");

        let string: String = format_as_bytesize(2097152, 1024);
        assert_eq!(string, "2.0 MiB");

        let string: String = format_as_bytesize(2097152, 1000);
        assert_eq!(string, "2.1 MB");

        let string: String = format_as_bytesize(3221225472, 1024);
        assert_eq!(string, "3.0 GiB");
    }
}
