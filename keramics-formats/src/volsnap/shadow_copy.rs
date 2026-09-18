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
use keramics_datetime::DateTime;
use keramics_types::Uuid;

use super::store_block::VolsnapStoreBlock;
use super::store_metadata::VolsnapStoreMetadata;

/// Volume Shadow Snapshot (volsnap) shadow copy.
pub struct VolsnapShadowCopy {
    /// Size.
    pub size: u64,

    /// Creation time.
    pub creation_time: DateTime,

    /// Value to indicate type 2 catalog entry was read.
    pub type2_entry_read: bool,

    /// Store metadata offset.
    pub store_metadata_offset: u64,

    /// Store block list offset.
    pub store_block_list_offset: u64,

    /// Store range list offset.
    pub store_range_list_offset: u64,

    /// Store bitmap offset.
    pub store_bitmap_offset: u64,

    /// Store previous bitmap offset.
    pub store_previous_bitmap_offset: u64,

    /// Value to indicate type 3 catalog entry was read.
    pub type3_entry_read: bool,

    /// Copy identifier.
    pub copy_identifier: Uuid,

    /// Copy set identifier.
    pub copy_set_identifier: Uuid,

    /// Attribute flags.
    pub attribute_flags: u32,

    /// Value to indicate store metadata was read.
    pub store_metadata_read: bool,
}

impl VolsnapShadowCopy {
    /// Creates a new shadow copy.
    pub fn new() -> Self {
        Self {
            size: 0,
            creation_time: DateTime::NotSet,
            type2_entry_read: false,
            store_metadata_offset: 0,
            store_block_list_offset: 0,
            store_range_list_offset: 0,
            store_bitmap_offset: 0,
            store_previous_bitmap_offset: 0,
            type3_entry_read: false,
            copy_identifier: Uuid::new(),
            copy_set_identifier: Uuid::new(),
            attribute_flags: 0,
            store_metadata_read: false,
        }
    }

    /// Reads store metadata
    pub fn read_store_metadata(
        &mut self,
        data_stream: &DataStreamReference,
        offset: u64,
    ) -> Result<(), ErrorTrace> {
        let mut store_block: VolsnapStoreBlock = VolsnapStoreBlock::new();

        match store_block.read_at_position(data_stream, SeekFrom::Start(offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read store metadata block at offset: {} (0x{:08x})",
                        offset, offset
                    ),
                );
                return Err(error);
            }
        }
        if store_block.block_type != 4 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported store metadata block - unsupported block type",
            ));
        }
        if store_block.current_block_offset != 0 && store_block.current_block_offset != offset {
            return Err(keramics_core::error_trace_new!(
                "Unsupported store metadata block - current block offset value out of bounds",
            ));
        }
        if store_block.next_block_offset != 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported store metadata block - unsupported next block offset",
            ));
        }
        let data_end_offset: usize = 128 + (store_block.store_metadata_size as usize);

        if store_block.store_metadata_size < 64 || data_end_offset > store_block.data.len() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported store metadata block - invalid store metadata size value out of bounds",
            ));
        }
        keramics_core::debug_trace_data_and_structure!(
            "VolsnapStoreMetadata",
            offset + 128,
            &store_block.data[128..data_end_offset],
            store_block.store_metadata_size,
            VolsnapStoreMetadata::debug_read_data(&store_block.data[128..])
        );
        let mut store_metadata: VolsnapStoreMetadata = VolsnapStoreMetadata::new();

        match store_metadata.read_data(&store_block.data[128..]) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read store metadata");
                return Err(error);
            }
        }
        self.copy_identifier = store_metadata.copy_identifier;
        self.copy_set_identifier = store_metadata.copy_set_identifier;
        self.attribute_flags = store_metadata.attribute_flags;
        self.store_metadata_read = true;

        // TODO: read operating machine string
        // TODO: read service machine string

        Ok(())
    }
}
