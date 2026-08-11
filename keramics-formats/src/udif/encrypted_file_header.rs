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
use keramics_layout_map::LayoutMap;
use keramics_types::{bytes_to_u32_be, bytes_to_u64_be};

use super::constants::*;
use super::encryption_type::UdifEncryptionType;
use super::key_protector_descriptor::UdifKeyProtectorDescriptor;

#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "big",
        field(name = "signature", data_type = "ByteString<8>"),
        field(name = "format_version", data_type = "u32"),
        field(name = "block_initialization_vector_size", data_type = "u32"),
        field(name = "block_encryption_mode", data_type = "u32"),
        field(name = "block_encryption_method", data_type = "u32", format = "hex"),
        field(name = "block_key_size", data_type = "u32"),
        field(name = "hmac_method", data_type = "u32"),
        field(name = "hmac_key_size", data_type = "u32"),
        field(name = "identifier", data_type = "Uuid"),
        field(name = "block_size", data_type = "u32"),
        field(name = "data_fork_size", data_type = "u64"),
        field(name = "data_fork_offset", data_type = "u64"),
        field(name = "number_of_key_protectors", data_type = "u32"),
        field(name = "unknown3", data_type = "[u8; 436]", format = "hex"),
    ),
    methods("debug_read_data", "read_at_position")
)]
/// Universal Disk Image Format (UDIF) encrypted file header.
pub struct UdifEncryptedFileHeader {
    /// Format version.
    pub format_version: u32,

    /// Block size.
    pub block_size: u32,

    /// Initialization vector size.
    pub initialization_vector_size: u32,

    /// Encryption type.
    pub encryption_type: UdifEncryptionType,

    /// HMAC method.
    pub hmac_method: u32,

    /// Initialization vector encryption method.
    pub hmac_key_size: u32,

    /// Data fork size.
    pub data_fork_size: u64,

    /// Data fork offset.
    pub data_fork_offset: u64,

    /// Key protector descriptors.
    pub key_protector_descriptors: Vec<UdifKeyProtectorDescriptor>,
}

impl UdifEncryptedFileHeader {
    /// Creates a new encrypted file header.
    pub fn new() -> Self {
        Self {
            format_version: 0,
            block_size: 0,
            initialization_vector_size: 0,
            encryption_type: UdifEncryptionType::new(),
            hmac_method: 0,
            hmac_key_size: 0,
            data_fork_size: 0,
            data_fork_offset: 0,
            key_protector_descriptors: Vec::new(),
        }
    }

    /// Reads the file encrypted header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        if data_size < 512 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..8] != UDIF_ENCRYPTED_FILE_HEADER_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        self.format_version = bytes_to_u32_be!(data, 8);

        if self.format_version != 2 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported format version: {}",
                self.format_version
            )));
        }
        self.initialization_vector_size = bytes_to_u32_be!(data, 12);
        self.encryption_type.mode = bytes_to_u32_be!(data, 16);
        self.encryption_type.method = bytes_to_u32_be!(data, 20);
        self.encryption_type.key_size = (bytes_to_u32_be!(data, 24) / 8) as usize;
        self.hmac_method = bytes_to_u32_be!(data, 28);
        self.hmac_key_size = bytes_to_u32_be!(data, 32);

        self.block_size = bytes_to_u32_be!(data, 52);
        self.data_fork_size = bytes_to_u64_be!(data, 56);
        self.data_fork_offset = bytes_to_u64_be!(data, 64);

        let number_of_key_protectors: u32 = bytes_to_u32_be!(data, 72);

        if (number_of_key_protectors as usize) > (data_size - 76) / 20 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of key protectors value out of bounds"
            ));
        }
        let mut data_offset: usize = 76;

        for value_index in 0..number_of_key_protectors {
            keramics_core::debug_trace_structure!(UdifKeyProtectorDescriptor::debug_read_data(
                &data[data_offset..]
            ));
            let mut key_protector_descriptor: UdifKeyProtectorDescriptor =
                UdifKeyProtectorDescriptor::new();

            match key_protector_descriptor.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read key protector descriptor: {}", value_index)
                    );
                    return Err(error);
                }
            }
            data_offset += 20;

            self.key_protector_descriptors
                .push(key_protector_descriptor);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x65, 0x6e, 0x63, 0x72, 0x63, 0x64, 0x73, 0x61, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x05, 0x80, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x80,
            0x00, 0x00, 0x00, 0x5b, 0x00, 0x00, 0x00, 0xa0, 0x12, 0xd1, 0x4a, 0x6e, 0xfa, 0x1f,
            0x4a, 0x6b, 0xad, 0x03, 0x3e, 0x72, 0x09, 0x63, 0xb1, 0x05, 0x00, 0x00, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0xe0, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x68, 0x00, 0x00,
            0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe8, 0x00, 0x00, 0x00, 0x14,
            0x83, 0x97, 0x30, 0xbe, 0x23, 0x31, 0xc6, 0x9d, 0xf4, 0xf7, 0x29, 0xff, 0xe8, 0xa1,
            0x0c, 0x26, 0x65, 0x3b, 0xea, 0x94, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x1f, 0x24, 0xe2, 0x57, 0x12, 0xc2,
            0xd7, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xc0, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x06,
            0x00, 0x00, 0x00, 0x30, 0x32, 0x31, 0xe2, 0x0a, 0xa6, 0x42, 0x88, 0x9a, 0x7e, 0x08,
            0x7c, 0xb8, 0x7c, 0x84, 0xba, 0x1c, 0xd5, 0x28, 0x64, 0x00, 0x7c, 0xfe, 0xa6, 0x77,
            0x79, 0x6a, 0x6f, 0x52, 0xe1, 0x6b, 0x26, 0x09, 0x69, 0x6d, 0xde, 0x92, 0x30, 0xae,
            0xb5, 0x60, 0x3a, 0xeb, 0x1f, 0x70, 0xf6, 0x70, 0x1b, 0xe6, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifEncryptedFileHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.format_version, 2);
        assert_eq!(test_struct.initialization_vector_size, 16);
        assert_eq!(test_struct.encryption_type.mode, 5);
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.encryption_type.key_size, 16);
        assert_eq!(test_struct.hmac_method, 91);
        assert_eq!(test_struct.hmac_key_size, 160);
        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.data_fork_size, 65536);
        assert_eq!(test_struct.data_fork_offset, 122880);
        assert_eq!(test_struct.key_protector_descriptors.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = UdifEncryptedFileHeader::new();
        let result = test_struct.read_data(&test_data[0..511]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = UdifEncryptedFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_format_version() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[8] = 0xff;

        let mut test_struct = UdifEncryptedFileHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = UdifEncryptedFileHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.format_version, 2);
        assert_eq!(test_struct.initialization_vector_size, 16);
        assert_eq!(test_struct.encryption_type.mode, 5);
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.encryption_type.key_size, 16);
        assert_eq!(test_struct.hmac_method, 0x0000005b);
        assert_eq!(test_struct.hmac_key_size, 160);
        assert_eq!(test_struct.block_size, 4096);
        assert_eq!(test_struct.data_fork_size, 65536);
        assert_eq!(test_struct.data_fork_offset, 122880);
        assert_eq!(test_struct.key_protector_descriptors.len(), 1);

        Ok(())
    }
}
