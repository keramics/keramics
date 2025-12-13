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

use std::io::SeekFrom;

use keramics_core::{DataStreamReference, ErrorTrace};

use super::sector_table::{VmdkSectorTable, VmdkSectorTableEntry};
use super::sparse_cowd_file_header::VmdkSparseCowdFileHeader;

/// VMware Virtual Disk (VMDK) sparse Copy-On-Write Disk (COWD) file.
pub struct VmdkSparseCowdFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Grain table size.
    grain_table_size: u64,

    /// Sectors per grain.
    pub sectors_per_grain: u32,

    /// Grain size.
    grain_size: u64,

    /// Grain directory.
    grain_directory: VmdkSectorTable,
}

impl VmdkSparseCowdFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            grain_table_size: 0,
            sectors_per_grain: 0,
            grain_size: 0,
            grain_directory: VmdkSectorTable::new(),
        }
    }

    /// Retrieves the grain data offset.
    pub fn get_grain_data_offset(&mut self, extent_offset: u64) -> Result<u64, ErrorTrace> {
        let grain_index: u64 = extent_offset / self.grain_size;
        let grain_directory_index: u64 = grain_index / self.grain_table_size;

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
        let mut grain_table: VmdkSectorTable = VmdkSectorTable::new();

        let grain_table_offset: u64 = (entry.sector_number as u64) * 512;

        grain_table.set_range(grain_table_offset, (self.grain_table_size / 4) as u32);
        let grain_table_index: u64 = grain_index % self.grain_table_size;

        if grain_table_index > u32::MAX as u64 {
            return Err(keramics_core::error_trace_new!(format!(
                "Invalid grain table: {} index value out of bounds",
                grain_directory_index
            )));
        }
        let entry: VmdkSectorTableEntry =
            match grain_table.read_entry(data_stream, grain_table_index as u32) {
                Ok(sector_number) => sector_number,
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read grain table: {} entry: {}",
                            grain_directory_index, grain_table_index
                        )
                    );
                    return Err(error);
                }
            };
        let grain_data_offset: u64 = (entry.sector_number as u64) * 512;

        Ok(grain_data_offset)
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let mut file_header: VmdkSparseCowdFileHeader = VmdkSparseCowdFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        if file_header.grain_directory_start_sector == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid grain directory start sector value out of bounds"
            ));
        }
        if file_header.number_of_grain_directory_entries == 0 {
            return Err(keramics_core::error_trace_new!(
                "Invalid number of grain directory entries value out of bounds"
            ));
        }
        let grain_directory_offset: u64 = (file_header.grain_directory_start_sector as u64) * 512;

        self.grain_table_size = 4096 * 512;
        self.sectors_per_grain = file_header.sectors_per_grain;
        self.grain_size = (file_header.sectors_per_grain as u64) * 512;
        self.grain_directory.set_range(
            grain_directory_offset,
            file_header.number_of_grain_directory_entries,
        );
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    fn get_file() -> Result<VmdkSparseCowdFile, ErrorTrace> {
        let mut file: VmdkSparseCowdFile = VmdkSparseCowdFile::new();

        let path_string: String = get_test_data_path("vmdk/ext2.cowd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_get_grain_data_offset() -> Result<(), ErrorTrace> {
        let mut file: VmdkSparseCowdFile = get_file()?;

        let grain_data_offset: u64 = file.get_grain_data_offset(0)?;
        assert_eq!(grain_data_offset, 65536);

        Ok(())
    }

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file: VmdkSparseCowdFile = VmdkSparseCowdFile::new();

        let path_string: String = get_test_data_path("vmdk/ext2.cowd");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;

        file.read_data_stream(&data_stream)?;
        assert_eq!(file.grain_table_size, 2097152);
        assert_eq!(file.sectors_per_grain, 128);
        assert_eq!(file.grain_size, 65536);

        Ok(())
    }
}
