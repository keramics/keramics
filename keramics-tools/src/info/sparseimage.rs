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
use keramics_formats::sparseimage::SparseImageFile;

use crate::formatters::ByteSize;

/// Information about a Mac OS sparse image (.sparseimage) file.
struct SparseImageFileInfo {
    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Block size.
    pub block_size: u32,
}

impl SparseImageFileInfo {
    /// Creates new file information.
    fn new() -> Self {
        Self {
            media_size: 0,
            bytes_per_sector: 0,
            block_size: 0,
        }
    }
}

impl fmt::Display for SparseImageFileInfo {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Sparse image (.sparseimage) information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);

        writeln!(formatter, "    Media size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;
        if self.block_size < 1024 {
            writeln!(
                formatter,
                "    Band size\t\t\t\t\t: {} bytes",
                self.block_size,
            )?;
        } else {
            let byte_size: ByteSize = ByteSize::new(self.block_size as u64, 1024);

            writeln!(formatter, "    Band size\t\t\t\t\t: {}", byte_size)?;
        }
        writeln!(formatter)
    }
}

/// Information about a Mac OS sparse image (.sparseimage) file.
pub struct SparseImageInfo {}

impl SparseImageInfo {
    /// Retrieves the file information.
    fn get_file_information(sparseimage_file: &SparseImageFile) -> SparseImageFileInfo {
        let mut file_information: SparseImageFileInfo = SparseImageFileInfo::new();

        file_information.media_size = sparseimage_file.media_size;
        file_information.bytes_per_sector = sparseimage_file.bytes_per_sector;
        file_information.block_size = sparseimage_file.block_size;

        file_information
    }

    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<SparseImageFile, ErrorTrace> {
        let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

        match sparseimage_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open sparseimage file");
                return Err(error);
            }
        };
        Ok(sparseimage_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let sparseimage_file: SparseImageFile = match Self::open_file(data_stream) {
            Ok(sparseimage_file) => sparseimage_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: SparseImageFileInfo = Self::get_file_information(&sparseimage_file);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus.sparseimage");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let sparseimage_file: SparseImageFile = SparseImageInfo::open_file(&data_stream)?;
        let test_struct: SparseImageFileInfo =
            SparseImageInfo::get_file_information(&sparseimage_file);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Sparse image (.sparseimage) information:\n",
            "    Media size\t\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "    Bytes per sector\t\t\t\t: 512 bytes\n",
            "    Band size\t\t\t\t\t: 1.0 MiB (1048576 bytes)\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus.sparseimage");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let sparseimage_file: SparseImageFile = SparseImageInfo::open_file(&data_stream)?;
        let test_struct: SparseImageFileInfo =
            SparseImageInfo::get_file_information(&sparseimage_file);

        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(test_struct.block_size, 1048576);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
