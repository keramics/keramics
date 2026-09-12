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
