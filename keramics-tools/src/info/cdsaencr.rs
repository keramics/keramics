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
use keramics_formats::cdsaencr::{CdsaEncrContainer, CdsaEncrEncryptionType};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

/// Information about a Mac OS Encrypted Encoding container.
struct CdsaEncrContainerInfo {
    /// Container identifier.
    pub container_identifier: Uuid,

    /// Format version.
    pub format_version: u32,

    /// Block size.
    pub block_size: u32,

    /// Encryption type.
    pub encryption_type: CdsaEncrEncryptionType,
}

impl CdsaEncrContainerInfo {
    /// Creates new container information.
    fn new() -> Self {
        Self {
            container_identifier: Uuid::new(),
            format_version: 0,
            block_size: 0,
            encryption_type: CdsaEncrEncryptionType::new(),
        }
    }
}

impl fmt::Display for CdsaEncrContainerInfo {
    /// Formats container information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Encrypted Encoding container (cdsaencr) information:"
        )?;
        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}",
            self.format_version
        )?;
        let byte_size: ByteSize = ByteSize::new(self.block_size as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Container identifier\t\t\t: {}",
            self.container_identifier
        )?;
        writeln!(formatter, "    Encryption information:")?;
        writeln!(
            formatter,
            "        Encryption method\t\t\t: {}",
            self.encryption_type
        )?;
        // TODO: print human readable encryption method
        // TODO: print key protectors
        // TODO: print identifier

        writeln!(formatter)
    }
}

/// Information about a Mac OS Encrypted Encoding container.
pub struct CdsaEncrInfo {}

impl CdsaEncrInfo {
    /// Retrieves the container information.
    fn get_container_information(cdsaencr_container: &CdsaEncrContainer) -> CdsaEncrContainerInfo {
        let mut container_information: CdsaEncrContainerInfo = CdsaEncrContainerInfo::new();

        container_information.format_version = cdsaencr_container.get_format_version();
        container_information.block_size = cdsaencr_container.get_block_size();
        container_information.container_identifier =
            cdsaencr_container.get_container_identifier().clone();
        container_information.encryption_type = cdsaencr_container.get_encryption_type().clone();

        container_information
    }

    /// Opens a container.
    fn open_container(data_stream: &DataStreamReference) -> Result<CdsaEncrContainer, ErrorTrace> {
        let mut cdsaencr_container: CdsaEncrContainer = CdsaEncrContainer::new();

        match cdsaencr_container.read_data_stream(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open encrypted container");
                return Err(error);
            }
        }
        Ok(cdsaencr_container)
    }

    /// Prints information about a container.
    pub fn print_container(data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let cdsaencr_container: CdsaEncrContainer = match Self::open_container(data_stream) {
            Ok(cdsaencr_container) => cdsaencr_container,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open container");
                return Err(error);
            }
        };
        let container_information: CdsaEncrContainerInfo =
            Self::get_container_information(&cdsaencr_container);

        print!("{}", container_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::info::tests::assert_lines_eq;

    #[test]
    fn test_container_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_aes256.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let cdsaencr_container: CdsaEncrContainer = CdsaEncrInfo::open_container(&data_stream)?;
        let test_struct: CdsaEncrContainerInfo =
            CdsaEncrInfo::get_container_information(&cdsaencr_container);

        let expected_string: &str = concat!(
            "Encrypted Encoding container (cdsaencr) information:\n",
            "    Format version\t\t\t\t: 1\n",
            "    Block size\t\t\t\t\t: 512 bytes\n",
            "    Container identifier\t\t\t: 6dde706c-61d2-45ff-9046-c86b3912bfeb\n",
            "    Encryption information:\n",
            "        Encryption method\t\t\t: AES-256-CBC-IV8\n",
            "\n"
        );
        assert_lines_eq(test_struct.to_string().as_str(), expected_string);

        Ok(())
    }

    #[test]
    fn test_get_container_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_aes256.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let cdsaencr_container: CdsaEncrContainer = CdsaEncrInfo::open_container(&data_stream)?;
        let test_struct: CdsaEncrContainerInfo =
            CdsaEncrInfo::get_container_information(&cdsaencr_container);

        assert_eq!(test_struct.format_version, 1);
        assert_eq!(test_struct.block_size, 512);
        assert_eq!(
            test_struct.container_identifier.to_string(),
            "6dde706c-61d2-45ff-9046-c86b3912bfeb"
        );
        assert_eq!(test_struct.encryption_type.method, 0x80000001);
        assert_eq!(test_struct.encryption_type.mode, 5);
        assert_eq!(test_struct.encryption_type.key_size, 32);

        Ok(())
    }

    // TODO: add tests for open_container
    // TODO: add tests for print_container
}
