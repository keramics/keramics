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

use std::cmp::min;
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::ErrorTrace;

use crate::traits::BlockReader;

use super::block_overlay_range::VolsnapBlockOverlayRange;
use super::block_range::VolsnapBlockRange;
use super::shadow_copy::VolsnapShadowCopy;
use super::volume::VolsnapVolume;

/// Volume Shadow Snapshot (volsnap) block reader.
pub struct VolsnapBlockReader {
    /// Snapshot (or source) volume.
    snapshot_volume: Arc<VolsnapVolume>,

    /// Storage volume.
    storage_volume: Option<Arc<VolsnapVolume>>,

    /// Block size.
    block_size: u32,

    /// Active shadow copy index.
    active_shadow_copy_index: usize,

    /// Number of shadow copies.
    number_of_shadow_copies: usize,

    /// Size.
    size: u64,
}

impl VolsnapBlockReader {
    /// Creates a block reader.
    pub fn new(
        snapshot_volume: &Arc<VolsnapVolume>,
        storage_volume: Option<&Arc<VolsnapVolume>>,
        block_size: u32,
        shadow_copy: &VolsnapShadowCopy,
    ) -> Self {
        let number_of_shadow_copies: usize = snapshot_volume.shadow_copies.len();

        Self {
            snapshot_volume: snapshot_volume.clone(),
            storage_volume: storage_volume.cloned(),
            block_size,
            active_shadow_copy_index: shadow_copy.store_index,
            number_of_shadow_copies,
            size: shadow_copy.size,
        }
    }

    /// Determines if the block range is sparse.
    pub fn check_if_sparse(
        &self,
        shadow_copy_index: usize,
        offset: u64,
    ) -> Result<bool, ErrorTrace> {
        let shadow_copy: &VolsnapShadowCopy = match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_index(shadow_copy_index)
        {
            Some(shadow_copy) => shadow_copy,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing shadow copy: {}",
                    shadow_copy_index
                )));
            }
        };
        let in_bitmap: bool = match shadow_copy.store_bitmap.check_if_set(offset) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to determine if block offset: {} (0x{:08x}) is set in (current) bitmap",
                        offset, offset
                    ),
                );
                return Err(error);
            }
        };
        let in_previous_bitmap: bool = if shadow_copy.store_previous_bitmap_offset == 0 {
            true
        } else {
            match shadow_copy.store_previous_bitmap.check_if_set(offset) {
                Ok(result) => result,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to determine if block offset: {} (0x{:08x}) is set in previous bitmap",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            }
        };
        let has_reverse_block_descriptor: bool =
            match shadow_copy.reverse_block_tree.get_value(offset) {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to retrieve reverse block descriptor for offset: {} (0x{:08x})",
                            offset, offset
                        ),
                    );
                    return Err(error);
                }
            };
        Ok(in_bitmap && in_previous_bitmap && !has_reverse_block_descriptor)
    }

    /// Determines the block range.
    pub fn get_range(
        &self,
        shadow_copy_index: usize,
        offset: u64,
    ) -> Result<VolsnapBlockRange, ErrorTrace> {
        let shadow_copy: &VolsnapShadowCopy = match self
            .snapshot_volume
            .shadow_copies
            .get_value_by_index(shadow_copy_index)
        {
            Some(shadow_copy) => shadow_copy,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing shadow copy: {}",
                    shadow_copy_index
                )));
            }
        };
        let relative_block_offset: u64 = offset % (self.block_size as u64);
        let mut range_offset: u64 = offset;
        let mut range_size: u32 = self.block_size - (relative_block_offset as u32);
        let mut in_block_list: bool = false;
        let mut is_forwarder: bool = false;

        match shadow_copy.forward_block_tree.get_value(offset) {
            Ok(Some(block_descriptor)) => {
                range_offset = relative_block_offset
                    + if block_descriptor.is_forwarder() {
                        block_descriptor.relative_offset
                    } else {
                        block_descriptor.offset
                    };
                if shadow_copy_index != self.active_shadow_copy_index {
                    in_block_list = !block_descriptor.is_overlay();
                } else {
                    if block_descriptor.is_overlay() {
                        let overlay_range: VolsnapBlockOverlayRange = block_descriptor
                            .get_overlay_range(
                                relative_block_offset,
                                self.snapshot_volume.bytes_per_sector,
                            );
                        if overlay_range.is_set {
                            range_offset = block_descriptor.offset + relative_block_offset;
                        }
                        range_size = overlay_range.end_offset - (relative_block_offset as u32);
                        in_block_list = overlay_range.is_set;
                    } else if let Some(overlay_block_descriptor) = &block_descriptor.overlay {
                        let overlay_range: VolsnapBlockOverlayRange = overlay_block_descriptor
                            .get_overlay_range(
                                relative_block_offset,
                                self.snapshot_volume.bytes_per_sector,
                            );
                        if overlay_range.is_set {
                            range_offset = overlay_block_descriptor.offset + relative_block_offset;
                        }
                        range_size = overlay_range.end_offset - (relative_block_offset as u32);
                        in_block_list = true;
                    } else {
                        in_block_list = true;
                    }
                }
                is_forwarder = block_descriptor.is_forwarder();
            }
            Ok(None) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve forward block descriptor for offset: {} (0x{:08x})",
                        offset, offset
                    ),
                );
                return Err(error);
            }
        }
        Ok(VolsnapBlockRange::new(
            range_offset,
            range_size,
            in_block_list,
            is_forwarder,
        ))
    }
}

impl BlockReader for VolsnapBlockReader {
    /// Retrieves the size of the data.
    fn get_size(&self) -> u64 {
        self.size
    }

    /// Reads data based on the block ranges in the block tree.
    fn read_data_from_blocks(&mut self, data: &mut [u8], offset: u64) -> Result<usize, ErrorTrace> {
        let read_size: usize = data.len();
        let mut data_offset: usize = 0;
        let mut current_offset: u64 = offset;

        while data_offset < read_size {
            if current_offset >= self.size {
                break;
            }
            let mut block_range: VolsnapBlockRange = match self
                .get_range(self.active_shadow_copy_index, current_offset)
            {
                Ok(block_range) => block_range,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to determine block range for offset: {} (0x{:08x}) in shadow copy: {}",
                            current_offset, current_offset, self.active_shadow_copy_index
                        ),
                    );
                    return Err(error);
                }
            };
            if !block_range.in_block_list || block_range.is_forwarder {
                let mut shadow_copy_index = self.active_shadow_copy_index + 1;
                let mut block_offset: u64 = block_range.offset;

                while shadow_copy_index < self.number_of_shadow_copies {
                    let next_block_range: VolsnapBlockRange = match self
                        .get_range(shadow_copy_index, block_offset)
                    {
                        Ok(block_range) => block_range,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine block range for offset: {} (0x{:08x}) in shadow copy: {}",
                                    block_offset, block_offset, shadow_copy_index
                                ),
                            );
                            return Err(error);
                        }
                    };
                    block_range.offset = next_block_range.offset;
                    block_range.size = min(block_range.size, next_block_range.size);
                    block_range.in_block_list = next_block_range.in_block_list;
                    block_range.is_forwarder = next_block_range.is_forwarder;

                    if next_block_range.in_block_list && !next_block_range.is_forwarder {
                        break;
                    }
                    block_offset = next_block_range.offset;
                    shadow_copy_index += 1;
                }
            }
            if !block_range.in_block_list {
                if self.active_shadow_copy_index + 1 == self.number_of_shadow_copies {
                    block_range.is_sparse = match self
                        .check_if_sparse(self.active_shadow_copy_index, current_offset)
                    {
                        Ok(result) => result,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to determine block range for offset: {} (0x{:08x}) in shadow copy: {}",
                                    current_offset, current_offset, self.active_shadow_copy_index
                                ),
                            );
                            return Err(error);
                        }
                    };
                }
            }
            let range_read_size: usize = min(read_size - data_offset, block_range.size as usize);
            let data_end_offset: usize = data_offset + range_read_size;

            if block_range.is_sparse {
                data[data_offset..data_end_offset].fill(0);
            } else if block_range.in_block_list {
                match self.snapshot_volume.data_stream.as_ref() {
                    Some(data_stream) => {
                        keramics_core::data_stream_read_exact_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(block_range.offset)
                        );
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing snapshot volume data stream"
                        ));
                    }
                }
            } else {
                match self.snapshot_volume.data_stream.as_ref() {
                    Some(data_stream) => {
                        keramics_core::data_stream_read_exact_at_position!(
                            data_stream,
                            &mut data[data_offset..data_end_offset],
                            SeekFrom::Start(current_offset)
                        );
                    }
                    None => {
                        return Err(keramics_core::error_trace_new!(
                            "Missing snapshot volume data stream"
                        ));
                    }
                }
            }
            data_offset = data_end_offset;
            current_offset += range_read_size as u64;
        }
        Ok(data_offset)
    }
}
