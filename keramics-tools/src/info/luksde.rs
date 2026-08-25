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
use keramics_formats::luksde::{LuksCredential, LuksEncryptedVolume};
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

/// Information about Linux Unified Key Setup (LUKS) Disk Encryption encrypted volume.
struct LuksEncryptedVolumeInfo<'a> {
    /// Encrypte volume.
    encrypted_volume: &'a LuksEncryptedVolume,
}

impl<'a> LuksEncryptedVolumeInfo<'a> {
    /// Creates new encrypted_volume information.
    fn new(encrypted_volume: &'a LuksEncryptedVolume) -> Self {
        Self { encrypted_volume }
    }
}

impl<'a> fmt::Display for LuksEncryptedVolumeInfo<'a> {
    /// Formats encrypted_volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Linux Unified Key Setup (LUKS) Disk Encryption information:",
        )?;
        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}",
            self.encrypted_volume.get_format_version()
        )?;
        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.encrypted_volume.get_identifier()
        )?;
        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.encrypted_volume.get_bytes_per_sector()
        )?;
        let byte_size: ByteSize = ByteSize::new(self.encrypted_volume.get_volume_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        writeln!(formatter)?;

        writeln!(formatter, "    Encryption information:")?;
        writeln!(
            formatter,
            "        Encryption method\t\t\t: {}",
            self.encrypted_volume.get_encryption_type()
        )?;
        // TODO: print key slots

        if self.encrypted_volume.is_locked() {
            writeln!(formatter, "        Is locked")?;
        }
        writeln!(formatter)
    }
}

/// Information about Linux Unified Key Setup (LUKS) Disk Encryption.
pub struct LuksInfo {}

impl LuksInfo {
    /// Opens a encrypted_volume.
    fn open_encrypted_volume(
        data_stream: &DataStreamReference,
    ) -> Result<LuksEncryptedVolume, ErrorTrace> {
        let mut luks_volume: LuksEncryptedVolume = LuksEncryptedVolume::new();

        match luks_volume.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to open LUKS encrypted volume"
                );
                return Err(error);
            }
        }
        if luks_volume.is_locked() {
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<LuksCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(LuksCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match luks_volume.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock image");
                    return Err(error);
                }
            }
        }
        Ok(luks_volume)
    }

    /// Prints information about a encrypted_volume.
    pub fn print_encrypted_volume(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let luks_volume: LuksEncryptedVolume = match Self::open_encrypted_volume(data_stream) {
            Ok(luks_volume) => luks_volume,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open encrypted_volume");
                return Err(error);
            }
        };
        let encrypted_volume_information: LuksEncryptedVolumeInfo =
            LuksEncryptedVolumeInfo::new(&luks_volume);

        print!("{}", encrypted_volume_information);

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
    fn test_encrypted_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/luksde/luks1.raw");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let luks_volume: LuksEncryptedVolume = LuksInfo::open_encrypted_volume(&data_stream)?;

        let test_struct: LuksEncryptedVolumeInfo = LuksEncryptedVolumeInfo::new(&luks_volume);

        let expected_string: &str = concat!(
            "Linux Unified Key Setup (LUKS) Disk Encryption information:\n",
            "    Format version\t\t\t\t: 1\n",
            "    Identifier\t\t\t\t\t: 20bc2795-63f3-4dc4-80d8-07911913a031\n",
            "    Bytes per sector\t\t\t\t: 512\n",
            "    Size\t\t\t\t\t: 2.0 MiB (2097152 bytes)\n",
            "\n",
            "    Encryption information:\n",
            "        Encryption method\t\t\t: AES-256-CBC\n",
            "        Is locked\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_encrypted_volume
    // TODO: add tests for print_encrypted_volume
}
