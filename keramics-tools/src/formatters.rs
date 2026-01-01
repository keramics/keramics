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

pub struct ByteSize {
    /// The value.
    pub value: u64,

    /// The base.
    pub base: u64,
}

impl ByteSize {
    const UNITS: [&'static str; 9] = ["", "K", "M", "G", "T", "P", "E", "Z", "Y"];

    /// Creates a new byte size.
    pub fn new(value: u64, base: u64) -> Self {
        Self { value, base }
    }

    /// Retrieves a human readable byte size.
    fn get_human_readable(&self) -> String {
        let mut factor: u64 = 1;
        let mut next_factor: u64 = self.base;
        let mut units_index: usize = 0;

        while next_factor <= self.value {
            factor = next_factor;
            next_factor *= self.base;
            units_index += 1;
        }
        if units_index == 0 {
            format!("{} B", self.value)
        } else {
            let float_value: f64 = (self.value as f64) / (factor as f64);

            let mut base_string: &'static str = "B";

            if self.base == 1024 {
                base_string = "iB";
            }
            format!(
                "{:.1} {}{}",
                float_value,
                Self::UNITS[units_index],
                base_string
            )
        }
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value < 1024 {
            write!(formatter, "{} bytes", self.value)
        } else {
            let human_readable: String = self.get_human_readable();

            write!(formatter, "{} ({} bytes)", human_readable, self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_human_readable() {
        let byte_size: ByteSize = ByteSize::new(512, 1024);
        let string: String = byte_size.get_human_readable();
        assert_eq!(string, "512 B");

        let byte_size: ByteSize = ByteSize::new(1024, 1024);
        let string: String = byte_size.get_human_readable();
        assert_eq!(string, "1.0 KiB");

        let byte_size: ByteSize = ByteSize::new(2097152, 1024);
        let string: String = byte_size.get_human_readable();
        assert_eq!(string, "2.0 MiB");

        let byte_size: ByteSize = ByteSize::new(2097152, 1000);
        let string: String = byte_size.get_human_readable();
        assert_eq!(string, "2.1 MB");

        let byte_size: ByteSize = ByteSize::new(3221225472, 1024);
        let string: String = byte_size.get_human_readable();
        assert_eq!(string, "3.0 GiB");
    }

    #[test]
    fn test_to_string() {
        let byte_size: ByteSize = ByteSize::new(512, 1024);
        let string: String = byte_size.to_string();
        assert_eq!(string, "512 bytes");

        let byte_size: ByteSize = ByteSize::new(1024, 1024);
        let string: String = byte_size.to_string();
        assert_eq!(string, "1.0 KiB (1024 bytes)");

        let byte_size: ByteSize = ByteSize::new(2097152, 1024);
        let string: String = byte_size.to_string();
        assert_eq!(string, "2.0 MiB (2097152 bytes)");

        let byte_size: ByteSize = ByteSize::new(2097152, 1000);
        let string: String = byte_size.to_string();
        assert_eq!(string, "2.1 MB (2097152 bytes)");

        let byte_size: ByteSize = ByteSize::new(3221225472, 1024);
        let string: String = byte_size.to_string();
        assert_eq!(string, "3.0 GiB (3221225472 bytes)");
    }
}
