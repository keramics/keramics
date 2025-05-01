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

use std::io;

use core::mediator::{Mediator, MediatorReference};
use types::Ucs2String;

use super::cluster_group::NtfsClusterGroup;
use super::mft_attribute_header::NtfsMftAttributeHeader;
use super::mft_attribute_non_resident::NtfsMftAttributeNonResident;
use super::mft_attribute_resident::NtfsMftAttributeResident;

/// New Technologies File System (NTFS) Master File Table (MFT) attribute.
pub struct NtfsMftAttribute {
    /// Mediator.
    mediator: MediatorReference,

    /// Attribute type.
    pub attribute_type: u32,

    /// Attribute size.
    pub attribute_size: u32,

    /// Name.
    pub name: Option<Ucs2String>,

    /// Allocated data size.
    pub allocated_data_size: u64,

    /// Data size.
    pub data_size: u64,

    /// Valid data size.
    pub valid_data_size: u64,

    /// Compression unit size.
    pub compression_unit_size: u32,

    /// Compressed data size.
    pub compressed_data_size: u64,

    /// Resident data.
    pub resident_data: Vec<u8>,

    /// Data cluster groups.
    pub data_cluster_groups: Vec<NtfsClusterGroup>,

    /// Non-resident flag.
    pub non_resident_flag: u8,

    /// Data flags.
    pub data_flags: u16,
}

impl NtfsMftAttribute {
    /// Creates a new MFT attribute.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            attribute_type: 0,
            attribute_size: 0,
            name: None,
            allocated_data_size: 0,
            data_size: 0,
            valid_data_size: 0,
            compression_unit_size: 0,
            compressed_data_size: 0,
            resident_data: Vec::new(),
            data_cluster_groups: Vec::new(),
            non_resident_flag: 0,
            data_flags: 0,
        }
    }

    /// Appends a data cluster group.
    pub fn append_data_cluster_group(&mut self, cluster_group: NtfsClusterGroup) {
        self.data_cluster_groups.push(cluster_group);
    }

    /// Determines if the MFT attribute is compressed.
    pub fn is_compressed(&self) -> bool {
        self.data_flags & 0x00ff != 0
    }

    /// Determines if the MFT attribute is resident.
    pub fn is_resident(&self) -> bool {
        self.non_resident_flag & 0x01 == 0
    }

    /// Determines if the MFT attribute is sparse.
    pub fn is_sparse(&self) -> bool {
        self.data_flags & 0x8000 != 0
    }

    /// Reads the MFT attribute from a buffer.
    pub fn read_data(&mut self, data: &[u8]) -> io::Result<()> {
        let data_size: usize = data.len();

        let mut mft_attribute_header: NtfsMftAttributeHeader = NtfsMftAttributeHeader::new();

        if self.mediator.debug_output {
            self.mediator
                .debug_print(NtfsMftAttributeHeader::debug_read_data(data));
        }
        mft_attribute_header.read_data(data)?;

        self.attribute_type = mft_attribute_header.attribute_type;
        self.attribute_size = mft_attribute_header.attribute_size;
        self.non_resident_flag = mft_attribute_header.non_resident_flag;
        self.data_flags = mft_attribute_header.data_flags;

        let mut data_offset: usize = 16;

        if self.data_flags & 0x00ff > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unsupported data flags: {}", self.data_flags),
            ));
        }
        // TODO: check if compression type is 0 when cluster block size > 4096
        let mut resident_attribute: NtfsMftAttributeResident = NtfsMftAttributeResident::new();
        let mut non_resident_attribute: NtfsMftAttributeNonResident =
            NtfsMftAttributeNonResident::new();

        if self.is_resident() {
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(NtfsMftAttributeResident::debug_read_data(
                        &data[data_offset..],
                    ));
            }
            resident_attribute.read_data(&data[data_offset..])?;

            data_offset += 8;
        } else {
            if self.mediator.debug_output {
                self.mediator
                    .debug_print(NtfsMftAttributeNonResident::debug_read_data(
                        &data[data_offset..],
                    ));
            }
            non_resident_attribute.read_data(&data[data_offset..])?;

            if self.mediator.debug_output {
                if self.is_compressed() && non_resident_attribute.compression_unit_size == 0 {
                    self.mediator.debug_print(format!(
                        "Attribute data flags set compression type but no compression unit size set\n",
                    ));
                }
            }
            let non_resident_data_size: usize = if non_resident_attribute.compression_unit_size == 0
            {
                48
            } else {
                56
            };
            data_offset += non_resident_data_size;
        }
        if mft_attribute_header.name_size > 0 {
            let name_offset: usize = mft_attribute_header.name_offset as usize;

            if name_offset < data_offset || name_offset >= data_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid name offset: {} value out of bounds", name_offset),
                ));
            }
            // TODO: debug print unknown data

            let name_size: usize = (mft_attribute_header.name_size as usize) * 2;

            self.read_name(data, name_offset, name_size)?;

            data_offset = name_offset + name_size;
        }
        if self.is_resident() {
            if resident_attribute.data_size > 0 {
                let resident_data_offset: usize = resident_attribute.data_offset as usize;

                if resident_data_offset < data_offset || resident_data_offset >= data_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid resident data offset: {} value out of bounds",
                            mft_attribute_header.name_size
                        ),
                    ));
                }
                // TODO: debug print unknown data

                let resident_data_size: usize = resident_attribute.data_size as usize;
                let resident_data_end_offset: usize = resident_data_offset + resident_data_size;

                if resident_data_end_offset > data_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Invalid resident data size: {} value out of bounds",
                            mft_attribute_header.name_size
                        ),
                    ));
                }
                if self.mediator.debug_output {
                    self.mediator.debug_print(format!(
                        "NtfsMftAttribute resident data of size: {} at offset: {} (0x{:08x})\n",
                        resident_attribute.data_size,
                        resident_attribute.data_offset,
                        resident_attribute.data_offset,
                    ));
                    self.mediator.debug_print_data(
                        &data[resident_data_offset..resident_data_end_offset],
                        true,
                    );
                }
                self.resident_data = vec![0; resident_data_size];
                self.resident_data
                    .copy_from_slice(&data[resident_data_offset..resident_data_end_offset]);
            }
            self.data_size = resident_attribute.data_size as u64;
        } else {
            let data_runs_offset: usize = non_resident_attribute.data_runs_offset as usize;

            if data_runs_offset < data_offset || data_runs_offset >= data_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid data runs offset: {} value out of bounds",
                        data_runs_offset
                    ),
                ));
            }
            // TODO: debug print unknown data

            let mut cluster_group: NtfsClusterGroup = NtfsClusterGroup::new(
                non_resident_attribute.data_first_vcn,
                non_resident_attribute.data_last_vcn,
            );
            cluster_group.read_data_runs(data, data_runs_offset)?;

            self.data_cluster_groups.push(cluster_group);

            self.allocated_data_size = non_resident_attribute.allocated_data_size;
            self.data_size = non_resident_attribute.data_size;
            self.valid_data_size = non_resident_attribute.valid_data_size;
            self.compression_unit_size = non_resident_attribute.compression_unit_size;
            self.compressed_data_size = non_resident_attribute.compressed_data_size;
        }
        // TODO: debug print unknown data

        Ok(())
    }

    /// Reads the name from a buffer.
    fn read_name(&mut self, data: &[u8], name_offset: usize, name_size: usize) -> io::Result<()> {
        let name_end_offset: usize = name_offset + name_size;

        if name_end_offset > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid name size: {} value out of bounds", name_size),
            ));
        }
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "NtfsMftAttributeName data of size: {} at offset: {} (0x{:08x})\n",
                name_size, name_offset, name_offset,
            ));
            self.mediator
                .debug_print_data(&data[name_offset..name_end_offset], true);
        }
        let name: Ucs2String = Ucs2String::from_le_bytes(&data[name_offset..name_end_offset]);

        self.name = Some(name);

        Ok(())
    }

    /// Sorts the data cluster groups.
    pub fn sort_data_cluster_groups(&mut self) {
        self.data_cluster_groups
            .sort_by_key(|cluster_group| cluster_group.first_vcn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x90, 0x00, 0x00, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x04, 0x18, 0x00, 0x00, 0x00,
            0x11, 0x00, 0x38, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x24, 0x00, 0x53, 0x00,
            0x44, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00,
            0x28, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
    }

    // TODO: add tests for append_data_cluster_group
    // TODO: add tests for is_compressed
    // TODO: add tests for is_resident
    // TODO: add tests for is_sparse

    #[test]
    fn test_read_data() -> io::Result<()> {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = NtfsMftAttribute::new();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.attribute_type, 0x00000090);
        assert_eq!(test_struct.attribute_size, 88);
        assert_eq!(test_struct.name.unwrap().to_string(), "$SDH");
        assert_eq!(test_struct.data_size, 56);
        assert_eq!(test_struct.data_cluster_groups.len(), 0);
        assert_eq!(test_struct.non_resident_flag, 0x00);
        assert_eq!(test_struct.data_flags, 0x0000);

        Ok(())
    }

    // TODO: add tests for read_name
    // TODO: add tests for sort_data_cluster_groups
}
