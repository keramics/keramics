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
use keramics_formats::cdsaencr::CdsaEncrCredential;
use keramics_formats::udif::{UdifCompressionMethod, UdifImage};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::Uuid;
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

use super::constants::*;

/// Information about an Universal Disk Image Format (UDIF) image.
struct UdifImageInfo<'a> {
    /// Image.
    image: &'a UdifImage,
}

impl<'a> UdifImageInfo<'a> {
    const COMPRESSION_METHODS: &'static [(UdifCompressionMethod, &'static str); 6] = &[
        (UdifCompressionMethod::Adc, "ADC"),
        (UdifCompressionMethod::Bzip2, "bzip2"),
        (UdifCompressionMethod::Lzfse, "LZFSE/LZVN"),
        (UdifCompressionMethod::Lzma, "LZMA"),
        (UdifCompressionMethod::None, "Uncompressed"),
        (UdifCompressionMethod::Zlib, "zlib"),
    ];

    /// Creates new image information.
    fn new(image: &'a UdifImage) -> Self {
        Self { image }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(compression_method: &UdifCompressionMethod) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }
}

impl<'a> fmt::Display for UdifImageInfo<'a> {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Universal Disk Image Format (UDIF) information:")?;

        let segment_set_identifier: &Uuid = self.image.get_segment_set_identifier();

        let value_string: String;
        let segment_set_identifier_string: &str = if segment_set_identifier.is_nil() {
            NOT_SET_VALUE
        } else {
            value_string = format!("{}", segment_set_identifier);
            value_string.as_str()
        };
        writeln!(
            formatter,
            "    Segment set identifier\t\t\t: {}",
            segment_set_identifier_string
        )?;
        writeln!(
            formatter,
            "    Number of segments\t\t\t\t: {}",
            self.image.get_number_of_segments()
        )?;
        // TODO: print (segment) set identifier

        if let Some(encryption_type) = self.image.get_encryption_type() {
            writeln!(formatter, "    Encryption information:")?;
            writeln!(
                formatter,
                "        Encryption method\t\t\t: {}",
                encryption_type
            )?;
            // TODO: print human readable encryption method
            // TODO: print key protectors
            // TODO: print identifier
        }
        writeln!(formatter)?;

        writeln!(formatter, "    Compression information:")?;

        let compression_method_string: &str =
            Self::get_compression_method_string(self.image.get_compression_method());
        writeln!(
            formatter,
            "        Compression method\t\t\t: {}",
            compression_method_string
        )?;
        writeln!(formatter)?;

        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.image.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.image.get_bytes_per_sector()
        )?;
        writeln!(formatter)
    }
}

/// Information about an Universal Disk Image Format (UDIF) image.
pub struct UdifInfo {}

impl UdifInfo {
    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<UdifImage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut udif_image: UdifImage = UdifImage::new();

        let file_name: PathComponent = match path_buf.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Unsupported file name"));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!("Missing file name"));
            }
        };
        match udif_image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open UDIF image");
                return Err(error);
            }
        }
        Ok(udif_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        // TODO: fallback to file if image open fails

        let mut udif_image: UdifImage = match Self::open_image(path_buf) {
            Ok(udif_image) => udif_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        if udif_image.is_locked() {
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<CdsaEncrCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(CdsaEncrCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match udif_image.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock image");
                    return Err(error);
                }
            }
        }
        let image_information: UdifImageInfo = UdifImageInfo::new(&udif_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let udif_image: UdifImage = UdifInfo::open_image(&path_buf)?;
        let test_struct: UdifImageInfo = UdifImageInfo::new(&udif_image);

        let expected_string: &str = concat!(
            "Universal Disk Image Format (UDIF) information:\n",
            "    Segment set identifier\t\t\t: Not set (0)\n",
            "    Number of segments\t\t\t\t: 1\n",
            "\n",
            "    Compression information:\n",
            "        Compression method\t\t\t: zlib\n",
            "\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 1.9 MiB (1964032 bytes)\n",
            "        Bytes per sector\t\t\t: 512\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
