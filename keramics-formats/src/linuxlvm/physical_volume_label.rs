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

use keramics_checksums::ReversedCrc32Context;
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use super::data_area_descriptor::LinuxLvmDataAreaDescriptor;
use super::physical_volume_header::LinuxLvmPhysicalVolumeHeader;
use super::physical_volume_label_header::LinuxLvmPhysicalVolumeLabelHeader;

/// Linux Logical Volume Manager (LVM) physical volume label.
pub struct LinuxLvmPhysicalVolumeLabel {
    /// Identifier.
    pub identifier: ByteString,

    /// Volume size.
    pub volume_size: u64,

    /// Data area descriptors.
    pub data_area_descriptors: Vec<LinuxLvmDataAreaDescriptor>,

    /// Metadata area descriptors.
    pub metadata_area_descriptors: Vec<LinuxLvmDataAreaDescriptor>,
}

impl LinuxLvmPhysicalVolumeLabel {
    /// Creates a new physical volume label.
    pub fn new() -> Self {
        Self {
            identifier: ByteString::new_with_encoding(&CharacterEncoding::Ascii),
            volume_size: 0,
            data_area_descriptors: Vec::new(),
            metadata_area_descriptors: Vec::new(),
        }
    }

    /// Reads the parent locator from a buffer.
    fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        keramics_core::debug_trace_structure!(LinuxLvmPhysicalVolumeLabelHeader::debug_read_data(
            &data
        ));
        let mut volume_label_header: LinuxLvmPhysicalVolumeLabelHeader =
            LinuxLvmPhysicalVolumeLabelHeader::new();

        match volume_label_header.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read physical volume label header",
                );
                return Err(error);
            }
        }
        if volume_label_header.checksum != 0 {
            let mut crc32_context: ReversedCrc32Context =
                ReversedCrc32Context::new(0xedb88320, 0xf597a6cf ^ 0xffffffff);

            crc32_context.update(&data[20..]);

            let calculated_checksum: u32 = crc32_context.finalize() ^ 0xffffffff;

            if volume_label_header.checksum != calculated_checksum {
                return Err(keramics_core::error_trace_new!(format!(
                    "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} checksums",
                    volume_label_header.checksum, calculated_checksum
                )));
            }
        }
        if volume_label_header.data_offset != 32 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported physical volume header offset"
            ));
        }
        keramics_core::debug_trace_structure!(LinuxLvmPhysicalVolumeHeader::debug_read_data(
            &data[32..]
        ));
        let mut volume_header: LinuxLvmPhysicalVolumeHeader = LinuxLvmPhysicalVolumeHeader::new();

        match volume_header.read_data(&data[32..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read physical volume header",
                );
                return Err(error);
            }
        }
        self.identifier = volume_header.identifier;
        self.volume_size = volume_header.volume_size;

        let mut data_offset: usize = 72;
        let mut data_area_logical_offset: u64 = 0;

        for (descriptor_index, chunk) in data[data_offset..].chunks_exact(16).enumerate() {
            data_offset += 16;

            if chunk == &[0; 16] {
                break;
            }
            keramics_core::debug_trace_structure!(LinuxLvmDataAreaDescriptor::debug_read_data(
                chunk
            ));
            let mut data_area_descriptor: LinuxLvmDataAreaDescriptor =
                LinuxLvmDataAreaDescriptor::new();

            match data_area_descriptor.read_data(chunk) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read data area descriptor: {}", descriptor_index),
                    );
                    return Err(error);
                }
            }
            // TODO: only allow size to be 0 if there is 1 data area descriptor
            if data_area_logical_offset == 0 && data_area_descriptor.size == 0 {
                data_area_descriptor.size = self.volume_size;
            }
            data_area_descriptor.logical_offset = data_area_logical_offset;
            data_area_logical_offset += data_area_descriptor.size;

            self.data_area_descriptors.push(data_area_descriptor);
        }
        data_area_logical_offset = 0;

        for (descriptor_index, chunk) in data[data_offset..].chunks_exact(16).enumerate() {
            if chunk == &[0; 16] {
                break;
            }
            keramics_core::debug_trace_structure!(LinuxLvmDataAreaDescriptor::debug_read_data(
                chunk
            ));
            let mut data_area_descriptor: LinuxLvmDataAreaDescriptor =
                LinuxLvmDataAreaDescriptor::new();

            match data_area_descriptor.read_data(chunk) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read data area descriptor: {}", descriptor_index),
                    );
                    return Err(error);
                }
            }
            data_area_descriptor.logical_offset = data_area_logical_offset;
            data_area_logical_offset += data_area_descriptor.size;

            self.metadata_area_descriptors.push(data_area_descriptor);
        }
        Ok(())
    }

    /// Reads the physical volume label from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        let mut data: [u8; 512] = [0; 512];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        keramics_core::debug_trace_data!("LinuxLvmPhysicalVolumeLabel", offset, &data, 512);

        match self.read_data(&data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read physical volume label at offset: {} (0x{:08x})",
                        offset, offset
                    )
                );
                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x4c, 0x41, 0x42, 0x45, 0x4c, 0x4f, 0x4e, 0x45, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x04, 0xfd, 0x07, 0x3d, 0x20, 0x00, 0x00, 0x00, 0x4c, 0x56, 0x4d, 0x32,
            0x20, 0x30, 0x30, 0x31, 0x6b, 0x36, 0x58, 0x5a, 0x5a, 0x66, 0x48, 0x63, 0x69, 0x79,
            0x6b, 0x6b, 0x78, 0x66, 0x63, 0x46, 0x7a, 0x41, 0x32, 0x36, 0x57, 0x48, 0x51, 0x61,
            0x53, 0x6f, 0x58, 0x70, 0x58, 0x63, 0x32, 0x49, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0xf0, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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

        let mut test_struct = LinuxLvmPhysicalVolumeLabel::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(
            test_struct.identifier,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![
                    0x6b, 0x36, 0x58, 0x5a, 0x5a, 0x66, 0x48, 0x63, 0x69, 0x79, 0x6b, 0x6b, 0x78,
                    0x66, 0x63, 0x46, 0x7a, 0x41, 0x32, 0x36, 0x57, 0x48, 0x51, 0x61, 0x53, 0x6f,
                    0x58, 0x70, 0x58, 0x63, 0x32, 0x49,
                ]
            },
        );
        assert_eq!(test_struct.volume_size, 16777216);
        assert_eq!(test_struct.data_area_descriptors.len(), 1);
        assert_eq!(test_struct.metadata_area_descriptors.len(), 1);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = LinuxLvmPhysicalVolumeLabel::new();
        let result = test_struct.read_data(&test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = LinuxLvmPhysicalVolumeLabel::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(
            test_struct.identifier,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![
                    0x6b, 0x36, 0x58, 0x5a, 0x5a, 0x66, 0x48, 0x63, 0x69, 0x79, 0x6b, 0x6b, 0x78,
                    0x66, 0x63, 0x46, 0x7a, 0x41, 0x32, 0x36, 0x57, 0x48, 0x51, 0x61, 0x53, 0x6f,
                    0x58, 0x70, 0x58, 0x63, 0x32, 0x49,
                ]
            },
        );
        assert_eq!(test_struct.volume_size, 16777216);
        assert_eq!(test_struct.data_area_descriptors.len(), 1);
        assert_eq!(test_struct.metadata_area_descriptors.len(), 1);

        Ok(())
    }
}
