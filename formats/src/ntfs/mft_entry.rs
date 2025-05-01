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

use std::collections::HashMap;
use std::io;

use core::mediator::{Mediator, MediatorReference};
use core::DataStreamReference;
use types::Ucs2String;

use super::constants::*;
use super::fixup_values::apply_fixup_values;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_attribute_group::NtfsMftAttributeGroup;
use super::mft_entry_header::NtfsMftEntryHeader;

const END_OF_ATTRIBUTES_MARKER: [u8; 4] = [0xff; 4];

/// New Technologies File System (NTFS) Master File Table (MFT) entry.
pub struct NtfsMftEntry {
    /// Mediator.
    mediator: MediatorReference,

    /// Data.
    data: Vec<u8>,

    /// Sequence number.
    pub sequence_number: u16,

    /// Base record file reference.
    pub base_record_file_reference: u64,

    /// Journal sequence number.
    pub journal_sequence_number: u64,

    /// Attributes offset.
    attributes_offset: u16,

    /// Value to indicate the MFT entry is empty.
    pub is_empty: bool,

    /// Value to indicate the MFT entry is bad.
    pub is_bad: bool,

    /// Value to indicate the MFT entry is allocated (used).
    pub is_allocated: bool,

    /// Attributes.
    pub attributes: Vec<NtfsMftAttribute>,

    /// Attribute groups per name.
    attribute_groups: HashMap<Option<Ucs2String>, NtfsMftAttributeGroup>,

    /// Indexes of data attributes in the attributes vector.
    data_attributes: Vec<usize>,

    /// Value to indicate the attributes were read.
    read_attributes: bool,
}

impl NtfsMftEntry {
    /// Creates a new MFT entry.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data: Vec::new(),
            sequence_number: 0,
            base_record_file_reference: 0,
            journal_sequence_number: 0,
            attributes_offset: 0,
            is_empty: false,
            is_bad: false,
            is_allocated: false,
            attributes: Vec::new(),
            attribute_groups: HashMap::new(),
            data_attributes: Vec::new(),
            read_attributes: false,
        }
    }

    /// Adds an attribute.
    fn add_attribute(&mut self, mut attribute: NtfsMftAttribute) -> io::Result<()> {
        if !self.attribute_groups.contains_key(&attribute.name) {
            let attribute_name: Option<Ucs2String> = match &attribute.name {
                Some(name) => Some(name.clone()),
                None => None,
            };
            let attribute_group: NtfsMftAttributeGroup = NtfsMftAttributeGroup::new();
            self.attribute_groups
                .insert(attribute_name, attribute_group);
        }
        let attribute_group: &mut NtfsMftAttributeGroup =
            match self.attribute_groups.get_mut(&attribute.name) {
                Some(attribute_group) => attribute_group,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Missing attribute group"),
                    ));
                }
            };
        match attribute_group.get_attribute_index(attribute.attribute_type) {
            Some(existing_attribute_index) => {
                let existing_attribute: &mut NtfsMftAttribute =
                    match self.attributes.get_mut(*existing_attribute_index) {
                        Some(existing_attribute) => existing_attribute,
                        None => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!("Missing attribute: {}", existing_attribute_index),
                            ));
                        }
                    };
                // TODO: check attribute.data_size

                if attribute.data_flags != existing_attribute.data_flags {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Unsupported data flags: 0x{:04x} expected: 0x{:04x}",
                            attribute.data_flags, existing_attribute.data_flags
                        ),
                    ));
                }
                // TODO: check for overlapping clusters
                for cluster_group in attribute.data_cluster_groups.drain(..) {
                    existing_attribute.append_data_cluster_group(cluster_group);
                }
                existing_attribute.sort_data_cluster_groups();
            }
            None => {
                let attribute_index: usize = self.attributes.len();
                let attribute_type: u32 = attribute.attribute_type;

                if attribute_type == NTFS_ATTRIBUTE_TYPE_DATA {
                    self.data_attributes.push(attribute_index);
                }
                attribute_group.add_attribute_index(attribute_type, attribute_index);

                self.attributes.push(attribute);
            }
        };
        Ok(())
    }

    /// Retrieves a specific attribute.
    pub fn get_attribute(
        &self,
        name: &Option<Ucs2String>,
        attribute_type: u32,
    ) -> Option<&NtfsMftAttribute> {
        match self.attribute_groups.get(name) {
            Some(attribute_group) => self.get_attribute_for_group(attribute_group, attribute_type),
            None => None,
        }
    }

    /// Retrieves a specific attribute for a specific attribute group.
    pub fn get_attribute_for_group(
        &self,
        attribute_group: &NtfsMftAttributeGroup,
        attribute_type: u32,
    ) -> Option<&NtfsMftAttribute> {
        match attribute_group.get_attribute_index(attribute_type) {
            Some(attribute_index) => self.attributes.get(*attribute_index),
            None => None,
        }
    }

    /// Retrieves a specific attribute group.
    pub fn get_attribute_group(&self, name: &Option<Ucs2String>) -> Option<&NtfsMftAttributeGroup> {
        self.attribute_groups.get(name)
    }

    /// Retrieves a specific data attribute.
    pub fn get_data_attribute_by_index(&self, attribute_index: usize) -> Option<&NtfsMftAttribute> {
        match self.data_attributes.get(attribute_index) {
            Some(attribute_index) => self.attributes.get(*attribute_index),
            None => None,
        }
    }

    /// Retrieves the number of data attributes.
    pub fn get_number_of_data_attributes(&self) -> usize {
        self.data_attributes.len()
    }

    /// Determines if the MFT entry has a specific attribute group.
    pub fn has_attribute_group(&self, name: &Option<Ucs2String>) -> bool {
        self.attribute_groups.contains_key(name)
    }

    /// Reads the attributes.
    pub fn read_attributes(&mut self) -> io::Result<()> {
        if !self.read_attributes {
            let mut data_offset: usize = self.attributes_offset as usize;
            let data_size: usize = self.data.len();

            loop {
                if data_offset > data_size - 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid data offset: {} value out of bounds", data_offset,),
                    ));
                }
                let data_end_offset: usize = data_offset + 4;
                if self.data[data_offset..data_end_offset] == END_OF_ATTRIBUTES_MARKER {
                    break;
                }
                let mut mft_attribute: NtfsMftAttribute = NtfsMftAttribute::new();

                mft_attribute.read_data(&self.data[data_offset..])?;
                data_offset += mft_attribute.attribute_size as usize;

                // TODO: handle attributes list

                self.add_attribute(mft_attribute)?;
            }
            self.read_attributes = true;
        }
        Ok(())
    }

    /// Reads the MFT entry from a buffer.
    fn read_data(&mut self, data: &mut [u8]) -> io::Result<()> {
        if data[0..4] == [4; 0] {
            self.is_empty = true;

            return Ok(());
        }
        if data[0..4] == NTFS_BAD_MFT_ENTRY_SIGNATURE {
            self.is_bad = true;

            return Ok(());
        }
        let data_size: usize = data.len();
        let mut mft_entry_header: NtfsMftEntryHeader = NtfsMftEntryHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(NtfsMftEntryHeader::debug_read_data(data));
        }
        mft_entry_header.read_data(data)?;

        if mft_entry_header.mft_entry_size as usize != data_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Mismatch between MFT entry size in header: {} and boot record: {}",
                    mft_entry_header.mft_entry_size, data_size,
                ),
            ));
        }
        if mft_entry_header.fixup_values_offset < 42
            || mft_entry_header.fixup_values_offset as usize > data_size
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid fix-up values offset: {} value out of bounds",
                    mft_entry_header.fixup_values_offset,
                ),
            ));
        }
        // In NTFS 1.2 the fix-up values offset can point to wfixupPattern.
        let header_size: u16 = if mft_entry_header.fixup_values_offset == 42 {
            42
        } else {
            48
        };
        if mft_entry_header.attributes_offset < header_size
            || mft_entry_header.attributes_offset as usize > data_size
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid attributes offset: {} value out of bounds",
                    mft_entry_header.attributes_offset,
                ),
            ));
        }
        if mft_entry_header.fixup_values_offset >= mft_entry_header.attributes_offset {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Fix-up values offset: {} exceeds attributes offset: {}",
                    mft_entry_header.fixup_values_offset, mft_entry_header.attributes_offset,
                ),
            ));
        }
        // TODO: set is_corrupted (or equiv) when fix-up values are corrupted.
        apply_fixup_values(
            data,
            mft_entry_header.fixup_values_offset,
            mft_entry_header.number_of_fixup_values,
        )?;
        self.sequence_number = mft_entry_header.sequence_number;
        self.base_record_file_reference = mft_entry_header.base_record_file_reference;
        self.journal_sequence_number = mft_entry_header.journal_sequence_number;
        self.attributes_offset = mft_entry_header.attributes_offset;
        self.is_allocated = true;

        Ok(())
    }

    /// Reads the MFT entry from a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u32,
        position: io::SeekFrom,
    ) -> io::Result<()> {
        // Note that 42 is the minimum MFT entry size and 65535 is chosen given the fix-up values
        // and attributes offsets of the MFT entry are 16-bit.
        if data_size < 42 || data_size > 65535 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported MFT entry data size: {} value out of bounds",
                    data_size
                ),
            ));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 = match data_stream.write() {
            Ok(mut data_stream) => data_stream.read_exact_at_position(&mut data, position)?,
            Err(error) => return Err(core::error_to_io_error!(error)),
        };
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "NtfsMftEntry data of size: {} at offset: {} (0x{:08x})\n",
                data_size, offset, offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        self.read_data(&mut data)?;

        self.data = data;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::open_fake_data_stream;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x46, 0x49, 0x4c, 0x45, 0x30, 0x00, 0x03, 0x00, 0x52, 0x51, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x38, 0x00, 0x01, 0x00, 0xa8, 0x01, 0x00, 0x00,
            0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0xe7, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x96, 0xd5, 0x86, 0xa0,
            0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5, 0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5,
            0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5, 0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01,
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00,
            0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x03, 0x00,
            0x4a, 0x00, 0x00, 0x00, 0x18, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x96, 0xd5, 0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5, 0x86, 0xa0,
            0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5, 0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01, 0x96, 0xd5,
            0x86, 0xa0, 0x08, 0x60, 0xd5, 0x01, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x04, 0x03, 0x24, 0x00, 0x4d, 0x00, 0x46, 0x00, 0x54, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x04,
            0xfa, 0x00, 0x21, 0x3c, 0x85, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xb0, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00, 0x40, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x01, 0xf9, 0x00, 0x21, 0x01,
            0xe7, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
            0x00, 0x00, 0x00, 0x00, 0x21, 0x04, 0xfa, 0x00, 0x21, 0x3c, 0x85, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x40, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x10,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x21, 0x01, 0xf9, 0x00, 0x21, 0x01, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x02, 0x00,
        ];
    }

    // TODO: add tests for add_attribute
    // TODO: add tests for get_attribute
    // TODO: add tests for get_attribute_for_group
    // TODO: add tests for get_attribute_group
    // TODO: add tests for get_data_attribute_by_index
    // TODO: add tests for get_number_of_data_attributes
    // TODO: add tests for has_attribute_group

    #[test]
    fn test_read_attributes() -> io::Result<()> {
        let mut test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftEntry::new();
        test_struct.read_data(&mut test_data)?;

        test_struct.data = test_data;
        test_struct.read_attributes()?;

        assert_eq!(test_struct.attribute_groups.len(), 1);

        let attribute_name: Option<Ucs2String> = None;
        let attribute_group: &NtfsMftAttributeGroup =
            test_struct.attribute_groups.get(&attribute_name).unwrap();
        assert_eq!(attribute_group.attributes.len(), 4);

        Ok(())
    }

    #[test]
    fn test_read_data() -> io::Result<()> {
        let mut test_data: Vec<u8> = get_test_data();

        assert_eq!(test_data[510..512], test_data[48..50]);
        assert_eq!(test_data[1022..1024], test_data[48..50]);

        let mut test_struct = NtfsMftEntry::new();
        test_struct.read_data(&mut test_data)?;

        assert_eq!(test_data[510..512], test_data[50..52]);
        assert_eq!(test_data[1022..1024], test_data[52..54]);

        assert_eq!(test_struct.sequence_number, 1);
        assert_eq!(test_struct.base_record_file_reference, 0);
        assert_eq!(test_struct.journal_sequence_number, 1069394);
        assert_eq!(test_struct.is_empty, false);
        assert_eq!(test_struct.is_bad, false);
        assert_eq!(test_struct.is_allocated, true);
        assert_eq!(test_struct.attribute_groups.len(), 0);

        Ok(())
    }

    #[test]
    fn test_read_data_with_unsupported_signature() {
        let mut test_data: Vec<u8> = get_test_data();
        test_data[0] = 0xff;

        let mut test_struct = NtfsMftEntry::new();
        let result = test_struct.read_data(&mut test_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_at_position() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();
        let test_data_size: u32 = test_data.len() as u32;
        let data_stream: DataStreamReference = open_fake_data_stream(test_data);

        let mut test_struct = NtfsMftEntry::new();
        test_struct.read_at_position(&data_stream, test_data_size, io::SeekFrom::Start(0))?;

        assert_eq!(test_struct.sequence_number, 1);
        assert_eq!(test_struct.base_record_file_reference, 0);
        assert_eq!(test_struct.journal_sequence_number, 1069394);
        assert_eq!(test_struct.is_empty, false);
        assert_eq!(test_struct.is_bad, false);
        assert_eq!(test_struct.is_allocated, true);
        assert_eq!(test_struct.attribute_groups.len(), 0);

        Ok(())
    }
}
