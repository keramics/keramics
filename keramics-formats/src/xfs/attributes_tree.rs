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

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use crate::indexed_hash_map::IndexedHashMap;

use super::attribute::XfsAttribute;
use super::attributes_tree_leaf_entry::XfsAttributesTreeLeafEntry;
use super::attributes_tree_local_value::XfsAttributesTreeLocalValue;
use super::attributes_tree_remote_value::XfsAttributesTreeRemoteValue;
use super::block_tree_branch_entry::XfsBlockTreeBranchEntry;
use super::block_tree_branch_header::XfsBlockTreeBranchHeader;
use super::block_tree_leaf_header::XfsBlockTreeLeafHeader;
use super::file_system_block::XfsFileSystemBlock;
use super::packed_extent::XfsPackedExtent;

/// X File System (XFS) attributes tree.
pub struct XfsAttributesTree {
    /// Character encoding.
    character_encoding: CharacterEncoding,

    /// Format version.
    format_version: u16,

    /// Allocation group size.
    allocation_group_size: u32,

    /// Block size.
    block_size: u32,

    /// Block number bit shift.
    block_number_bit_shift: u64,

    /// Relative block number bit mask.
    relative_block_number_bit_mask: u64,
}

impl XfsAttributesTree {
    /// Creates a new attributes tree.
    pub fn new(
        character_encoding: &CharacterEncoding,
        format_version: u16,
        allocation_group_size: u32,
        number_of_relative_block_number_bits: u32,
        block_size: u32,
    ) -> Self {
        Self {
            character_encoding: character_encoding.clone(),
            format_version,
            allocation_group_size,
            block_size,
            block_number_bit_shift: number_of_relative_block_number_bits as u64,
            relative_block_number_bit_mask: (1 << (number_of_relative_block_number_bits as u64))
                - 1,
        }
    }

    /// Reads the attributes attributes.
    pub fn read_attributes(
        &self,
        data_stream: &DataStreamReference,
        extents: &[XfsPackedExtent],
        attributes: &mut IndexedHashMap<ByteString, XfsAttribute>,
    ) -> Result<(), ErrorTrace> {
        let mut read_block_numbers: HashSet<u32> = HashSet::new();

        match self.read_attributes_from_node(
            data_stream,
            0,
            extents,
            attributes,
            &mut read_block_numbers,
        ) {
            Ok(_) => Ok(()),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read attributes from root node: 0"
                );
                Err(error)
            }
        }
    }

    /// Reads the attributes attributes from a attributes tree branch node.
    pub fn read_attributes_from_branch_node(
        &self,
        file_system_block: &XfsFileSystemBlock,
        data_stream: &DataStreamReference,
        extents: &[XfsPackedExtent],
        attributes: &mut IndexedHashMap<ByteString, XfsAttribute>,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<(), ErrorTrace> {
        let branch_header_offset: usize;
        let branch_header_size: usize;

        if self.format_version < 5 {
            if file_system_block.signature != 0xfebe {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported file system block signature: 0x{:04x}",
                    file_system_block.signature
                )));
            }
            branch_header_offset = 12;
            branch_header_size = 4;
        } else {
            if file_system_block.signature != 0x3ebe {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported file system block signature: 0x{:04x}",
                    file_system_block.signature
                )));
            }
            branch_header_offset = 56;
            branch_header_size = 8;
        }
        let data_size: usize = file_system_block.data.len();
        let branch_header_end_offset: usize = branch_header_offset + branch_header_size;

        if branch_header_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid branch header size value out of bounds"
            ));
        }
        keramics_core::debug_trace_structure!(XfsBlockTreeBranchHeader::debug_read_data(
            &file_system_block.data[branch_header_offset..branch_header_end_offset]
        ));
        let mut branch_header: XfsBlockTreeBranchHeader = XfsBlockTreeBranchHeader::new();

        match branch_header
            .read_data(&file_system_block.data[branch_header_offset..branch_header_end_offset])
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block tree branch header"
                );
                return Err(error);
            }
        }
        let entries_data_end_offset: usize =
            branch_header_end_offset + ((branch_header.number_of_entries as usize) * 8);

        if entries_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of entries value out of bounds"
            ));
        }
        let mut data_offset: usize = branch_header_end_offset;

        for entry_index in 0..branch_header.number_of_entries {
            keramics_core::debug_trace_structure!(XfsBlockTreeBranchEntry::debug_read_data(
                &file_system_block.data[data_offset..]
            ));
            let mut entry: XfsBlockTreeBranchEntry = XfsBlockTreeBranchEntry::new();

            match entry.read_data(&file_system_block.data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read attributes tree branch entry: {}",
                            entry_index
                        ),
                    );
                    return Err(error);
                }
            }
            data_offset += 8;

            match self.read_attributes_from_node(
                data_stream,
                entry.block_number,
                extents,
                attributes,
                read_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read attributes from root node: {}",
                            entry.block_number
                        ),
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads the attributes attributes from a attributes tree leaf node.
    pub fn read_attributes_from_leaf_node(
        &self,
        file_system_block: &XfsFileSystemBlock,
        attributes: &mut IndexedHashMap<ByteString, XfsAttribute>,
    ) -> Result<(), ErrorTrace> {
        let leaf_header_offset: usize;
        let leaf_header_size: usize;

        if self.format_version < 5 {
            if file_system_block.signature != 0xfbee {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported file system block signature: 0x{:04x}",
                    file_system_block.signature
                )));
            }
            leaf_header_offset = 12;
            leaf_header_size = 20;
        } else {
            if file_system_block.signature != 0x3bee {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unsupported file system block signature: 0x{:04x}",
                    file_system_block.signature
                )));
            }
            leaf_header_offset = 56;
            leaf_header_size = 24;
        }
        let data_size: usize = file_system_block.data.len();
        let leaf_header_end_offset: usize = leaf_header_offset + leaf_header_size;

        if leaf_header_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid leaf header size value out of bounds"
            ));
        }
        keramics_core::debug_trace_structure!(XfsBlockTreeLeafHeader::debug_read_data(
            &file_system_block.data[leaf_header_offset..leaf_header_end_offset]
        ));
        let mut leaf_header: XfsBlockTreeLeafHeader = XfsBlockTreeLeafHeader::new();

        match leaf_header
            .read_data(&file_system_block.data[leaf_header_offset..leaf_header_end_offset])
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read block tree leaf header"
                );
                return Err(error);
            }
        }
        let entries_data_end_offset: usize =
            leaf_header_end_offset + ((leaf_header.number_of_entries as usize) * 8);

        if entries_data_end_offset > data_size {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of entries value out of bounds"
            ));
        }
        let mut data_offset: usize = leaf_header_end_offset;

        for entry_index in 0..leaf_header.number_of_entries {
            keramics_core::debug_trace_structure!(XfsAttributesTreeLeafEntry::debug_read_data(
                &file_system_block.data[data_offset..]
            ));
            let mut entry: XfsAttributesTreeLeafEntry = XfsAttributesTreeLeafEntry::new();

            match entry.read_data(&file_system_block.data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to read attributes tree leaf entry: {}", entry_index),
                    );
                    return Err(error);
                }
            }
            data_offset += 8;

            let value_offset: usize = entry.value_offset as usize;

            if value_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - value offset value out of bounds",
                    entry_index
                )));
            }
            let value_size: usize = if entry.attribute_flags & 0x01 == 0 {
                9
            } else {
                3
            };
            let value_end_offset: usize = value_offset + value_size;

            if value_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - value size value out of bounds",
                    entry_index
                )));
            }
            let name_size: usize;
            let value_data_size: usize;

            if entry.attribute_flags & 0x01 == 0 {
                keramics_core::debug_trace_structure!(
                    XfsAttributesTreeRemoteValue::debug_read_data(
                        &file_system_block.data[value_offset..]
                    )
                );
                let mut value: XfsAttributesTreeRemoteValue = XfsAttributesTreeRemoteValue::new();

                match value.read_data(&file_system_block.data[value_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read attributes tree leaf entry: {} remote value",
                                entry_index
                            ),
                        );
                        return Err(error);
                    }
                }
                value_data_size = value.value_data_size as usize;
                name_size = value.name_size as usize;
            } else {
                keramics_core::debug_trace_structure!(
                    XfsAttributesTreeLocalValue::debug_read_data(
                        &file_system_block.data[value_offset..]
                    )
                );
                let mut value: XfsAttributesTreeLocalValue = XfsAttributesTreeLocalValue::new();

                match value.read_data(&file_system_block.data[value_offset..]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to read attributes tree leaf entry: {} local value",
                                entry_index
                            ),
                        );
                        return Err(error);
                    }
                }
                value_data_size = value.value_data_size as usize;
                name_size = value.name_size as usize;
            }
            let name_end_offset: usize = value_end_offset + name_size;

            if name_size == 0 || name_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Invalid entry: {} - name size value out of bounds",
                    entry_index
                )));
            }
            let name: ByteString = XfsAttribute::read_name(
                &self.character_encoding,
                entry.attribute_flags,
                &file_system_block.data[value_end_offset..name_end_offset],
            );
            if entry.attribute_flags & 0x01 == 0 {
                todo!();
            } else {
                let value_data_end_offset: usize = name_end_offset + value_data_size;

                if value_data_end_offset > data_size {
                    return Err(keramics_core::error_trace_new!(format!(
                        "Invalid entry: {} - value data size value out of bounds",
                        entry_index
                    )));
                }
                let value_data: Vec<u8> =
                    file_system_block.data[name_end_offset..value_data_end_offset].to_vec();

                attributes.insert(name, XfsAttribute::InlineData(value_data));
            }
        }
        Ok(())
    }

    /// Reads the attributes attributes from a attributes tree node.
    pub fn read_attributes_from_node(
        &self,
        data_stream: &DataStreamReference,
        logical_block_number: u32,
        extents: &[XfsPackedExtent],
        attributes: &mut IndexedHashMap<ByteString, XfsAttribute>,
        read_block_numbers: &mut HashSet<u32>,
    ) -> Result<(), ErrorTrace> {
        if read_block_numbers.contains(&logical_block_number) {
            return Err(keramics_core::error_trace_new!(format!(
                "Attributes tree node: {} already read",
                logical_block_number
            )));
        }
        let extent_index: usize = match extents.binary_search_by(|extent| {
            let extent_end_block_number: u64 =
                extent.logical_block_number + (extent.number_of_blocks as u64);

            if (logical_block_number as u64) >= extent_end_block_number {
                Ordering::Less
            } else if (logical_block_number as u64) < extent.logical_block_number {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            Ok(extent_index) => extent_index,
            Err(_) => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Missing extent for node: {}",
                    logical_block_number
                )));
            }
        };
        let extent: &XfsPackedExtent = match extents.get(extent_index) {
            Some(extent) => extent,
            None => {
                return Err(keramics_core::error_trace_new!(format!(
                    "Unable to retrieve extent: {} for node: {}",
                    extent_index, logical_block_number
                )));
            }
        };
        let allocation_group_index: u64 =
            extent.physical_block_number >> self.block_number_bit_shift;
        let allocation_group_block_number: u64 =
            allocation_group_index * (self.allocation_group_size as u64);
        let relative_block_number: u64 =
            extent.physical_block_number & self.relative_block_number_bit_mask;
        let extent_block_number: u64 = (logical_block_number as u64) - extent.logical_block_number;
        let physical_block_number: u64 = relative_block_number + extent_block_number;

        let block_offset: u64 =
            (allocation_group_block_number + physical_block_number) * (self.block_size as u64);

        let mut file_system_block: XfsFileSystemBlock = XfsFileSystemBlock::new();

        match file_system_block.read_at_position(
            data_stream,
            self.block_size,
            SeekFrom::Start(block_offset),
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read allocation group: {} file system block: {}",
                        allocation_group_index, physical_block_number
                    )
                );
                return Err(error);
            }
        }
        read_block_numbers.insert(logical_block_number);

        if file_system_block.signature == 0x3bee || file_system_block.signature == 0xfbee {
            match self.read_attributes_from_leaf_node(&file_system_block, attributes) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read attributes from attributes tree leaf node: {}",
                            logical_block_number
                        ),
                    );
                    return Err(error);
                }
            }
        } else if file_system_block.signature == 0x3ebe || file_system_block.signature == 0xfebe {
            match self.read_attributes_from_branch_node(
                &file_system_block,
                data_stream,
                extents,
                attributes,
                read_block_numbers,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read attributes from attributes tree branch node: {}",
                            logical_block_number
                        ),
                    );
                    return Err(error);
                }
            }
        } else {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported allocation group: {} file system block: {} signature: 0x{:04x}",
                allocation_group_index, physical_block_number, file_system_block.signature
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::xfs::enums::XfsExtentType;

    /// Builds a synthetic XFS attributes leaf block for testing.
    fn build_attributes_leaf_block(
        format_version: u16,
        block_size: usize,
        name: &[u8],
        value: &[u8],
    ) -> Vec<u8> {
        let mut block: Vec<u8> = vec![0; block_size];
        let (signature, header_offset, entry_offset): (u16, usize, usize) = if format_version < 5 {
            (0xfbee, 12, 32)
        } else {
            (0x3bee, 56, 80)
        };
        block[8..10].copy_from_slice(&signature.to_be_bytes());
        let header_end_offset: usize = header_offset + 2;
        block[header_offset..header_end_offset].copy_from_slice(&1_u16.to_be_bytes());

        // Leaf entry: hash(4) + value_offset(2) + attribute_flags(1, 0x01 = local) + pad(1).
        let value_offset: u16 = 256;
        let entry_value_offset: usize = entry_offset + 4;
        let entry_flags_offset: usize = entry_offset + 6;
        block[entry_value_offset..entry_flags_offset].copy_from_slice(&value_offset.to_be_bytes());
        block[entry_flags_offset] = 0x01;

        // Local value at offset 256: value_data_size(2) + name_size(1) + name + value_data.
        let value_start: usize = value_offset as usize;
        let name_size_offset: usize = value_start + 2;
        let name_offset: usize = value_start + 3;
        let name_end_offset: usize = name_offset + name.len();
        let value_end_offset: usize = name_end_offset + value.len();
        let value_data_size: u16 = value.len() as u16;
        block[value_start..name_size_offset].copy_from_slice(&value_data_size.to_be_bytes());
        block[name_size_offset] = name.len() as u8;
        block[name_offset..name_end_offset].copy_from_slice(name);
        block[name_end_offset..value_end_offset].copy_from_slice(value);

        block
    }

    #[test]
    fn test_read_attributes_v4_and_v5() -> Result<(), ErrorTrace> {
        let block_size: usize = 512;

        for format_version in [4, 5] {
            let mut image: Vec<u8> = vec![0; 2 * block_size];
            let (branch_signature, header_offset, entry_offset): (u16, usize, usize) =
                if format_version < 5 {
                    (0xfebe, 12, 16)
                } else {
                    (0x3ebe, 56, 64)
                };
            // Block 0: branch node pointing to logical block 1.
            image[8..10].copy_from_slice(&branch_signature.to_be_bytes());
            let header_end_offset: usize = header_offset + 2;
            image[header_offset..header_end_offset].copy_from_slice(&1_u16.to_be_bytes());
            let entry_block_offset: usize = entry_offset + 4;
            let entry_block_end_offset: usize = entry_offset + 8;
            image[entry_block_offset..entry_block_end_offset].copy_from_slice(&1_u32.to_be_bytes());

            // Block 1: leaf node with a local inline attribute.
            let leaf: Vec<u8> =
                build_attributes_leaf_block(format_version, block_size, b"attr1", b"val1");
            let leaf_end_offset: usize = 2 * block_size;
            image[block_size..leaf_end_offset].copy_from_slice(&leaf);

            let data_stream: DataStreamReference = open_fake_data_stream(&image);
            let attributes_tree: XfsAttributesTree = XfsAttributesTree::new(
                &CharacterEncoding::Utf8,
                format_version,
                16,
                4,
                block_size as u32,
            );
            let extents: [XfsPackedExtent; 1] = [XfsPackedExtent {
                number_of_blocks: 2,
                physical_block_number: 0,
                logical_block_number: 0,
                extent_type: XfsExtentType::InFile,
            }];
            let mut attributes: IndexedHashMap<ByteString, XfsAttribute> = IndexedHashMap::new();
            attributes_tree.read_attributes(&data_stream, &extents, &mut attributes)?;

            assert_eq!(attributes.len(), 1);
            let attribute_name: ByteString = ByteString::from("user.attr1");
            let attribute: &XfsAttribute = attributes.get_value_by_key(&attribute_name).unwrap();
            let XfsAttribute::InlineData(data) = attribute;
            assert_eq!(data, b"val1");
        }
        Ok(())
    }

    #[test]
    fn test_read_attributes_with_errors() {
        let block_size: usize = 512;
        let mut image: Vec<u8> = vec![0; 2 * block_size];
        image[8..10].copy_from_slice(&0x1234_u16.to_be_bytes());
        let block1_end_offset: usize = 2 * block_size;
        let block1: &mut [u8] = &mut image[block_size..block1_end_offset];
        block1[8..10].copy_from_slice(&0xfebe_u16.to_be_bytes());
        block1[12..14].copy_from_slice(&1_u16.to_be_bytes());
        block1[20..24].copy_from_slice(&1_u32.to_be_bytes());

        let data_stream: DataStreamReference = open_fake_data_stream(&image);
        let attributes_tree_v4: XfsAttributesTree =
            XfsAttributesTree::new(&CharacterEncoding::Utf8, 4, 16, 4, block_size as u32);
        let attributes_tree_v5: XfsAttributesTree =
            XfsAttributesTree::new(&CharacterEncoding::Utf8, 5, 16, 4, block_size as u32);
        let extents: [XfsPackedExtent; 1] = [XfsPackedExtent {
            number_of_blocks: 2,
            physical_block_number: 0,
            logical_block_number: 0,
            extent_type: XfsExtentType::InFile,
        }];
        let mut attributes: IndexedHashMap<ByteString, XfsAttribute> = IndexedHashMap::new();

        // Test missing extent for root block 0.
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes(&data_stream, &[], &mut attributes);
        assert!(result.is_err());

        // Test unsupported block signature at block 0.
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes(&data_stream, &extents, &mut attributes);
        assert!(result.is_err());

        // Test branch node cycle detection at block 1.
        let mut read_blocks: HashSet<u32> = HashSet::new();
        let result: Result<(), ErrorTrace> = attributes_tree_v4.read_attributes_from_node(
            &data_stream,
            1,
            &extents,
            &mut attributes,
            &mut read_blocks,
        );
        assert!(result.is_err());

        // Test branch node errors for v4 and v5 (wrong signature, short data, entries out of bounds).
        for (tree, valid_signature, header_offset) in [
            (&attributes_tree_v4, 0xfebe, 12),
            (&attributes_tree_v5, 0x3ebe, 56),
        ] {
            let mut block: XfsFileSystemBlock = XfsFileSystemBlock::new();
            let mut read_blocks: HashSet<u32> = HashSet::new();
            block.signature = 0x0000;
            let result: Result<(), ErrorTrace> = tree.read_attributes_from_branch_node(
                &block,
                &data_stream,
                &extents,
                &mut attributes,
                &mut read_blocks,
            );
            assert!(result.is_err());

            block.signature = valid_signature;
            block.data = vec![0; header_offset];
            let result: Result<(), ErrorTrace> = tree.read_attributes_from_branch_node(
                &block,
                &data_stream,
                &extents,
                &mut attributes,
                &mut read_blocks,
            );
            assert!(result.is_err());

            block.data = vec![0; header_offset + 12];
            let header_end_offset: usize = header_offset + 2;
            block.data[header_offset..header_end_offset].copy_from_slice(&10_u16.to_be_bytes());
            let result: Result<(), ErrorTrace> = tree.read_attributes_from_branch_node(
                &block,
                &data_stream,
                &extents,
                &mut attributes,
                &mut read_blocks,
            );
            assert!(result.is_err());
        }

        // Test leaf node errors for v4 and v5.
        for (tree, valid_signature, header_offset) in [
            (&attributes_tree_v4, 0xfbee, 12),
            (&attributes_tree_v5, 0x3bee, 56),
        ] {
            let mut leaf_block: XfsFileSystemBlock = XfsFileSystemBlock::new();
            leaf_block.signature = 0x0000;
            let result: Result<(), ErrorTrace> =
                tree.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
            assert!(result.is_err());

            leaf_block.signature = valid_signature;
            leaf_block.data = vec![0; header_offset + 4];
            let result: Result<(), ErrorTrace> =
                tree.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
            assert!(result.is_err());
        }

        // Test leaf entry out-of-bounds cases on v4 leaf node.
        let mut leaf_block: XfsFileSystemBlock = XfsFileSystemBlock::new();
        leaf_block.signature = 0xfbee;
        leaf_block.data = vec![0; 48];
        leaf_block.data[12..14].copy_from_slice(&10_u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());

        // Value offset out of bounds.
        leaf_block.data = vec![0; 64];
        leaf_block.data[12..14].copy_from_slice(&1_u16.to_be_bytes());
        leaf_block.data[36..38].copy_from_slice(&100_u16.to_be_bytes());
        leaf_block.data[38] = 0x01;
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());

        // Value size out of bounds (both local and remote flag).
        leaf_block.data[36..38].copy_from_slice(&63_u16.to_be_bytes());
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());

        leaf_block.data[38] = 0x00;
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());

        // Zero name size.
        leaf_block.data[36..38].copy_from_slice(&48_u16.to_be_bytes());
        leaf_block.data[38] = 0x01;
        leaf_block.data[50] = 0;
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());

        // Value data size out of bounds.
        leaf_block.data[48..50].copy_from_slice(&50_u16.to_be_bytes());
        leaf_block.data[50] = 2;
        let result: Result<(), ErrorTrace> =
            attributes_tree_v4.read_attributes_from_leaf_node(&leaf_block, &mut attributes);
        assert!(result.is_err());
    }
}
