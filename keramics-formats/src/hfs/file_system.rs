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

use crate::path::Path;

use super::attributes_file::HfsAttributesFile;
use super::block_ranges::HfsBlockRanges;
use super::catalog_file::HfsCatalogFile;
use super::constants::*;
use super::directory_entry::HfsDirectoryEntry;
use super::enums::HfsFormat;
use super::extents_overflow_file::HfsExtentsOverflowFile;
use super::file_entry::HfsFileEntry;
use super::fork_descriptor::HfsForkDescriptor;
use super::master_directory_block::HfsMasterDirectoryBlock;
use super::volume_header::HfsVolumeHeader;

/// Hierarchical File System (HFS) file system.
pub struct HfsFileSystem {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format.
    format: HfsFormat,

    /// Block size.
    pub block_size: u32,

    /// Data area block number.
    data_area_block_number: u16,

    /// Extents overflow file.
    extents_overflow_file: Arc<HfsExtentsOverflowFile>,

    /// Catalog file.
    catalog_file: Arc<HfsCatalogFile>,

    /// Attributes file.
    attributes_file: Arc<HfsAttributesFile>,
}

impl HfsFileSystem {
    /// Creates a new file system.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format: HfsFormat::HfsPlus,
            block_size: 0,
            data_area_block_number: 0,
            extents_overflow_file: Arc::new(HfsExtentsOverflowFile::new()),
            catalog_file: Arc::new(HfsCatalogFile::new()),
            attributes_file: Arc::new(HfsAttributesFile::new()),
        }
    }

    /// Retrieves the file entry for a specific identifier (CNID).
    pub fn get_file_entry_by_identifier(
        &self,
        identifier: u32,
    ) -> Result<Option<HfsFileEntry>, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let directory_entry: HfsDirectoryEntry = match self
            .catalog_file
            .get_directory_entry_by_identifier(data_stream, identifier)
        {
            Ok(Some(directory_entry)) => directory_entry,
            Ok(None) => return Ok(None),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve directory entry: {} from catalog file",
                        identifier
                    )
                );
                return Err(error);
            }
        };
        let mut file_entry: HfsFileEntry = HfsFileEntry::new(
            data_stream,
            self.block_size,
            self.data_area_block_number,
            &self.catalog_file,
            &self.extents_overflow_file,
            &self.attributes_file,
            directory_entry,
        );
        match file_entry.read_indirect_node() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read indirect node");
                return Err(error);
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the file entry for a specific path.
    pub fn get_file_entry_by_path(&self, path: &Path) -> Result<Option<HfsFileEntry>, ErrorTrace> {
        if path.is_empty() || path.is_relative() {
            return Ok(None);
        }
        let mut file_entry: HfsFileEntry = match self.get_root_directory() {
            Ok(file_entry) => file_entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to retrieve root directory");
                return Err(error);
            }
        };
        // TODO: cache file entries.
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
        match file_entry.read_indirect_node() {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read indirect node");
                return Err(error);
            }
        }
        Ok(Some(file_entry))
    }

    /// Retrieves the root directory (file entry).
    pub fn get_root_directory(&self) -> Result<HfsFileEntry, ErrorTrace> {
        match self.get_file_entry_by_identifier(HFS_ROOT_DIRECTORY_IDENTIFIER) {
            Ok(Some(file_entry)) => Ok(file_entry),
            Ok(None) => Err(keramics_core::error_trace_new!(format!(
                "Missing file entry: {}",
                HFS_ROOT_DIRECTORY_IDENTIFIER
            ))),
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to retrieve file entry: {}",
                        HFS_ROOT_DIRECTORY_IDENTIFIER
                    )
                );
                Err(error)
            }
        }
    }

    /// Reads the attributes file.
    fn read_attributes_file(
        &mut self,
        data_stream: &DataStreamReference,
        fork_descriptor: &HfsForkDescriptor,
    ) -> Result<(), ErrorTrace> {
        if fork_descriptor.size == 0 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut block_ranges: HfsBlockRanges = HfsBlockRanges::new();

        match block_ranges.read_fork_descriptor(
            self.data_area_block_number,
            HFS_ATTRIBUTES_FILE_IDENTIFIER,
            fork_descriptor,
            data_stream,
            &self.extents_overflow_file,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine block ranges from fork descriptor"
                );
                return Err(error);
            }
        }
        match Arc::get_mut(&mut self.attributes_file) {
            Some(attributes_file) => {
                match attributes_file.initialize(
                    &self.format,
                    self.block_size,
                    fork_descriptor.size,
                    block_ranges.ranges,
                    data_stream,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize attributes file"
                        );
                        return Err(error);
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to attributes file"
                ));
            }
        }
        Ok(())
    }

    /// Reads the catalog file.
    fn read_catalog_file(
        &mut self,
        data_stream: &DataStreamReference,
        fork_descriptor: &HfsForkDescriptor,
    ) -> Result<(), ErrorTrace> {
        if fork_descriptor.size == 0 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut block_ranges: HfsBlockRanges = HfsBlockRanges::new();

        match block_ranges.read_fork_descriptor(
            self.data_area_block_number,
            HFS_CATALOG_FILE_IDENTIFIER,
            fork_descriptor,
            data_stream,
            &self.extents_overflow_file,
        ) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine block ranges from fork descriptor"
                );
                return Err(error);
            }
        }
        match Arc::get_mut(&mut self.catalog_file) {
            Some(catalog_file) => {
                match catalog_file.initialize(
                    &self.format,
                    self.block_size,
                    fork_descriptor.size,
                    block_ranges.ranges,
                    data_stream,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize catalog file"
                        );
                        return Err(error);
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to attributes file"
                ));
            }
        }
        Ok(())
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

    /// Reads the extents overflow file.
    fn read_extents_overflow_file(
        &mut self,
        data_stream: &DataStreamReference,
        fork_descriptor: &HfsForkDescriptor,
    ) -> Result<(), ErrorTrace> {
        if fork_descriptor.size == 0 {
            return Err(keramics_core::error_trace_new!("Unsupported file size"));
        }
        let mut block_ranges: HfsBlockRanges = HfsBlockRanges::new();

        match block_ranges.read_extents(self.data_area_block_number, &fork_descriptor.extents) {
            Ok(result) => result,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    "Unable to determine block ranges from fork descriptor"
                );
                return Err(error);
            }
        }
        if block_ranges.number_of_blocks < fork_descriptor.number_of_blocks {
            return Err(keramics_core::error_trace_new!(
                "Unsupported extents overflow"
            ));
        }
        match Arc::get_mut(&mut self.extents_overflow_file) {
            Some(extents_overflow_file) => {
                match extents_overflow_file.initialize(
                    &self.format,
                    self.block_size,
                    fork_descriptor.size,
                    block_ranges.ranges,
                    data_stream,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to initialize extents overflow file"
                        );
                        return Err(error);
                    }
                }
            }
            None => {
                return Err(keramics_core::error_trace_new!(
                    "Unable to obtain mutable reference to extents overflow file"
                ));
            }
        }
        Ok(())
    }

    /// Reads metadata from the master directory block or volume header.
    fn read_metadata(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let mut data: Vec<u8> = vec![0; 512];

        let offset: u64 = keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(1024)
        );
        match &data[0..2] {
            HFS_MASTER_DIRECTORY_BLOCK_SIGNATURE => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsMasterBootRecord",
                    offset,
                    &data,
                    512,
                    HfsMasterDirectoryBlock::debug_read_data(&data)
                );
                let mut master_directory_block: HfsMasterDirectoryBlock =
                    HfsMasterDirectoryBlock::new();

                match master_directory_block.read_data(&data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read master directory block"
                        );
                        return Err(error);
                    }
                }
                self.format = HfsFormat::Hfs;
                self.block_size = master_directory_block.block_size;
                self.data_area_block_number = master_directory_block.data_area_block_number;

                let fork_descriptor: HfsForkDescriptor = HfsForkDescriptor {
                    size: master_directory_block.extents_overflow_file_size as u64,
                    number_of_blocks: master_directory_block
                        .extents_overflow_file_size
                        .div_ceil(self.block_size),
                    extents: master_directory_block.extents_overflow_file_extents,
                };
                match self.read_extents_overflow_file(data_stream, &fork_descriptor) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read extents overflow file"
                        );
                        return Err(error);
                    }
                }
                let fork_descriptor: HfsForkDescriptor = HfsForkDescriptor {
                    size: master_directory_block.catalog_file_size as u64,
                    number_of_blocks: master_directory_block
                        .catalog_file_size
                        .div_ceil(self.block_size),
                    extents: master_directory_block.catalog_file_extents,
                };
                match self.read_catalog_file(data_stream, &fork_descriptor) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read catalog file");
                        return Err(error);
                    }
                }
            }
            HFSPLUS_VOLUME_HEADER_SIGNATURE | HFSX_VOLUME_HEADER_SIGNATURE => {
                keramics_core::debug_trace_data_and_structure!(
                    "HfsVolumeHeader",
                    offset,
                    &data,
                    512,
                    HfsVolumeHeader::debug_read_data(&data)
                );
                let mut volume_header: HfsVolumeHeader = HfsVolumeHeader::new();

                match volume_header.read_data(&data) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read volume header"
                        );
                        return Err(error);
                    }
                }
                self.format = volume_header.format;
                self.block_size = volume_header.block_size;

                match self.read_extents_overflow_file(
                    data_stream,
                    &volume_header.extents_overflow_file_fork_descriptor,
                ) {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(
                            error,
                            "Unable to read extents overflow file"
                        );
                        return Err(error);
                    }
                }
                match self
                    .read_catalog_file(data_stream, &volume_header.catalog_file_fork_descriptor)
                {
                    Ok(_) => {}
                    Err(mut error) => {
                        keramics_core::error_trace_add_frame!(error, "Unable to read catalog file");
                        return Err(error);
                    }
                }
                if volume_header.attributes_file_fork_descriptor.size > 0 {
                    match self.read_attributes_file(
                        data_stream,
                        &volume_header.attributes_file_fork_descriptor,
                    ) {
                        Ok(_) => {}
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to read attributes file"
                            );
                            return Err(error);
                        }
                    }
                }
            }
            _ => return Err(keramics_core::error_trace_new!("Unsupported signature")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file_system(path_string: &str) -> Result<HfsFileSystem, ErrorTrace> {
        let mut file_system: HfsFileSystem = HfsFileSystem::new();

        let test_data_path_string: String = get_test_data_path(path_string);
        let path_buf: PathBuf = PathBuf::from(test_data_path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        Ok(file_system)
    }

    // Tests with HFS.

    #[test]
    fn test_get_file_entry_by_identifier_with_hfs() -> Result<(), ErrorTrace> {
        let file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let file_entry: HfsFileEntry = file_system.get_file_entry_by_identifier(18)?.unwrap();
        assert_eq!(file_entry.identifier, 18);

        let result: Option<HfsFileEntry> = file_system.get_file_entry_by_identifier(0xffffffff)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_directory_with_hfs() -> Result<(), ErrorTrace> {
        let file_system: HfsFileSystem = get_file_system("hfs/hfs.raw")?;

        let root_directory: HfsFileEntry = file_system.get_root_directory()?;
        assert_eq!(root_directory.identifier, HFS_ROOT_DIRECTORY_IDENTIFIER);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_hfs() -> Result<(), ErrorTrace> {
        let mut file_system: HfsFileSystem = HfsFileSystem::new();

        let path_string: String = get_test_data_path("hfs/hfs.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.format, HfsFormat::Hfs);

        Ok(())
    }

    // Tests with HFS+.

    #[test]
    fn test_get_file_entry_by_identifier_with_hfsplus() -> Result<(), ErrorTrace> {
        let file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let file_entry: HfsFileEntry = file_system.get_file_entry_by_identifier(21)?.unwrap();
        assert_eq!(file_entry.identifier, 21);

        let result: Option<HfsFileEntry> = file_system.get_file_entry_by_identifier(0xffffffff)?;
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_get_root_directory_with_hfsplus() -> Result<(), ErrorTrace> {
        let file_system: HfsFileSystem = get_file_system("hfs/hfsplus.raw")?;

        let root_directory: HfsFileEntry = file_system.get_root_directory()?;
        assert_eq!(root_directory.identifier, HFS_ROOT_DIRECTORY_IDENTIFIER);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_hfsplus() -> Result<(), ErrorTrace> {
        let mut file_system: HfsFileSystem = HfsFileSystem::new();

        let path_string: String = get_test_data_path("hfs/hfsplus.raw");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file_system.read_data_stream(&data_stream)?;

        assert_eq!(file_system.format, HfsFormat::HfsPlus);

        Ok(())
    }
}
