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

use keramics_core::ErrorTrace;

use super::enums::EwfNamingSchema;

/// Expert Witness Compression Format (EWF) segment file.
pub struct EwfSegmentFile {}

impl EwfSegmentFile {
    /// Determines the extension for a given segment number.
    pub fn get_extension(
        segment_number: u16,
        naming_schema: &EwfNamingSchema,
    ) -> Result<String, ErrorTrace> {
        if segment_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment number: 0"
            ));
        }
        let mut extension: [u32; 3] = [0; 3];

        let first_character: u32 = match naming_schema {
            EwfNamingSchema::E01UpperCase => 0x45, // 'E'
            EwfNamingSchema::L01UpperCase => 0x4c, // 'L'
            EwfNamingSchema::S01UpperCase => 0x53, // 'S'
            EwfNamingSchema::E01LowerCase => 0x65, // 'e'
            EwfNamingSchema::L01LowerCase => 0x6c, // 'l'
            EwfNamingSchema::S01LowerCase => 0x73, // 's'
        };
        if segment_number < 100 {
            extension[2] = 0x30 + (segment_number % 10) as u32;
            extension[1] = 0x30 + (segment_number / 10) as u32;
            extension[0] = first_character;
        } else {
            let base_character: u32 = match naming_schema {
                EwfNamingSchema::E01UpperCase
                | EwfNamingSchema::L01UpperCase
                | EwfNamingSchema::S01UpperCase => 0x41, // 'A'
                EwfNamingSchema::E01LowerCase
                | EwfNamingSchema::L01LowerCase
                | EwfNamingSchema::S01LowerCase => 0x61, // 'a'
            };
            let mut extension_segment_number: u32 = (segment_number as u32) - 100;

            extension[2] = base_character + (extension_segment_number % 26) as u32;
            extension_segment_number /= 26;

            extension[1] = base_character + (extension_segment_number % 26) as u32;
            extension_segment_number /= 26;

            extension[0] = first_character + extension_segment_number;
        }
        let last_character: u32 = match naming_schema {
            EwfNamingSchema::E01UpperCase
            | EwfNamingSchema::L01UpperCase
            | EwfNamingSchema::S01UpperCase => 0x5a, // 'Z'
            EwfNamingSchema::E01LowerCase
            | EwfNamingSchema::L01LowerCase
            | EwfNamingSchema::S01LowerCase => 0x7a, // 'z'
        };
        if extension[0] > last_character {
            return Err(keramics_core::error_trace_new!(format!(
                "Unsupported segment number: {} value exceeds maximum for naming schema",
                segment_number,
            )));
        }
        let mut segment_extension: String = String::new();

        for code_point in extension.iter() {
            match char::from_u32(*code_point) {
                Some(character) => segment_extension.push(character),
                None => {
                    return Err(keramics_core::error_trace_new!(
                        "Unable to encode string - code point outside of supported range"
                    ));
                }
            }
        }
        Ok(segment_extension)
    }

    /// Determines the file name.
    pub fn get_file_name(
        name: &String,
        segment_number: u16,
        naming_schema: Option<&EwfNamingSchema>,
    ) -> Result<String, ErrorTrace> {
        match naming_schema {
            Some(naming_schema) => {
                let segment_extension: String =
                    match Self::get_extension(segment_number, naming_schema) {
                        Ok(extension) => extension,
                        Err(mut error) => {
                            keramics_core::error_trace_add_frame!(
                                error,
                                "Unable to determine extension"
                            );
                            return Err(error);
                        }
                    };
                Ok(format!("{}.{}", name, segment_extension))
            }
            None => Ok(name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension() -> Result<(), ErrorTrace> {
        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "E01");

        let extension: String = EwfSegmentFile::get_extension(99, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "E99");

        let extension: String = EwfSegmentFile::get_extension(100, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EAA");

        let extension: String = EwfSegmentFile::get_extension(125, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EAZ");

        let extension: String = EwfSegmentFile::get_extension(126, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "EBA");

        let extension: String = EwfSegmentFile::get_extension(776, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "FAA");

        let extension: String =
            EwfSegmentFile::get_extension(14296, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "ZAA");

        let extension: String =
            EwfSegmentFile::get_extension(14971, &EwfNamingSchema::E01UpperCase)?;
        assert_eq!(extension, "ZZZ");

        let result = EwfSegmentFile::get_extension(14972, &EwfNamingSchema::E01UpperCase);
        assert!(result.is_err());

        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::L01UpperCase)?;
        assert_eq!(extension, "L01");

        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::S01UpperCase)?;
        assert_eq!(extension, "S01");

        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::E01LowerCase)?;
        assert_eq!(extension, "e01");

        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::L01LowerCase)?;
        assert_eq!(extension, "l01");

        let extension: String = EwfSegmentFile::get_extension(1, &EwfNamingSchema::S01LowerCase)?;
        assert_eq!(extension, "s01");

        Ok(())
    }

    #[test]
    fn test_get_file_name() -> Result<(), ErrorTrace> {
        let base_name: String = String::from("image");

        let name: String =
            EwfSegmentFile::get_file_name(&base_name, 1, Some(&EwfNamingSchema::E01UpperCase))?;
        assert_eq!(name, "image.E01");

        let name: String = EwfSegmentFile::get_file_name(&base_name, 1, None)?;
        assert_eq!(name, "image");

        Ok(())
    }
}
