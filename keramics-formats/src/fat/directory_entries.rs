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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::{ByteString, Ucs2String};

use super::block_allocation_table::FatBlockAllocationTable;
use super::constants::*;
use super::directory_entry::FatDirectoryEntry;
use super::directory_entry_type::FatDirectoryEntryType;
use super::long_name_directory_entry::FatLongNameDirectoryEntry;
use super::short_name_directory_entry::FatShortNameDirectoryEntry;
use super::string::FatString;

/// File Allocation Table (FAT) directory entries.
pub struct FatDirectoryEntries {
    /// Mediator.
    mediator: MediatorReference,

    /// Case folding mappings.
    pub case_folding_mappings: Arc<HashMap<u16, u16>>,

    /// Entries.
    pub entries: BTreeMap<Ucs2String, FatDirectoryEntry>,

    /// Volume label.
    pub volume_label: Option<ByteString>,

    /// Value to indicate the directory entries were read.
    is_read: bool,
}

impl FatDirectoryEntries {
    /// Creates new directory entries.
    pub fn new(case_folding_mappings: &Arc<HashMap<u16, u16>>) -> Self {
        Self {
            mediator: Mediator::current(),
            case_folding_mappings: case_folding_mappings.clone(),
            entries: BTreeMap::new(),
            volume_label: None,
            is_read: false,
        }
    }

    /// Retrieves a specific directory entry.
    pub fn get_entry_by_index(&self, entry_index: usize) -> Option<&FatDirectoryEntry> {
        match self.entries.iter().nth(entry_index) {
            Some((_, entry)) => Some(entry),
            None => None,
        }
    }

    /// Retrieves a specific directory entry by name.
    pub fn get_entry_by_name(
        &self,
        name: &FatString,
    ) -> Result<Option<&FatDirectoryEntry>, ErrorTrace> {
        let lookup_name: Ucs2String = match name.get_lookup_name(&self.case_folding_mappings) {
            Ok(string) => string,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to determine lookup name");
                return Err(error);
            }
        };
        match self.entries.get_key_value(&lookup_name) {
            Some((_, entry)) => Ok(Some(entry)),
            None => Ok(None),
        }
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
        last_vfat_sequence_number: &mut u8,
        long_name_entries: &mut Vec<FatLongNameDirectoryEntry>,
    ) -> Result<(), ErrorTrace> {
        let mut safe_last_vfat_sequence_number: u8 = *last_vfat_sequence_number;

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
            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "FatDirectoryEntry data of size: 32 at offset: {} (0x{:08x})\n",
                    directory_entry_offset, directory_entry_offset
                ));
                self.mediator
                    .debug_print_data(&data[data_offset..data_end_offset], true);
            }
            match FatDirectoryEntryType::read_data(&data[data_offset..]) {
                FatDirectoryEntryType::LongName => {
                    if self.mediator.debug_output {
                        self.mediator
                            .debug_print(FatLongNameDirectoryEntry::debug_read_data(
                                &data[data_offset..data_end_offset],
                            ));
                    }
                    let mut entry: FatLongNameDirectoryEntry = FatLongNameDirectoryEntry::new();

                    match entry.read_data(&data[data_offset..data_end_offset]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read long name directory entry"
                            );
                            return Err(error);
                        }
                    }
                    let vfat_sequence_number: u8 = entry.sequence_number & 0x1f;

                    if entry.sequence_number & 0x40 != 0 {
                        // TODO: warn about incomplete long name entry sequence
                        long_name_entries.clear();
                    } else if vfat_sequence_number + 1 != safe_last_vfat_sequence_number {
                        return Err(keramics_core::error_trace_new!(format!(
                            "VFAT long name sequence number mismatch at offset: {} (0x{:08x})",
                            directory_entry_offset, directory_entry_offset
                        )));
                    }
                    long_name_entries.push(entry);

                    safe_last_vfat_sequence_number = vfat_sequence_number;
                }
                FatDirectoryEntryType::ShortName => {
                    if self.mediator.debug_output {
                        self.mediator
                            .debug_print(FatShortNameDirectoryEntry::debug_read_data(
                                &data[data_offset..data_end_offset],
                            ));
                    }
                    let mut entry: FatShortNameDirectoryEntry = FatShortNameDirectoryEntry::new();

                    match entry.read_data(&data[data_offset..data_end_offset]) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read short name directory entry"
                            );
                            return Err(error);
                        }
                    }
                    if entry.file_attribute_flags & 0x58 == FAT_FILE_ATTRIBUTE_FLAG_VOLUME_LABEL {
                        self.volume_label = Some(entry.name);
                    } else {
                        // Ignore "." and ".."
                        if entry.name == "." || entry.name == ".." {
                            // TODO: add support to handle special directory entries
                        } else {
                            let mut directory_entry: FatDirectoryEntry =
                                FatDirectoryEntry::new(directory_entry_offset as u32, entry);
                            directory_entry.set_long_name(long_name_entries);

                            let lookup_name: Ucs2String =
                                directory_entry.get_lookup_name(&self.case_folding_mappings);
                            self.entries.insert(lookup_name, directory_entry);
                        }
                    }
                }
                FatDirectoryEntryType::Terminator => {
                    break;
                }
                FatDirectoryEntryType::Unallocated => {
                    // TODO: add support for recovering unallocated entries
                }
            }
            data_offset = data_end_offset;
            directory_entry_offset += 32;
        }
        *last_vfat_sequence_number = safe_last_vfat_sequence_number;

        Ok(())
    }

    /// Reads the directories entries starting at a specific cluster block in a data stream.
    pub fn read_at_cluster_block(
        &mut self,
        data_stream: &DataStreamReference,
        block_allocation_table: &Arc<FatBlockAllocationTable>,
        mut cluster_block_number: u32,
    ) -> Result<(), ErrorTrace> {
        let largest_cluster_block_number: u32 =
            block_allocation_table.get_largest_cluster_block_number();

        let mut data: Vec<u8> = vec![0; block_allocation_table.cluster_block_size as usize];

        let mut read_cluster_block_numbers: HashSet<u32> = HashSet::new();
        let mut last_vfat_sequence_number: u8 = 0;
        let mut long_name_entries: Vec<FatLongNameDirectoryEntry> = Vec::new();

        while cluster_block_number >= 2 && cluster_block_number < largest_cluster_block_number {
            if read_cluster_block_numbers.contains(&cluster_block_number) {
                return Err(keramics_core::error_trace_new!(format!(
                    "Cluster block: {} already read",
                    cluster_block_number
                )));
            }
            let offset: u64 = block_allocation_table.first_cluster_offset
                + (((cluster_block_number - 2) as u64)
                    * (block_allocation_table.cluster_block_size as u64));

            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(offset)
            );
            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "FatDirectoryEntries cluster block: {} data of size: {} at offset: {} (0x{:08x})\n",
                    cluster_block_number, block_allocation_table.cluster_block_size, offset, offset
                ));
                self.mediator.debug_print_data(&data, true);
            }
            match self.read_data(
                &data,
                offset,
                &mut last_vfat_sequence_number,
                &mut long_name_entries,
            ) {
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
        self.is_read = true;

        Ok(())
    }

    /// Reads the directories entries a specific position in a data stream.
    pub fn read_at_position(
        &mut self,
        data_stream: &DataStreamReference,
        data_size: u32,
        position: SeekFrom,
    ) -> Result<(), ErrorTrace> {
        // 65536 entries x 32 bytes = 2097152 bytes (2 MiB)
        if data_size < 32 || data_size > 2097152 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported directory entries data size: {} value out of bounds",
                data_size
            )));
        }
        let mut data: Vec<u8> = vec![0; data_size as usize];

        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, &mut data, position);
        if self.mediator.debug_output {
            self.mediator.debug_print(format!(
                "FatDirectoryEntries data of size: {} at offset: {} (0x{:08x})\n",
                data_size, offset, offset
            ));
            self.mediator.debug_print_data(&data, true);
        }
        let mut last_vfat_sequence_number: u8 = 0;
        let mut long_name_entries: Vec<FatLongNameDirectoryEntry> = Vec::new();

        match self.read_data(
            &data,
            offset,
            &mut last_vfat_sequence_number,
            &mut long_name_entries,
        ) {
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
        self.is_read = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_fake_data_stream;
    use keramics_encodings::CharacterEncoding;
    use keramics_types::constants::UCS2_CASE_MAPPINGS;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x46, 0x41, 0x54, 0x31, 0x32, 0x5f, 0x54, 0x45, 0x53, 0x54, 0x20, 0x08, 0x00, 0x00,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x41, 0x65, 0x00, 0x6d, 0x00, 0x70, 0x00, 0x74, 0x00, 0x79,
            0x00, 0x0f, 0x00, 0xc9, 0x66, 0x00, 0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x00, 0x00,
            0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x45, 0x4d, 0x50, 0x54, 0x59, 0x46,
            0x7e, 0x31, 0x20, 0x20, 0x20, 0x20, 0x00, 0x7d, 0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b,
            0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41, 0x74,
            0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x0f, 0x00, 0x81, 0x69, 0x00,
            0x72, 0x00, 0x31, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff,
            0xff, 0xff, 0x54, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10,
            0x00, 0x7d, 0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b,
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
    }

    #[test]
    fn test_read_data() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();

        let case_folding_mappings: Arc<HashMap<u16, u16>> = Arc::new(
            UCS2_CASE_MAPPINGS
                .into_iter()
                .collect::<HashMap<u16, u16>>(),
        );
        let mut test_struct = FatDirectoryEntries::new(&case_folding_mappings);
        let mut last_vfat_sequence_number: u8 = 0;
        let mut long_name_entries: Vec<FatLongNameDirectoryEntry> = Vec::new();

        test_struct.read_data(
            &test_data,
            0,
            &mut last_vfat_sequence_number,
            &mut long_name_entries,
        )?;

        assert_eq!(test_struct.entries.len(), 2);
        assert_eq!(
            test_struct.volume_label,
            Some(ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'F', b'A', b'T', b'1', b'2', b'_', b'T', b'E', b'S', b'T'],
            })
        );

        let entry: &FatDirectoryEntry = test_struct.get_entry_by_index(0).unwrap();
        assert_eq!(entry.long_name, Some(Ucs2String::from("emptyfile")));
        assert_eq!(
            entry.short_name.name,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'E', b'M', b'P', b'T', b'Y', b'F', b'~', b'1'],
            }
        );
        assert_eq!(entry.short_name.file_attribute_flags, 0x20);

        Ok(())
    }

    // TODO: add tests for read_at_cluster_block

    #[test]
    fn test_read_at_position() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data();
        let data_stream: DataStreamReference = open_fake_data_stream(&test_data);

        let case_folding_mappings: Arc<HashMap<u16, u16>> = Arc::new(
            UCS2_CASE_MAPPINGS
                .into_iter()
                .collect::<HashMap<u16, u16>>(),
        );
        let mut test_struct = FatDirectoryEntries::new(&case_folding_mappings);
        test_struct.read_at_position(&data_stream, 512, SeekFrom::Start(0))?;

        assert_eq!(test_struct.entries.len(), 2);
        assert_eq!(
            test_struct.volume_label,
            Some(ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'F', b'A', b'T', b'1', b'2', b'_', b'T', b'E', b'S', b'T'],
            })
        );

        let entry: &FatDirectoryEntry = test_struct.get_entry_by_index(0).unwrap();
        assert_eq!(entry.long_name, Some(Ucs2String::from("emptyfile")));
        assert_eq!(
            entry.short_name.name,
            ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'E', b'M', b'P', b'T', b'Y', b'F', b'~', b'1'],
            }
        );
        assert_eq!(entry.short_name.file_attribute_flags, 0x20);

        Ok(())
    }
}
