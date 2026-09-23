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

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::cpio::{CpioArchive, CpioFormat};

/// Information about a Copy in and out (CPIO) archive.
struct CpioArchiveInfo<'a> {
    /// Container.
    archive: &'a CpioArchive,
}

impl<'a> CpioArchiveInfo<'a> {
    const FORMATS: &'static [(CpioFormat, &'static str); 4] = &[
        (CpioFormat::BinaryBigEndian, "binary big-endian"),
        (CpioFormat::BinaryLittleEndian, "binary little-endian"),
        (CpioFormat::NewAscii, "new ASCII (newc)"),
        (CpioFormat::PortableAscii, "portable ASCII (odc)"),
    ];

    /// Creates new archive information.
    fn new(archive: &'a CpioArchive) -> Self {
        Self { archive }
    }

    /// Retrieves the format as a string.
    pub fn get_format_string(&self, format: &CpioFormat) -> &str {
        Self::FORMATS
            .binary_search_by(|(key, _)| key.cmp(format))
            .map_or_else(|_| "Unknown", |index| Self::FORMATS[index].1)
    }
}

impl<'a> fmt::Display for CpioArchiveInfo<'a> {
    /// Formats archive information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Copy in and out (CPIO) archive information:")?;

        let format_string: &str = self.get_format_string(self.archive.get_format());
        writeln!(formatter, "    Format\t\t\t\t\t: {}", format_string)?;

        writeln!(formatter)
    }
}

/// Information about Copy in and out (CPIO) format.
pub struct CpioInfo {}

impl CpioInfo {
    /// Opens an archive.
    fn open_archive(data_stream: &DataStreamReference) -> Result<CpioArchive, ErrorTrace> {
        let mut cpio_archive: CpioArchive = CpioArchive::new();

        match cpio_archive.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open archive");
                return Err(error);
            }
        }
        Ok(cpio_archive)
    }

    /// Prints information about an archive.
    pub fn print_archive(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let cpio_archive: CpioArchive = match Self::open_archive(data_stream) {
            Ok(cpio_archive) => cpio_archive,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open archive");
                return Err(error);
            }
        };
        let archive_information: CpioArchiveInfo = CpioArchiveInfo::new(&cpio_archive);

        print!("{}", archive_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::assert_lines_eq;

    fn get_archive() -> Result<CpioArchive, ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/cpio/bin_le.cpio");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        CpioInfo::open_archive(&data_stream)
    }

    #[test]
    fn test_archive_information_fmt() -> Result<(), ErrorTrace> {
        let cpio_archive: CpioArchive = get_archive()?;

        let test_struct: CpioArchiveInfo = CpioArchiveInfo::new(&cpio_archive);

        let expected_string: &str = concat!(
            "Copy in and out (CPIO) archive information:\n",
            "    Format\t\t\t\t\t: binary little-endian\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_archive
    // TODO: add tests for print_archive
}
