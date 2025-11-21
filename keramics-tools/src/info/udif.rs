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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::udif::{UdifCompressionMethod, UdifFile};

use crate::formatters::ByteSize;

/// Information about an Universal Disk Image Format (UDIF) file.
struct UdifFileInfo {
    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Compression method.
    pub compression_method: UdifCompressionMethod,
}

impl UdifFileInfo {
    const COMPRESSION_METHODS: &[(UdifCompressionMethod, &'static str); 6] = &[
        (UdifCompressionMethod::Adc, "ADC"),
        (UdifCompressionMethod::Bzip2, "bzip2"),
        (UdifCompressionMethod::Lzfse, "LZFSE/LZVN"),
        (UdifCompressionMethod::Lzma, "LZMA"),
        (UdifCompressionMethod::None, "Uncompressed"),
        (UdifCompressionMethod::Zlib, "zlib"),
    ];

    /// Creates new file information.
    fn new() -> Self {
        Self {
            media_size: 0,
            bytes_per_sector: 0,
            compression_method: UdifCompressionMethod::None,
        }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(&self) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(&self.compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }
}

impl fmt::Display for UdifFileInfo {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Universal Disk Image Format (UDIF) information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "    Media size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;
        let compression_method_string: &str = self.get_compression_method_string();
        writeln!(
            formatter,
            "    Compression method\t\t\t\t: {}",
            compression_method_string
        )?;
        writeln!(formatter)
    }
}

/// Information about an Universal Disk Image Format (UDIF) file.
pub struct UdifInfo {}

impl UdifInfo {
    /// Retrieves the file information.
    fn get_file_information(udif_file: &UdifFile) -> UdifFileInfo {
        let mut file_information: UdifFileInfo = UdifFileInfo::new();

        file_information.media_size = udif_file.media_size;
        file_information.bytes_per_sector = udif_file.bytes_per_sector;
        file_information.compression_method = udif_file.compression_method.clone();

        file_information
    }

    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<UdifFile, ErrorTrace> {
        let mut udif_file: UdifFile = UdifFile::new();

        match udif_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open UDIF file");
                return Err(error);
            }
        }
        Ok(udif_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let udif_file: UdifFile = match Self::open_file(data_stream) {
            Ok(udif_file) => udif_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: UdifFileInfo = Self::get_file_information(&udif_file);

        print!("{}", file_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    #[test]
    fn test_file_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let udif_file: UdifFile = UdifInfo::open_file(&data_stream)?;
        let test_struct: UdifFileInfo = UdifInfo::get_file_information(&udif_file);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Universal Disk Image Format (UDIF) information:\n",
            "    Media size\t\t\t\t\t: 1.9 MiB (1964032 bytes)\n",
            "    Bytes per sector\t\t\t\t: 512 bytes\n",
            "    Compression method\t\t\t\t: zlib\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let udif_file: UdifFile = UdifInfo::open_file(&data_stream)?;
        let test_struct: UdifFileInfo = UdifInfo::get_file_information(&udif_file);

        assert_eq!(test_struct.media_size, 1964032);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.compression_method, UdifCompressionMethod::Zlib);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
