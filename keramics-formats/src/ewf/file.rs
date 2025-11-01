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
    ) -> Result<u64, ErrorTrace> {
        let data_stream: &DataStreamReference = match self.data_stream.as_ref() {
            Some(data_stream) => data_stream,
            None => {
                return Err(keramics_core::error_trace_new!("Missing data stream"));
            }
        };
        let offset: u64 =
            keramics_core::data_stream_read_exact_at_position!(data_stream, data, position);
        Ok(offset)
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        match self.read_sections(data_stream) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read sections");
                return Err(error);
            }
        }
        self.data_stream = Some(data_stream.clone());

        Ok(())
    }

    /// Reads the file header and section headers.
    fn read_sections(&mut self, data_stream: &DataStreamReference) -> Result<(), ErrorTrace> {
        let file_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        let mut file_header: EwfFileHeader = EwfFileHeader::new();

        match file_header.read_at_position(data_stream, SeekFrom::Start(0)) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to read file header");
                return Err(error);
            }
        }
        let mut file_offset: u64 = 13;

        while file_offset < file_size {
            let mut section_header: EwfSectionHeader = EwfSectionHeader::new();

            match section_header.read_at_position(data_stream, SeekFrom::Start(file_offset)) {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read section header at offset: {} (0x{:08x})",
                            file_offset, file_offset
                        )
                    );
                    return Err(error);
                }
            }

            let mut is_last_section: bool = false;
            let mut section_size: u64 = section_header.size;

            if section_header.section_type == EWF_SECTION_TYPE_DONE
                || section_header.section_type == EWF_SECTION_TYPE_NEXT
            {
                if section_size == 0 {
                    section_size = 76;
                } else if section_size != 76 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported done or next section size"
                    ));
                }
                if section_header.next_offset != file_offset {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported done or next section header next offset does not align with file offset"
                    ));
                }
                is_last_section = true;
            } else {
                if section_header.next_offset <= file_offset {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported section next offset"
                    ));
                }
                let calculated_section_size: u64 = section_header.next_offset - file_offset;

                if section_size == 0 {
                    section_size = calculated_section_size;
                } else if section_size != calculated_section_size {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported section size value does not align with next offset"
                    ));
                }
                if section_size < 76 {
                    return Err(keramics_core::error_trace_new!(
                        "Unsupported section size value too small"
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
            return Err(keramics_core::error_trace_new!(
                "Unsupported trailing data after last section header"
            ));
        }
        self.segment_number = file_header.segment_number;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use keramics_core::open_os_data_stream;

    use crate::tests::get_test_data_path;

    // TODO: add tests for read_exact_at_position

    #[test]
    fn test_read_data_stream() -> Result<(), ErrorTrace> {
        let mut file = EwfFile::new();

        let path_buf: PathBuf = PathBuf::from(get_test_data_path("ewf/ext2.E01").as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        file.read_data_stream(&data_stream)?;

        assert_eq!(file.segment_number, 1);
        assert_eq!(file.sections.len(), 10);

        Ok(())
    }

    // TODO: add tests for read_sections
}
