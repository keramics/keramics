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

use keramics_core::{DataStreamReference, ErrorTrace};

use crate::util::calculate_alignment_padding;

use super::enums::CpioFormat;
use super::file_record::CpioFileRecord;
use super::file_record_bin_be::CpioBinaryFileRecordBigEndian;
use super::file_record_bin_le::CpioBinaryFileRecordLittleEndian;
use super::file_record_newc::CpioNewAsciiFileRecord;
use super::file_record_odc::CpioPortableAsciiFileRecord;

/// Copy in and out (CPIO) archive.
pub struct CpioArchive {
    /// Data stream.
    data_stream: Option<DataStreamReference>,

    /// Format.
    format: CpioFormat,

    /// File records.
    file_records: Vec<CpioFileRecord>,
}

impl CpioArchive {
    /// Creates an archive.
    pub fn new() -> Self {
        Self {
            data_stream: None,
            format: CpioFormat::NotSet,
            file_records: Vec::new(),
        }
    }

    /// Reads a data stream.
    pub fn read_data_stream(
        &mut self,
        data_stream: &DataStreamReference,
    ) -> Result<(), ErrorTrace> {
        let data_stream_size: u64 = keramics_core::data_stream_get_size!(data_stream);

        let mut data: [u8; 6] = [0; 6];

        keramics_core::data_stream_read_exact_at_position!(
            data_stream,
            &mut data,
            SeekFrom::Start(0)
        );
        self.format = if &data[0..6] == b"070701" || &data[0..6] == b"070702" {
            CpioFormat::NewAscii
        } else if &data[0..6] == b"070707" {
            CpioFormat::PortableAscii
        } else if &data[0..2] == &[0x71, 0xc7] {
            CpioFormat::BinaryBigEndian
        } else if &data[0..2] == &[0xc7, 0x71] {
            CpioFormat::BinaryLittleEndian
        } else {
            CpioFormat::NotSet
        };
        let file_record_size: usize = match &self.format {
            CpioFormat::BinaryBigEndian | CpioFormat::BinaryLittleEndian => 26,
            CpioFormat::NewAscii => 110,
            CpioFormat::PortableAscii => 76,
            _ => return Err(keramics_core::error_trace_new!("Unsupported format")),
        };
        let mut file_record_data: Vec<u8> = vec![0; file_record_size];
        let mut data_offset: u64 = 0;

        while data_offset < data_stream_size {
            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut file_record_data,
                SeekFrom::Start(data_offset)
            );
            let mut file_record: CpioFileRecord = CpioFileRecord::new();

            let result: Result<(), ErrorTrace> = match &self.format {
                CpioFormat::BinaryBigEndian => {
                    keramics_core::debug_trace_data_and_structure!(
                        "CpioBinaryFileRecordBigEndian",
                        data_offset,
                        &file_record_data,
                        file_record_size,
                        CpioBinaryFileRecordBigEndian::debug_read_data(&file_record_data),
                    );
                    CpioBinaryFileRecordBigEndian::read_data(&mut file_record, &file_record_data)
                }
                CpioFormat::BinaryLittleEndian => {
                    keramics_core::debug_trace_data_and_structure!(
                        "CpioBinaryFileRecordLittleEndian",
                        data_offset,
                        &file_record_data,
                        file_record_size,
                        CpioBinaryFileRecordLittleEndian::debug_read_data(&file_record_data),
                    );
                    CpioBinaryFileRecordLittleEndian::read_data(&mut file_record, &file_record_data)
                }
                CpioFormat::NewAscii => {
                    keramics_core::debug_trace_data!(
                        "CpioNewAsciiFileRecord",
                        data_offset,
                        &file_record_data,
                        file_record_size,
                    );
                    CpioNewAsciiFileRecord::read_data(&mut file_record, &file_record_data)
                }
                CpioFormat::PortableAscii => {
                    keramics_core::debug_trace_data!(
                        "CpioPortableAsciiFileRecord",
                        data_offset,
                        &file_record_data,
                        file_record_size,
                    );
                    CpioPortableAsciiFileRecord::read_data(&mut file_record, &file_record_data)
                }
                _ => Err(keramics_core::error_trace_new!("Unsupported format")),
            };
            match result {
                Ok(_) => {}
                Err(mut error) => {
                    keramics_core::error_trace_add_frame!(
                        error,
                        format!(
                            "Unable to read file record at offset: {} (0x{:08x})",
                            data_offset, data_offset
                        )
                    );
                    return Err(error);
                }
            }
            data_offset += file_record_size as u64;

            if file_record.path_size > 32768 {
                return Err(keramics_core::error_trace_new!(
                    "Invalid file record path size value out of bounds"
                ));
            }
            let mut path_data: Vec<u8> = vec![0; file_record.path_size as usize];

            keramics_core::data_stream_read_exact_at_position!(
                data_stream,
                &mut path_data,
                SeekFrom::Start(data_offset)
            );
            keramics_core::debug_trace_data!(
                "CpioFileRecordPath",
                data_offset,
                &path_data,
                file_record.path_size,
            );
            // TODO: parse path to reconstruct hierarchy.

            data_offset += file_record.path_size as u64;

            let alignment_padding: usize = match &self.format {
                CpioFormat::BinaryBigEndian | CpioFormat::BinaryLittleEndian => {
                    calculate_alignment_padding(data_offset as usize, 2)
                }
                CpioFormat::NewAscii => calculate_alignment_padding(data_offset as usize, 4),
                _ => 0,
            };
            if alignment_padding > 0 {
                // TODO: debug print alignment padding.

                data_offset += alignment_padding as u64;
            }
            file_record.data_offset = data_offset;

            data_offset += file_record.data_size as u64;

            let alignment_padding: usize = match &self.format {
                CpioFormat::BinaryBigEndian | CpioFormat::BinaryLittleEndian => {
                    calculate_alignment_padding(data_offset as usize, 2)
                }
                CpioFormat::NewAscii => calculate_alignment_padding(data_offset as usize, 4),
                _ => 0,
            };
            if alignment_padding > 0 {
                // TODO: debug print alignment padding.

                data_offset += alignment_padding as u64;
            }
            if path_data == b"TRAILER!!!\0" {
                break;
            }
            self.file_records.push(file_record);
        }
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

    #[test]
    fn test_read_data_stream_with_bin_le() -> Result<(), ErrorTrace> {
        keramics_core::mediator::Mediator { debug_output: true }.make_current();

        let mut archive: CpioArchive = CpioArchive::new();

        let path_string: String = get_test_data_path("cpio/bin_le.cpio");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        archive.read_data_stream(&data_stream)?;

        assert_eq!(archive.format, CpioFormat::BinaryLittleEndian);
        assert_eq!(archive.file_records.len(), 21);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_newc() -> Result<(), ErrorTrace> {
        let mut archive: CpioArchive = CpioArchive::new();

        let path_string: String = get_test_data_path("cpio/newc.cpio");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        archive.read_data_stream(&data_stream)?;

        assert_eq!(archive.format, CpioFormat::NewAscii);
        assert_eq!(archive.file_records.len(), 21);

        Ok(())
    }

    #[test]
    fn test_read_data_stream_with_odc() -> Result<(), ErrorTrace> {
        let mut archive: CpioArchive = CpioArchive::new();

        let path_string: String = get_test_data_path("cpio/odc.cpio");
        let path_buf: PathBuf = PathBuf::from(path_string.as_str());
        let data_stream: DataStreamReference = open_os_data_stream(&path_buf)?;
        archive.read_data_stream(&data_stream)?;

        assert_eq!(archive.format, CpioFormat::PortableAscii);
        assert_eq!(archive.file_records.len(), 21);

        Ok(())
    }
}
