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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::{FileResolverReference, open_os_file_resolver};

use crate::formatters::ByteSize;

/// Information about a Mac OS sparse bundle (.sparsebundle) directory.
struct SparseBundleImageInfo {
    /// Block size.
    pub block_size: u32,

    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,
}

impl SparseBundleImageInfo {
    /// Creates new image information.
    fn new() -> Self {
        Self {
            block_size: 0,
            media_size: 0,
            bytes_per_sector: 0,
        }
    }
}

impl fmt::Display for SparseBundleImageInfo {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Sparse bundle (.sparsebundle) information:")?;

        let byte_size: ByteSize = ByteSize::new(self.block_size as u64, 1024);
        writeln!(formatter, "    Band size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;
        writeln!(formatter)
    }
}

/// Information about a Mac OS sparse bundle (.sparsebundle) image.
pub struct SparseBundleInfo {}

impl SparseBundleInfo {
    /// Retrieves the image information.
    fn get_image_information(sparsebundle_image: &SparseBundleImage) -> SparseBundleImageInfo {
        let mut image_information: SparseBundleImageInfo = SparseBundleImageInfo::new();

        image_information.block_size = sparsebundle_image.block_size;
        image_information.media_size = sparsebundle_image.media_size;
        image_information.bytes_per_sector = sparsebundle_image.bytes_per_sector;

        image_information
    }

    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<SparseBundleImage, ErrorTrace> {
        let file_resolver: FileResolverReference = match open_os_file_resolver(path_buf) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut sparsebundle_image: SparseBundleImage = SparseBundleImage::new();

        match sparsebundle_image.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open sparsebundle image");
                return Err(error);
            }
        }
        Ok(sparsebundle_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let sparsebundle_image: SparseBundleImage = match Self::open_image(path_buf) {
            Ok(sparsebundle_image) => sparsebundle_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        let image_information: SparseBundleImageInfo =
            Self::get_image_information(&sparsebundle_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::info::tests::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/sparsebundle/hfsplus.sparsebundle");
        let sparsebundle_image: SparseBundleImage = SparseBundleInfo::open_image(&path_buf)?;
        let test_struct: SparseBundleImageInfo =
            SparseBundleInfo::get_image_information(&sparsebundle_image);

        let expected_string: &str = concat!(
            "Sparse bundle (.sparsebundle) information:\n",
            "    Band size\t\t\t\t\t: 8.0 MiB (8388608 bytes)\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512 bytes\n",
            "\n"
        );
        assert_lines_eq(test_struct.to_string().as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_get_image_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/sparsebundle/hfsplus.sparsebundle");
        let sparsebundle_image: SparseBundleImage = SparseBundleInfo::open_image(&path_buf)?;
        let test_struct: SparseBundleImageInfo =
            SparseBundleInfo::get_image_information(&sparsebundle_image);

        assert_eq!(test_struct.block_size, 8388608);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
