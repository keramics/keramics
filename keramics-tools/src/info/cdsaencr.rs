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
use keramics_formats::cdsaencr::CdsaEncrContainer;

use crate::formatters::ByteSize;

/// Information about a Mac OS Encrypted Encoding container.
struct CdsaEncrContainerInfo<'a> {
    /// Container.
    container: &'a CdsaEncrContainer,
}

impl<'a> CdsaEncrContainerInfo<'a> {
    /// Creates new container information.
    fn new(container: &'a CdsaEncrContainer) -> Self {
        Self { container }
    }
}

impl<'a> fmt::Display for CdsaEncrContainerInfo<'a> {
    /// Formats container information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Encrypted Encoding container (cdsaencr) information:"
        )?;
        writeln!(
            formatter,
            "    Format version\t\t\t\t: {}",
            self.container.get_format_version()
        )?;
        let byte_size: ByteSize = ByteSize::new(self.container.get_block_size() as u64, 1024);
        writeln!(formatter, "    Block size\t\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "    Container identifier\t\t\t: {}",
            self.container.get_container_identifier()
        )?;
        writeln!(formatter, "    Encryption information:")?;
        writeln!(
            formatter,
            "        Encryption method\t\t\t: {}",
            self.container.get_encryption_type()
        )?;
        // TODO: print key protectors
        // TODO: print identifier

        writeln!(formatter)
    }
}

/// Information about a Mac OS Encrypted Encoding container.
pub struct CdsaEncrInfo {}

impl CdsaEncrInfo {
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
            CdsaEncrContainerInfo::new(&cdsaencr_container);

        print!("{}", container_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::assert_lines_eq;

    #[test]
    fn test_container_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/udif/hfsplus_aes256.dmg");
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        let cdsaencr_container: CdsaEncrContainer = CdsaEncrInfo::open_container(&data_stream)?;

        let test_struct: CdsaEncrContainerInfo = CdsaEncrContainerInfo::new(&cdsaencr_container);

        let expected_string: &str = concat!(
            "Encrypted Encoding container (cdsaencr) information:\n",
            "    Format version\t\t\t\t: 1\n",
            "    Block size\t\t\t\t\t: 512 bytes\n",
            "    Container identifier\t\t\t: 6dde706c-61d2-45ff-9046-c86b3912bfeb\n",
            "    Encryption information:\n",
            "        Encryption method\t\t\t: AES-256-CBC-IV8\n",
            "\n"
        );
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_container
    // TODO: add tests for print_container
}
