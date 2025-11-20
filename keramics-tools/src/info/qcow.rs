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

use std::collections::HashMap;
use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::qcow::{QcowCompressionMethod, QcowEncryptionMethod, QcowFile};
use keramics_types::ByteString;

use crate::formatters::format_as_bytesize;

/// Information about a QEMU Copy-On-Write (QCOW) file.
struct QcowFileInfo {
    /// Format version.
    pub format_version: u32,

    /// Media size.
    pub media_size: u64,

    /// Compression method.
    pub compression_method: QcowCompressionMethod,

    /// Encryption method.
    pub encryption_method: QcowEncryptionMethod,

    /// Backing file name.
    pub backing_file_name: Option<ByteString>,
}

impl QcowFileInfo {
    /// Creates new file information.
    fn new() -> Self {
        Self {
            format_version: 0,
            media_size: 0,
            compression_method: QcowCompressionMethod::Unknown,
            encryption_method: QcowEncryptionMethod::Unknown,
            backing_file_name: None,
        }
    }
}

impl fmt::Display for QcowFileInfo {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "QEMU Copy-On-Write (QCOW) information:\n")?;

        write!(
            formatter,
            "    Format version\t\t\t\t: {}\n",
            self.format_version
        )?;

        if self.media_size < 1024 {
            write!(
                formatter,
                "    Media size\t\t\t\t\t: {} bytes\n",
                self.media_size
            )?;
        } else {
            let media_size_string: String = format_as_bytesize(self.media_size, 1024);
            write!(
                formatter,
                "    Media size\t\t\t\t\t: {} ({} bytes)\n",
                media_size_string, self.media_size
            )?;
        }
        let compression_methods = HashMap::<QcowCompressionMethod, &'static str>::from([
            (QcowCompressionMethod::Unknown, "Unknown"),
            (QcowCompressionMethod::Zlib, "zlib"),
        ]);
        let compression_method_string: &str =
            compression_methods.get(&self.compression_method).unwrap();

        write!(
            formatter,
            "    Compression method\t\t\t\t: {}\n",
            compression_method_string
        )?;
        let encryption_methods = HashMap::<QcowEncryptionMethod, &'static str>::from([
            (QcowEncryptionMethod::AesCbc128, "AES-CBC 128-bit"),
            (QcowEncryptionMethod::Luks, "Linux Unified Key Setup (LUKS)"),
            (QcowEncryptionMethod::None, "None"),
            (QcowEncryptionMethod::Unknown, "Unknown"),
        ]);
        let encryption_method_string: &str =
            encryption_methods.get(&self.encryption_method).unwrap();

        write!(
            formatter,
            "    Encryption method\t\t\t\t: {}\n",
            encryption_method_string
        )?;

        if let Some(backing_file_name) = &self.backing_file_name {
            write!(
                formatter,
                "    Backing file name\t\t\t\t: {}\n",
                backing_file_name
            )?;
        }
        // TODO: print feature flags.
        // TODO: print snapshot information.

        write!(formatter, "\n")
    }
}

/// Information about a QEMU Copy-On-Write (QCOW) file.
pub struct QcowInfo {}

impl QcowInfo {
    /// Retrieves the file information.
    fn get_file_information(qcow_file: &QcowFile) -> QcowFileInfo {
        let mut file_information: QcowFileInfo = QcowFileInfo::new();

        file_information.format_version = qcow_file.format_version;
        file_information.media_size = qcow_file.media_size;
        file_information.compression_method = qcow_file.compression_method.clone();
        file_information.encryption_method = qcow_file.encryption_method.clone();
        file_information.backing_file_name = qcow_file.get_backing_file_name().cloned();

        file_information
    }

    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<QcowFile, ErrorTrace> {
        let mut qcow_file: QcowFile = QcowFile::new();

        match qcow_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open QCOW file");
                return Err(error);
            }
        };
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
        let file_information: QcowFileInfo = Self::get_file_information(&qcow_file);

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
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow2");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let qcow_file: QcowFile = QcowInfo::open_file(&data_stream)?;
        let test_struct: QcowFileInfo = QcowInfo::get_file_information(&qcow_file);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "QEMU Copy-On-Write (QCOW) information:\n",
            "    Format version\t\t\t\t: 3\n",
            "    Media size\t\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "    Compression method\t\t\t\t: zlib\n",
            "    Encryption method\t\t\t\t: None\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/qcow/ext2.qcow2");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let qcow_file: QcowFile = QcowInfo::open_file(&data_stream)?;
        let test_struct: QcowFileInfo = QcowInfo::get_file_information(&qcow_file);

        assert_eq!(test_struct.format_version, 3);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.compression_method, QcowCompressionMethod::Zlib);
        assert_eq!(test_struct.backing_file_name, None);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
