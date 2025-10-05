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

use std::io;

use keramics_layout_map::LayoutMap;
use keramics_types::bytes_to_u16_le;

use super::constants::*;
use super::mft_attribute::NtfsMftAttribute;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        member(field(name = "unknown1", data_type = "[u8; 8]")),
        member(field(name = "major_format_version", data_type = "u8")),
        member(field(name = "minor_format_version", data_type = "u8")),
        member(field(name = "volume_flags", data_type = "u16")),
    ),
    method(name = "debug_read_data")
)]
/// New Technologies File System (NTFS) volume information ($VOLUME_INFORMATION).
pub struct NtfsVolumeInformation {
    /// Major format version
    pub major_format_version: u8,

    /// Minor format version
    pub minor_format_version: u8,

    /// Volume flags.
    pub volume_flags: u16,
}

impl NtfsVolumeInformation {
    /// Creates new volume information.
    pub fn new() -> Self {
        Self {
            major_format_version: 0,
            minor_format_version: 0,
            volume_flags: 0,
        }
    }

    /// Reads the volume information from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported NTFS volume information data size"),
            ));
        }
        self.major_format_version = data[8];
        self.minor_format_version = data[9];
        self.volume_flags = bytes_to_u16_le!(data, 10);

        Ok(())
    }

    /// Reads the volume information from a MFT attribute.
    pub fn from_attribute(mft_attribute: &NtfsMftAttribute) -> io::Result<Self> {
        if mft_attribute.attribute_type != NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported attribute type: 0x{:08x}.",
                    mft_attribute.attribute_type
                ),
            ));
        }
        if mft_attribute.is_compressed() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported compressed $VOLUME_INFORMATION attribute.",
            ));
        }
        if !mft_attribute.is_resident() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported non-resident $VOLUME_INFORMATION attribute.",
            ));
        }
        let mut volume_information: NtfsVolumeInformation = NtfsVolumeInformation::new();
        volume_information.read_data(&mft_attribute.resident_data)?;

        Ok(volume_information)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x80, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_struct = NtfsVolumeInformation::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.major_format_version, 3);
        assert_eq!(test_struct.minor_format_version, 1);
        assert_eq!(test_struct.volume_flags, 0x0080);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsVolumeInformation::new();
        let result = test_struct.read_data(&test_data[0..11]);
        assert!(result.is_err());
    }

    // TODO: add tests for from_attribute
}
