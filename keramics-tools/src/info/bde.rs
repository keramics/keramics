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
use keramics_formats::bde::{
    BdeCredential, BdeEncryptedVolume, BdeKeyProtector, BdeKeyProtectorType,
};
use keramics_vfs::{VfsCredential, VfsCredentialStore};

use crate::formatters::ByteSize;

/// Information about BitLocker disk encryption (BDE) encrypted volume.
struct BdeEncryptedVolumeInfo<'a> {
    /// Encrypte volume.
    encrypted_volume: &'a BdeEncryptedVolume,
}

impl<'a> BdeEncryptedVolumeInfo<'a> {
    /// Creates new encrypted_volume information.
    fn new(encrypted_volume: &'a BdeEncryptedVolume) -> Self {
        Self { encrypted_volume }
    }
}

impl<'a> fmt::Display for BdeEncryptedVolumeInfo<'a> {
    /// Formats encrypted_volume information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "BitLocker disk encryption (BDE) information:")?;

        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.encrypted_volume.get_identifier()
        )?;
        if let Some(description) = self.encrypted_volume.get_description() {
            writeln!(formatter, "    Description\t\t\t\t\t: {}", description)?;
        }
        writeln!(
            formatter,
            "    Bytes per sector\t\t\t\t: {}",
            self.encrypted_volume.get_bytes_per_sector()
        )?;
        let byte_size: ByteSize = ByteSize::new(self.encrypted_volume.get_volume_size(), 1024);
        writeln!(formatter, "    Size\t\t\t\t\t: {}", byte_size)?;

        // TODO: print creation time

        writeln!(formatter)?;

        writeln!(formatter, "    Encryption information:")?;
        writeln!(
            formatter,
            "        Encryption method\t\t\t: {}",
            self.encrypted_volume.get_encryption_type()
        )?;
        if self.encrypted_volume.is_locked() {
            writeln!(formatter, "        Is locked")?;
        }
        writeln!(formatter)
    }
}

/// Information about BitLocker disk encryption (BDE).
pub struct BdeInfo {}

impl BdeInfo {
    /// Opens a encrypted_volume.
    fn open_encrypted_volume(
        data_stream: &DataStreamReference,
    ) -> Result<BdeEncryptedVolume, ErrorTrace> {
        let mut bde_volume: BdeEncryptedVolume = BdeEncryptedVolume::new();

        match bde_volume.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open BDE encrypted volume");
                return Err(error);
            }
        }
        if bde_volume.is_locked() {
            let credential_store: &VfsCredentialStore = VfsCredentialStore::current();
            let mut credentials: Vec<BdeCredential> = Vec::new();

            for vfs_credential in credential_store.iter() {
                match vfs_credential {
                    VfsCredential::Passphrase(passphrase) => {
                        credentials.push(BdeCredential::Passphrase(passphrase.clone()))
                    }
                    _ => {}
                }
            }
            match bde_volume.unlock(&credentials) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to unlock volume");
                    return Err(error);
                }
            }
        }
        Ok(bde_volume)
    }

    /// Prints information about a encrypted_volume.
    pub fn print_encrypted_volume(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let bde_volume: BdeEncryptedVolume = match Self::open_encrypted_volume(data_stream) {
            Ok(bde_volume) => bde_volume,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open volume");
                return Err(error);
            }
        };
        let encrypted_volume_information: BdeEncryptedVolumeInfo =
            BdeEncryptedVolumeInfo::new(&bde_volume);

        print!("{}", encrypted_volume_information);

        for key_protector_index in 0..bde_volume.get_number_of_key_protectors() {
            let bde_key_protector: &BdeKeyProtector =
                match bde_volume.get_key_protector_by_index(key_protector_index) {
                    Some(key_protector) => key_protector,
                    None => {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Missing key protector: {}",
                            key_protector_index
                        )));
                    }
                };
            println!("    Key protector: {}", key_protector_index + 1);
            println!(
                "        Identifier\t\t\t\t: {}",
                bde_key_protector.get_identifier()
            );

            let protector_type: &BdeKeyProtectorType = bde_key_protector.get_protector_type();
            println!("        Type\t\t\t\t\t: {}", protector_type);

            // TODO: print modification time

            println!();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use keramics_core::open_os_data_stream;
    use keramics_formats::RangeStream;
    use keramics_formats::vhd::VhdFile;

    use crate::assert_lines_eq;

    #[test]
    fn test_encrypted_volume_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/bde/bde_aes128.vhd");
        let os_data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let mut vhd_file: VhdFile = VhdFile::new();
        vhd_file.read_data_stream(&os_data_stream)?;

        let vhd_data_stream: DataStreamReference = vhd_file.get_data_stream().unwrap();
        let data_stream: DataStreamReference = Arc::new(RwLock::new(RangeStream::new(
            &vhd_data_stream,
            65536,
            65994752,
        )));
        let bde_volume: BdeEncryptedVolume = BdeInfo::open_encrypted_volume(&data_stream)?;

        let test_struct: BdeEncryptedVolumeInfo = BdeEncryptedVolumeInfo::new(&bde_volume);

        let expected_string: &str = concat!(
            "BitLocker disk encryption (BDE) information:\n",
            "    Identifier\t\t\t\t\t: fbdde069-e6b1-4cf9-8064-6b68d5955171\n",
            "    Description\t\t\t\t\t: TEST TestVolume 2026-09-04\n",
            "    Bytes per sector\t\t\t\t: 512\n",
            "    Size\t\t\t\t\t: 62.9 MiB (65994752 bytes)\n",
            "\n",
            "    Encryption information:\n",
            "        Encryption method\t\t\t: AES-128-CBC\n",
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
