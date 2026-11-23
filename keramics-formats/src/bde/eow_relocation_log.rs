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

use super::eow_relocation_log_entry::BdeEowRelocationLogEntry;
use super::eow_relocation_log_header::BdeEowRelocationLogHeader;

/// BitLocker Drive Encryption (BDE) Encrypt-on-Write (EOW) relocation log.
pub struct BdeEowRelocationLog {}

impl BdeEowRelocationLog {
    /// Creates a new Encrypt-on-Write (EOW) relocation log.
    pub fn new() -> Self {
        Self {}
    }

    /// Reads the Encrypt-on-Write (EOW) relocation log from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: usize,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        // Note that 16777216 is an arbitrary chosen limit.
        if !(1024..=16777216).contains(&data_size) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported Encrypt-on-Write (EOW) relocation log size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        let mut data_offset: usize = 0;
        let data_end_offset: usize = 512;

        keramics_core::debug_trace_data_and_structure!(
            "BdeEowRelocationLogHeader",
            offset,
            &data[data_offset..data_end_offset],
            512,
            BdeEowRelocationLogHeader::debug_read_data(&data[data_offset..data_end_offset])
        );
        let mut header: BdeEowRelocationLogHeader = BdeEowRelocationLogHeader::new();

        match header.read_data(&data[data_offset..data_end_offset]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read Encrypt-on-Write (EOW) relocation log header",
                );
                return Err(error);
            }
        }
        data_offset = data_end_offset;

        if header.entry_size < 1024 || (header.entry_size as usize) > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid relocation log header - entry size value out of bounds",
            ));
        }
        if (header.number_of_entries as usize) > (data_size - 1024) / (header.entry_size as usize) {
            return Err(keramics_core::error_trace_new!(
                "Invalid relocation log header - number of entries value out of bounds",
            ));
        }
        let sectors_data_size: usize = (header.entry_size as usize) - 1024;

        for entry_index in 0..header.number_of_entries {
            let data_end_offset: usize = data_offset + 512;

            keramics_core::debug_trace_data_and_structure!(
                format!("BdeEowRelocationLogEntry: {}", entry_index),
                offset + (data_offset as u64),
                &data[data_offset..data_end_offset],
                512,
                BdeEowRelocationLogEntry::debug_read_data(&data[data_offset..data_end_offset])
            );
            let mut entry: BdeEowRelocationLogEntry = BdeEowRelocationLogEntry::new();

            match entry.read_data(&data[data_offset..data_end_offset]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Encrypt-on-Write (EOW) relocation log entry: {}",
                            entry_index
                        )
                    );
                    return Err(error);
                }
            }
            data_offset = data_end_offset;

            let entry_sectors_data_size: usize = entry.encrypted_sectors_data_size as usize;

            if entry_sectors_data_size > data_size - data_offset {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid relocation log entry: {} - used entry sectors data size value out of bounds",
                    entry_index
                )));
            }
            let data_end_offset: usize = data_offset + entry_sectors_data_size;

            keramics_core::debug_trace_data!(
                "BdeEowRelocationLogSectorData",
                offset + (data_offset as u64),
                &data[data_offset..data_end_offset],
                sectors_data_size,
            );
            if entry.encrypted_sectors_data_checksum != 0 {
                let mut crc32_context: ReversedCrc32Context =
                    ReversedCrc32Context::new(0xedb88320, 0);

                crc32_context.update(&data[data_offset..data_end_offset]);

                let calculated_checksum: u32 = crc32_context.finalize();

                if entry.encrypted_sectors_data_checksum != calculated_checksum {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Mismatch between stored: 0x{:08x} and calculated: 0x{:08x} encrypted sectors data checksums",
                        entry.encrypted_sectors_data_checksum, calculated_checksum
                    )));
                }
            }
            data_offset += sectors_data_size;

            let data_end_offset: usize = data_offset + 512;

            keramics_core::debug_trace_data_and_structure!(
                format!("BdeEowRelocationLogBackupEntry: {}", entry_index),
                offset + (data_offset as u64),
                &data[data_offset..data_end_offset],
                512,
                BdeEowRelocationLogEntry::debug_read_data(&data[data_offset..data_end_offset])
            );
            let mut entry: BdeEowRelocationLogEntry = BdeEowRelocationLogEntry::new();

            match entry.read_data(&data[data_offset..data_end_offset]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read Encrypt-on-Write (EOW) relocation log backup entry: {}",
                            entry_index
                        )
                    );
                    return Err(error);
                }
            }
            data_offset = data_end_offset;
        }
        let data_end_offset: usize = data_offset + 512;

        keramics_core::debug_trace_data_and_structure!(
            "BdeEowRelocationLogBackupHeader",
            offset + (data_offset as u64),
            &data[data_offset..data_end_offset],
            512,
            BdeEowRelocationLogHeader::debug_read_data(&data[data_offset..data_end_offset])
        );
        let mut header: BdeEowRelocationLogHeader = BdeEowRelocationLogHeader::new();

        match header.read_data(&data[data_offset..data_end_offset]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read Encrypt-on-Write (EOW) relocation log backup header",
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

    /// Builds a synthetic EOW relocation log header for testing.
    fn build_header(number_of_entries: u16, entry_size: u32) -> [u8; 512] {
        let mut header: [u8; 512] = [0; 512];
        header[0..10].copy_from_slice(b"OLRDHEVF2\x00");
        header[12..14].copy_from_slice(&number_of_entries.to_le_bytes());
        header[30..34].copy_from_slice(&entry_size.to_le_bytes());
        header
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        // Build a valid 1-entry relocation log (512 header + 1536 entry + 512 backup header = 2560 bytes).
        let mut log_data: Vec<u8> = vec![0; 2560];
        let header: [u8; 512] = build_header(1, 1536);
        log_data[0..512].copy_from_slice(&header);
        log_data[2048..2560].copy_from_slice(&header);

        // Sector payload at offset 1024..1536 (512 bytes) with matching CRC-32 checksum.
        let sector_payload: [u8; 512] = [0x42; 512];
        log_data[1024..1536].copy_from_slice(&sector_payload);
        let mut crc32_context: ReversedCrc32Context = ReversedCrc32Context::new(0xedb88320, 0);
        crc32_context.update(&sector_payload);
        let checksum: u32 = crc32_context.finalize();

        // Entry header at 512..1024 and backup entry at 1536..2048.
        let entry_header: &mut [u8] = &mut log_data[512..1024];
        entry_header[20..24].copy_from_slice(&512_u32.to_le_bytes());
        entry_header[24..28].copy_from_slice(&checksum.to_le_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&log_data);
        let mut relocation_log: BdeEowRelocationLog = BdeEowRelocationLog::new();
        relocation_log.read_at_position(&data_stream, log_data.len(), SeekFrom::Start(0))?;

        Ok(())
    }

    #[test]
    fn test_read_at_position_with_errors() {
        let empty_stream: DataStreamReference = open_fake_data_stream(&[]);
        let mut relocation_log: BdeEowRelocationLog = BdeEowRelocationLog::new();

        // Test unsupported data sizes (< 1024 and > 16777216).
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&empty_stream, 512, SeekFrom::Start(0));
        assert!(result.is_err());

        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&empty_stream, 16777217, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test invalid header signature.
        let invalid_header_data: Vec<u8> = vec![0; 1024];
        let invalid_header_stream: DataStreamReference =
            open_fake_data_stream(&invalid_header_data);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&invalid_header_stream, 1024, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test entry_size < 1024.
        let mut small_entry_data: Vec<u8> = vec![0; 1024];
        let header: [u8; 512] = build_header(0, 512);
        small_entry_data[0..512].copy_from_slice(&header);
        let stream: DataStreamReference = open_fake_data_stream(&small_entry_data);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&stream, 1024, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test number_of_entries out of bounds.
        let mut too_many_entries: Vec<u8> = vec![0; 2048];
        let header: [u8; 512] = build_header(5, 1024);
        too_many_entries[0..512].copy_from_slice(&header);
        let stream: DataStreamReference = open_fake_data_stream(&too_many_entries);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&stream, 2048, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test entry sectors data size out of bounds.
        let mut out_of_bounds_sectors_data: Vec<u8> = vec![0; 2560];
        let header: [u8; 512] = build_header(1, 1536);
        out_of_bounds_sectors_data[0..512].copy_from_slice(&header);
        let entry_header: &mut [u8] = &mut out_of_bounds_sectors_data[512..1024];
        entry_header[20..24].copy_from_slice(&4096_u32.to_le_bytes());
        let stream: DataStreamReference = open_fake_data_stream(&out_of_bounds_sectors_data);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&stream, 2560, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test CRC-32 checksum mismatch.
        let mut invalid_checksum_data: Vec<u8> = vec![0; 2560];
        let header: [u8; 512] = build_header(1, 1536);
        invalid_checksum_data[0..512].copy_from_slice(&header);
        let entry_header: &mut [u8] = &mut invalid_checksum_data[512..1024];
        entry_header[20..24].copy_from_slice(&512_u32.to_le_bytes());
        entry_header[24..28].copy_from_slice(&0xdeadbeef_u32.to_le_bytes());
        let stream: DataStreamReference = open_fake_data_stream(&invalid_checksum_data);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&stream, 2560, SeekFrom::Start(0));
        assert!(result.is_err());

        // Test invalid backup header signature.
        let mut invalid_backup_data: Vec<u8> = vec![0; 1024];
        let header: [u8; 512] = build_header(0, 1024);
        invalid_backup_data[0..512].copy_from_slice(&header);
        let stream: DataStreamReference = open_fake_data_stream(&invalid_backup_data);
        let result: Result<(), ErrorTrace> =
            relocation_log.read_at_position(&stream, 1024, SeekFrom::Start(0));
        assert!(result.is_err());
    }
}
