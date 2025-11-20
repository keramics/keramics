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
use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::vhd::{VhdDiskType, VhdFile};
use keramics_types::{Ucs2String, Uuid};

use crate::formatters::ByteSize;

/// Information about a Virtual Hard Disk (VHD) file.
struct VhdFileInfo {
    /// Disk type.
    pub disk_type: VhdDiskType,

    /// Media size.
    pub media_size: u64,

    /// Bytes per sector.
    pub bytes_per_sector: u16,

    /// Identifier.
    pub identifier: Uuid,

    /// Parent identifier.
    pub parent_identifier: Option<Uuid>,

    /// Parent name.
    pub parent_name: Option<Ucs2String>,
}

impl VhdFileInfo {
    /// Creates new file information.
    fn new() -> Self {
        Self {
            disk_type: VhdDiskType::Unknown,
            media_size: 0,
            bytes_per_sector: 0,
            identifier: Uuid::new(),
            parent_identifier: None,
            parent_name: None,
        }
    }
}

impl fmt::Display for VhdFileInfo {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Virtual Hard Disk (VHD) information:\n")?;

        write!(formatter, "    Format version\t\t\t\t: 1.0\n")?;

        let disk_types = HashMap::<VhdDiskType, &'static str>::from([
            (VhdDiskType::Differential, "Differential"),
            (VhdDiskType::Dynamic, "Dynamic"),
            (VhdDiskType::Fixed, "Fixed"),
            (VhdDiskType::Unknown, "Unknown"),
        ]);
        let disk_type_string: &str = disk_types.get(&self.disk_type).unwrap();

        write!(formatter, "    Disk type\t\t\t\t\t: {}\n", disk_type_string)?;

        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);

        write!(formatter, "    Media size\t\t\t\t\t: {}\n", byte_size)?;

        write!(
            formatter,
            "    Bytes per sector\t\t\t\t: {} bytes\n",
            self.bytes_per_sector
        )?;
        write!(formatter, "    Identifier\t\t\t\t\t: {}\n", self.identifier)?;

        if let Some(parent_identifier) = &self.parent_identifier {
            write!(
                formatter,
                "    Parent identifier\t\t\t\t: {}\n",
                parent_identifier
            )?;
        }
        if let Some(parent_name) = &self.parent_name {
            write!(formatter, "    Parent name\t\t\t\t\t: {}\n", parent_name)?;
        }
        write!(formatter, "\n")
    }
}

/// Information about a Virtual Hard Disk (VHD) file.
pub struct VhdInfo {}

impl VhdInfo {
    /// Retrieves the file information.
    fn get_file_information(vhd_file: &VhdFile) -> VhdFileInfo {
        let mut file_information: VhdFileInfo = VhdFileInfo::new();

        file_information.disk_type = vhd_file.disk_type.clone();
        file_information.media_size = vhd_file.media_size;
        file_information.bytes_per_sector = vhd_file.bytes_per_sector;
        file_information.identifier = vhd_file.identifier.clone();
        file_information.parent_identifier = vhd_file.parent_identifier.clone();
        file_information.parent_name = vhd_file.parent_name.clone();

        file_information
    }

    /// Opens a file.
    fn open_file(data_stream: &DataStreamReference) -> Result<VhdFile, ErrorTrace> {
        let mut vhd_file: VhdFile = VhdFile::new();

        match vhd_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VHD file");
                return Err(error);
            }
        };
        Ok(vhd_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let vhd_file: VhdFile = match Self::open_file(data_stream) {
            Ok(vhd_file) => vhd_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: VhdFileInfo = Self::get_file_information(&vhd_file);

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
        let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ext2.vhd");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let vhd_file: VhdFile = VhdInfo::open_file(&data_stream)?;
        let test_struct: VhdFileInfo = VhdInfo::get_file_information(&vhd_file);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Virtual Hard Disk (VHD) information:\n",
            "    Format version\t\t\t\t: 1.0\n",
            "    Disk type\t\t\t\t\t: Dynamic\n",
            "    Media size\t\t\t\t\t: 4.0 MiB (4212736 bytes)\n",
            "    Bytes per sector\t\t\t\t: 512 bytes\n",
            "    Identifier\t\t\t\t\t: 4f75d18f-d5ef-438e-b326-d60da6c9ed67\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_file_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vhd/ext2.vhd");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let vhd_file: VhdFile = VhdInfo::open_file(&data_stream)?;
        let test_struct: VhdFileInfo = VhdInfo::get_file_information(&vhd_file);

        assert_eq!(test_struct.disk_type, VhdDiskType::Dynamic);
        assert_eq!(test_struct.media_size, 4212736);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(
            test_struct.identifier.to_string(),
            "4f75d18f-d5ef-438e-b326-d60da6c9ed67"
        );
        assert_eq!(test_struct.parent_identifier, None);
        assert_eq!(test_struct.parent_name, None);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
