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

use super::enums::SplitRawNamingSchema;

/// Split raw storage media image segment file.
pub struct SplitRawSegmentFile {}

impl SplitRawSegmentFile {
    /// Determines the file name.
    pub fn get_file_name(
        name: &String,
        mut segment_number: u16,
        number_of_segment_files: u16,
        naming_schema: &SplitRawNamingSchema,
        name_first_segment_number: u16,
        name_suffix_size: usize,
    ) -> Result<String, ErrorTrace> {
        if segment_number == 0 {
            return Err(keramics_core::error_trace_new!(
                "Unsupported segment number: 0"
            ));
        }
        match naming_schema {
            SplitRawNamingSchema::Alphabetic => {
                let mut segment_suffix: Vec<char> = Vec::new();

                segment_number = (segment_number - 1) + name_first_segment_number;
                while segment_number > 0 {
                    let remainder: u16 = segment_number % 26;
                    segment_number /= 26;

                    match char::from_u32((remainder + 0x61) as u32) {
                        Some(character) => segment_suffix.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                if segment_suffix.len() > name_suffix_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid segment suffix value exceeds size"
                    ));
                }
                while segment_suffix.len() < name_suffix_size {
                    segment_suffix.push('a');
                }
                Ok(format!(
                    "{}{}",
                    name,
                    segment_suffix.iter().rev().collect::<String>()
                ))
            }
            SplitRawNamingSchema::Numeric => {
                let mut segment_suffix: Vec<char> = Vec::new();

                // TODO: add hexadecimal support
                segment_number = (segment_number - 1) + name_first_segment_number;
                while segment_number > 0 {
                    let remainder: u16 = segment_number % 10;
                    segment_number /= 10;

                    match char::from_u32((remainder + 0x30) as u32) {
                        Some(character) => segment_suffix.push(character),
                        None => {
                            return Err(keramics_core::error_trace_new!(
                                "Unable to encode string - code point outside of supported range"
                            ));
                        }
                    }
                }
                if segment_suffix.len() > name_suffix_size {
                    return Err(keramics_core::error_trace_new!(
                        "Invalid segment suffix value exceeds size"
                    ));
                }
                while segment_suffix.len() < name_suffix_size {
                    segment_suffix.push('0');
                }
                Ok(format!(
                    "{}{}",
                    name,
                    segment_suffix.iter().rev().collect::<String>()
                ))
            }
            SplitRawNamingSchema::XOfN => Ok(format!(
                "{}{}of{}",
                name, segment_number, number_of_segment_files
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_name() -> Result<(), ErrorTrace> {
        let name: String = SplitRawSegmentFile::get_file_name(
            &String::from("image"),
            1,
            99,
            &SplitRawNamingSchema::Alphabetic,
            0,
            2,
        )?;
        assert_eq!(name, "imageaa");

        let name: String = SplitRawSegmentFile::get_file_name(
            &String::from("image"),
            1,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            1,
        )?;
        assert_eq!(name, "image1");

        let name: String = SplitRawSegmentFile::get_file_name(
            &String::from("image."),
            1,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            3,
        )?;
        assert_eq!(name, "image.001");

        let name: String = SplitRawSegmentFile::get_file_name(
            &String::from("image."),
            1,
            99,
            &SplitRawNamingSchema::XOfN,
            1,
            1,
        )?;
        assert_eq!(name, "image.1of99");

        let result: Result<String, ErrorTrace> = SplitRawSegmentFile::get_file_name(
            &String::from("image"),
            0,
            99,
            &SplitRawNamingSchema::Numeric,
            1,
            1,
        );
        assert!(result.is_err());

        Ok(())
    }
}
