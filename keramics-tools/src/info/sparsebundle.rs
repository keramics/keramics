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
use keramics_formats::sparsebundle::SparseBundleImage;
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

/// Information about a Mac OS sparse bundle (.sparsebundle) image.
struct SparseBundleImageInfo<'a> {
    /// Image.
    image: &'a SparseBundleImage,
}

impl<'a> SparseBundleImageInfo<'a> {
    /// Creates new image information.
    fn new(image: &'a SparseBundleImage) -> Self {
        Self { image }
    }
}

impl<'a> fmt::Display for SparseBundleImageInfo<'a> {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Sparse bundle (.sparsebundle) information:")?;

        let byte_size: ByteSize = ByteSize::new(self.image.get_block_size() as u64, 1024);
        writeln!(formatter, "    Band size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)?;

        if let Some(encryption_type) = self.image.get_encryption_type() {
            writeln!(formatter, "    Encryption information:")?;
            writeln!(
                formatter,
                "        Encryption method\t\t\t: {}",
                encryption_type
            )?;
            // TODO: print key protectors
            // TODO: print identifier

            if self.image.is_locked() {
                writeln!(formatter, "        Is locked")?;
            }
            writeln!(formatter)?;
        }
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

/// Information about a Mac OS sparse bundle (.sparsebundle) image.
pub struct SparseBundleInfo {}

impl SparseBundleInfo {
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

        let file_name: PathComponent = PathComponent::from("Info.plist");
        match sparsebundle_image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        }
        if sparsebundle_image.is_locked() {
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
            match sparsebundle_image.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock image");
                    return Err(error);
                }
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
            SparseBundleImageInfo::new(&sparsebundle_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/sparsebundle/hfsplus.sparsebundle");
        let sparsebundle_image: SparseBundleImage = SparseBundleInfo::open_image(&path_buf)?;
        let test_struct: SparseBundleImageInfo = SparseBundleImageInfo::new(&sparsebundle_image);

        let expected_string: &str = concat!(
            "Sparse bundle (.sparsebundle) information:\n",
            "    Band size\t\t\t\t\t: 8.0 MiB (8388608 bytes)\n",
            "\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
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
