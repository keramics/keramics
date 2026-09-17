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

use keramics_datetime::DateTime;

use super::constants::*;

/// Windows file attribute flags information.
pub struct WindowsFileAttributeFlagsInfo {
    /// Flags.
    flags: u16,
}

impl WindowsFileAttributeFlagsInfo {
    /// Creates new file attribute flags information.
    pub fn new(flags: u16) -> Self {
        Self { flags }
    }
}

impl fmt::Display for WindowsFileAttributeFlagsInfo {
    /// Formats file attribute flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0001 != 0 {
            writeln!(
                formatter,
                "        0x0001: Is read-only (FILE_ATTRIBUTE_READ_ONLY)"
            )?;
        }
        if self.flags & 0x0002 != 0 {
            writeln!(
                formatter,
                "        0x0002: Is hidden (FILE_ATTRIBUTE_HIDDEN)"
            )?;
        }
        if self.flags & 0x0004 != 0 {
            writeln!(
                formatter,
                "        0x0004: Is system (FILE_ATTRIBUTE_SYSTEM)"
            )?;
        }

        if self.flags & 0x0010 != 0 {
            writeln!(
                formatter,
                "        0x0010: Is directory (FILE_ATTRIBUTE_DIRECTORY)"
            )?;
        }
        if self.flags & 0x0020 != 0 {
            writeln!(
                formatter,
                "        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)"
            )?;
        }
        if self.flags & 0x0040 != 0 {
            writeln!(
                formatter,
                "        0x0040: Is device (FILE_ATTRIBUTE_DEVICE)"
            )?;
        }
        if self.flags & 0x0080 != 0 {
            writeln!(
                formatter,
                "        0x0080: Is normal (FILE_ATTRIBUTE_NORMAL)"
            )?;
        }
        Ok(())
    }
}

/// Filetime date and time information.
pub struct FiletimeDateTimeInfo<'a> {
    /// Flags.
    date_time: &'a DateTime,
}

impl<'a> FiletimeDateTimeInfo<'a> {
    /// Creates new date and time information.
    pub fn new(date_time: &'a DateTime) -> Self {
        Self { date_time }
    }
}

impl<'a> fmt::Display for FiletimeDateTimeInfo<'a> {
    /// Formats date and time information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.date_time {
            DateTime::Filetime(filetime) => {
                write!(formatter, "{}+00:00", filetime.to_iso8601_string())
            }
            DateTime::NotSet => write!(formatter, "{}", NOT_SET_VALUE),
            _ => write!(formatter, "Unsupported date time"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_datetime::Filetime;

    use crate::assert_lines_eq;

    #[test]
    fn test_file_attribute_flags_information_fmt() {
        let test_struct: WindowsFileAttributeFlagsInfo = WindowsFileAttributeFlagsInfo::new(0x0020);

        let expected_string: &str = "        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)\n";

        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);
    }

    #[test]
    fn test_filetime_date_time_information_fmt() {
        let date_time: DateTime = DateTime::Filetime(Filetime::new(0x01cb3a623d0a17ce));
        let test_struct: FiletimeDateTimeInfo = FiletimeDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, "2010-08-12T21:06:31.5468750+00:00");

        let date_time: DateTime = DateTime::NotSet;
        let test_struct: FiletimeDateTimeInfo = FiletimeDateTimeInfo::new(&date_time);
        let string: String = test_struct.to_string();
        assert_eq!(string, NOT_SET_VALUE);
    }
}
