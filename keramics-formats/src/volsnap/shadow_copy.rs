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

use crate::block_tree::BlockTree;

use super::block_descriptor::VolsnapBlockDescriptor;
use super::store_bitmap::VolsnapStoreBitmap;
use super::store_block::VolsnapStoreBlock;
use super::store_block_list::VolsnapStoreBlockList;
use super::store_metadata::VolsnapStoreMetadata;

#[cfg(feature = "debug-trace")]
use super::store_range_list::VolsnapStoreRangeList;

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

    /// Store index.
    pub store_index: usize,

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

    /// Store bitmap.
    pub store_bitmap: VolsnapStoreBitmap,

    /// Store previous bitmap.
    pub store_previous_bitmap: VolsnapStoreBitmap,

    /// Forward block tree.
    pub forward_block_tree: BlockTree<VolsnapBlockDescriptor>,

    /// Reverse block tree.
    pub reverse_block_tree: BlockTree<VolsnapBlockDescriptor>,
}

impl VolsnapShadowCopy {
    /// Creates a new shadow copy.
    pub fn new(block_size: u16) -> Self {
        Self {
            size: 0,
            creation_time: DateTime::NotSet,
            type2_entry_read: false,
            store_metadata_offset: 0,
            store_block_list_offset: 0,
            store_range_list_offset: 0,
            store_bitmap_offset: 0,
            store_previous_bitmap_offset: 0,
            store_index: 0,
            type3_entry_read: false,
            copy_identifier: Uuid::new(),
            copy_set_identifier: Uuid::new(),
            attribute_flags: 0,
            store_metadata_read: false,
            store_bitmap: VolsnapStoreBitmap::new(block_size),
            store_previous_bitmap: VolsnapStoreBitmap::new(block_size),
            forward_block_tree: BlockTree::<VolsnapBlockDescriptor>::new(0, 0, 0),
            reverse_block_tree: BlockTree::<VolsnapBlockDescriptor>::new(0, 0, 0),
        }
    }

    /// Initializes the block trees.
    pub fn initialize_block_trees(
        &self,
        store_block_list: &VolsnapStoreBlockList,
        forward_block_tree: &mut BlockTree<VolsnapBlockDescriptor>,
        reverse_block_tree: &mut BlockTree<VolsnapBlockDescriptor>,
    ) -> Result<(), ErrorTrace> {
        if !self.type2_entry_read {
            return Err(keramics_core::error_trace_new!(
                "Missing type 2 catalog entry"
            ));
        }
        for block_descriptor in store_block_list.block_descriptors.iter() {
            if block_descriptor.is_unused() {
                continue;
            }
            let mut original_offset: u64 = block_descriptor.original_offset;

            if !block_descriptor.is_overlay() {
                let mut relative_offset: u64 = 0;

                let found_reverse_block_descriptor: bool = match reverse_block_tree
                    .get_value(original_offset)
                {
                    Ok(Some(reverse_block_descriptor)) => {
                        original_offset = reverse_block_descriptor.original_offset;
                        relative_offset = reverse_block_descriptor.relative_offset;

                        true
                    }
                    Ok(None) => false,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve block descriptor: {} (0x{:08x}) from reverse block tree",
                                original_offset, original_offset
                            )
                        );
                        return Err(error);
                    }
                };
                if found_reverse_block_descriptor {
                    match reverse_block_tree.remove_value(relative_offset, 16384) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to remove block descriptor: {} (0x{:08x}) from reverse block tree",
                                    relative_offset, relative_offset
                                )
                            );
                            return Err(error);
                        }
                    }
                }
            }
            if block_descriptor.is_forwarder() {
                if original_offset == block_descriptor.relative_offset {
                    continue;
                }
            }
            let mut forward_block_descriptor: VolsnapBlockDescriptor = block_descriptor.clone();
            forward_block_descriptor.original_offset = original_offset;

            match forward_block_tree.get_value(original_offset) {
                Ok(Some(existing_block_descriptor)) => {
                    if forward_block_descriptor.is_overlay() {
                        if existing_block_descriptor.is_overlay() {
                            forward_block_descriptor.bitmap |= existing_block_descriptor.bitmap;
                        } else if let Some(overlay_block_descriptor) =
                            &existing_block_descriptor.overlay
                        {
                            forward_block_descriptor.bitmap |= overlay_block_descriptor.bitmap;
                        } else {
                            let overlay: Option<Box<VolsnapBlockDescriptor>> =
                                Some(Box::new(forward_block_descriptor));

                            forward_block_descriptor = existing_block_descriptor.clone();
                            forward_block_descriptor.overlay = overlay;
                        }
                    } else {
                        if existing_block_descriptor.is_overlay() {
                            forward_block_descriptor.overlay =
                                Some(Box::new(existing_block_descriptor.clone()));
                        } else {
                            forward_block_descriptor.overlay =
                                existing_block_descriptor.overlay.clone();
                        }
                    }
                }
                Ok(None) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve block descriptor: {} (0x{:08x}) from forward block tree",
                            original_offset, original_offset
                        )
                    );
                    return Err(error);
                }
            };
            match forward_block_tree.insert_value(original_offset, 16384, forward_block_descriptor)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to insert block descriptor: {} (0x{:08x}) into forward block tree",
                            original_offset, original_offset
                        )
                    );
                    return Err(error);
                }
            }
            if block_descriptor.is_forwarder() {
                let relative_offset: u64 = block_descriptor.relative_offset;
                let reverse_block_descriptor: VolsnapBlockDescriptor = block_descriptor.clone();

                match reverse_block_tree.insert_value(
                    relative_offset,
                    16384,
                    reverse_block_descriptor,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to insert block descriptor: {} (0x{:08x}) into reverse block tree",
                                relative_offset, relative_offset
                            )
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }

    /// Reads the store metadata.
    pub fn read_store_metadata(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        if !self.type3_entry_read {
            return Err(keramics_core::error_trace_new!(
                "Missing store metadata offset"
            ));
        }
        let mut store_block: VolsnapStoreBlock = VolsnapStoreBlock::new();

        match store_block.read_at_position(data_stream, SeekFrom::Start(self.store_metadata_offset))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read store metadata block at offset: {} (0x{:08x})",
                        self.store_metadata_offset, self.store_metadata_offset
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
        if store_block.current_block_offset != 0
            && store_block.current_block_offset != self.store_metadata_offset
        {
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
            self.store_metadata_offset + 128,
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

        match self
            .store_bitmap
            .read_at_offset(data_stream, self.store_bitmap_offset)
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read store bitmap");
                return Err(error);
            }
        }
        if self.store_previous_bitmap_offset != 0 {
            match self
                .store_previous_bitmap
                .read_at_offset(data_stream, self.store_previous_bitmap_offset)
            {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read store previous bitmap"
                    );
                    return Err(error);
                }
            }
        }
        #[cfg(feature = "debug-trace")]
        {
            let mut store_range_list: VolsnapStoreRangeList = VolsnapStoreRangeList::new();

            match store_range_list.read_at_offset(data_stream, self.store_range_list_offset) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read store range list");
                    return Err(error);
                }
            }
        }
        let mut store_block_list: VolsnapStoreBlockList = VolsnapStoreBlockList::new();

        match store_block_list.read_at_offset(data_stream, self.store_block_list_offset) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read store block list");
                return Err(error);
            }
        }
        if self.type2_entry_read {
            let mut forward_block_tree: BlockTree<VolsnapBlockDescriptor> =
                BlockTree::<VolsnapBlockDescriptor>::new(self.size, 0, 16384);
            let mut reverse_block_tree: BlockTree<VolsnapBlockDescriptor> =
                BlockTree::<VolsnapBlockDescriptor>::new(self.size, 0, 16384);

            match self.initialize_block_trees(
                &store_block_list,
                &mut forward_block_tree,
                &mut reverse_block_tree,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to initialize block trees"
                    );
                    return Err(error);
                }
            }
            self.forward_block_tree = forward_block_tree;
            self.reverse_block_tree = reverse_block_tree;
        }
        Ok(())
    }
}
