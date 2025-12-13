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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::vhdx::{VhdxDiskType, VhdxFile};
use keramics_types::{Ucs2String, Uuid};

use crate::formatters::ByteSize;

/// Information about a Virtual Hard Disk (VHDX) file.
struct VhdxFileInfo {
    /// Format version.
    pub format_version: u16,

    /// Disk type.
    pub disk_type: VhdxDiskType,

    /// Identifier.
    pub identifier: Uuid,

    /// Parent identifier.
    pub parent_identifier: Option<Uuid>,

    /// Parent name.
    pub parent_name: Option<Ucs2String>,

    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,
}

impl VhdxFileInfo {
    const DISK_TYPES: &[(VhdxDiskType, &'static str); 3] = &[
        (VhdxDiskType::Differential, "Differential"),
        (VhdxDiskType::Dynamic, "Dynamic"),
        (VhdxDiskType::Fixed, "Fixed"),
    ];

    /// Creates new file information.
    fn new() -> Self {
        Self {
            format_version: 0,
            disk_type: VhdxDiskType::Unknown,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
            media_size: 0,
            bytes_per_sector: 0,
        }
    }

    /// Retrieves the disk type as a string.
    pub fn get_disk_type_string(&self) -> &str {
        Self::DISK_TYPES
            .binary_search_by(|(key, _)| key.cmp(&self.disk_type))
            .map_or_else(|_| "Unknown", |index| Self::DISK_TYPES[index].1)
    }
}

impl fmt::Display for VhdxFileInfo {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Virtual Hard Disk (VHDX) information:")?;

        writeln!(
            formatter,
            "    Format version\t\t\t\t: 2.{}",
            self.format_version
        )?;
        let disk_type_string: &str = self.get_disk_type_string();
        writeln!(formatter, "    Disk type\t\t\t\t\t: {}", disk_type_string)?;
        writeln!(formatter, "    Identifier\t\t\t\t\t: {}", self.identifier)?;

        if self.parent_identifier.is_some() || self.parent_name.is_some() {
            writeln!(formatter, "    Parent information:")?;

            if let Some(parent_identifier) = &self.parent_identifier {
                writeln!(
                    formatter,
                    "        Identifier\t\t\t\t: {}",
                    parent_identifier
                )?;
            }
            if let Some(parent_name) = &self.parent_name {
                writeln!(formatter, "        Name\t\t\t\t\t: {}", parent_name)?;
            }
        }
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {} bytes",
            self.bytes_per_sector
        )?;
        writeln!(formatter)
    }
}

/// Information about a Virtual Hard Disk (VHDX) file.
pub struct VhdxInfo {}

impl VhdxInfo {
    /// Retrieves the file information.
    fn get_file_information(vhdx_file: &VhdxFile) -> VhdxFileInfo {
        let mut file_information: VhdxFileInfo = VhdxFileInfo::new();

        file_information.format_version = vhdx_file.format_version;
        file_information.disk_type = vhdx_file.disk_type.clone();
        file_information.identifier = vhdx_file.identifier.clone();
        file_information.parent_identifier = vhdx_file.parent_identifier.clone();
        file_information.parent_name = vhdx_file.parent_name.clone();
        file_information.media_size = vhdx_file.media_size;
        file_information.bytes_per_sector = vhdx_file.bytes_per_sector;

        file_information
    }

    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<VhdxFile, ErrorTrace> {
        let mut vhdx_file: VhdxFile = VhdxFile::new();

        match vhdx_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHDX file");
                return Err(error);
            }
        }
        Ok(vhdx_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let vhdx_file: VhdxFile = match Self::open_file(data_stream) {
            Ok(vhdx_file) => vhdx_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: VhdxFileInfo = Self::get_file_information(&vhdx_file);

        print!("{}", file_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    #[test]
    fn test_file_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vhdx/ext2.vhdx");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let vhdx_file: VhdxFile = VhdxInfo::open_file(&data_stream)?;
        let test_struct: VhdxFileInfo = VhdxInfo::get_file_information(&vhdx_file);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Virtual Hard Disk (VHDX) information:\n",
            "    Format version\t\t\t\t: 2.1\n",
            "    Disk type\t\t\t\t\t: Fixed\n",
            "    Identifier\t\t\t\t\t: ee10a932-6284-f448-aaab-ab839f90ddef\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512 bytes\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vhdx/ext2.vhdx");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let vhdx_file: VhdxFile = VhdxInfo::open_file(&data_stream)?;
        let test_struct: VhdxFileInfo = VhdxInfo::get_file_information(&vhdx_file);

        assert_eq!(test_struct.format_version, 1);
        assert_eq!(test_struct.disk_type, VhdxDiskType::Fixed);
        assert_eq!(
            test_struct.identifier.to_string(),
            "ee10a932-6284-f448-aaab-ab839f90ddef"
        );
        assert_eq!(test_struct.parent_identifier, None);
        assert_eq!(test_struct.parent_name, None);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.bytes_per_sector, 512);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
