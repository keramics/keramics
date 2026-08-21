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

use std::collections::HashSet;
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{Ucs2CharacterMappings, Ucs2String};

use crate::indexed_hash_map::IndexedHashMap;
use crate::path_component::PathComponent;

use super::allocation_bitmap_record::ExFatAllocationBitmapRecord;
use super::block_allocation_table::ExFatBlockAllocationTable;
use super::case_folding_mappings_record::ExFatCaseFoldingMappingsRecord;
use super::constants::*;
use super::data_stream_record::ExFatDataStreamRecord;
use super::directory_entry::ExFatDirectoryEntry;
use super::directory_entry_type::ExFatDirectoryEntryType;
use super::file_entry_record::ExFatFileEntryRecord;
use super::file_name_record::ExFatFileNameRecord;
use super::volume_label_record::ExFatVolumeLabelRecord;

/// Extensible File Allocation Table (exFAT) directory entries.
pub struct ExFatDirectoryEntries {
    /// Entries.
    pub entries: IndexedHashMap<Ucs2String, ExFatDirectoryEntry>,

    /// Case folding mappings.
    pub case_folding_mappings: Arc<Ucs2CharacterMappings>,

    /// Volume label.
    pub volume_label: Option<Ucs2String>,

    /// Value to indicate the directory entries are those of the root directory.
    is_root: bool,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl ExFatDirectoryEntries {
    /// Creates new directory entries.
    pub fn new(case_folding_mappings: &Arc<Ucs2CharacterMappings>, is_root: bool) -> Self {
        Self {
            entries: IndexedHashMap::new(),
            case_folding_mappings: case_folding_mappings.clone(),
            volume_label: None,
            is_root,
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(&self, entry_index: usize) -> Option<&ExFatDirectoryEntry> {
        self.entries.get_value_by_index(entry_index)
    }

    /// Retrieves a specific directory entry by name.
    pub fn get_entry_by_name(
        &self,
        name: &PathComponent,
    ) -> Result<Option<&ExFatDirectoryEntry>, ErrorTrace> {
        let lookup_name: Ucs2String =
            match name.to_ucs2_string_with_case_folding(&self.case_folding_mappings) {
                Ok(ucs2_string) => ucs2_string,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to convert path component to UCS-2 string with case folding"
                    );
                    return Err(error);
                }
            };
        Ok(self.entries.get_value_by_key(&lookup_name))
    }

    /// Retrieves the number of entries.
    pub fn get_number_of_entries(&self) -> usize {
        self.entries.len()
    }

    /// Determines if the directory entries were read.
    pub fn is_read(&self) -> bool {
        return self.is_read;
    }

    /// Reads the directory entries from a buffer.
    fn read_data(
        &mut self,
        data: &[u8],
        mut directory_entry_offset: u64,
        entries: &mut Vec<ExFatDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut data_offset: usize = 0;
        let data_size: usize = data.len();

        while data_offset < data_size {
            let data_end_offset: usize = data_offset + 32;

            if data_end_offset > data_size {
                return Err(keramics_core::error_trace_new!(format!(
                    "Insufficient data for directory entry at offset: {}",
                    data_offset
                )));
            }
            keramics_core::debug_trace_data!(
                "ExFatDirectoryEntry",
                directory_entry_offset,
                &data[data_offset..data_end_offset],
                32
            );
            match ExFatDirectoryEntryType::read_data(&data[data_offset..]) {
                ExFatDirectoryEntryType::AllocationBitmap => {
                    keramics_core::debug_trace_structure!(
                        ExFatAllocationBitmapRecord::debug_read_data(&data[data_offset..])
                    );
                    if !self.is_root {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported allocation bitmap record in non-root directory"
                        ));
                    }
                }
                ExFatDirectoryEntryType::CaseFoldingMappings => {
                    keramics_core::debug_trace_structure!(
                        ExFatCaseFoldingMappingsRecord::debug_read_data(&data[data_offset..])
                    );
                    if !self.is_root {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported case folding mappings record in non-root directory"
                        ));
                    }
                }
                ExFatDirectoryEntryType::DataStream => {
                    keramics_core::debug_trace_structure!(ExFatDataStreamRecord::debug_read_data(
                        &data[data_offset..]
                    ));
                    let mut data_stream_record: ExFatDataStreamRecord =
                        ExFatDataStreamRecord::new();

                    match data_stream_record.read_data(&data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read data stream record"
                            );
                            return Err(error);
                        }
                    }
                    match entries.last_mut() {
                        Some(directory_entry) => {
                            // TODO: check if data stream values are already set in directory entry.

                            directory_entry.valid_data_size = data_stream_record.valid_data_size;
                            directory_entry.data_start_cluster =
                                data_stream_record.data_start_cluster;
                            directory_entry.data_size = data_stream_record.data_size;

                            if data_stream_record.flags & 0x02 != 0 {
                                directory_entry.data_stream_no_fat_chain = true;
                            }
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Missing last directory entry"
                            ));
                        }
                    }
                }
                ExFatDirectoryEntryType::FileEntry => {
                    keramics_core::debug_trace_structure!(ExFatFileEntryRecord::debug_read_data(
                        &data[data_offset..]
                    ));
                    let mut directory_entry: ExFatDirectoryEntry =
                        ExFatDirectoryEntry::new(directory_entry_offset);

                    match directory_entry
                        .file_entry_record
                        .read_data(&data[data_offset..])
                    {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read file entry record"
                            );
                            return Err(error);
                        }
                    }
                    entries.push(directory_entry);
                }
                ExFatDirectoryEntryType::FileName => {
                    keramics_core::debug_trace_structure!(ExFatFileNameRecord::debug_read_data(
                        &data[data_offset..]
                    ));
                    let mut file_name_record: ExFatFileNameRecord = ExFatFileNameRecord::new();

                    match file_name_record.read_data(&data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read file name record"
                            );
                            return Err(error);
                        }
                    }
                    match entries.last_mut() {
                        Some(directory_entry) => {
                            directory_entry.name.append(&mut file_name_record.name);
                        }
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Missing last directory entry"
                            ));
                        }
                    }
                }
                ExFatDirectoryEntryType::Terminator => {
                    break;
                }
                ExFatDirectoryEntryType::VolumeIdentifier => {}
                ExFatDirectoryEntryType::VolumeLabel => {
                    keramics_core::debug_trace_structure!(ExFatVolumeLabelRecord::debug_read_data(
                        &data[data_offset..]
                    ));
                    if !self.is_root {
                        return Err(keramics_core::error_trace_new!(
                            "Unsupported volume label record in non-root directory"
                        ));
                    }
                    let mut volume_label_record: ExFatVolumeLabelRecord =
                        ExFatVolumeLabelRecord::new();

                    match volume_label_record.read_data(&data[data_offset..]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read volume label record"
                            );
                            return Err(error);
                        }
                    }
                    // TODO: check if first or second directory entry.

                    if self.volume_label.is_some() {
                        return Err(keramics_core::error_trace_new!("Volume label already set"));
                    }
                    self.volume_label = Some(volume_label_record.name);
                }
                _ => {}
            }
            data_offset = data_end_offset;
            directory_entry_offset += 32;
        }
        Ok(())
    }

    /// Finalizes the directories entries after read.
    fn read_finalize(&mut self, entries: &mut Vec<ExFatDirectoryEntry>) {
        for directory_entry in entries.drain(..) {
            let lookup_name: Ucs2String =
                directory_entry.get_lookup_name(&self.case_folding_mappings);
            self.entries.insert(lookup_name, directory_entry);
        }
        self.is_read = true;
    }

    /// Reads the directories entries starting at a specific cluster block in a data stream.
    pub fn read_at_cluster_block(
        &mut self,
        data_stream: &DataStreamReference,
        block_allocation_table: &Arc<ExFatBlockAllocationTable>,
        mut cluster_block_number: u32,
    ) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; block_allocation_table.cluster_block_size as usize];

        let mut read_cluster_block_numbers: HashSet<u32> = HashSet::new();
        let mut entries: Vec<ExFatDirectoryEntry> = Vec::new();

        while cluster_block_number >= 2 && cluster_block_number < EXFAT_LARGEST_CLUSTER_BLOCK_NUMBER
        {
            if read_cluster_block_numbers.contains(&cluster_block_number) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Cluster block: {} already read",
                    cluster_block_number
                )));
            }
            let offset: u64 = block_allocation_table.first_cluster_offset
                + (((cluster_block_number - 2) as u64)
                    * (block_allocation_table.cluster_block_size as u64));

            keramics_core::data_stream_read_exact_at_position_with_debug_trace_data!(
                format!(
                    "ExFatDirectoryEntries cluster block: {}",
                    cluster_block_number
                ),
                data_stream,
                &mut data,
                block_allocation_table.cluster_block_size,
                SeekFrom::Start(offset)
            );
            match self.read_data(&data, offset, &mut entries) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read directory entries at {} (0x{:08x})",
                            offset, offset
                        )
                    );
                    return Err(error);
                }
            }
            read_cluster_block_numbers.insert(cluster_block_number);

            cluster_block_number =
                match block_allocation_table.read_entry(data_stream, cluster_block_number) {
                    Ok(entry) => entry,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read next cluster block number from block allocation table"
                        );
                        return Err(error);
                    }
                };
        }
        self.read_finalize(&mut entries);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x83, 0x0a, 0x65, 0x00, 0x78, 0x00, 0x66, 0x00, 0x61, 0x00, 0x74, 0x00, 0x5f, 0x00,
            0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x81, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x82, 0x00,
            0x00, 0x00, 0x0d, 0xd3, 0x19, 0xe6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0xcc, 0x16, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x85, 0x02, 0xeb, 0x8b, 0x20, 0x00, 0x00, 0x00, 0xcd, 0x62, 0x15, 0x5d,
            0xcd, 0x62, 0x15, 0x5d, 0xcd, 0x62, 0x15, 0x5d, 0x25, 0x25, 0x80, 0x80, 0x80, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x01, 0x00, 0x09, 0x6b, 0x8e, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc1, 0x00, 0x65, 0x00,
            0x6d, 0x00, 0x70, 0x00, 0x74, 0x00, 0x79, 0x00, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00,
            0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x85, 0x02, 0x21, 0xdd, 0x10, 0x00, 0x00, 0x00, 0xcd, 0x62, 0x15, 0x5d, 0xcd, 0x62,
            0x15, 0x5d, 0xcd, 0x62, 0x15, 0x5d, 0x26, 0x26, 0x80, 0x80, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xc0, 0x03, 0x00, 0x08, 0x55, 0xc7, 0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc1, 0x00, 0x74, 0x00, 0x65, 0x00,
            0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x69, 0x00, 0x72, 0x00, 0x31, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    fn get_directory_entries() -> Result<ExFatDirectoryEntries, ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mappings: Arc<Ucs2CharacterMappings> =
            Arc::new(Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice()));

        let mut directory_entries: ExFatDirectoryEntries =
            ExFatDirectoryEntries::new(&mappings, true);

        let mut entries: Vec<ExFatDirectoryEntry> = Vec::new();
        directory_entries.read_data(&test_data, 0, &mut entries)?;
        directory_entries.read_finalize(&mut entries);

        Ok(directory_entries)
    }

    #[test]
    fn test_get_entry_by_index() -> Result<(), ErrorTrace> {
        let test_struct: ExFatDirectoryEntries = get_directory_entries()?;

        let entry: Option<&ExFatDirectoryEntry> = test_struct.get_entry_by_index(0);
        assert!(entry.is_some());

        let entry: Option<&ExFatDirectoryEntry> = test_struct.get_entry_by_index(99);
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_entry_by_name() -> Result<(), ErrorTrace> {
        let test_struct: ExFatDirectoryEntries = get_directory_entries()?;

        let name: PathComponent = PathComponent::Ucs2String(Ucs2String::from("emptyfile"));
        let entry: Option<&ExFatDirectoryEntry> = test_struct.get_entry_by_name(&name)?;
        assert!(entry.is_some());

        let name: PathComponent = PathComponent::Ucs2String(Ucs2String::from("bogus"));
        let entry: Option<&ExFatDirectoryEntry> = test_struct.get_entry_by_name(&name)?;
        assert!(entry.is_none());

        Ok(())
    }

    #[test]
    fn test_get_number_of_entries() -> Result<(), ErrorTrace> {
        let test_struct: ExFatDirectoryEntries = get_directory_entries()?;

        assert_eq!(test_struct.get_number_of_entries(), 2);

        Ok(())
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let mappings: Arc<Ucs2CharacterMappings> =
            Arc::new(Ucs2CharacterMappings::from(UCS2_CASE_MAPPINGS.as_slice()));

        let mut test_struct: ExFatDirectoryEntries = ExFatDirectoryEntries::new(&mappings, true);

        assert_eq!(test_struct.entries.len(), 0);

        let mut entries: Vec<ExFatDirectoryEntry> = Vec::new();
        test_struct.read_data(&test_data, 0, &mut entries)?;
        assert_eq!(entries.len(), 2);

        assert_eq!(
            test_struct.volume_label,
            Some(Ucs2String::from("exfat_test"))
        );
        Ok(())
    }

    // TODO: add tests for read_at_cluster_block
}
