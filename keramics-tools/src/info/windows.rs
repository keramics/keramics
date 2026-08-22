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
    /// Formats partition file attribute flags information for display.
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::assert_lines_eq;

    #[test]
    fn test_file_attribute_flags_information_fmt() {
        let test_struct: WindowsFileAttributeFlagsInfo = WindowsFileAttributeFlagsInfo::new(0x0020);

        let expected_string: &str =
            concat!("        0x0020: Should be archived (FILE_ATTRIBUTE_ARCHIVE)\n",);
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);
    }
}
