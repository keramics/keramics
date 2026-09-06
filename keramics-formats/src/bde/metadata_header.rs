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

use keramics_core::ErrorTrace;
use keramics_datetime::{DateTime, Filetime};
use keramics_layout_map::LayoutMap;
use keramics_types::{Uuid, bytes_to_u16_le, bytes_to_u32_le};

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "metadata_size", data_type = "u32"),
        field(name = "format_version", data_type = "u32"),
        field(name = "metadata_header_size", data_type = "u32"),
        field(name = "metadata_size_copy", data_type = "u32"),
        field(name = "volume_identifier", data_type = "Uuid"),
        field(name = "next_nonce_counter", data_type = "u32"),
        field(name = "encryption_method", data_type = "u16", format = "hex"),
        field(name = "encryption_method_copy", data_type = "u16", format = "hex"),
        field(name = "creation_time", data_type = "Filetime"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) metadata header.
pub struct BdeMetadataHeader {
    /// Metadata size.
    pub metadata_size: u32,

    /// Volume identifier.
    pub volume_identifier: Uuid,

    /// Encryption method.
    pub encryption_method: u16,

    /// Creation time.
    pub creation_time: DateTime,
}

impl BdeMetadataHeader {
    /// Creates a new metadata header.
    pub fn new() -> Self {
        Self {
            metadata_size: 0,
            volume_identifier: Uuid::new(),
            encryption_method: 0,
            creation_time: DateTime::NotSet,
        }
    }

    /// Reads the metadata header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 48 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.metadata_size = bytes_to_u32_le!(data, 0);
        self.volume_identifier = Uuid::from_le_bytes(&data[16..32]);
        self.encryption_method = bytes_to_u16_le!(data, 36);

        let filetime: Filetime = Filetime::from_bytes(&data[40..]);

        self.creation_time = if filetime.timestamp == 0 {
            DateTime::NotSet
        } else {
            DateTime::Filetime(filetime)
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xf2, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0xf2, 0x01,
            0x00, 0x00, 0x69, 0xe0, 0xdd, 0xfb, 0xb1, 0xe6, 0xf9, 0x4c, 0x80, 0x64, 0x6b, 0x68,
            0xd5, 0x95, 0x51, 0x71, 0x08, 0x00, 0x00, 0x00, 0x02, 0x80, 0x02, 0x80, 0x73, 0x1c,
            0x01, 0x13, 0x3a, 0x3c, 0xdd, 0x01,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.metadata_size, 498);
        assert_eq!(
            test_struct.volume_identifier.to_string(),
            "fbdde069-e6b1-4cf9-8064-6b68d5955171",
        );
        assert_eq!(test_struct.encryption_method, 0x8002);
        assert_eq!(
            test_struct.creation_time,
            DateTime::Filetime(Filetime {
                timestamp: 0x1dd3c3a13011c73
            })
        );
        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeMetadataHeader::new();
        let result = test_struct.read_data(&test_data[0..47]);
        assert!(result.is_err());
    }
}
