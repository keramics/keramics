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

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};

use super::attribute_list_entry::NtfsAttributeListEntry;
use super::block_reader::NtfsBlockReader;
use super::block_stream::NtfsBlockStream;
use super::constants::*;
use super::mft_attribute::NtfsMftAttribute;

/// New Technologies File System (NTFS) attribute list.
pub struct NtfsAttributeList {
    /// Entries.
    pub entries: Vec<NtfsAttributeListEntry>,
}

impl NtfsAttributeList {
    /// Creates a new attribute list.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Reads the attribute list from a buffer.
    fn read_data(&mut self, data: &[u8]) -> Result<(), ErrorTrace> {
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();

        while data_offset < data_size {
            keramics_core::debug_trace_structure!(NtfsAttributeListEntry::debug_read_data(
                &data[data_offset..]
            ));

            let mut entry: NtfsAttributeListEntry = NtfsAttributeListEntry::new();

            match entry.read_data(&data[data_offset..]) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read attribute list entry"
                    );
                    return Err(error);
                }
            }
            // TODO: read name.
            // TODO: print remaining data.

            data_offset += entry.attribute_size as usize;

            self.entries.push(entry);
        }
        Ok(())
    }

    /// Reads the attribute list from a MFT attribute.
    pub fn read_attribute(
        &mut self,
        data_attribute: &NtfsMftAttribute,
        data_stream: &DataStreamReference,
        cluster_block_size: u32,
    ) -> Result<(), ErrorTrace> {
        if data_attribute.attribute_type != NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported attribute type: 0x{:08x}",
                data_attribute.attribute_type
            )));
        }
        if data_attribute.is_compressed() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported compressed $ATTRIBUTE_LIST attribute"
            ));
        }
        if data_attribute.is_resident() {
            match self.read_data(&data_attribute.resident_data) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to read resident attribute list"
                    );
                    return Err(error);
                }
            }
        } else {
            let mut block_reader: NtfsBlockReader =
                NtfsBlockReader::new(data_stream, cluster_block_size);

            match block_reader.open(data_attribute) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to open block reader");
                    return Err(error);
                }
            }
            let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(block_reader);

            let mut cluster_block: Vec<u8> = vec![0; cluster_block_size as usize];

            loop {
                let read_count: usize = match block_stream.read(&mut cluster_block) {
                    Ok(read_count) => read_count,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read attribute list from block stream"
                        );
                        return Err(error);
                    }
                };
                if read_count == 0 {
                    break;
                }
                match self.read_data(&cluster_block[..read_count]) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read non-resident attribute list"
                        );
                        return Err(error);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;

    use crate::ntfs::cluster_group::NtfsClusterGroup;
    use crate::ntfs::data_run::{NtfsDataRun, NtfsDataRunType};

    fn get_test_data() -> Vec<u8> {
        vec![
            0x20, 0x00, 0x00, 0x00, 0x70, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x0f, 0x00, 0x58, 0x01, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x20, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x44, 0x43, 0x00, 0x00, 0x00, 0x00,
            0x30, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x28, 0x00, 0x04, 0x1a, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x24, 0x00, 0x53, 0x00, 0x44, 0x00, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x90, 0x00, 0x00, 0x00, 0x28, 0x00, 0x04, 0x1a, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x11, 0x00,
            0x24, 0x00, 0x53, 0x00, 0x44, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x90, 0x00, 0x00, 0x00, 0x28, 0x00, 0x04, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x10, 0x00, 0x24, 0x00,
            0x53, 0x00, 0x49, 0x00, 0x49, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x00,
            0x00, 0x00, 0x28, 0x00, 0x04, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xc8, 0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03, 0x00, 0x24, 0x00, 0x53, 0x00,
            0x44, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x00, 0x00, 0x00,
            0x28, 0x00, 0x04, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x24, 0x00, 0x53, 0x00, 0x49, 0x00,
            0x49, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x00, 0x00, 0x00, 0x28, 0x00,
            0x04, 0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x08, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x05, 0x00, 0x24, 0x00, 0x53, 0x00, 0x44, 0x00, 0x48, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x00, 0x00, 0x00, 0x28, 0x00, 0x04, 0x1a,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc8, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x06, 0x00, 0x24, 0x00, 0x53, 0x00, 0x49, 0x00, 0x49, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let mut test_struct: NtfsAttributeList = NtfsAttributeList::new();

        let test_data: Vec<u8> = get_test_data();
        test_struct.read_data(&test_data)?;

        assert_eq!(test_struct.entries.len(), 1);

        // Test with truncated entry data.
        let mut invalid_attribute_list: NtfsAttributeList = NtfsAttributeList::new();
        let result: Result<(), ErrorTrace> = invalid_attribute_list.read_data(&test_data[0..10]);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_read_attribute() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let empty_stream: DataStreamReference = open_fake_data_stream(&[]);

        // Test with resident $ATTRIBUTE_LIST attribute.
        let mut resident_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        resident_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST;
        resident_attribute.resident_data = test_data.clone();
        let mut attribute_list: NtfsAttributeList = NtfsAttributeList::new();
        attribute_list.read_attribute(&resident_attribute, &empty_stream, 4096)?;
        assert_eq!(attribute_list.entries.len(), 1);

        // Test with non-resident $ATTRIBUTE_LIST attribute backed by a cluster group.
        let mut cluster_data: Vec<u8> = vec![0; 4096];
        cluster_data[0..test_data.len()].copy_from_slice(&test_data);
        let cluster_stream: DataStreamReference = open_fake_data_stream(&cluster_data);
        let mut cluster_group: NtfsClusterGroup = NtfsClusterGroup::new(0, 0);
        let data_run: NtfsDataRun = NtfsDataRun {
            block_number: 0,
            number_of_blocks: 1,
            run_type: NtfsDataRunType::InFile,
        };
        cluster_group.data_runs.push(data_run);
        let mut non_resident_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        non_resident_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST;
        non_resident_attribute.non_resident_flag = 1;
        non_resident_attribute.data_size = test_data.len() as u64;
        non_resident_attribute.allocated_data_size = 4096;
        non_resident_attribute.valid_data_size = test_data.len() as u64;
        non_resident_attribute
            .data_cluster_groups
            .push(cluster_group);
        let mut non_resident_list: NtfsAttributeList = NtfsAttributeList::new();
        non_resident_list.read_attribute(&non_resident_attribute, &cluster_stream, 4096)?;
        assert_eq!(non_resident_list.entries.len(), 1);

        // Test with unsupported attribute type.
        let mut wrong_type_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        wrong_type_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_DATA;
        let mut list: NtfsAttributeList = NtfsAttributeList::new();
        let result: Result<(), ErrorTrace> =
            list.read_attribute(&wrong_type_attribute, &empty_stream, 4096);
        assert!(result.is_err());

        // Test with compressed $ATTRIBUTE_LIST attribute.
        let mut compressed_attribute: NtfsMftAttribute = NtfsMftAttribute::new();
        compressed_attribute.attribute_type = NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST;
        compressed_attribute.data_flags = 0x0001;
        let result: Result<(), ErrorTrace> =
            list.read_attribute(&compressed_attribute, &empty_stream, 4096);
        assert!(result.is_err());

        // Test with invalid resident attribute data.
        let mut invalid_resident: NtfsMftAttribute = NtfsMftAttribute::new();
        invalid_resident.attribute_type = NTFS_ATTRIBUTE_TYPE_ATTRIBUTE_LIST;
        invalid_resident.resident_data = vec![0; 4];
        let result: Result<(), ErrorTrace> =
            list.read_attribute(&invalid_resident, &empty_stream, 4096);
        assert!(result.is_err());

        Ok(())
    }
}
