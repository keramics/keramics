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
use keramics_formats::qcow::{QcowCompressionMethod, QcowCredential, QcowFile};
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

/// QEMU Copy-On-Write (QCOW) compatibility feature flags information.
struct QcowCompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl QcowCompatibilityFeatureFlagsInfo {
    /// Creates new compatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for QcowCompatibilityFeatureFlagsInfo {
    /// Formats compatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: (QCOW2_COMPAT_LAZY_REFCOUNTS)"
            )?;
        }
        Ok(())
    }
}

/// QEMU Copy-On-Write (QCOW) file information.
struct QcowFileInfo<'a> {
    /// File.
    file: &'a QcowFile,
}

impl<'a> QcowFileInfo<'a> {
    const COMPRESSION_METHODS: &'static [(QcowCompressionMethod, &'static str); 2] = &[
        (QcowCompressionMethod::Zlib, "zlib"),
        (QcowCompressionMethod::Zstd, "zstd"),
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
        let flags: u64 = self.file.get_compatible_feature_flags();
        writeln!(
            formatter,
            "    Compatible features\t\t\t\t: 0x{:016x}",
            flags
        )?;
        let flags_info: QcowCompatibilityFeatureFlagsInfo =
            QcowCompatibilityFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let flags: u64 = self.file.get_incompatible_feature_flags();
        writeln!(
            formatter,
            "    Incompatible features\t\t\t: 0x{:016x}",
            flags
        )?;
        let flags_info: QcowIncompatibilityFeatureFlagsInfo =
            QcowIncompatibilityFeatureFlagsInfo::new(flags);
        writeln!(formatter, "{}", flags_info)?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_block_size() as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        let compression_method_string: &str =
            self.get_compression_method_string(self.file.get_compression_method());
        writeln!(
            formatter,
            "    Compression method\t\t\t\t: {}",
            compression_method_string
        )?;
        if let Some(backing_file_name) = &self.file.get_backing_file_name() {
            writeln!(
                formatter,
                "    Backing file name\t\t\t\t: {}",
                backing_file_name
            )?;
        }
        writeln!(formatter)?;

        if let Some(encryption_type) = self.file.get_encryption_type() {
            writeln!(formatter, "    Encryption information:")?;
            writeln!(
                formatter,
                "        Encryption method\t\t\t: {}",
                encryption_type
            )?;
            if self.file.is_locked() {
                writeln!(formatter, "        Is locked")?;
            }
            writeln!(formatter)?;
        }
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        // TODO: print snapshot information.

        writeln!(formatter)
    }
}

/// QEMU Copy-On-Write (QCOW) incompatibility feature flags information.
struct QcowIncompatibilityFeatureFlagsInfo {
    /// Flags.
    flags: u64,
}

impl QcowIncompatibilityFeatureFlagsInfo {
    /// Creates new incompatibility feature flags information.
    fn new(flags: u64) -> Self {
        Self { flags }
    }
}

impl fmt::Display for QcowIncompatibilityFeatureFlagsInfo {
    /// Formats incompatibility feature flags information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.flags & 0x0000000000000001 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000001: (QCOW2_INCOMPAT_DIRTY)"
            )?;
        }
        if self.flags & 0x0000000000000002 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000002: (QCOW2_INCOMPAT_CORRUPT)"
            )?;
        }
        if self.flags & 0x0000000000000004 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000004: (QCOW2_INCOMPAT_DATA_FILE)"
            )?;
        }
        if self.flags & 0x0000000000000008 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000008: (QCOW2_INCOMPAT_COMPRESSION)"
            )?;
        }
        if self.flags & 0x0000000000000010 != 0 {
            writeln!(
                formatter,
                "        0x0000000000000010: (QCOW2_INCOMPAT_EXTL2)"
            )?;
        }
        Ok(())
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
        if qcow_file.is_locked() {
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<QcowCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(QcowCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match qcow_file.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock file");
                    return Err(error);
                }
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
            "    Compatible features\t\t\t\t: 0x0000000000000000\n",
            "\n",
            "    Incompatible features\t\t\t: 0x0000000000000000\n",
            "\n",
            "    Block size\t\t\t\t\t: 64.0 KiB (65536 bytes)\n",
            "    Compression method\t\t\t\t: zlib\n",
            "\n",
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
