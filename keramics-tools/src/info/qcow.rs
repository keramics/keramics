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
use keramics_formats::qcow::{QcowCompressionMethod, QcowEncryptionMethod, QcowFile};

use crate::formatters::ByteSize;

/// Information about a QEMU Copy-On-Write (QCOW) file.
struct QcowFileInfo<'a> {
    /// File.
    file: &'a QcowFile,
}

impl<'a> QcowFileInfo<'a> {
    const COMPRESSION_METHODS: &'static [(QcowCompressionMethod, &'static str); 1] =
        &[(QcowCompressionMethod::Zlib, "zlib")];

    const ENCRYPTION_METHODS: &'static [(QcowEncryptionMethod, &'static str); 3] = &[
        (QcowEncryptionMethod::AesCbc128, "AES-CBC 128-bit"),
        (QcowEncryptionMethod::Luks, "Linux Unified Key Setup (LUKS)"),
        (QcowEncryptionMethod::None, "None"),
    ];

    /// Creates new file information.
    fn new(file: &'a QcowFile) -> Self {
        Self { file }
    }

    /// Retrieves the compression method as a string.
    pub fn get_compression_method_string(
        &self,
        compression_method: &QcowCompressionMethod,
    ) -> &str {
        Self::COMPRESSION_METHODS
            .binary_search_by(|(key, _)| key.cmp(compression_method))
            .map_or_else(|_| "Unknown", |index| Self::COMPRESSION_METHODS[index].1)
    }

    /// Retrieves the encryption method as a string.
    pub fn get_encryption_method_string(&self, encryption_method: &QcowEncryptionMethod) -> &str {
        Self::ENCRYPTION_METHODS
            .binary_search_by(|(key, _)| key.cmp(encryption_method))
            .map_or_else(|_| "Unknown", |index| Self::ENCRYPTION_METHODS[index].1)
    }
}

impl<'a> fmt::Display for QcowFileInfo<'a> {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "QEMU Copy-On-Write (QCOW) information:")?;

        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}",
            self.file.get_format_version()
        )?;
        let compression_method_string: &str =
            self.get_compression_method_string(self.file.get_compression_method());
        writeln!(
            formatter,
            "    Compression method\t\t\t\t: {}",
            compression_method_string
        )?;

        let encryption_method_string: &str =
            self.get_encryption_method_string(self.file.get_encryption_method());
        writeln!(
            formatter,
            "    Encryption method\t\t\t\t: {}",
            encryption_method_string
        )?;

        if let Some(backing_file_name) = &self.file.get_backing_file_name() {
            writeln!(
                formatter,
                "    Backing file name\t\t\t\t: {}",
                backing_file_name
            )?;
        }
        // TODO: print feature flags.

        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        // TODO: print snapshot information.

        writeln!(formatter)
    }
}

/// Information about a QEMU Copy-On-Write (QCOW) file.
pub struct QcowInfo {}

impl QcowInfo {
    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<QcowFile, ErrorTrace> {
        let mut qcow_file: QcowFile = QcowFile::new();

        match qcow_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open QCOW file");
                return Err(error);
            }
        }
        Ok(qcow_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let qcow_file: QcowFile = match Self::open_file(data_stream) {
            Ok(qcow_file) => qcow_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: QcowFileInfo = QcowFileInfo::new(&qcow_file);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow2");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let qcow_file: QcowFile = QcowInfo::open_file(&data_stream)?;
        let test_struct: QcowFileInfo = QcowFileInfo::new(&qcow_file);

        let expected_string: &str = concat!(
            "QEMU Copy-On-Write (QCOW) information:\n",
            "    Format version\t\t\t\t: 3\n",
            "    Compression method\t\t\t\t: zlib\n",
            "    Encryption method\t\t\t\t: None\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
