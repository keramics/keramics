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
use keramics_formats::cdsaencr::CdsaEncrCredential;
use keramics_formats::sparseimage::SparseImageFile;
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

/// Information about a Mac OS sparse image (.sparseimage) file.
struct SparseImageFileInfo<'a> {
    /// File.
    file: &'a SparseImageFile,
}

impl<'a> SparseImageFileInfo<'a> {
    /// Creates new file information.
    fn new(file: &'a SparseImageFile) -> Self {
        Self { file }
    }
}

impl<'a> fmt::Display for SparseImageFileInfo<'a> {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Sparse image (.sparseimage) information:")?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_block_size() as u64, 1024);
        writeln!(formatter, "    Band size\t\t\t\t\t: {}", byte_size)?;

        if let Some(encryption_type) = &self.file.get_encryption_type() {
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
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {} bytes",
            self.file.get_bytes_per_sector()
        )?;
        writeln!(formatter)
    }
}

/// Information about a Mac OS sparse image (.sparseimage) file.
pub struct SparseImageInfo {}

impl SparseImageInfo {
    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<SparseImageFile, ErrorTrace> {
        let mut sparseimage_file: SparseImageFile = SparseImageFile::new();

        match sparseimage_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        }
        if sparseimage_file.is_locked() {
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
            match sparseimage_file.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock file");
                    return Err(error);
                }
            }
        }
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
        let file_information: SparseImageFileInfo = SparseImageFileInfo::new(&sparseimage_file);

        print!("{}", file_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::assert_lines_eq;

    #[test]
    fn test_file_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/sparseimage/hfsplus.sparseimage");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let sparseimage_file: SparseImageFile = SparseImageInfo::open_file(&data_stream)?;
        let test_struct: SparseImageFileInfo = SparseImageFileInfo::new(&sparseimage_file);

        let expected_string: &str = concat!(
            "Sparse image (.sparseimage) information:\n",
            "    Band size\t\t\t\t\t: 1.0 MiB (1048576 bytes)\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512 bytes\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
