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
use keramics_formats::cdsaencr::{CdsaEncrCredential, CdsaEncrEncryptionType};
use keramics_formats::udif::{UdifCompressionMethod, UdifImage};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::Uuid;
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

use super::constants::*;

/// Information about an Universal Disk Image Format (UDIF) image.
struct UdifImageInfo {
    /// Segment set identifier.
    pub segment_set_identifier: Uuid,

    /// Number of segments.
    pub number_of_segments: u32,

    /// Compression method.
    pub compression_method: UdifCompressionMethod,

    /// Encryption type.
    pub encryption_type: Option<CdsaEncrEncryptionType>,

    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,
}

impl UdifImageInfo {
    const COMPRESSION_METHODS: &[(UdifCompressionMethod, &'static str); 6] = &[
        (UdifCompressionMethod::Adc, "ADC"),
        (UdifCompressionMethod::Bzip2, "bzip2"),
        (UdifCompressionMethod::Lzfse, "LZFSE/LZVN"),
        (UdifCompressionMethod::Lzma, "LZMA"),
        (UdifCompressionMethod::None, "Uncompressed"),
        (UdifCompressionMethod::Zlib, "zlib"),
    ];

    /// Creates new image information.
    fn new() -> Self {
        Self {
            segment_set_identifier: Uuid::new(),
            number_of_segments: 0,
            compression_method: UdifCompressionMethod::None,
            encryption_type: None,
            media_size: 0,
            bytes_per_sector: 0,
        }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(&self) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(&self.compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }
}

impl fmt::Display for UdifImageInfo {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Universal Disk Image Format (UDIF) information:")?;

        let value_string: String;
        let segment_set_identifier_string: &str = if self.segment_set_identifier.is_nil() {
            NOT_SET_VALUE
        } else {
            value_string = format!("{}", self.segment_set_identifier);
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
            self.number_of_segments
        )?;
        // TODO: print (segment) set identifier

        if let Some(encryption_type) = &self.encryption_type {
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
        writeln!(formatter, "    Compression information:")?;

        let compression_method_string: &str = self.get_compression_method_string();
        writeln!(
            formatter,
            "        Compression method\t\t\t: {}",
            compression_method_string
        )?;
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.bytes_per_sector
        )?;
        writeln!(formatter)
    }
}

/// Information about an Universal Disk Image Format (UDIF) image.
pub struct UdifInfo {}

impl UdifInfo {
    /// Retrieves the image information.
    fn get_image_information(udif_image: &UdifImage) -> UdifImageInfo {
        let mut image_information: UdifImageInfo = UdifImageInfo::new();

        image_information.segment_set_identifier = udif_image.get_segment_set_identifier().clone();
        image_information.number_of_segments = udif_image.get_number_of_segments();
        image_information.compression_method = udif_image.get_compression_method().clone();
        image_information.encryption_type = udif_image.get_encryption_type().cloned();
        image_information.media_size = udif_image.get_media_size();
        image_information.bytes_per_sector = udif_image.get_bytes_per_sector();

        image_information
    }

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
        let image_information: UdifImageInfo = Self::get_image_information(&udif_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::info::tests::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let udif_image: UdifImage = UdifInfo::open_image(&path_buf)?;
        let test_struct: UdifImageInfo = UdifInfo::get_image_information(&udif_image);

        let expected_string: &str = concat!(
            "Universal Disk Image Format (UDIF) information:\n",
            "    Segment set identifier\t\t\t: N/A (not set)\n",
            "    Number of segments\t\t\t\t: 1\n",
            "    Compression information:\n",
            "        Compression method\t\t\t: zlib\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 1.9 MiB (1964032 bytes)\n",
            "        Bytes per sector\t\t\t: 512\n",
            "\n"
        );
        assert_lines_eq(test_struct.to_string().as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_get_image_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_zlib.dmg");
        let udif_image: UdifImage = UdifInfo::open_image(&path_buf)?;
        let test_struct: UdifImageInfo = UdifInfo::get_image_information(&udif_image);

        assert_eq!(
            test_struct.segment_set_identifier.to_string(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(test_struct.number_of_segments, 1);
        assert_eq!(test_struct.compression_method, UdifCompressionMethod::Zlib);
        assert_eq!(test_struct.encryption_type, None);
        assert_eq!(test_struct.media_size, 1964032);
        assert_eq!(test_struct.bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
