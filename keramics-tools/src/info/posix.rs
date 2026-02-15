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

/// POSIX file mode information.
pub struct PosixFileModeInfo {
    /// Flags.
    file_mode: u16,
}

impl PosixFileModeInfo {
    /// Creates new file mode information.
    pub fn new(file_mode: u16) -> Self {
        Self { file_mode }
    }

    /// Retrieves a file mode string representation.
    fn get_file_mode_string(file_mode: u16) -> String {
        let mut string_parts: Vec<&str> = vec!["-"; 10];

        if file_mode & 0x0001 != 0 {
            string_parts[9] = "x";
        }
        if file_mode & 0x0002 != 0 {
            string_parts[8] = "w";
        }
        if file_mode & 0x0004 != 0 {
            string_parts[7] = "r";
        }
        if file_mode & 0x0008 != 0 {
            string_parts[6] = "x";
        }
        if file_mode & 0x0010 != 0 {
            string_parts[5] = "w";
        }
        if file_mode & 0x0020 != 0 {
            string_parts[4] = "r";
        }
        if file_mode & 0x0040 != 0 {
            string_parts[3] = "x";
        }
        if file_mode & 0x0080 != 0 {
            string_parts[2] = "w";
        }
        if file_mode & 0x0100 != 0 {
            string_parts[1] = "r";
        }
        string_parts[0] = match file_mode & 0xf000 {
            0x1000 => "p",
            0x2000 => "c",
            0x4000 => "d",
            0x6000 => "b",
            0xa000 => "l",
            0xc000 => "s",
            0xe000 => "w",
            _ => "-",
        };
        string_parts.join("")
    }
}

impl fmt::Display for PosixFileModeInfo {
    /// Formats partition file mode information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let string: String = Self::get_file_mode_string(self.file_mode);

        write!(formatter, "{} (0o{:0o})", string, self.file_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_mode_string() {
        let string: String = PosixFileModeInfo::get_file_mode_string(0x1000);
        assert_eq!(string, "p---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0x2000);
        assert_eq!(string, "c---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0x4000);
        assert_eq!(string, "d---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0x6000);
        assert_eq!(string, "b---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0xa000);
        assert_eq!(string, "l---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0xc000);
        assert_eq!(string, "s---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0xe000);
        assert_eq!(string, "w---------");

        let string: String = PosixFileModeInfo::get_file_mode_string(0x81ff);
        assert_eq!(string, "-rwxrwxrwx");
    }

    #[test]
    fn test_file_mode_information_fmt() {
        let test_struct: PosixFileModeInfo = PosixFileModeInfo::new(0x81a4);

        let string: String = test_struct.to_string();
        assert_eq!(string, "-rw-r--r-- (0o100644)");
    }
}
