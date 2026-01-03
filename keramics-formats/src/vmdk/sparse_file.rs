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

use keramics_compression::ZlibContext;
use keramics_core::{DataStreamReference, ErrorTrace};

use crate::block_tree::BlockTree;

use super::block_range::{VmdkBlockRange, VmdkBlockRangeType};
use super::compressed_grain_header::VmdkCompressedGrainHeader;
use super::constants::*;
use super::enums::VmdkCompressionMethod;
use super::sector_table::{VmdkSectorTable, VmdkSectorTableEntry};
use super::sparse_file_header::VmdkSparseFileHeader;

/// VMware Virtual Disk (VMDK) sparse file.
pub struct VmdkSparseFile {
    /// Data stream.
    pub(super) data_stream: Option<DataStreamReference>,

    /// Number of grain table entries.
    number_of_grain_table_entries: u32,

    /// Sectors per grain.
    pub sectors_per_grain: u64,

    /// Grain size.
    pub(super) grain_size: u64,

    /// Grain directory.
    grain_directory: VmdkSectorTable,

    /// Block tree.
    pub(super) block_tree: BlockTree<VmdkBlockRange>,

    /// Compression method.
    pub compression_method: VmdkCompressionMethod,
}

impl VmdkSparseFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            number_of_grain_table_entries: 0,
            sectors_per_grain: 0,
            grain_size: 0,
            grain_directory: VmdkSectorTable::new(),
            block_tree: BlockTree::<VmdkBlockRange>::new(0, 0, 0),
            compression_method: VmdkCompressionMethod::None,
        }
    }

    /// Reads a compressed grain.
    pub(super) fn read_compressed_grain(
        &mut self,
        mut grain_offset: u64,
        data: &mut [u8],
    ) -> Result<(), ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let mut compressed_grain_header: VmdkCompressedGrainHeader =
            VmdkCompressedGrainHeader::new();

        match compressed_grain_header.read_at_position(data_stream, SeekFrom::Start(grain_offset)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read compressed grain header at offset: {} (0x{:08x})",
                        grain_offset, grain_offset
                    )
                );
                return Err(error);
            }
        }
        grain_offset += 12;

        // Note that 16777216 is an arbitrary chosen limit.
        if compressed_grain_header.compressed_data_size == 0
            || compressed_grain_header.compressed_data_size > 16777216
        {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid compressed data size: {} value out of bounds",
                compressed_grain_header.compressed_data_size
            )));
        }
        let mut compressed_data: Vec<u8> =
            vec![0; compressed_grain_header.compressed_data_size as usize];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut compressed_data,
            SeekFrom::Start(grain_offset)
        );
        keramics_core::debug_trace_data!(
            "VmdkCompressedGrain",
            grain_offset,
            &compressed_data,
            compressed_grain_header.compressed_data_size
        );
        let mut zlib_context: ZlibContext = ZlibContext::new();

        match zlib_context.decompress(&compressed_data, data) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decompress grain data");
                return Err(error);
            }
        }
        Ok(())
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut file_header: VmdkSparseFileHeader = VmdkSparseFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        // file_header.primary_grain_directory_start_sector == -1 &&
        // file_header.compression_methods == Deflate
        // TODO: read secondary file header at file_size - 1024

        if file_header.number_of_grain_table_entries == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of grain table entries value out of bounds"
            ));
        }
        let grain_table_number_of_sectors: u64 =
            (file_header.number_of_grain_table_entries as u64) * file_header.sectors_per_grain;

        if grain_table_number_of_sectors > (u32::MAX / 512) as u64 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of grain table sectors value out of bounds"
            ));
        }
        // Note that 16777216 is an arbitrary chosen limit.
        if file_header.sectors_per_grain > 16777216 / 512 {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported sectors per grain: {} value out of bounds",
                file_header.sectors_per_grain
            )));
        }
        self.sectors_per_grain = file_header.sectors_per_grain;

        if file_header.maximum_data_number_of_sectors == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid maximum data number of sectors value out of bounds"
            ));
        }
        let number_of_grain_directory_entries: u64 = file_header
            .maximum_data_number_of_sectors
            .div_ceil(grain_table_number_of_sectors);

        if number_of_grain_directory_entries > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of grain directory entries value out of bounds"
            ));
        }
        // TODO: add support for GD_AT_END
        let grain_directory_offset: u64 =
            if file_header.flags & VMDK_SPARSE_FILE_FLAG_USE_SECONDARY_GRAIN_DIRECTORY == 0 {
                if file_header.primary_grain_directory_start_sector == 0
                    || file_header.primary_grain_directory_start_sector > u64::MAX / 512
                {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid primary grain directory start sector value out of bounds"
                    ));
                }
                file_header.primary_grain_directory_start_sector * 512
            } else {
                if file_header.secondary_grain_directory_start_sector == 0
                    || file_header.secondary_grain_directory_start_sector > u64::MAX / 512
                {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid secondary grain directory start sector value out of bounds"
                    ));
                }
                file_header.secondary_grain_directory_start_sector * 512
            };
        self.number_of_grain_table_entries = file_header.number_of_grain_table_entries;
        self.grain_size = (file_header.sectors_per_grain as u64) * 512;
        self.grain_directory.set_range(
            grain_directory_offset,
            number_of_grain_directory_entries as u32,
        );
        // TODO: check backup grain directory

        if file_header.flags & VMDK_SPARSE_FILE_FLAG_HAS_GRAIN_COMPRESSION != 0 {
            self.compression_method = VmdkCompressionMethod::Zlib;
        }
        let block_tree_data_size: u64 = number_of_grain_directory_entries
            * (self.number_of_grain_table_entries as u64)
            * self.grain_size;
        self.block_tree = BlockTree::<VmdkBlockRange>::new(
            block_tree_data_size,
            grain_table_number_of_sectors * 512,
            self.grain_size,
        );
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads a specific grain directory entry and fills the block tree.
    pub(super) fn read_grain_directory_entry(
        &mut self,
        extent_offset: u64,
    ) -> Result<(), ErrorTrace> {
        let grain_index: u64 = extent_offset / self.grain_size;
        let grain_directory_index: u64 = grain_index / (self.number_of_grain_table_entries as u64);

        if grain_directory_index > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(
                "Invalid grain directory index value out of bounds"
            ));
        }
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let entry: VmdkSectorTableEntry = match self
            .grain_directory
            .read_entry(data_stream, grain_directory_index as u32)
        {
            Ok(entry) => entry,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(
                    error,
                    format!(
                        "Unable to read grain directory entry: {}",
                        grain_directory_index
                    )
                );
                return Err(error);
            }
        };
        if entry.sector_number == 0 {
            let grain_extent_offset: u64 = grain_directory_index * self.grain_size;
            let grain_extent_size: u64 =
                (self.number_of_grain_table_entries as u64) * self.grain_size;

            let block_range: VmdkBlockRange = VmdkBlockRange::new(
                grain_extent_offset,
                0,
                grain_extent_size,
                VmdkBlockRangeType::InParentOrSparse,
            );
            match self
                .block_tree
                .insert_value(grain_extent_offset, grain_extent_size, block_range)
            {
                Ok(_) => {}
                Err(error) => {
                    return Err(keramics_core::error_trace_new_with_error!(
                        "Unable to insert block range into block tree",
                        error
                    ));
                }
            }
        } else {
            let grain_table_offset: u64 = (entry.sector_number as u64) * 512;

            match self.read_grain_table_entry(grain_table_offset, grain_index) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read grain table entry for grain directory entry: {}",
                            grain_directory_index
                        )
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    /// Reads a specific grain table entry and fills the block tree.
    fn read_grain_table_entry(
        &mut self,
        grain_table_offset: u64,
        grain_index: u64,
    ) -> Result<(), ErrorTrace> {
        let grain_table_index: u64 = grain_index % (self.number_of_grain_table_entries as u64);

        if grain_table_index > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid grain table index value out of bounds",
            )));
        }
        let mut grain_table: VmdkSectorTable = VmdkSectorTable::new();
        grain_table.set_range(grain_table_offset, self.number_of_grain_table_entries);

        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let entry: VmdkSectorTableEntry =
            match grain_table.read_entry(data_stream, grain_table_index as u32) {
                Ok(sector_number) => sector_number,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read grain table entry: {} at offset: {} (0x{:08x})",
                            grain_table_index, grain_table_offset, grain_table_offset
                        )
                    );
                    return Err(error);
                }
            };
        let grain_data_offset: u64 = (entry.sector_number as u64) * 512;
        let grain_extent_offset: u64 = grain_index * self.grain_size;

        let range_type: VmdkBlockRangeType = if entry.sector_number == 0 {
            VmdkBlockRangeType::InParentOrSparse
        } else if self.compression_method == VmdkCompressionMethod::Zlib {
            VmdkBlockRangeType::Compressed
        } else {
            VmdkBlockRangeType::InFile
        };
        let block_range: VmdkBlockRange = VmdkBlockRange::new(
            grain_extent_offset,
            grain_data_offset,
            self.grain_size,
            range_type,
        );
        match self
            .block_tree
            .insert_value(grain_extent_offset, self.grain_size, block_range)
        {
            Ok(_) => {}
            Err(error) => {
                return Err(keramics_core::error_trace_new_with_error!(
                    "Unable to insert block range into block tree",
                    error
                ));
            }
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

    fn get_file() -> Result<VmdkSparseFile, ErrorTrace> {
        let mut file: VmdkSparseFile = VmdkSparseFile::new();

        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    // TODO: add tests for read_compressed_grain

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: VmdkSparseFile = VmdkSparseFile::new();

        let path_string: String = get_test_data_path("vmdk/ext2.vmdk");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file.read_data_stream(&data_stream)?;
        assert_eq!(file.number_of_grain_table_entries, 512);
        assert_eq!(file.sectors_per_grain, 128);
        assert_eq!(file.grain_size, 65536);
        assert_eq!(file.compression_method, VmdkCompressionMethod::None);

        Ok(())
    }

    #[test]
    fn test_read_grain_directory_entry() -> Result<(), ErrorTrace> {
        let mut file: VmdkSparseFile = get_file()?;

        let result: Option<&VmdkBlockRange> = file.block_tree.get_value(0)?;
        assert_eq!(result, None);

        file.read_grain_directory_entry(0)?;

        let result: Option<&VmdkBlockRange> = file.block_tree.get_value(0)?;
        assert_eq!(
            result,
            Some(&VmdkBlockRange::new(
                0,
                65536,
                65536,
                VmdkBlockRangeType::InFile,
            ))
        );
        Ok(())
    }

    // TODO: add tests for read_grain_table_entry
}
