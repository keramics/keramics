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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_formats::pdi::PdiImage;
use keramics_formats::{FileResolverReference, open_os_file_resolver};

use crate::formatters::ByteSize;

/// Information about a Parallels Disk Image (PDI) image.
struct PdiImageInfo {
    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,
}

impl PdiImageInfo {
    /// Creates new image information.
    fn new() -> Self {
        Self {
            media_size: 0,
            bytes_per_sector: 0,
        }
    }
}

impl fmt::Display for PdiImageInfo {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Parallels Disk Image (PDI) information:")?;

        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;

        // TODO: print additional information

        writeln!(formatter)
    }
}

/// Information about a Parallels Disk Image (PDI) image.
pub struct PdiInfo {}

impl PdiInfo {
    /// Retrieves the image information.
    fn get_image_information(pdi_image: &PdiImage) -> PdiImageInfo {
        let mut image_information: PdiImageInfo = PdiImageInfo::new();

        image_information.media_size = pdi_image.media_size;
        image_information.bytes_per_sector = pdi_image.bytes_per_sector;

        image_information
    }

    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<PdiImage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut pdi_image: PdiImage = PdiImage::new();

        match pdi_image.open(&file_resolver) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VMDK image");
                return Err(error);
            }
        }
        Ok(pdi_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let pdi_image: PdiImage = match Self::open_image(path_buf) {
            Ok(pdi_image) => pdi_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        let image_information: PdiImageInfo = Self::get_image_information(&pdi_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/pdi/hfsplus.hdd/DiskDescriptor.xml");
        let pdi_image: PdiImage = PdiInfo::open_image(&path_buf)?;
        let test_struct: PdiImageInfo = PdiInfo::get_image_information(&pdi_image);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Parallels Disk Image (PDI) information:\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 32.0 MiB (33554432 bytes)\n",
            "        Bytes per sector\t\t\t: 512 bytes\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_image_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/pdi/hfsplus.hdd/DiskDescriptor.xml");
        let pdi_image: PdiImage = PdiInfo::open_image(&path_buf)?;
        let test_struct: PdiImageInfo = PdiInfo::get_image_information(&pdi_image);

        assert_eq!(test_struct.media_size, 33554432);
        assert_eq!(test_struct.bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
