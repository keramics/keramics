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

use std::io::SeekFrom;

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::bytes_to_u32_be;

use super::file_header_v1::QcowFileHeaderV1;
use super::file_header_v2::QcowFileHeaderV2;
use super::file_header_v3::QcowFileHeaderV3;

/// QEMU Copy-On-Write (QCOW) file header.
pub struct QcowFileHeader {
    /// Mediator.
    mediator: MediatorReference,

    /// Format version.
    pub format_version: u32,

    /// Header size.
    pub header_size: u32,

    /// Level 1 table number of references.
    pub level1_table_number_of_references: u32,

    /// Level 1 table offset.
    pub level1_table_offset: u64,

    /// Number of level 2 table bits.
    pub number_of_level2_table_bits: u32,

    /// Number of cluster block bits.
    pub number_of_cluster_block_bits: u32,

    /// Media size.
    pub media_size: u64,

    /// Compression method.
    pub compression_method: u8,

    /// Encryption method.
    pub encryption_method: u32,

    /// Backing file name offset.
    pub backing_file_name_offset: u64,

    /// Backing file name size.
    pub backing_file_name_size: u32,

    /// Number of snapshots.
    pub number_of_snapshots: u32,

    /// Snapshots offset.
    pub snapshots_offset: u64,
}

impl QcowFileHeader {
    /// Creates a new file header.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            format_version: 0,
            header_size: 0,
            level1_table_number_of_references: 0,
            level1_table_offset: 0,
            number_of_level2_table_bits: 0,
            number_of_cluster_block_bits: 0,
            media_size: 0,
            compression_method: 0,
            encryption_method: 0,
            backing_file_name_offset: 0,
            backing_file_name_size: 0,
            number_of_snapshots: 0,
            snapshots_offset: 0,
        }
    }

    /// Reads the file header a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        let mut data: [u8; 112] = [0; 112];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "QcowFileHeader data of size: {} at offset: {} (0x{:08x})\n",
                data.len(),
                offset,
                offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.format_version = bytes_to_u32_be!(data, 4);

        match self.format_version {
            1 => {
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(QcowFileHeaderV1::debug_read_data(&data));
                }
                match QcowFileHeaderV1::read_data(self, &data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read version 1 file header"
                        );
                        return Err(error);
                    }
                }
                self.header_size = 48;
            }
            2 => {
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(QcowFileHeaderV2::debug_read_data(&data));
                }
                match QcowFileHeaderV2::read_data(self, &data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read version 2 file header"
                        );
                        return Err(error);
                    }
                }
                self.header_size = 72;
                self.number_of_level2_table_bits = self.number_of_cluster_block_bits - 3;
            }
            3 => {
                if self.mediator.debug_output {
                    self.mediator
                        .debug_print(QcowFileHeaderV3::debug_read_data(&data));
                }
                match QcowFileHeaderV3::read_data(self, &data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read version 3 file header"
                        );
                        return Err(error);
                    }
                }
                self.number_of_level2_table_bits = self.number_of_cluster_block_bits - 3;
            }
            _ => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported format version: {}",
                    self.format_version
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    fn get_test_data_v3() -> Vec<u8> {
        return vec![
            0x51, 0x46, 0x49, 0xfb, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_at_position_v3() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_v3();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = QcowFileHeader::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0))?;

        assert_eq!(test_struct.format_version, 3);
        assert_eq!(test_struct.backing_file_name_offset, 0);
        assert_eq!(test_struct.backing_file_name_size, 0);
        assert_eq!(test_struct.number_of_cluster_block_bits, 16);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.encryption_method, 0);
        assert_eq!(test_struct.level1_table_number_of_references, 1);
        assert_eq!(test_struct.level1_table_offset, 196608);
        assert_eq!(test_struct.number_of_snapshots, 0);
        assert_eq!(test_struct.snapshots_offset, 0);
        assert_eq!(test_struct.header_size, 112);
        assert_eq!(test_struct.compression_method, 0);

        Ok(())
    }
}
