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
use keramics_types::bytes_to_u32_le;

use super::constants::*;

/// Linux Logical Volume Manager (LVM) physical volume label header.
#[derive(LayoutMap)]
#[layout_map(
    structure(
        byte_order = "little",
        field(name = "signature", data_type = "ByteString<8>"),
        field(name = "sector_number", data_type = "u64"),
        field(name = "checksum", data_type = "u32"),
        field(name = "data_offset", data_type = "u32"),
        field(name = "type_indicator", data_type = "ByteString<8>"),
    ),
    methods("debug_read_data")
)]
pub struct LinuxLvmPhysicalVolumeLabelHeader {
    /// Checksum.
    pub checksum: u32,

    /// Data offset.
    pub data_offset: u32,
}

impl LinuxLvmPhysicalVolumeLabelHeader {
    /// Creates a new physical volume label header.
    pub fn new() -> Self {
        Self {
            checksum: 0,
            data_offset: 0,
        }
    }

    /// Reads the physical volume label header from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        if data.len() < 32 {
            return Err(keramics_core::error_trace_new!("Unsupported data size"));
        }
        if &data[0..8] != LINUX_LVM_PHYSICAL_VOLUME_SIGNATURE {
            return Err(keramics_core::error_trace_new!("Unsupported signature"));
        }
        if &data[24..32] != LINUX_LVM_PHYSICAL_VOLUME_TYPE_INDICATOR {
            return Err(keramics_core::error_trace_new!(
                "Unsupported type indicator"
            ));
        }
        self.checksum = bytes_to_u32_le!(data, 16);
        self.data_offset = bytes_to_u32_le!(data, 20);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        vec![
            0x4c, 0x41, 0x42, 0x45, 0x4c, 0x4f, 0x4e, 0x45, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x04, 0xfd, 0x07, 0x3d, 0x20, 0x00, 0x00, 0x00, 0x4c, 0x56, 0x4d, 0x32,
            0x20, 0x30, 0x30, 0x31,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = LinuxLvmPhysicalVolumeLabelHeader::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.checksum, 0x3d07fd04);
        assert_eq!(test_struct.data_offset, 0x00000020);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = LinuxLvmPhysicalVolumeLabelHeader::new();

        let test_data: Vec<u8> = get_test_data();
        let result = test_struct.read_data(&test_data[0..31]);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = LinuxLvmPhysicalVolumeLabelHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_type_indicator() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[24] = 0xff;

        let mut test_struct = LinuxLvmPhysicalVolumeLabelHeader::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }
}
