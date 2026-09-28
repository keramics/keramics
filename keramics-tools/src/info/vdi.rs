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

use std::fmt;

use keramics_core::{DataStreamReference, ErrorTrace};
use keramics_formats::vdi::VdiFile;

use crate::formatters::ByteSize;

/// Information about a Virtual Disk Image (VDI) file.
struct VdiFileInfo<'a> {
    /// File.
    file: &'a VdiFile,
}

impl<'a> VdiFileInfo<'a> {
    /// Creates new file information.
    fn new(file: &'a VdiFile) -> Self {
        Self { file }
    }
}

impl<'a> fmt::Display for VdiFileInfo<'a> {
    /// Formats file information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Virtual Disk Image (VDI) information:")?;

        let (major_format_version, minor_format_version): (u16, u16) =
            self.file.get_format_version();
        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}.{}",
            major_format_version, minor_format_version
        )?;

        writeln!(
            formatter,
            "    Identifier\t\t\t\t\t: {}",
            self.file.get_identifier()
        )?;
        if let Some(parent_identifier) = self.file.get_parent_identifier() {
            writeln!(formatter, "    Parent information:")?;

            writeln!(
                formatter,
                "        Identifier\t\t\t\t: {}",
                parent_identifier
            )?;
        }
        writeln!(formatter, "    Media information:")?;

        let byte_size: ByteSize = ByteSize::new(self.file.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.file.get_bytes_per_sector()
        )?;
        writeln!(formatter)
    }
}

/// Information about Virtual Disk Image (VDI) format.
pub struct VdiInfo {}

impl VdiInfo {
    /// Opens a file.
    pub(super) fn open_file(data_stream: &DataStreamReference) -> Result<VdiFile, ErrorTrace> {
        let mut vdi_file: VdiFile = VdiFile::new();

        match vdi_file.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open VDI file");
                return Err(error);
            }
        }
        Ok(vdi_file)
    }

    /// Prints information about a file.
    pub fn print_file(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let vdi_file: VdiFile = match Self::open_file(data_stream) {
            Ok(vdi_file) => vdi_file,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open file");
                return Err(error);
            }
        };
        let file_information: VdiFileInfo = VdiFileInfo::new(&vdi_file);

        print!("{}", file_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::assert_lines_eq;

    fn get_file() -> Result<VdiFile, ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/vdi/ext2.vdi");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        VdiInfo::open_file(&data_stream)
    }

    #[test]
    fn test_file_information_fmt() -> Result<(), ErrorTrace> {
        let vdi_file: VdiFile = get_file()?;

        let test_struct: VdiFileInfo = VdiFileInfo::new(&vdi_file);

        let expected_string: &str = concat!(
            "Virtual Disk Image (VDI) information:\n",
            "    Format version\t\t\t\t: 1.1\n",
            "    Identifier\t\t\t\t\t: 88437ae8-9631-4c48-81c3-e787cc586501\n",
            "    Media information:\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Bytes per sector\t\t\t: 512\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_file
    // TODO: add tests for print_file
}
