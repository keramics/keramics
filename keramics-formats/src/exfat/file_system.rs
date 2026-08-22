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
use std::sync::Arc;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::constants::UCS2_CASE_MAPPINGS;
use keramics_types::{Ucs2CharacterMappings, Ucs2String};

use crate::path::Path;

use super::block_allocation_table::ExFatBlockAllocationTable;
use super::boot_record::ExFatBootRecord;
use super::data_stream_record::ExFatDataStreamRecord;
use super::directory_entries::ExFatDirectoryEntries;
use super::directory_entry::ExFatDirectoryEntry;
use super::directory_entry_type::ExFatDirectoryEntryType;
use super::file_entry::ExFatFileEntry;
use super::file_name_record::ExFatFileNameRecord;

/// Extensible File Allocation Table (exFAT) file system.
pub struct ExFatFileSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    bytes_per_sector: u16,

    /// Cluster block size.
    cluster_block_size: u32,

    /// First cluster offset.
    first_cluster_offset: u64,

    /// Root directory cluster block number.
    root_directory_cluster_block_number: u32,

    /// Block allocation table.
    block_allocation_table: Option<Arc<ExFatBlockAllocationTable>>,

    /// Case folding mappings.
    case_folding_mappings: Arc<Ucs2CharacterMappings>,

    /// Volume serial number.
    volume_serial_number: u32,

    /// Volume label.
    volume_label: Option<Ucs2String>,
}

impl ExFatFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            cluster_block_size: 0,
            first_cluster_offset: 0,
            root_directory_cluster_block_number: 0,
            block_allocation_table: None,
            case_folding_mappings: Arc::new(Ucs2CharacterMappings::from(
                UCS2_CASE_MAPPINGS.as_slice(),
            )),
            volume_serial_number: 0,
            volume_label: None,
        }
    }

    /// Retrieves the bytes per sector.
    pub fn get_bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&Ucs2String> {
        self.volume_label.as_ref()
    }

    /// Retrieves the file entry for a specific identifier (directory entry offset).
    pub fn get_file_entry_by_identifier(
        &self,
        file_entry_identifier: u64,
    ) -> Result<ExFatFileEntry, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let block_allocation_table: &Arc<ExFatBlockAllocationTable> =
            match self.block_allocation_table.as_ref() {
                Some(block_allocation_table) => block_allocation_table,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table"
                    ));
                }
            };
        let directory_entry: Option<ExFatDirectoryEntry> = if file_entry_identifier == 0 {
            None
        } else {
            match self.read_directory_entry_by_identifier(data_stream, file_entry_identifier) {
                Ok(directory_entry) => Some(directory_entry),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read directory entry");
                    return Err(error);
                }
            }
        };
        Ok(ExFatFileEntry::new(
            data_stream,
            block_allocation_table,
            file_entry_identifier,
            directory_entry,
            ExFatDirectoryEntries::new(&self.case_folding_mappings, false),
        ))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        path: &Path,
    ) -> Result<Option<ExFatFileEntry>, ErrorTrace> {
        if path.is_empty() || path.is_relative() {
            return Ok(None);
        }
        let mut file_entry: ExFatFileEntry = match self.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        for path_component in path.components[1..].iter() {
            file_entry = match file_entry.get_sub_file_entry_by_name(path_component) {
                Ok(Some(file_entry)) => file_entry,
                Ok(None) => return Ok(None),
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!("Unable to retrieve sub file entry: {}", path_component)
                    );
                    return Err(error);
                }
            };
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<ExFatFileEntry, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let block_allocation_table: &Arc<ExFatBlockAllocationTable> =
            match self.block_allocation_table.as_ref() {
                Some(block_allocation_table) => block_allocation_table,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table"
                    ));
                }
            };
        let mut directory_entries: ExFatDirectoryEntries =
            ExFatDirectoryEntries::new(&self.case_folding_mappings, true);

        match directory_entries.read_at_cluster_block(
            data_stream,
            &block_allocation_table,
            self.root_directory_cluster_block_number,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read root directory from cluster block: {}",
                        self.root_directory_cluster_block_number
                    )
                );
                return Err(error);
            }
        }
        Ok(ExFatFileEntry::new(
            data_stream,
            block_allocation_table,
            0,
            None,
            directory_entries,
        ))
    }

    /// Reads a file system from a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_metadata(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read metadata");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads a directory entry for a specific identifier (directory entry offset).
    fn read_directory_entry_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        file_entry_identifier: u64,
    ) -> Result<ExFatDirectoryEntry, ErrorTrace> {
        let mut directory_entry: ExFatDirectoryEntry =
            ExFatDirectoryEntry::new(file_entry_identifier);

        match directory_entry
            .file_entry_record
            .read_at_position(data_stream, SeekFrom::Start(file_entry_identifier))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read file entry record at offset: {} (0x{:08x})",
                        file_entry_identifier, file_entry_identifier
                    )
                );
                return Err(error);
            }
        }
        let first_directory_entry_offset: u64 = if file_entry_identifier < self.first_cluster_offset
        {
            (self.root_directory_cluster_block_number as u64) * (self.cluster_block_size as u64)
        } else {
            let cluster_block_number: u32 =
                (2 + ((file_entry_identifier as u64) - self.first_cluster_offset)
                    / (self.cluster_block_size as u64)) as u32;

            self.first_cluster_offset
                + (((cluster_block_number - 2) as u64) * (self.cluster_block_size as u64))
        };
        let last_directory_entry_offset: u64 =
            first_directory_entry_offset + (self.cluster_block_size as u64);

        let mut directory_entry_offset: u64 = file_entry_identifier + 32;
        let mut data: Vec<u8> = vec![0; 32];

        // TODO: add support to scan next cluster block.
        while directory_entry_offset < last_directory_entry_offset {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(directory_entry_offset)
            );
            keramics_core::debug_trace_data!(
                "ExFatDirectoryEntry",
                directory_entry_offset,
                &data,
                32
            );
            match ExFatDirectoryEntryType::read_data(&data) {
                ExFatDirectoryEntryType::DataStream => {
                    keramics_core::debug_trace_structure!(ExFatDataStreamRecord::debug_read_data(
                        &data
                    ));
                    let mut data_stream_record: ExFatDataStreamRecord =
                        ExFatDataStreamRecord::new();

                    match data_stream_record.read_data(&data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read data stream record"
                            );
                            return Err(error);
                        }
                    }
                    directory_entry.valid_data_size = data_stream_record.valid_data_size;
                    directory_entry.data_start_cluster = data_stream_record.data_start_cluster;
                    directory_entry.data_size = data_stream_record.data_size;

                    if data_stream_record.flags & 0x02 != 0 {
                        directory_entry.data_stream_no_fat_chain = true;
                    }
                }
                ExFatDirectoryEntryType::FileName => {
                    keramics_core::debug_trace_structure!(ExFatFileNameRecord::debug_read_data(
                        &data
                    ));
                    let mut file_name_record: ExFatFileNameRecord = ExFatFileNameRecord::new();

                    match file_name_record.read_data(&data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read file name record"
                            );
                            return Err(error);
                        }
                    }
                    directory_entry.name.append(&mut file_name_record.name);
                }
                _ => {
                    break;
                }
            }
            directory_entry_offset += 32;
        }
        Ok(directory_entry)
    }

    /// Reads the boot record and root directory.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut boot_record: ExFatBootRecord = ExFatBootRecord::new();

        match boot_record.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read boot record");
                return Err(error);
            }
        }
        self.bytes_per_sector = boot_record.bytes_per_sector;
        self.root_directory_cluster_block_number = boot_record.root_directory_cluster_block_number;
        self.volume_serial_number = boot_record.volume_serial_number;

        let mut number_of_clusters: u64 = boot_record.number_of_sectors as u64;
        number_of_clusters -= (boot_record.number_of_allocation_tables as u64)
            * (boot_record.allocation_table_size as u64);
        number_of_clusters /= boot_record.sectors_per_cluster_block as u64;

        let allocation_table_offset: u64 =
            (boot_record.allocation_table_offset as u64) * (boot_record.bytes_per_sector as u64);

        self.first_cluster_offset =
            (boot_record.cluster_heap_start_sector as u64) * (boot_record.bytes_per_sector as u64);
        self.cluster_block_size =
            (boot_record.bytes_per_sector as u32) * (boot_record.sectors_per_cluster_block as u32);

        let block_allocation_table: Arc<ExFatBlockAllocationTable> =
            Arc::new(ExFatBlockAllocationTable::new(
                allocation_table_offset,
                number_of_clusters as u32,
                self.first_cluster_offset,
                self.cluster_block_size,
            ));
        let mut directory_entries: ExFatDirectoryEntries =
            ExFatDirectoryEntries::new(&self.case_folding_mappings, true);

        match directory_entries.read_at_cluster_block(
            data_stream,
            &block_allocation_table,
            self.root_directory_cluster_block_number,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read root directory from cluster block: {}",
                        self.root_directory_cluster_block_number
                    )
                );
                return Err(error);
            }
        }
        if directory_entries.volume_label.is_some() {
            self.volume_label = directory_entries.volume_label;
        }
        self.block_allocation_table = Some(block_allocation_table);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<ExFatFileSystem, ErrorTrace> {
        let mut file_system: ExFatFileSystem = ExFatFileSystem::new();

        let path_string: String = get_test_data_path("exfat/exfat.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_bytes_per_sector() -> Result<(), ErrorTrace> {
        let file_system: ExFatFileSystem = get_file_system()?;

        let bytes_per_sector: u16 = file_system.get_bytes_per_sector();
        assert_eq!(bytes_per_sector, 512);

        Ok(())
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let file_system: ExFatFileSystem = get_file_system()?;

        let volume_label: Option<&Ucs2String> = file_system.get_volume_label();
        assert_eq!(volume_label, Some(Ucs2String::from("exfat_test")).as_ref());
        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: ExFatFileSystem = get_file_system()?;

        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_identifier(0x00000000)?;
        assert_eq!(file_entry.identifier, 0x00000000);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert!(name.is_none());

        let result: Result<ExFatFileEntry, ErrorTrace> =
            file_system.get_file_entry_by_identifier(0x00201a00);
        assert!(result.is_err());

        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_identifier(0x00201a80)?;
        assert_eq!(file_entry.identifier, 0x00201a80);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert_eq!(name, Some(Ucs2String::from("emptyfile")).as_ref());

        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_identifier(0x00201cc0)?;
        assert_eq!(file_entry.identifier, 0x00201cc0);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert_eq!(
            name,
            Some(Ucs2String::from(
                "My long, very long file name, so very long"
            ))
            .as_ref()
        );
        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: ExFatFileSystem = get_file_system()?;

        let path: Path = Path::from("/");
        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00000000);

        let path: Path = Path::from("/emptyfile");
        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00201a80);

        let path: Path = Path::from("/testdir1/testfile1");
        let file_entry: ExFatFileEntry = file_system.get_file_entry_by_path(&path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00201c00);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert_eq!(name, Some(Ucs2String::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: ExFatFileSystem = get_file_system()?;

        let file_entry: ExFatFileEntry = file_system.get_root_directory()?;

        assert_eq!(file_entry.identifier, 0x00000000);

        Ok(())
    }

    // TODO: add tests for read_directory_entry_by_identifier

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let mut file_system: ExFatFileSystem = ExFatFileSystem::new();

        let path_string: String = get_test_data_path("exfat/exfat.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert_eq!(file_system.volume_serial_number, 0x7aef7302);
        assert_eq!(
            file_system.volume_label,
            Some(Ucs2String::from("exfat_test"))
        );
        Ok(())
    }
}
