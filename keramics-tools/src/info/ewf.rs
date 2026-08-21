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
use std::path::PathBuf;

use keramics_core::ErrorTrace;
use keramics_core::formatters::format_as_string;
use keramics_formats::ewf::{EwfHeaderValueType, EwfImage, EwfMediaType};
use keramics_formats::{FileResolverReference, PathComponent, open_os_file_resolver};
use keramics_types::Uuid;

use crate::formatters::ByteSize;

/// Information about an Expert Witness Compression Format (EWF) image.
struct EwfImageInfo<'a> {
    /// Image.
    image: &'a EwfImage,
}

impl<'a> EwfImageInfo<'a> {
    const MEDIA_TYPES: &'static [(EwfMediaType, &'static str); 4] = &[
        (EwfMediaType::FixedDisk, "fixed disk"),
        (EwfMediaType::LogicalEvidence, "logical evidence"),
        (EwfMediaType::OpticalDisk, "optical disk (CD/DVD/BD)"),
        (EwfMediaType::RemoveableDisk, "removable disk"),
    ];

    /// Create new image information.
    pub fn new(image: &'a EwfImage) -> Self {
        Self { image }
    }

    /// Retrieves the media type as a string.
    pub fn get_media_type_string(&self, media_type: &EwfMediaType) -> &str {
        Self::MEDIA_TYPES
            .binary_search_by(|(key, _)| key.cmp(media_type))
            .map_or_else(|_| "Unknown", |index| Self::MEDIA_TYPES[index].1)
    }
}

impl<'a> fmt::Display for EwfImageInfo<'a> {
    /// Formats image information for display.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            formatter,
            "Expert Witness Compression Format (EWF) information:"
        )?;

        // TODO: print file format

        let segment_set_identifier: &Uuid = self.image.get_segment_set_identifier();
        if !segment_set_identifier.is_nil() {
            writeln!(
                formatter,
                "    Segment set identifier\t\t\t: {}",
                segment_set_identifier
            )?;
        }
        writeln!(
            formatter,
            "    Sectors per chunk\t\t\t\t: {}",
            self.image.get_sectors_per_chunk(),
        )?;
        let error_granularity: u32 = self.image.get_error_granularity();

        if error_granularity == 1 {
            writeln!(
                formatter,
                "    Error granularity\t\t\t\t: {} sector",
                error_granularity
            )?;
        } else {
            writeln!(
                formatter,
                "    Error granularity\t\t\t\t: {} sectors",
                error_granularity
            )?;
        }
        // TODO: print compression method

        writeln!(formatter)?;

        writeln!(formatter, "    Media information:")?;

        // TODO: print media type (combine with is physical)
        let media_type: &EwfMediaType = self.image.get_media_type();
        let media_type_string: &str = self.get_media_type_string(media_type);
        writeln!(
            formatter,
            "        Media type\t\t\t\t: {}",
            media_type_string,
        )?;
        let byte_size: ByteSize = ByteSize::new(self.image.get_media_size(), 1024);
        writeln!(formatter, "        Media size\t\t\t\t: {}", byte_size)?;

        if *media_type != EwfMediaType::LogicalEvidence {
            writeln!(
                formatter,
                "        Number of sectors\t\t\t: {}",
                self.image.get_number_of_sectors()
            )?;
            writeln!(
                formatter,
                "        Bytes per sector\t\t\t: {}",
                self.image.get_bytes_per_sector()
            )?;
        }
        let md5_hash: &[u8] = self.image.get_md5_hash();

        if md5_hash != &[0; 16] {
            let hash_string: String = format_as_string(md5_hash);
            writeln!(formatter, "        MD5\t\t\t\t\t: {}", hash_string)?;
        }
        let sha1_hash: &[u8] = self.image.get_sha1_hash();

        if sha1_hash != &[0; 20] {
            let hash_string: String = format_as_string(sha1_hash);
            writeln!(formatter, "        SHA1\t\t\t\t\t: {}", hash_string)?;
        }
        writeln!(formatter)?;

        writeln!(formatter, "    Case information:")?;

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
            if let Some(header_value) = self.image.get_header_value(&header_value_type) {
                let header_value_string: String = header_value.to_string();

                if header_value_string.is_empty() {
                    continue;
                }
                if header_value_type == EwfHeaderValueType::PasswordHash
                    && header_value_string == "0"
                {
                    continue;
                } else if (header_value_type == EwfHeaderValueType::AcquisitionDate
                    || header_value_type == EwfHeaderValueType::SystemDate)
                    && header_value_string == "1970-01-01T00:00:00"
                {
                    continue;
                }
                writeln!(
                    formatter,
                    "        {}{}: {}",
                    description,
                    "\t".repeat((40 - description.len()).div_ceil(8)),
                    header_value_string,
                )?;
            }
        }
        // TODO: print optical disk session information
        // TODO: print error information

        writeln!(formatter)
    }
}

/// Information about an Expert Witness Compression Format (EWF) image.
pub struct EwfInfo {}

impl EwfInfo {
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
        let mut ewf_image: EwfImage = EwfImage::new();

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
        let image_information: EwfImageInfo = EwfImageInfo::new(&ewf_image);

        print!("{}", image_information);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::assert_lines_eq;

    #[test]
    fn test_image_information_fmt() -> Result<(), ErrorTrace> {
        let path_buf: PathBuf = PathBuf::from("../test_data/ewf/ext2.E01");
        let ewf_image: EwfImage = EwfInfo::open_image(&path_buf)?;

        let test_struct: EwfImageInfo = EwfImageInfo::new(&ewf_image);

        let expected_string: &str = concat!(
            "Expert Witness Compression Format (EWF) information:\n",
            "    Sectors per chunk\t\t\t\t: 64\n",
            "    Error granularity\t\t\t\t: 64 sectors\n",
            "\n",
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
        let string: String = test_struct.to_string();
        assert_lines_eq!(string.as_str(), expected_string);

        Ok(())
    }

    // TODO: add tests for open_image
    // TODO: add tests for print_image
}
