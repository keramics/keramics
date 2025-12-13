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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_core::formatters::format_as_string;
use keramics_formats::ewf::{EwfHeaderValueType, EwfImage, EwfMediaType};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

/// Information about an Expert Witness Compression Format (EWF) image.
struct EwfImageInfo {
    /// Segment file set identifier.
    pub set_identifier: Uuid,

    /// Sectors per chunk.
    pub sectors_per_chunk: u32,

    /// Error granularity.
    pub error_granularity: u32,

    /// Media type.
    pub media_type: EwfMediaType,

    /// Media size.
    pub media_size: u64,

    /// Number of sectors.
    pub number_of_sectors: u32,

    /// Bytes per sector.
    pub bytes_per_sector: u32,

    /// MD5 hash.
    pub md5_hash: [u8; 16],

    /// SHA1 hash.
    pub sha1_hash: [u8; 20],

    /// Header values.
    pub header_values: Vec<(&'static str, String)>,
}

impl EwfImageInfo {
    const MEDIA_TYPES: &[(EwfMediaType, &'static str); 4] = &[
        (EwfMediaType::FixedDisk, "fixed disk"),
        (EwfMediaType::LogicalEvidence, "logical evidence"),
        (EwfMediaType::OpticalDisk, "optical disk (CD/DVD/BD)"),
        (EwfMediaType::RemoveableDisk, "removable disk"),
    ];

    /// Create new image information.
    pub fn new() -> Self {
        Self {
            set_identifier: Uuid::new(),
            sectors_per_chunk: 0,
            error_granularity: 0,
            media_type: EwfMediaType::Unknown,
            media_size: 0,
            number_of_sectors: 0,
            bytes_per_sector: 0,
            md5_hash: [0; 16],
            sha1_hash: [0; 20],
            header_values: Vec::new(),
        }
    }

    /// Retrieves the media type as a string.
    pub fn get_media_type_string(&self) -> &str {
        Self::MEDIA_TYPES
            .binary_search_by(|(key, _)| key.cmp(&self.media_type))
            .map_or_else(|_| "Unknown", |index| Self::MEDIA_TYPES[index].1)
    }
}

impl fmt::Display for EwfImageInfo {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Expert Witness Compression Format (EWF) information:"
        )?;

        // TODO: print file format
        if !self.set_identifier.is_nil() {
            writeln!(
                formatter,
                "    Set identifier\t\t\t\t: {}",
                self.set_identifier
            )?;
        }
        writeln!(
            formatter,
            "    Sectors per chunk\t\t\t\t: {}",
            self.sectors_per_chunk
        )?;
        writeln!(
            formatter,
            "    Error granularity\t\t\t\t: {} sectors",
            self.error_granularity
        )?;
        // TODO: print compression method

        writeln!(formatter, "    Media information:")?;

        // TODO: print media type (combine with is physical)
        let media_type_string: &str = self.get_media_type_string();
        writeln!(
            formatter,
            "        Media type\t\t\t\t: {}",
            media_type_string,
        )?;
        let byte_size: ByteSize = ByteSize::new(self.media_size, 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        writeln!(
            formatter,
            "        Number of sectors\t\t\t: {}",
            self.number_of_sectors
        )?;
        writeln!(
            formatter,
            "        Bytes per sector\t\t\t: {}",
            self.bytes_per_sector
        )?;
        if self.md5_hash != [0; 16] {
            let hash_string: String = format_as_string(&self.md5_hash);
            writeln!(formatter, "        MD5\t\t\t\t\t: {}", hash_string)?;
        }
        if self.sha1_hash != [0; 20] {
            let hash_string: String = format_as_string(&self.sha1_hash);
            writeln!(formatter, "        SHA1\t\t\t\t\t: {}", hash_string)?;
        }
        writeln!(formatter)?;

        writeln!(formatter, "    Case information:")?;

        for (description, value) in &self.header_values {
            writeln!(
                formatter,
                "        {}{}: {}",
                description,
                "\t".repeat((40 - description.len()).div_ceil(8)),
                value,
            )?;
        }
        // TODO: print case information
        //
        // TODO: print optical disk session information
        // TODO: print error information

        writeln!(formatter)
    }
}

/// Information about an Expert Witness Compression Format (EWF) image.
pub struct EwfInfo {}

impl EwfInfo {
    /// Retrieves the image information.
    fn get_image_information(ewf_image: &EwfImage) -> EwfImageInfo {
        let mut image_information: EwfImageInfo = EwfImageInfo::new();

        image_information.set_identifier = ewf_image.set_identifier.clone();
        image_information.sectors_per_chunk = ewf_image.sectors_per_chunk;
        image_information.error_granularity = ewf_image.error_granularity;
        image_information.media_type = ewf_image.media_type.clone();
        image_information.media_size = ewf_image.media_size;
        image_information.number_of_sectors = ewf_image.number_of_sectors;
        image_information.bytes_per_sector = ewf_image.bytes_per_sector;
        image_information
            .md5_hash
            .copy_from_slice(&ewf_image.md5_hash);
        image_information
            .sha1_hash
            .copy_from_slice(&ewf_image.sha1_hash);

        let header_values: [(EwfHeaderValueType, &str); 15] = [
            (EwfHeaderValueType::CaseNumber, "Case number"),
            (EwfHeaderValueType::Description, "Description"),
            (EwfHeaderValueType::ExaminerName, "Examiner name"),
            (EwfHeaderValueType::EvidenceNumber, "Evidence number"),
            (EwfHeaderValueType::Notes, "Notes"),
            (EwfHeaderValueType::AcquisitionDate, "Acquisition date"),
            (EwfHeaderValueType::SystemDate, "System date"),
            (EwfHeaderValueType::Platform, "Operating system used"),
            (EwfHeaderValueType::Version, "Software version used"),
            (EwfHeaderValueType::PasswordHash, "Password"),
            (EwfHeaderValueType::CompressionLevel, "Compression level"),
            (EwfHeaderValueType::Model, "Model"),
            (EwfHeaderValueType::SerialNumber, "Serial number"),
            (EwfHeaderValueType::DeviceLabel, "Device label"),
            (EwfHeaderValueType::ProcessIdentifier, "Process identifier"),
        ];
        for (header_value_type, description) in header_values {
            if let Some(header_value) = ewf_image.get_header_value(&header_value_type) {
                let header_value_string: String = header_value.to_string();

                if header_value_string.is_empty() {
                    continue;
                }
                if header_value_type == EwfHeaderValueType::PasswordHash
                    && header_value_string == "0"
                {
                    continue;
                }
                image_information
                    .header_values
                    .push((description, header_value_string));
            }
        }
        image_information
    }

    /// Opens an image.
    fn open_image(path_buf: &PathBuf) -> Result<EwfImage, ErrorTrace> {
        let mut base_path: PathBuf = path_buf.clone();
        base_path.pop();

        let file_resolver: FileResolverReference = match open_os_file_resolver(&base_path) {
            Ok(file_resolver) => file_resolver,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to create file resolver");
                return Err(error);
            }
        };
        let mut ewf_image: EwfImage = EwfImage::new();

        let file_name: PathComponent = match path_buf.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => PathComponent::from(file_name),
                None => {
                    return Err(keramics_core::error_trace_new!("Unsupported file name"));
                }
            },
            None => {
                return Err(keramics_core::error_trace_new!("Missing file name"));
            }
        };
        match ewf_image.open(&file_resolver, &file_name) {
            Ok(_) => {}
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open EWF image");
                return Err(error);
            }
        }
        Ok(ewf_image)
    }

    /// Prints information about an image.
    pub fn print_image(path_buf: &PathBuf) -> Result<(), ErrorTrace> {
        let ewf_image: EwfImage = match Self::open_image(path_buf) {
            Ok(ewf_image) => ewf_image,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to open image");
                return Err(error);
            }
        };
        let image_information: EwfImageInfo = Self::get_image_information(&ewf_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/ewf/ext2.E01");
        let ewf_image: EwfImage = EwfInfo::open_image(&path_buf)?;
        let test_struct: EwfImageInfo = EwfInfo::get_image_information(&ewf_image);

        let string: String = test_struct.to_string();
        let expected_string: &str = concat!(
            "Expert Witness Compression Format (EWF) information:\n",
            "    Sectors per chunk\t\t\t\t: 64\n",
            "    Error granularity\t\t\t\t: 64 sectors\n",
            "    Media information:\n",
            "        Media type\t\t\t\t: fixed disk\n",
            "        Media size\t\t\t\t: 4.0 MiB (4194304 bytes)\n",
            "        Number of sectors\t\t\t: 8192\n",
            "        Bytes per sector\t\t\t: 512\n",
            "        MD5\t\t\t\t\t: b1760d0b35a512ef56970df4e6f8c5d6\n",
            "\n",
            "    Case information:\n",
            "        Case number\t\t\t\t: case\n",
            "        Description\t\t\t\t: description\n",
            "        Examiner name\t\t\t\t: examiner\n",
            "        Evidence number\t\t\t\t: evidence\n",
            "        Notes\t\t\t\t\t: notes\n",
            "        Acquisition date\t\t\t: 2025-09-17T17:46:01\n",
            "        System date\t\t\t\t: 2025-09-17T17:46:01\n",
            "        Operating system used\t\t\t: Linux\n",
            "        Software version used\t\t\t: 20140817\n",
            "\n"
        );
        assert_eq!(string, expected_string);

        Ok(())
    }

    #[test]
    fn test_get_image_information() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/ewf/ext2.E01");
        let ewf_image: EwfImage = EwfInfo::open_image(&path_buf)?;
        let test_struct: EwfImageInfo = EwfInfo::get_image_information(&ewf_image);

        assert_eq!(test_struct.sectors_per_chunk, 64);
        assert_eq!(test_struct.error_granularity, 64);
        assert_eq!(test_struct.media_type, EwfMediaType::FixedDisk);
        assert_eq!(test_struct.media_size, 4194304);
        assert_eq!(test_struct.number_of_sectors, 8192);
        assert_eq!(test_struct.bytes_per_sector, 512);
        assert_eq!(
            test_struct.md5_hash,
            [
                0xb1, 0x76, 0x0d, 0x0b, 0x35, 0xa5, 0x12, 0xef, 0x56, 0x97, 0x0d, 0xf4, 0xe6, 0xf8,
                0xc5, 0xd6,
            ]
        );
        assert_eq!(test_struct.sha1_hash, [0; 20]);
        assert_eq!(
            test_struct.header_values,
            vec![
                ("Case number", String::from("case")),
                ("Description", String::from("description")),
                ("Examiner name", String::from("examiner")),
                ("Evidence number", String::from("evidence")),
                ("Notes", String::from("notes")),
                ("Acquisition date", String::from("2025-09-17T17:46:01")),
                ("System date", String::from("2025-09-17T17:46:01")),
                ("Operating system used", String::from("Linux")),
                ("Software version used", String::from("20140817")),
            ]
        );
        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
