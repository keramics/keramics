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
use std::rc::Rc;
use std::sync::Arc;

use crate::types::{SharedValue, Ucs2String};
use crate::vfs::{VfsDataStreamReference, VfsFileSystem, VfsPath};

use super::boot_record::NtfsBootRecord;
use super::constants::*;
use super::file_entry::NtfsFileEntry;
use super::master_file_table::NtfsMasterFileTable;
use super::mft_attribute::NtfsMftAttribute;
use super::mft_entry::NtfsMftEntry;
use super::path::NtfsPath;
use super::volume_information_attribute::NtfsVolumeInformationAttribute;

/// New Technologies File System (NTFS).
pub struct NtfsFileSystem {
    /// Data stream.
    data_stream: VfsDataStreamReference,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Cluster block size.
    pub cluster_block_size: u32,

    /// MFT entry size.
    pub mft_entry_size: u32,

    /// Index entry size.
    pub index_entry_size: u32,

    /// Master File Table (MFT).
    mft: Rc<NtfsMasterFileTable>,

    /// $VOLUME_INFORMATION attribute of the "$Volume" metadata file.
    volume_information_attribute: Option<NtfsVolumeInformationAttribute>,

    /// Volume label from the $VOLUME_NAME attribute of the "$Volume" metadata file.
    volume_label: Option<Ucs2String>,

    /// Volume serial number.
    pub volume_serial_number: u64,
}

impl NtfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: SharedValue::none(),
            bytes_per_sector: 0,
            cluster_block_size: 0,
            mft_entry_size: 0,
            index_entry_size: 0,
            mft: Rc::new(NtfsMasterFileTable::new()),
            volume_information_attribute: None,
            volume_label: None,
            volume_serial_number: 0,
        }
    }

    /// Retrieves the format version.
    pub fn get_format_version(&self) -> Option<(u8, u8)> {
        match &self.volume_information_attribute {
            Some(volume_information_attribute) => Some((
                volume_information_attribute.major_format_version,
                volume_information_attribute.minor_format_version,
            )),
            None => None,
        }
    }

    /// Retrieves the volume flags.
    pub fn get_volume_flags(&self) -> Option<u16> {
        match &self.volume_information_attribute {
            Some(volume_information_attribute) => Some(volume_information_attribute.volume_flags),
            None => None,
        }
    }

    /// Retrieves the volume label.
    pub fn get_volume_label(&self) -> Option<&Ucs2String> {
        self.volume_label.as_ref()
    }

    /// Retrieves the file entry for a specific identifier (MFT entry number).
    pub fn get_file_entry_by_identifier(&self, mft_entry_number: u64) -> io::Result<NtfsFileEntry> {
        if mft_entry_number >= self.mft.number_of_entries {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Invalid MFT entry number: {} value out of bounds",
                    mft_entry_number
                ),
            ));
        }
        let mut mft_entry: NtfsMftEntry =
            self.mft.get_entry(&self.data_stream, mft_entry_number)?;
        mft_entry.read_attributes()?;

        let file_entry: NtfsFileEntry = NtfsFileEntry::new(
            &self.data_stream,
            &self.mft,
            mft_entry_number,
            mft_entry,
            None,
            None,
        );
        Ok(file_entry)
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &NtfsPath) -> io::Result<Option<NtfsFileEntry>> {
        if path.is_empty() || path.components[0].len() != 0 {
            return Ok(None);
        }
        let mut file_entry: NtfsFileEntry = self.get_root_directory()?;

        // TODO: cache file entries.
        for path_component in path.components[1..].iter() {
            file_entry = match file_entry.get_sub_file_entry_by_name(path_component)? {
                Some(file_entry) => file_entry,
                None => return Ok(None),
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> io::Result<NtfsFileEntry> {
        self.get_file_entry_by_identifier(NTFS_ROOT_DIRECTORY_IDENTIFIER)
    }

    // TODO: add method to retrieve USN journal file entry

    /// Opens a file system.
    pub fn open(&mut self, file_system: &Arc<VfsFileSystem>, path: &VfsPath) -> io::Result<()> {
        self.data_stream = match file_system.get_data_stream_by_path_and_name(path, None)? {
            Some(data_stream) => data_stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("No such file: {}", path.to_string()),
                ))
            }
        };
        self.read_metadata()
    }

    /// Reads the boot record, master file table and security descriptors.
    fn read_metadata(&mut self) -> io::Result<()> {
        let mut boot_record: NtfsBootRecord = NtfsBootRecord::new();
        boot_record.read_at_position(&self.data_stream, io::SeekFrom::Start(0))?;

        self.bytes_per_sector = boot_record.bytes_per_sector;
        self.cluster_block_size = boot_record.cluster_block_size;
        self.mft_entry_size = boot_record.mft_entry_size;
        self.index_entry_size = boot_record.index_entry_size;
        self.volume_serial_number = boot_record.volume_serial_number;

        self.read_master_file_table(boot_record.mft_block_number)?;
        self.read_volume_information()?;

        // TODO: read security descriptors, MFT entry 9 ($Secure)
        // let mft_entry: NtfsMftEntry = self.mft.get_entry(&self.data_stream, 9)?;

        Ok(())
    }

    /// Reads the master file table.
    fn read_master_file_table(&mut self, mft_block_number: u64) -> io::Result<()> {
        if mft_block_number > u64::MAX / (self.cluster_block_size as u64) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unsupported MFT block number: {} value out of bounds",
                    mft_block_number
                ),
            ));
        }
        let mft_offset: u64 = mft_block_number * (self.cluster_block_size as u64);

        let mut mft_entry: NtfsMftEntry = NtfsMftEntry::new();

        mft_entry.read_at_position(
            &self.data_stream,
            self.mft_entry_size,
            io::SeekFrom::Start(mft_offset),
        )?;
        if mft_entry.is_bad {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported marked bad MFT entry.",
            ));
        }
        if !mft_entry.is_allocated {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported unallocated MFT entry.",
            ));
        }
        mft_entry.read_attributes()?;

        let data_attribute: &NtfsMftAttribute =
            match mft_entry.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_DATA) {
                Some(mft_attribute) => mft_attribute,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported MFT missing $Data attribute.",
                    ))
                }
            };
        match Rc::get_mut(&mut self.mft) {
            Some(mft) => {
                mft.initialize(self.cluster_block_size, self.mft_entry_size, data_attribute)?
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unable to initialize master file table"),
                ));
            }
        };
        // TODO: read for attribute list if appropriate

        Ok(())
    }

    /// Reads the volume information from the $Volume metadata file.
    fn read_volume_information(&mut self) -> io::Result<()> {
        let mut mft_entry: NtfsMftEntry = self
            .mft
            .get_entry(&self.data_stream, NTFS_VOLUME_INFORMATION_FILE_IDENTIFIER)?;
        if mft_entry.is_allocated {
            mft_entry.read_attributes()?;

            match mft_entry.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_VOLUME_NAME) {
                Some(mft_attribute) => {
                    if !mft_attribute.is_resident() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported non-resident $VOLUME_NAME attribute.",
                        ));
                    }
                    if mft_attribute.is_compressed() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported compressed $VOLUME_NAME attribute.",
                        ));
                    }
                    let volume_label: Ucs2String =
                        Ucs2String::from_le_bytes(&mft_attribute.resident_data);

                    self.volume_label = Some(volume_label);
                }
                None => {}
            };
            match mft_entry.get_attribute(&None, NTFS_ATTRIBUTE_TYPE_VOLUME_INFORMATION) {
                Some(mft_attribute) => {
                    if !mft_attribute.is_resident() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported non-resident $VOLUME_INFORMATION attribute.",
                        ));
                    }
                    if mft_attribute.is_compressed() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Unsupported compressed $VOLUME_INFORMATION attribute.",
                        ));
                    }
                    let mut volume_information_attribute: NtfsVolumeInformationAttribute =
                        NtfsVolumeInformationAttribute::new();
                    volume_information_attribute.read_data(&mft_attribute.resident_data)?;

                    self.volume_information_attribute = Some(volume_information_attribute);
                }
                None => {}
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::vfs::{VfsContext, VfsPath};

    #[test]
    fn test_open() -> io::Result<()> {
        let mut vfs_context: VfsContext = VfsContext::new();

        let vfs_path: VfsPath = VfsPath::Os {
            location: "../test_data/ntfs/ntfs.raw".to_string(),
        };
        let vfs_file_system: Arc<VfsFileSystem> = vfs_context.open_file_system(&vfs_path)?;

        let mut file_system: NtfsFileSystem = NtfsFileSystem::new();

        file_system.open(&vfs_file_system, &vfs_path)?;

        assert_eq!(file_system.bytes_per_sector, 512);
        assert_eq!(file_system.cluster_block_size, 4096);
        assert_eq!(file_system.mft_entry_size, 1024);
        assert_eq!(file_system.index_entry_size, 4096);
        assert_eq!(file_system.volume_serial_number, 0x39fc0da25d085bcb);

        Ok(())
    }
}
