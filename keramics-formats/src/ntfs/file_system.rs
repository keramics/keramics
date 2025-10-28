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

use keramics_core::{DataStream, DataStreamReference, ErrorTrace};
use keramics_types::{Ucs2String, bytes_to_u16_le};

use super::block_stream::NtfsBlockStream;
use super::boot_record::NtfsBootRecord;
use super::constants::*;
use super::file_entry::NtfsFileEntry;
use super::master_file_table::NtfsMasterFileTable;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_attributes::NtfsMftAttributes;
use super::mft_entry::NtfsMftEntry;
use super::path::NtfsPath;
use super::volume_information::NtfsVolumeInformation;

/// New Technologies File System (NTFS).
pub struct NtfsFileSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Cluster block size.
    pub cluster_block_size: u32,

    /// MFT entry size.
    pub mft_entry_size: u32,

    /// Index entry size.
    pub index_entry_size: u32,

    /// Master File Table (MFT).
    mft: Arc<NtfsMasterFileTable>,

    /// Case folding mappings.
    case_folding_mappings: Arc<HashMap<u16, u16>>,

    /// Volume information from the $VOLUME_INFORMATION attribute of the "$Volume" metadata file.
    volume_information: Option<NtfsVolumeInformation>,

    /// Volume label from the $VOLUME_NAME attribute of the "$Volume" metadata file.
    volume_label: Option<Ucs2String>,

    /// Volume serial number.
    pub volume_serial_number: u64,
}

impl NtfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            bytes_per_sector: 0,
            cluster_block_size: 0,
            mft_entry_size: 0,
            index_entry_size: 0,
            mft: Arc::new(NtfsMasterFileTable::new()),
            case_folding_mappings: Arc::new(HashMap::new()),
            volume_information: None,
            volume_label: None,
            volume_serial_number: 0,
        }
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> Option<(u8, u8)> {
        match &self.volume_information {
            Some(volume_information) => Some((
                volume_information.major_format_version,
                volume_information.minor_format_version,
            )),
            None => None,
        }
    }

    /// Retrieves the volume flags.
    pub fn get_volume_flags(&self) -> Option<u16> {
        match &self.volume_information {
            Some(volume_information) => Some(volume_information.volume_flags),
            None => None,
        }
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&Ucs2String> {
        self.volume_label.as_ref()
    }

    /// Retrieves the file entry for a specific identifier (MFT entry number).
    pub fn get_file_entry_by_identifier(
        &self,
        mft_entry_number: u64,
    ) -> Result<NtfsFileEntry, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        if mft_entry_number >= self.mft.number_of_entries {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid MFT entry number: {} value out of bounds",
                mft_entry_number
            )));
        }
        let mft_entry: NtfsMftEntry = match self.mft.get_entry(data_stream, mft_entry_number) {
            Ok(mft_entry) => mft_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!("Unable to retrieve MFT entry: {}", mft_entry_number)
                );
                return Err(error);
            }
        };
        let mut file_entry: NtfsFileEntry = NtfsFileEntry::new(
            data_stream,
            &self.mft,
            &self.case_folding_mappings,
            mft_entry_number,
            mft_entry,
            None,
            None,
        );
        match file_entry.read_attributes() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        Ok(file_entry)
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(
        &self,
        path: &NtfsPath,
    ) -> Result<Option<NtfsFileEntry>, ErrorTrace> {
        if path.is_empty() || path.components[0].len() != 0 {
            return Ok(None);
        }
        let mut file_entry: NtfsFileEntry = match self.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            let result: Option<NtfsFileEntry> =
                match file_entry.get_sub_file_entry_by_name(path_component) {
                    Ok(result) => result,
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            format!("Unable to retrieve sub file entry: {}", path.to_string())
                        );
                        return Err(error);
                    }
                };
            file_entry = match result {
                Some(file_entry) => file_entry,
                None => return Ok(None),
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<NtfsFileEntry, ErrorTrace> {
        match self.get_file_entry_by_identifier(NTFS_ROOT_DIRECTORY_IDENTIFIER) {
            Ok(file_entry) => Ok(file_entry),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        }
    }

    // TODO: add method to retrieve USN journal file entry

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
        match self.read_volume_information(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read volume information");
                return Err(error);
            }
        }
        match self.read_case_folding_mappings(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read case folding mappings"
                );
                return Err(error);
            }
        }
        // TODO: read security descriptors, MFT entry 9 ($Secure)

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the boot record and master file table.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut boot_record: NtfsBootRecord = NtfsBootRecord::new();

        match boot_record.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read boot record");
                return Err(error);
            }
        }
        self.bytes_per_sector = boot_record.bytes_per_sector;
        self.cluster_block_size = boot_record.cluster_block_size;
        self.mft_entry_size = boot_record.mft_entry_size;
        self.index_entry_size = boot_record.index_entry_size;
        self.volume_serial_number = boot_record.volume_serial_number;

        match self.read_master_file_table(data_stream, boot_record.mft_block_number) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read master file table");
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads the case folding mappings from the $UpCase metadata file.
    fn read_case_folding_mappings(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mft_entry: NtfsMftEntry = match self
            .mft
            .get_entry(data_stream, NTFS_CASE_FOLDING_MAPPIINGS_FILE_IDENTIFIER)
        {
            Ok(mft_entry) => mft_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve MFT entry: {}",
                        NTFS_CASE_FOLDING_MAPPIINGS_FILE_IDENTIFIER
                    )
                );
                return Err(error);
            }
        };
        if mft_entry.is_bad {
            return Err(keramics_core::error_trace_new!(
                "Unsupported marked bad MFT entry"
            ));
        }
        if !mft_entry.is_allocated {
            return Err(keramics_core::error_trace_new!(
                "Unsupported unallocated MFT entry"
            ));
        }
        let mut mft_attributes: NtfsMftAttributes = NtfsMftAttributes::new();

        match mft_entry.read_attributes(&mut mft_attributes) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        if mft_attributes.attribute_list.is_some() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported MFT entry with attribute list"
            ));
        }
        let data_attribute: &NtfsMftAttribute =
            match mft_attributes.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(data_attribute) => data_attribute,
                None => {
                    return Err(keramics_core::error_trace_new!("Missing $Data attribute"));
                }
            };
        if data_attribute.is_resident() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported resident $Data attribute"
            ));
        }
        if data_attribute.is_compressed() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported compressed $Data attribute"
            ));
        }
        let mut block_stream: NtfsBlockStream = NtfsBlockStream::new(self.mft.cluster_block_size);

        match block_stream.open(data_stream, data_attribute) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open block stream");
                return Err(error);
            }
        }
        let mut data: Vec<u8> = vec![0; 131072];

        match block_stream.read_exact_at_position(&mut data, SeekFrom::Start(0)) {
            Ok(offset) => offset,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to read case folding mappings"
                );
                return Err(error);
            }
        };
        match Arc::get_mut(&mut self.case_folding_mappings) {
            Some(case_folding_mappings) => {
                let mut data_offset: usize = 0;
                for character_value in 0..=65535 {
                    let value_16bit: u16 = bytes_to_u16_le!(data, data_offset);
                    data_offset += 2;

                    if character_value != value_16bit {
                        case_folding_mappings.insert(character_value, value_16bit);
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to initialize case folding mappings"
                ));
            }
        };
        Ok(())
    }

    /// Reads the master file table.
    fn read_master_file_table(
        &mut self,
        data_stream: &DataStreamReference,
        mft_block_number: u64,
    ) -> Result<(), ErrorTrace> {
        if mft_block_number > u64::MAX / (self.cluster_block_size as u64) {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported MFT block number: {} value out of bounds",
                mft_block_number
            )));
        }
        match Arc::get_mut(&mut self.mft) {
            Some(mft) => match mft.initialize(
                self.cluster_block_size,
                self.mft_entry_size,
                data_stream,
                mft_block_number,
            ) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        "Unable to initialize master file table"
                    );
                    return Err(error);
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to master file table"
                ));
            }
        };
        Ok(())
    }

    /// Reads the volume information from the $Volume metadata file.
    fn read_volume_information(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mft_entry: NtfsMftEntry = match self
            .mft
            .get_entry(data_stream, NTFS_VOLUME_INFORMATION_FILE_IDENTIFIER)
        {
            Ok(mft_entry) => mft_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve MFT entry: {}",
                        NTFS_VOLUME_INFORMATION_FILE_IDENTIFIER
                    )
                );
                return Err(error);
            }
        };
        if mft_entry.is_bad {
            return Err(keramics_core::error_trace_new!(
                "Unsupported marked bad MFT entry"
            ));
        }
        if !mft_entry.is_allocated {
            return Err(keramics_core::error_trace_new!(
                "Unsupported unallocated MFT entry"
            ));
        }
        let mut mft_attributes: NtfsMftAttributes = NtfsMftAttributes::new();

        match mft_entry.read_attributes(&mut mft_attributes) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read attributes");
                return Err(error);
            }
        }
        if mft_attributes.attribute_list.is_some() {
            return Err(keramics_core::error_trace_new!(
                "Unsupported MFT entry with attribute list"
            ));
        }
        match mft_attributes.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_VOLUME_NAME) {
            Some(mft_attribute) => {
                if !mft_attribute.is_resident() {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported non-resident $VOLUME_NAME attribute"
                    ));
                }
                if mft_attribute.is_compressed() {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported compressed $VOLUME_NAME attribute"
                    ));
                }
                let volume_label: Ucs2String =
                    Ucs2String::from_le_bytes(&mft_attribute.resident_data);

                self.volume_label = Some(volume_label);
            }
            None => {}
        };
        match mft_attributes.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION) {
            Some(mft_attribute) => {
                let volume_information: NtfsVolumeInformation =
                    match NtfsVolumeInformation::from_attribute(mft_attribute) {
                        Ok(volume_information) => volume_information,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to create volume information from attribute"
                            );
                            return Err(error);
                        }
                    };
                self.volume_information = Some(volume_information);
            }
            None => {}
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file_system() -> Result<NtfsFileSystem, ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    #[test]
    fn test_get_format_version() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let format_version: Option<(u8, u8)> = file_system.get_format_version();
        assert_eq!(format_version, Some((3, 1)));

        Ok(())
    }

    #[test]
    fn test_get_volume_flags() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let volume_flags: Option<u16> = file_system.get_volume_flags();
        assert_eq!(volume_flags, Some(0x0000));

        Ok(())
    }

    #[test]
    fn test_get_volume_label() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let volume_label: Option<&Ucs2String> = file_system.get_volume_label();
        let expected_label: Ucs2String = Ucs2String::from("ntfs_test");
        assert_eq!(volume_label, Some(&expected_label));

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_identifier() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let file_entry: NtfsFileEntry = file_system.get_file_entry_by_identifier(5)?;
        assert_eq!(file_entry.mft_entry_number, 5);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert!(name.is_none());

        let file_entry: NtfsFileEntry = file_system.get_file_entry_by_identifier(64)?;
        assert_eq!(file_entry.mft_entry_number, 64);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert!(name.is_none());

        Ok(())
    }

    #[test]
    fn test_get_file_entry_by_path() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let ntfs_path: NtfsPath = NtfsPath::from("\\");
        let file_entry: NtfsFileEntry = file_system.get_file_entry_by_path(&ntfs_path)?.unwrap();

        assert_eq!(file_entry.mft_entry_number, 5);

        let ntfs_path: NtfsPath = NtfsPath::from("\\emptyfile");
        let file_entry: NtfsFileEntry = file_system.get_file_entry_by_path(&ntfs_path)?.unwrap();

        assert_eq!(file_entry.mft_entry_number, 64);

        let ntfs_path: NtfsPath = NtfsPath::from("\\testdir1\\testfile1");
        let file_entry: NtfsFileEntry = file_system.get_file_entry_by_path(&ntfs_path)?.unwrap();

        assert_eq!(file_entry.mft_entry_number, 66);

        let name: Option<&Ucs2String> = file_entry.get_name();
        assert_eq!(name, Some(Ucs2String::from("testfile1")).as_ref());

        Ok(())
    }

    #[test]
    fn test_get_root_directory() -> Result<(), ErrorTrace> {
        let file_system: NtfsFileSystem = get_file_system()?;

        let root_directory: NtfsFileEntry = file_system.get_root_directory()?;
        assert_eq!(
            root_directory.mft_entry_number,
            NTFS_ROOT_DIRECTORY_IDENTIFIER
        );
        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert_eq!(file_system.cluster_block_size, 4096);
        assert_eq!(file_system.mft_entry_size, 1024);
        assert_eq!(file_system.index_entry_size, 4096);
        assert_eq!(file_system.volume_serial_number, 0x39fc0da25d085bcb);

        Ok(())
    }

    #[test]
    fn test_read_metadata() -> Result<(), ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_metadata(&data_stream)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert_eq!(file_system.cluster_block_size, 4096);
        assert_eq!(file_system.mft_entry_size, 1024);
        assert_eq!(file_system.index_entry_size, 4096);
        assert_eq!(file_system.volume_serial_number, 0x39fc0da25d085bcb);

        Ok(())
    }

    #[test]
    fn test_read_case_folding_mappings() -> Result<(), ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_metadata(&data_stream)?;

        assert_eq!(file_system.case_folding_mappings.len(), 0);

        file_system.read_case_folding_mappings(&data_stream)?;

        assert_eq!(file_system.case_folding_mappings.len(), 973);

        Ok(())
    }

    #[test]
    fn test_read_master_file_table() -> Result<(), ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file_system.cluster_block_size = 4096;
        file_system.mft_entry_size = 1024;

        file_system.read_master_file_table(&data_stream, 4)?;

        Ok(())
    }

    #[test]
    fn test_read_volume_information() -> Result<(), ErrorTrace> {
        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ntfs/ntfs.raw").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_metadata(&data_stream)?;

        assert!(file_system.volume_information.is_none());

        file_system.read_volume_information(&data_stream)?;

        assert!(file_system.volume_information.is_some());

        Ok(())
    }
}
