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
use keramics_types::{Uuid, bytes_to_u64_le};

use crate::indexed_hash_map::IndexedHashMap;

use super::catalog_block_header::VolsnapCatalogBlockHeader;
use super::catalog_entry_type2::VolsnapCatalogEntryType2;
use super::catalog_entry_type3::VolsnapCatalogEntryType3;
use super::shadow_copy::VolsnapShadowCopy;

/// Volume Shadow Snapshot (volsnap) catalog block.
pub struct VolsnapCatalogBlock {
    /// Current block offset.
    pub current_block_offset: u64,

    /// Next block offset.
    pub next_block_offset: u64,
}

impl VolsnapCatalogBlock {
    /// Creates a new catalog block.
    pub fn new() -> Self {
        Self {
            current_block_offset: 0,
            next_block_offset: 0,
        }
    }

    /// Reads the catalog block from a buffer.
    fn read_data(
        &mut self,
        data: &[u8],
        offset: u64,
        shadow_copies: &mut IndexedHashMap<Uuid, VolsnapShadowCopy>,
    ) -> Result<(), ErrorTrace> {
        let data_size: usize = data.len();

        keramics_core::debug_trace_data_and_structure!(
            "VolsnapCatalogBlockHeader",
            offset,
            &data[0..128],
            128,
            VolsnapCatalogBlockHeader::debug_read_data(data)
        );
        let mut block_header: VolsnapCatalogBlockHeader = VolsnapCatalogBlockHeader::new();

        match block_header.read_data(data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read catalog block header");
                return Err(error);
            }
        }
        self.current_block_offset = block_header.current_block_offset;
        self.next_block_offset = block_header.next_block_offset;

        let mut data_offset: usize = 128;
        let mut entry_index: usize = 0;

        while data_offset < data_size {
            let data_end_offset: usize = data_offset + 128;

            let entry_type: u64 = bytes_to_u64_le!(data, data_offset);

            match entry_type {
                0 | 1 => {
                    if data[data_offset + 8..data_end_offset] != [0; 120] {
                        return Err(keramics_core::error_trace_new!(format!(
                            "Unsupported type {} entry: {} - value is non-empty",
                            entry_type, entry_index
                        )));
                    }
                }
                2 => {
                    keramics_core::debug_trace_data_and_structure!(
                        format!("VolsnapCatalogEntryType2: {}", entry_index),
                        offset + (data_offset as u64),
                        &data[data_offset..data_end_offset],
                        128,
                        VolsnapCatalogEntryType2::debug_read_data(&data[data_offset..])
                    );
                    let mut entry: VolsnapCatalogEntryType2 = VolsnapCatalogEntryType2::new();

                    match entry.read_data(&data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read type 2 entry: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                    if !shadow_copies.contains_key(&entry.store_identifier) {
                        let shadow_copy: VolsnapShadowCopy = VolsnapShadowCopy::new();

                        _ = shadow_copies.insert(entry.store_identifier.clone(), shadow_copy);
                    }
                    let shadow_copy: &mut VolsnapShadowCopy = match shadow_copies
                        .get_value_by_key_mut(&entry.store_identifier)
                    {
                        Some(shadow_copy) => shadow_copy,
                        None => return Err(keramics_core::error_trace_new!("Missing shadow copy")),
                    };
                    shadow_copy.size = entry.size;
                    shadow_copy.creation_time = entry.creation_time;
                    shadow_copy.type2_entry_read = true;
                }
                3 => {
                    keramics_core::debug_trace_data_and_structure!(
                        format!("VolsnapCatalogEntryType3: {}", entry_index),
                        offset + (data_offset as u64),
                        &data[data_offset..data_end_offset],
                        128,
                        VolsnapCatalogEntryType3::debug_read_data(&data[data_offset..])
                    );
                    let mut entry: VolsnapCatalogEntryType3 = VolsnapCatalogEntryType3::new();

                    match entry.read_data(&data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!("Unable to read type 3 entry: {}", entry_index)
                            );
                            return Err(error);
                        }
                    }
                    if !shadow_copies.contains_key(&entry.store_identifier) {
                        let shadow_copy: VolsnapShadowCopy = VolsnapShadowCopy::new();

                        _ = shadow_copies.insert(entry.store_identifier.clone(), shadow_copy);
                    }
                    let shadow_copy: &mut VolsnapShadowCopy = match shadow_copies
                        .get_value_by_key_mut(&entry.store_identifier)
                    {
                        Some(shadow_copy) => shadow_copy,
                        None => return Err(keramics_core::error_trace_new!("Missing shadow copy")),
                    };
                    shadow_copy.store_block_list_offset = entry.store_block_list_offset;
                    shadow_copy.store_metadata_offset = entry.store_metadata_offset;
                    shadow_copy.store_range_list_offset = entry.store_range_list_offset;
                    shadow_copy.store_bitmap_offset = entry.store_bitmap_offset;
                    shadow_copy.store_previous_bitmap_offset = entry.store_previous_bitmap_offset;
                    shadow_copy.type3_entry_read = true;
                }
                _ => {
                    keramics_core::debug_trace_data!(
                        format!("VolsnapCatalogEntryType{}: {}", entry_type, entry_index),
                        offset + (data_offset as u64),
                        &data[data_offset..data_end_offset],
                        128,
                    );
                    return Err(keramics_core::error_trace_new!(format!(
                        "Unsupported type {} entry: {}",
                        entry_type, entry_index
                    )));
                }
            }
            data_offset = data_end_offset;
            entry_index += 1;
        }
        Ok(())
    }

    /// Reads the catalog block from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        position: SeekFrom,
        shadow_copies: &mut IndexedHashMap<Uuid, VolsnapShadowCopy>,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; 16384];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);

        match self.read_data(&data, offset, shadow_copies) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read catalog block at offset: {} (0x{:08x})",
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

    use std::io::SeekFrom;

    use keramics_core::{DataStreamReference, open_fake_data_stream};

    fn get_test_data() -> Vec<u8> {
        let mut test_data: Vec<u8> = vec![0; 16384];
        test_data[0..640].copy_from_slice(&[
            0x6b, 0x87, 0x08, 0x38, 0x76, 0xc1, 0x48, 0x4e, 0xb7, 0xae, 0x04, 0x04, 0x6e, 0x6c,
            0xc7, 0x52, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0,
            0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xef, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x9b, 0x81, 0x17, 0x9f, 0xf9, 0xb0, 0xf1, 0x11, 0x90, 0xdc,
            0x7c, 0xed, 0x8d, 0x4e, 0x4e, 0x79, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x43, 0x9f, 0xad, 0x0e, 0x45,
            0xdd, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40,
            0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x9b, 0x81, 0x17, 0x9f, 0xf9, 0xb0, 0xf1, 0x11,
            0x90, 0xdc, 0x7c, 0xed, 0x8d, 0x4e, 0x4e, 0x79, 0x00, 0x00, 0xa9, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x80, 0xa9, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xaa, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xef, 0x07, 0x00, 0x00, 0x00, 0x00, 0xa3, 0x81, 0x17, 0x9f, 0xf9, 0xb0,
            0xf1, 0x11, 0x90, 0xdc, 0x7c, 0xed, 0x8d, 0x4e, 0x4e, 0x79, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x25, 0x94,
            0x8f, 0xb9, 0x0e, 0x45, 0xdd, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x40, 0xa9, 0x04, 0x00, 0x00, 0x00, 0x00, 0xa3, 0x81, 0x17, 0x9f,
            0xf9, 0xb0, 0xf1, 0x11, 0x90, 0xdc, 0x7c, 0xed, 0x8d, 0x4e, 0x4e, 0x79, 0x00, 0x00,
            0xa9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xa9, 0x04, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x80, 0xaa, 0x04, 0x00, 0x00, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xaa, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        test_data
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VolsnapCatalogBlock::new();
        let mut shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy> = IndexedHashMap::new();
        test_struct.read_data(&test_data, 0, &mut shadow_copies)?;

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_data_size() {
        let mut test_struct = VolsnapCatalogBlock::new();

        let test_data: Vec<u8> = get_test_data();
        let mut shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy> = IndexedHashMap::new();
        let result = test_struct.read_data(&test_data[0..127], 0, &mut shadow_copies);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = VolsnapCatalogBlock::new();
        let mut shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy> = IndexedHashMap::new();
        let result = test_struct.read_data(&test_data, 0, &mut shadow_copies);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let mut test_struct = VolsnapCatalogBlock::new();
        let mut shadow_copies: IndexedHashMap<Uuid, VolsnapShadowCopy> = IndexedHashMap::new();
        test_struct.read_at_position(&data_stream, SeekFrom::Start(0), &mut shadow_copies)?;

        Ok(())
    }
}
