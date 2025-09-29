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
use std::io::SeekFrom;

use keramics_core::DataStreamReference;

use super::constants::*;
use super::file_header::EwfFileHeader;
use super::section_header::EwfSectionHeader;

/// Expert Witness Compression Format (EWF) file.
pub struct EwfFile {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Segment number.
    pub segment_number: u16,

    /// Sections.
    pub sections: Vec<EwfSectionHeader>,
}

impl EwfFile {
    /// Creates a new file.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            segment_number: 0,
            sections: Vec::new(),
        }
    }

    /// Reads an exact amount of data at a specific position.
    pub fn read_exact_at_position(
        &mut self,
        data: &mut [u8],
        position: SeekFrom,
    ) -> io::Result<u64> {
        match self.data_stream.as_ref() {
            Some(data_stream) => match data_stream.write() {
                Ok(mut data_stream) => data_stream.read_exact_at_position(data, position),
                Err(error) => return Err(keramics_core::error_to_io_error!(error)),
            },
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Missing data stream",
                ));
            }
        }
    }

    /// Reads a data stream.
    pub fn read_data_stream(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        self.read_sections(data_stream)?;

        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file header and section headers.
    fn read_sections(&mut self, data_stream: &DataStreamReference) -> io::Result<()> {
        let file_size: u64 = match data_stream.write() {
            Ok(mut data_stream) => data_stream.get_size()?,
            Err(error) => return Err(keramics_core::error_to_io_error!(error)),
        };
        let mut file_header: EwfFileHeader = EwfFileHeader::new();

        file_header.read_at_position(data_stream, io::SeekFrom::Start(0))?;

        let mut file_offset: u64 = 13;

        while file_offset < file_size {
            let mut section_header: EwfSectionHeader = EwfSectionHeader::new();

            section_header.read_at_position(data_stream, io::SeekFrom::Start(file_offset))?;

            let mut is_last_section: bool = false;
            let mut section_size: u64 = section_header.size;

            if section_header.section_type == EWF_SECTION_TYPE_DONE
                || section_header.section_type == EWF_SECTION_TYPE_NEXT
            {
                if section_size == 0 {
                    section_size = 76;
                } else if section_size != 76 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported done or next section size",
                    ));
                }
                if section_header.next_offset != file_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported done or next section header next offset does not align with file offset",
                    ));
                }
                is_last_section = true;
            } else {
                if section_header.next_offset <= file_offset {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported section next offset",
                    ));
                }
                let calculated_section_size: u64 = section_header.next_offset - file_offset;

                if section_size == 0 {
                    section_size = calculated_section_size;
                } else if section_size != calculated_section_size {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported section size value does not align with next offset",
                    ));
                }
                if section_size < 76 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported section size value too small",
                    ));
                }
            }
            section_header.size = section_size;

            self.sections.push(section_header);

            file_offset += section_size;

            if is_last_section {
                break;
            }
        }
        if file_offset != file_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported trailing data after last section header",
            ));
        }
        self.segment_number = file_header.segment_number;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keramics_core::open_os_data_stream;

    fn get_file() -> io::Result<EwfFile> {
        let mut file: EwfFile = EwfFile::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/ewf/ext2.E01")?;
        file.read_data_stream(&data_stream)?;

        Ok(file)
    }

    #[test]
    fn test_read_data_stream() -> io::Result<()> {
        let mut file = EwfFile::new();

        let data_stream: DataStreamReference = open_os_data_stream("../test_data/ewf/ext2.E01")?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.segment_number, 1);
        assert_eq!(file.sections.len(), 10);

        Ok(())
    }
}
