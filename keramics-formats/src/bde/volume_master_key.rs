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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encryption::AesCcmContext;
use keramics_layout_map::LayoutMap;
use keramics_types::{Uuid, bytes_to_u16_le};

use super::aes_ccm_encrypted_key::BdeAesCcmEncryptedKey;
use super::metadata_entry_header::BdeMetadataEntryHeader;
use super::metadata_property::BdeMetadataProperty;
use super::password::BdePassword;
use super::stretch_key::BdeStretchKey;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "identifier", data_type = "Uuid"),
        field(name = "modification_type", data_type = "Filetime"),
        field(name = "unknown1", data_type = "[u8; 2]"),
        field(name = "protector_type", data_type = "u16", format = "hex"),
    ),
    methods("debug_read_data")
)]
/// BitLocker disk encryption (BDE) volume master key.
pub struct BdeVolumeMasterKey {
    /// Identifier.
    pub identifier: Uuid,

    /// Protector type.
    pub protector_type: u16,

    /// Properties.
    pub properties: Vec<BdeMetadataProperty>,

    /// Data.
    data: Vec<u8>,

    /// Key.
    pub key: Vec<u8>,
}

impl BdeVolumeMasterKey {
    /// Creates a new volume master key.
    pub fn new() -> Self {
        Self {
            identifier: Uuid::new(),
            protector_type: 0,
            properties: Vec::new(),
            data: Vec::new(),
            key: Vec::new(),
        }
    }

    /// Reads the volume master key from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 28 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        self.identifier = Uuid::from_le_bytes(&data[0..16]);
        self.protector_type = bytes_to_u16_le!(data, 26);

        let mut data_offset: usize = 28;
        let mut entry_index: usize = 0;

        while data_offset < data_size - 8 {
            let data_end_offset: usize = data_offset + 8;

            if &data[data_offset..data_end_offset] == &[0; 8] {
                break;
            }
            keramics_core::debug_trace_structure!(BdeMetadataEntryHeader::debug_read_data(
                &data[data_offset..]
            ));
            let mut entry_header: BdeMetadataEntryHeader = BdeMetadataEntryHeader::new();

            match entry_header.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read metadata entry: {} header", entry_index),
                    );
                    return Err(error);
                }
            }
            if entry_header.entry_type != 0 {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid metadata entry: {} unsupported entry type: 0x{:04x}",
                    entry_index, entry_header.entry_type
                )));
            }
            if entry_header.entry_size < 8
                || (entry_header.entry_size as usize) > data_size - data_offset
            {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid metadata entry: {} size value out of bounds",
                    entry_index
                )));
            }
            let entry_data_size: usize = (entry_header.entry_size as usize) - 8;

            data_offset += 8;

            let data_end_offset: usize = data_offset + entry_data_size;

            keramics_core::debug_trace_data!(
                "BdeMetadataPropertyData",
                data_offset,
                &data[data_offset..data_end_offset],
                entry_data_size
            );
            self.properties.push(BdeMetadataProperty::new(
                entry_header.value_type,
                data_offset,
                entry_data_size,
            ));
            data_offset = data_end_offset;
            entry_index += 1;
        }
        Ok(())
    }

    /// Reads the volume master key from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u16,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        if data_size < 18 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported volume master key data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("BdeVolumeMasterKey", offset, &data, data_size);

        match self.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read volume master key at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        self.data = data;

        Ok(())
    }

    /// Reads the AES-CCM encrypted key property.
    fn read_aes_ccm_encrypted_key_property(&self) -> Result<BdeAesCcmEncryptedKey, ErrorTrace> {
        let data_size: usize = self.data.len();

        let property: &BdeMetadataProperty = match self
            .properties
            .iter()
            .find(|property| property.value_type == 0x0005)
        {
            Some(property) => property,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing AES-CCM encrypted key property"
                ));
            }
        };
        let data_offset: usize = property.offset;

        if data_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid AES-CCM encrypted key property data offset value out of bounds",
            ));
        }
        if property.size > data_size - data_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid AES-CCM encrypted key property data size value out of bounds",
            ));
        }
        let data_end_offset: usize = data_offset + property.size;

        keramics_core::debug_trace_structure!(BdeAesCcmEncryptedKey::debug_read_data(
            &self.data[data_offset..data_end_offset]
        ));
        let mut aes_ccm_encrypted_key: BdeAesCcmEncryptedKey = BdeAesCcmEncryptedKey::new();

        match aes_ccm_encrypted_key.read_data(&self.data[data_offset..data_end_offset]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read AES-CCM encrypted key"
                );
                return Err(error);
            }
        }
        Ok(aes_ccm_encrypted_key)
    }

    /// Reads the stretch key property.
    fn read_stretch_key_property(&self) -> Result<BdeStretchKey, ErrorTrace> {
        let data_size: usize = self.data.len();

        let property: &BdeMetadataProperty = match self
            .properties
            .iter()
            .find(|property| property.value_type == 0x0003)
        {
            Some(property) => property,
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Missing stretch key property"
                ));
            }
        };
        let data_offset: usize = property.offset;

        if data_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid stretch key property data offset value out of bounds",
            ));
        }
        if property.size > data_size - data_offset {
            return Err(keramics_core::error_trace_new!(
                "Invalid stretch key property data size value out of bounds",
            ));
        }
        let data_end_offset: usize = data_offset + property.size;

        keramics_core::debug_trace_structure!(BdeStretchKey::debug_read_data(
            &self.data[data_offset..data_end_offset]
        ));
        let mut stretch_key: BdeStretchKey = BdeStretchKey::new();

        match stretch_key.read_data(&self.data[data_offset..data_end_offset]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read stretch key");
                return Err(error);
            }
        }
        Ok(stretch_key)
    }

    /// Unlocks the key using a password.
    pub fn unlock_with_password(&mut self, password_hash: &[u8]) -> Result<bool, ErrorTrace> {
        if self.protector_type != 0x2000 {
            return Ok(false);
        }
        let data_size: usize = self.data.len();

        let mut stretch_key: BdeStretchKey = match self.read_stretch_key_property() {
            Ok(key) => key,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read stretch key property");
                return Err(error);
            }
        };
        let mut aes_ccm_encrypted_key: BdeAesCcmEncryptedKey =
            match self.read_aes_ccm_encrypted_key_property() {
                Ok(key) => key,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read AES-CCM encrypted key property"
                    );
                    return Err(error);
                }
            };
        let password_key: Vec<u8> =
            match BdePassword::calculate_key(&stretch_key.salt, password_hash) {
                Ok(password_hash) => password_hash,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to calculate password key"
                    );
                    return Err(error);
                }
            };
        let mut ccm_context: AesCcmContext = AesCcmContext::new(&aes_ccm_encrypted_key.nonce, &[]);

        match ccm_context.set_key(&password_key) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to set key of AES-CCM context"
                );
                return Err(error);
            }
        };
        let key_size: usize = aes_ccm_encrypted_key.encrypted_data.len();
        self.key = vec![0; key_size];
        let mut tag: Vec<u8> = vec![0; 16];

        match ccm_context.decrypt(
            &aes_ccm_encrypted_key.encrypted_data,
            &mut self.key,
            &mut tag,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to decrypt AES-CCM encrypted key"
                );
                return Err(error);
            }
        };
        Ok(&aes_ccm_encrypted_key.tag == &tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        vec![
            0xd8, 0xd4, 0x0a, 0x55, 0x07, 0xef, 0xe3, 0x4a, 0xb8, 0x1a, 0x02, 0xee, 0x00, 0x0e,
            0x85, 0x7b, 0xf0, 0x01, 0x7c, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x00, 0x00, 0x00, 0x20,
            0x6c, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x01, 0x10, 0x00, 0x00, 0xfe, 0xdc,
            0xfa, 0xa5, 0xe2, 0x6e, 0xe3, 0x88, 0x0d, 0x2b, 0xdb, 0x2e, 0xe4, 0xe4, 0x42, 0x8f,
            0x50, 0x00, 0x00, 0x00, 0x05, 0x00, 0x01, 0x00, 0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c,
            0xdd, 0x01, 0x02, 0x00, 0x00, 0x00, 0x1f, 0x36, 0xdb, 0xbe, 0x1c, 0x3e, 0x0a, 0xd2,
            0x95, 0x02, 0x90, 0x07, 0xf8, 0xd8, 0x1e, 0xa0, 0xb4, 0x83, 0x96, 0xae, 0x1a, 0xa4,
            0xb1, 0xa1, 0xab, 0x59, 0x0f, 0xda, 0xeb, 0xc7, 0x4c, 0x70, 0x23, 0x49, 0xb8, 0x86,
            0xbd, 0x9c, 0x79, 0x53, 0xc7, 0x51, 0x48, 0x09, 0x9b, 0x52, 0xeb, 0xe7, 0xc3, 0xd9,
            0x06, 0xf2, 0x47, 0x08, 0x0f, 0xeb, 0x46, 0x6a, 0x94, 0x19, 0x50, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x00, 0xa0, 0x7a, 0x71, 0x19, 0x3a, 0x3c, 0xdd, 0x01, 0x03, 0x00,
            0x00, 0x00, 0x61, 0x99, 0x52, 0x91, 0xa6, 0x7a, 0xf5, 0xb7, 0x7d, 0x49, 0x43, 0x15,
            0xae, 0x32, 0x2e, 0x1b, 0xed, 0x99, 0x40, 0x76, 0xcc, 0xd0, 0x22, 0x54, 0xd2, 0xcf,
            0x82, 0xfd, 0x2e, 0x92, 0x36, 0x53, 0xbb, 0x6a, 0xab, 0x0f, 0xd2, 0x50, 0x91, 0xff,
            0x7e, 0xa9, 0xe1, 0x0b, 0x61, 0xc6, 0x12, 0x52, 0xe3, 0xc7, 0x94, 0xd7, 0xe3, 0x92,
            0x04, 0x47, 0xbf, 0x42, 0x26, 0x7a,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeVolumeMasterKey::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.identifier.to_string(),
            "550ad4d8-ef07-4ae3-b81a-02ee000e857b",
        );
        assert_eq!(test_struct.protector_type, 0x2000);
        assert_eq!(test_struct.properties.len(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = BdeVolumeMasterKey::new();
        let result = test_struct.read_data(&test_data[0..27]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = BdeVolumeMasterKey::new();
        test_struct.read_at_position(&data_stream, 216, SeekFrom::Start(0))?;

        assert_eq!(
            test_struct.identifier.to_string(),
            "550ad4d8-ef07-4ae3-b81a-02ee000e857b",
        );
        assert_eq!(test_struct.protector_type, 0x2000);
        assert_eq!(test_struct.properties.len(), 2);

        Ok(())
    }
}
