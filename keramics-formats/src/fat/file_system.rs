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
use std::io::SeekFrom;
use std::sync::Arc;

use keramics_core::mediator::{Mediator, MediatorReference};
use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_types::ByteString;
use keramics_types::constants::UCS2_CASE_MAPPINGS;

use super::block_allocation_table::FatBlockAllocationTable;
use super::boot_record::FatBootRecord;
use super::directory_entries::FatDirectoryEntries;
use super::directory_entry::FatDirectoryEntry;
use super::directory_entry_type::FatDirectoryEntryType;
use super::enums::FatFormat;
use super::file_entry::FatFileEntry;
use super::long_name_directory_entry::FatLongNameDirectoryEntry;
use super::path::FatPath;
use super::short_name_directory_entry::FatShortNameDirectoryEntry;

/// File Allocation Table (FAT) file system.
pub struct FatFileSystem {
    /// Mediator.
    mediator: MediatorReference,

    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Cluster block size.
    cluster_block_size: u32,

    /// First cluster offset.
    pub first_cluster_offset: u64,

    /// Root directory offset.
    pub root_directory_offset: u64,

    /// Root directory size.
    pub root_directory_size: u32,

    /// Root directory cluster block number.
    pub root_directory_cluster_block_number: u32,

    /// Format.
    pub format: FatFormat,

    /// Block allocation table.
    block_allocation_table: Option<Arc<FatBlockAllocationTable>>,

    /// Case folding mappings.
    case_folding_mappings: Arc<HashMap<u16, u16>>,

    /// Volume serial number.
    pub volume_serial_number: u32,

    /// Volume label.
    volume_label: Option<ByteString>,

    /// Volume label stored in the root directory.
    root_directory_volume_label: Option<ByteString>,
}

impl FatFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            mediator: Mediator::current(),
            data_stream: None,
            bytes_per_sector: 0,
            cluster_block_size: 0,
            first_cluster_offset: 0,
            root_directory_offset: 0,
            root_directory_size: 0,
            root_directory_cluster_block_number: 0,
            format: FatFormat::Fat12,
            block_allocation_table: None,
            case_folding_mappings: Arc::new(
                UCS2_CASE_MAPPINGS
                    .into_iter()
                    .collect::<HashMap<u16, u16>>(),
            ),
            volume_serial_number: 0,
            volume_label: None,
            root_directory_volume_label: None,
        }
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&ByteString> {
        match self.root_directory_volume_label.as_ref() {
            Some(label) => return Some(label),
            None => self.volume_label.as_ref(),
        }
    }

    /// Retrieves the file entry for a specific identifier (inode number).
    pub fn get_file_entry_by_identifier(
        &self,
        file_entry_identifier: u32,
    ) -> Result<FatFileEntry, ErrorTrace> {
        if (file_entry_identifier as u64) == self.root_directory_offset {
            return self.get_root_directory();
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let block_allocation_table: &Arc<FatBlockAllocationTable> =
            match self.block_allocation_table.as_ref() {
                Some(block_allocation_table) => block_allocation_table,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table"
                    ));
                }
            };
        let directory_entry: FatDirectoryEntry =
            match self.read_directory_entry_by_identifier(data_stream, file_entry_identifier) {
                Ok(directory_entry) => directory_entry,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(error, "Unable to read directory entry");
                    return Err(error);
                }
            };
        Ok(FatFileEntry::new(
            data_stream,
            block_allocation_table,
            file_entry_identifier,
            Some(directory_entry),
            FatDirectoryEntries::new(&self.case_folding_mappings),
        ))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        path: &FatPath,
    ) -> Result<Option<FatFileEntry>, ErrorTrace> {
        if path.is_empty() || path.components[0].len() != 0 {
            return Ok(None);
        }
        let mut file_entry: FatFileEntry = match self.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            let result: Option<FatFileEntry> =
                match file_entry.get_sub_file_entry_by_name(path_component) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!(
                                "Unable to retrieve sub file entry: {}",
                                path_component.to_string()
                            )
                        );
                        return Err(error);
                    }
                };
            file_entry = match result {
                Some(file_entry) => file_entry,
                None => return Ok(None),
            };
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<FatFileEntry, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let block_allocation_table: &Arc<FatBlockAllocationTable> =
            match self.block_allocation_table.as_ref() {
                Some(block_allocation_table) => block_allocation_table,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table"
                    ));
                }
            };
        let directory_entries: FatDirectoryEntries = match self.read_root_directory(data_stream) {
            Ok(directory_entries) => directory_entries,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read root directory");
                return Err(error);
            }
        };
        Ok(FatFileEntry::new(
            data_stream,
            block_allocation_table,
            self.root_directory_offset as u32,
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

    /// Reads a directory entry for a specific identifier.
    fn read_directory_entry_by_identifier(
        &self,
        data_stream: &DataStreamReference,
        file_entry_identifier: u32,
    ) -> Result<FatDirectoryEntry, ErrorTrace> {
        // TODO: move code into DirectoryEntryScanner
        let mut short_name_entry: FatShortNameDirectoryEntry = FatShortNameDirectoryEntry::new();

        match short_name_entry
            .read_at_position(data_stream, SeekFrom::Start(file_entry_identifier as u64))
        {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read short name directory entry at offset: {} (0x{:08x})",
                        file_entry_identifier, file_entry_identifier
                    )
                );
                return Err(error);
            }
        }
        let mut directory_entry_offset: u64 = (file_entry_identifier as u64) - 32;

        let cluster_block_number: u32 = if directory_entry_offset < self.first_cluster_offset {
            0
        } else {
            (2 + ((file_entry_identifier as u64) - self.first_cluster_offset)
                / (self.cluster_block_size as u64)) as u32
        };
        let first_directory_entry_offset: u64 =
            if directory_entry_offset < self.first_cluster_offset {
                self.root_directory_offset
            } else {
                self.first_cluster_offset
                    + (((cluster_block_number - 2) as u64) * (self.cluster_block_size as u64))
            };
        let mut last_vfat_sequence_number: u8 = 0;
        let mut long_name_entries: Vec<FatLongNameDirectoryEntry> = Vec::new();
        let mut last_entry: bool = false;
        let mut data: [u8; 32] = [0; 32];

        while directory_entry_offset >= first_directory_entry_offset {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut data,
                SeekFrom::Start(directory_entry_offset)
            );
            if self.mediator.debug_output {
                self.mediator.debug_print(format!(
                    "FatDirectoryEntry data of size: 32 at offset: {} (0x{:08x})\n",
                    directory_entry_offset, directory_entry_offset
                ));
                self.mediator.debug_print_data(&data, true);
            }
            match FatDirectoryEntryType::read_data(&data) {
                FatDirectoryEntryType::LongName => {
                    let mut long_name_entry: FatLongNameDirectoryEntry =
                        FatLongNameDirectoryEntry::new();

                    match long_name_entry.read_data(&data) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                format!(
                                    "Unable to read long name directory entry at offset: {} (0x{:08x})",
                                    file_entry_identifier, file_entry_identifier
                                )
                            );
                            return Err(error);
                        }
                    }
                    let vfat_sequence_number: u8 = long_name_entry.sequence_number & 0x1f;

                    if long_name_entry.sequence_number & 0x40 != 0 {
                        last_entry = true;
                    } else if last_vfat_sequence_number != 0
                        && last_vfat_sequence_number + 1 != vfat_sequence_number
                    {
                        return Err(keramics_core::error_trace_new!(format!(
                            "VFAT long name sequence number mismatch at offset: {} (0x{:08x})",
                            directory_entry_offset, directory_entry_offset
                        )));
                    }
                    long_name_entries.push(long_name_entry);

                    last_vfat_sequence_number = vfat_sequence_number;
                }
                _ => last_entry = true,
            }
            if last_entry {
                break;
            }
            directory_entry_offset -= 32;
        }
        if !last_entry {
            // TODO: determine previous cluster block.
        }
        let mut directory_entry: FatDirectoryEntry =
            FatDirectoryEntry::new(file_entry_identifier, short_name_entry);

        if last_vfat_sequence_number & 0x40 == 0 {
            long_name_entries.reverse();

            directory_entry.set_long_name(&mut long_name_entries);
        }
        Ok(directory_entry)
    }

    /// Reads the boot record and root directory.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut boot_record: FatBootRecord = FatBootRecord::new();

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

        if !boot_record.volume_label.is_empty() {
            self.volume_label = Some(boot_record.volume_label)
        }
        let mut number_of_clusters: u64 = (boot_record.number_of_sectors as u64)
            - (boot_record.number_of_reserved_sectors as u64);
        number_of_clusters -= (boot_record.number_of_allocation_tables as u64)
            * (boot_record.allocation_table_size as u64);
        number_of_clusters /= boot_record.sectors_per_cluster_block as u64;

        self.format =
            if self.root_directory_cluster_block_number != 0 || number_of_clusters >= 65525 {
                FatFormat::Fat32
            } else if number_of_clusters >= 4085 {
                FatFormat::Fat16
            } else {
                FatFormat::Fat12
            };
        let allocation_table_offset: u64 =
            (boot_record.number_of_reserved_sectors as u64) * (boot_record.bytes_per_sector as u64);
        let allocation_table_size: u64 =
            (boot_record.allocation_table_size as u64) * (boot_record.bytes_per_sector as u64);

        self.first_cluster_offset = allocation_table_offset
            + ((boot_record.number_of_allocation_tables as u64) * allocation_table_size);

        self.cluster_block_size =
            (boot_record.bytes_per_sector as u32) * (boot_record.sectors_per_cluster_block as u32);

        if self.root_directory_cluster_block_number == 0 {
            self.root_directory_offset = self.first_cluster_offset;
            self.root_directory_size = (boot_record.number_of_root_directory_entries as u32) * 32;
            self.first_cluster_offset += self.root_directory_size as u64;
        }
        self.block_allocation_table = Some(Arc::new(FatBlockAllocationTable::new(
            &self.format,
            allocation_table_offset,
            number_of_clusters as u32,
            self.first_cluster_offset,
            self.cluster_block_size,
        )));
        let directory_entries: FatDirectoryEntries = match self.read_root_directory(data_stream) {
            Ok(directory_entries) => directory_entries,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read root directory");
                return Err(error);
            }
        };
        if directory_entries.volume_label.is_some() {
            self.root_directory_volume_label = directory_entries.volume_label;
        }
        Ok(())
    }

    /// Reads the root directory.
    fn read_root_directory(
        &self,
        data_stream: &DataStreamReference,
    ) -> Result<FatDirectoryEntries, ErrorTrace> {
        let block_allocation_table: &Arc<FatBlockAllocationTable> =
            match self.block_allocation_table.as_ref() {
                Some(block_allocation_table) => block_allocation_table,
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Missing block allocation table"
                    ));
                }
            };
        let mut directory_entries: FatDirectoryEntries =
            FatDirectoryEntries::new(&self.case_folding_mappings);

        if self.root_directory_size > 0 {
            match directory_entries.read_at_position(
                &data_stream,
                self.root_directory_size,
                SeekFrom::Start(self.root_directory_offset as u64),
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read root directory at offset: {} (0x{:08x})",
                            self.root_directory_offset, self.root_directory_offset
                        )
                    );
                    return Err(error);
                }
            }
        } else {
            match directory_entries.read_at_cluster_block(
                &data_stream,
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
        }
        Ok(directory_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;
    use keramics_encodings::CharacterEncoding;

    use crate::fat::FatString;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<FatFileSystem, ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("fat/fat12.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let file_system: FatFileSystem = get_file_system()?;

        let volume_label: Option<&ByteString> = file_system.get_volume_label();
        assert_eq!(
            volume_label,
            Some(ByteString {
                encoding: CharacterEncoding::Ascii,
                elements: vec![b'F', b'A', b'T', b'1', b'2', b'_', b'T', b'E', b'S', b'T']
            })
            .as_ref()
        );
        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: FatFileSystem = get_file_system()?;

        let file_entry: FatFileEntry = file_system.get_file_entry_by_identifier(0x00001a00)?;
        assert_eq!(file_entry.identifier, 0x00001a00);

        let name: Option<FatString> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry: FatFileEntry = file_system.get_file_entry_by_identifier(0x00001a40)?;
        assert_eq!(file_entry.identifier, 0x00001a40);

        let name: Option<FatString> = file_entry.get_name();
        assert_eq!(name, Some(FatString::from("emptyfile")));

        let file_entry: FatFileEntry = file_system.get_file_entry_by_identifier(0x00006340)?;
        assert_eq!(file_entry.identifier, 0x00006340);

        let name: Option<FatString> = file_entry.get_name();
        assert_eq!(
            name,
            Some(FatString::from(
                "My long, very long file name, so very long"
            ))
        );

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: FatFileSystem = get_file_system()?;

        let fat_path: FatPath = FatPath::from("/");
        let file_entry: FatFileEntry = file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00001a00);

        let fat_path: FatPath = FatPath::from("/emptyfile");
        let file_entry: FatFileEntry = file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00001a40);

        // TODO: add support for short names
        // let fat_path: FatPath = FatPath::from("/EMPTYF~1");
        // let file_entry: FatFileEntry = file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        // assert_eq!(file_entry.identifier, 0x00001a40);

        let fat_path: FatPath = FatPath::from("/testdir1/testfile1");
        let file_entry: FatFileEntry = file_system.get_file_entry_by_path(&fat_path)?.unwrap();

        assert_eq!(file_entry.identifier, 0x00006260);

        let name: Option<FatString> = file_entry.get_name();
        assert_eq!(name, Some(FatString::from("testfile1")));

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: FatFileSystem = get_file_system()?;

        let file_entry: FatFileEntry = file_system.get_root_directory()?;

        assert_eq!(file_entry.identifier, 0x00001a00);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("fat/fat12.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert!(file_system.format == FatFormat::Fat12);
        assert_eq!(file_system.volume_serial_number, 0x56f30d5b);
        assert_eq!(
            file_system.volume_label,
            Some(ByteString::from("FAT12_TEST"))
        );

        Ok(())
    }

    // TODO: add tests for read_directory_entry_by_identifier

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file_system: FatFileSystem = FatFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("fat/fat12.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_metadata(&data_stream)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert!(file_system.format == FatFormat::Fat12);
        assert_eq!(file_system.volume_serial_number, 0x56f30d5b);
        assert_eq!(
            file_system.volume_label,
            Some(ByteString::from("FAT12_TEST"))
        );

        Ok(())
    }

    // TODO: add tests for read_root_directory
}
