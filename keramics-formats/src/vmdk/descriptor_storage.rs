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

use keramics_encodings::CharacterEncoding;
use keramics_types::ByteString;

use super::descriptor_extent::VmdkDescriptorExtent;
use super::enums::{VmdkDescriptorExtentAccessMode, VmdkDescriptorExtentType, VmdkDiskType};

/// VMware Virtual Disk (VMDK) descriptor storage.
pub struct VmdkDescriptorStorage<'a> {
    /// Data.
    data: &'a [u8],

    /// Data size.
    data_size: usize,

    /// Data offset.
    data_offset: usize,
}

impl<'a> VmdkDescriptorStorage<'a> {
    /// Creates a new descriptor storage.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            data_size: data.len(),
            data_offset: 0,
        }
    }

    /// Retrieves the next line.
    pub fn next_line(&mut self) -> Option<&'a [u8]> {
        if self.data_offset >= self.data_size {
            return None;
        }
        let start_offset: usize = self.data_offset;
        let mut end_offset: usize = start_offset;

        let mut last_byte: u8 = 0;
        while self.data_offset < self.data_size {
            let byte: u8 = self.data[self.data_offset];
            self.data_offset += 1;

            if byte == 0x00 {
                return None;
            }
            // Break at newline (\n)
            if byte == b'\n' {
                // Ignore cariage return (\r)
                if last_byte == b'\r' {
                    end_offset -= 1;
                }
                break;
            }
            end_offset += 1;
            last_byte = byte;
        }
        Some(&self.data[start_offset..end_offset])
    }

    /// Parses a content identifier value, which is 32-bit integer represented in hexadecimal.
    pub fn parse_content_identifier_value(value: &[u8]) -> Option<u32> {
        if value.len() > 8 {
            None
        } else {
            let mut value_32bit: u32 = 0;

            for byte in value.iter() {
                let hexdigit: u8 = match byte {
                    b'0'..=b'9' => *byte - 0x30,
                    b'a'..=b'f' => (*byte - 0x61) + 10,
                    _ => return None,
                };
                value_32bit = (value_32bit << 4) | (hexdigit as u32);
            }
            Some(value_32bit)
        }
    }

    /// Parses a disk type value.
    pub fn parse_disk_type_value(value: &[u8]) -> Option<VmdkDiskType> {
        let value_size: usize = value.len();

        if value_size == 0 || value[0] != b'"' || value[value_size - 1] != b'"' {
            return None;
        }
        match &value[1..value_size - 1] {
            b"custom" => Some(VmdkDiskType::Custom),
            b"fulldevice" => Some(VmdkDiskType::Device),
            b"vmfs" => Some(VmdkDiskType::VmfsFlat),
            b"vmfsraw" => Some(VmdkDiskType::VmfsRaw),
            b"vmfsrawdevicemap" | b"vmfsrdm" => Some(VmdkDiskType::VmfsRdm),
            b"vmfspassthroughrawdevicemap" | b"vmfsrdmp" => Some(VmdkDiskType::VmfsRdmp),
            b"vmfssparse" => Some(VmdkDiskType::VmfsSparse),
            b"monolithicflat" => Some(VmdkDiskType::MonolithicFlat),
            b"streamoptimized" => Some(VmdkDiskType::StreamOptimized),
            b"2gbmaxextentflat" | b"twogbmaxextentflat" => Some(VmdkDiskType::Flat2GbExtent),
            b"2gbmaxextentsparse" | b"twogbmaxextentsparse" => Some(VmdkDiskType::Sparse2GbExtent),
            b"monolithicsparse" => Some(VmdkDiskType::MonolithicSparse),
            b"vmfspreallocated" => Some(VmdkDiskType::VmfsFlatPreAllocated),
            b"partitioneddevice" => Some(VmdkDiskType::DevicePartitioned),
            b"vmfseagerzeroedthick" => Some(VmdkDiskType::VmfsFlatZeroed),
            b"vmfsthin" => Some(VmdkDiskType::VmfsSparseThin),
            _ => Some(VmdkDiskType::Unknown),
        }
    }

    /// Parses an encoding value.
    pub fn parse_encoding_value(value: &[u8]) -> Option<CharacterEncoding> {
        let value_size: usize = value.len();

        if value_size == 0 || value[0] != b'"' || value[value_size - 1] != b'"' {
            return None;
        }
        // Replace underscores with hyphens.
        let mut normalized_value: Vec<u8> = value
            .iter()
            .skip(1)
            .take(value_size - 2)
            .map(|byte| if *byte == b'_' { b'-' } else { *byte })
            .collect::<Vec<u8>>();

        // Renames "cp" or "ms" prefix to "windows".
        if &normalized_value[0..2] == b"cp" || &normalized_value[0..2] == b"ms" {
            let mut renamed_value: Vec<u8> = b"windows".to_vec();
            renamed_value.extend_from_slice(&normalized_value[2..]);

            normalized_value = renamed_value;
        };

        // Remove hyphen from certain prefixes.
        if &normalized_value[0..4] == b"iso-" {
            normalized_value.remove(3);
        } else if &normalized_value[0..4] == b"koi-" {
            normalized_value.remove(3);
        } else if &normalized_value[0..8] == b"windows-" {
            normalized_value.remove(7);
        } else if &normalized_value[0..4] == b"utf-" {
            normalized_value.remove(3);
        };

        match normalized_value.as_slice() {
            // TODO: add CharacterEncoding::Windows950 support
            // b"big5" | b"windows950" => Some(CharacterEncoding::Windows950),
            b"gbk" | b"windows936" => Some(CharacterEncoding::Windows936),
            b"iso8859-1" => Some(CharacterEncoding::Iso8859_1),
            b"iso8859-2" => Some(CharacterEncoding::Iso8859_2),
            b"iso8859-3" => Some(CharacterEncoding::Iso8859_3),
            b"iso8859-4" => Some(CharacterEncoding::Iso8859_4),
            b"iso8859-5" => Some(CharacterEncoding::Iso8859_5),
            b"iso8859-6" => Some(CharacterEncoding::Iso8859_6),
            b"iso8859-7" => Some(CharacterEncoding::Iso8859_7),
            b"iso8859-8" => Some(CharacterEncoding::Iso8859_8),
            b"iso8859-9" => Some(CharacterEncoding::Iso8859_9),
            b"iso8859-10" => Some(CharacterEncoding::Iso8859_10),
            b"iso8859-11" => Some(CharacterEncoding::Iso8859_11),
            b"iso8859-13" => Some(CharacterEncoding::Iso8859_13),
            b"iso8859-14" => Some(CharacterEncoding::Iso8859_14),
            b"iso8859-15" => Some(CharacterEncoding::Iso8859_15),
            b"iso8859-16" => Some(CharacterEncoding::Iso8859_16),
            b"koi8r" => Some(CharacterEncoding::Koi8R),
            b"koi8u" => Some(CharacterEncoding::Koi8U),
            b"shift-jis" | b"windows932" => Some(CharacterEncoding::Windows932),
            b"utf-8" => Some(CharacterEncoding::Utf8),
            b"windows874" => Some(CharacterEncoding::Windows874),
            b"windows949" | b"windows949-2000" => Some(CharacterEncoding::Windows949),
            b"windows1250" => Some(CharacterEncoding::Windows1250),
            b"windows1251" => Some(CharacterEncoding::Windows1251),
            b"windows1252" => Some(CharacterEncoding::Windows1252),
            b"windows1253" => Some(CharacterEncoding::Windows1253),
            b"windows1254" => Some(CharacterEncoding::Windows1254),
            b"windows1255" => Some(CharacterEncoding::Windows1255),
            b"windows1256" => Some(CharacterEncoding::Windows1256),
            b"windows1257" => Some(CharacterEncoding::Windows1257),
            b"windows1258" => Some(CharacterEncoding::Windows1258),
            _ => None,
        }
    }

    /// Parses an extent.
    pub fn parse_extent(line: &[u8], encoding: &CharacterEncoding) -> Option<VmdkDescriptorExtent> {
        let line_size: usize = line.len();

        let filename_start_index: usize = line
            .iter()
            .position(|byte| *byte == b'"')
            .unwrap_or(line_size + 1);

        // Split the part of the line before the file name.
        let lowercase_line: Vec<u8> = Self::to_ascii_lowercase(&line[0..filename_start_index - 1]);
        let values: Vec<&[u8]> = lowercase_line
            .split(|byte| *byte == b' ')
            .collect::<Vec<&[u8]>>();

        let number_of_values: usize = values.len();

        if number_of_values < 3 {
            return None;
        }
        let access_mode: VmdkDescriptorExtentAccessMode = match values[0] {
            b"noaccess" => VmdkDescriptorExtentAccessMode::NoAccess,
            b"rdonly" => VmdkDescriptorExtentAccessMode::ReadOnly,
            b"rw" => VmdkDescriptorExtentAccessMode::ReadWrite,
            _ => VmdkDescriptorExtentAccessMode::Unknown,
        };
        let number_of_sectors: u64 = match Self::parse_integer_value(values[1]) {
            Some(value_64bit) => {
                if value_64bit == 0 || value_64bit > u64::MAX / 512 {
                    return None;
                }
                value_64bit
            }
            None => return None,
        };
        let extent_type: VmdkDescriptorExtentType = match values[2] {
            b"flat" => VmdkDescriptorExtentType::Flat,
            b"sparse" => VmdkDescriptorExtentType::Sparse,
            b"vmfs" => VmdkDescriptorExtentType::VmfsFlat,
            b"vmfsraw" => VmdkDescriptorExtentType::VmfsRaw,
            b"vmfsrdm" => VmdkDescriptorExtentType::VmfsRdm,
            b"vmfssparse" => VmdkDescriptorExtentType::VmfsSparse,
            b"zero" => VmdkDescriptorExtentType::Zero,
            _ => VmdkDescriptorExtentType::Unknown,
        };
        let filename_end_index: usize = if filename_start_index >= line_size {
            line.len()
        } else {
            match line.iter().rposition(|byte| *byte == b'"') {
                Some(index) => index,
                None => return None,
            }
        };
        let file_name: Option<ByteString> = if filename_end_index >= line_size {
            None
        } else {
            let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);
            byte_string.read_data(&line[filename_start_index + 1..filename_end_index]);

            Some(byte_string)
        };
        let mut start_sector: u64 = 0;

        if filename_end_index + 2 < line_size {
            // Split the part of the line after the file name.
            let lowercase_line: Vec<u8> = Self::to_ascii_lowercase(&line[filename_end_index + 2..]);

            let values: Vec<&[u8]> = lowercase_line
                .split(|byte| *byte == b' ')
                .collect::<Vec<&[u8]>>();

            if !values.is_empty() {
                match Self::parse_integer_value(values[0]) {
                    Some(value_64bit) => {
                        if value_64bit > u64::MAX / 512 {
                            return None;
                        }
                        start_sector = value_64bit;
                    }
                    None => return None,
                }
            }
        };
        Some(VmdkDescriptorExtent::new(
            start_sector,
            number_of_sectors,
            file_name,
            extent_type,
            access_mode,
        ))
    }

    /// Parses a file name.
    pub fn parse_file_name(line: &[u8], encoding: &CharacterEncoding) -> Option<ByteString> {
        let line_size: usize = line.len();

        let filename_start_index: usize = line
            .iter()
            .position(|byte| *byte == b'"')
            .unwrap_or(line_size + 1);

        let filename_end_index: usize = if filename_start_index >= line_size {
            line.len()
        } else {
            match line.iter().rposition(|byte| *byte == b'"') {
                Some(index) => index,
                None => return None,
            }
        };
        if filename_end_index >= line_size {
            None
        } else {
            let mut byte_string: ByteString = ByteString::new_with_encoding(encoding);
            byte_string.read_data(&line[filename_start_index + 1..filename_end_index]);

            Some(byte_string)
        }
    }

    /// Parses an integer value represented in decimal.
    pub fn parse_integer_value(value: &[u8]) -> Option<u64> {
        if value.len() > 20 {
            None
        } else {
            let mut value_64bit: u64 = 0;

            for byte in value.iter() {
                let digit: u8 = match byte {
                    b'0'..=b'9' => *byte - 0x30,
                    _ => return None,
                };
                value_64bit = (value_64bit * 10) + (digit as u64);
            }
            Some(value_64bit)
        }
    }

    /// Parses a key-value pair.
    pub fn parse_key_value_pair(line: &[u8]) -> Option<(&[u8], &[u8])> {
        let values: Vec<&[u8]> = line.split(|byte| *byte == b'=').collect::<Vec<&[u8]>>();

        if values.len() != 2 {
            None
        } else {
            Some((Self::trim(values[0]), Self::trim(values[1])))
        }
    }

    /// Trims leading and trailing whitespace from the line.
    #[inline(always)]
    pub fn trim(line: &[u8]) -> &[u8] {
        if line.is_empty() {
            line
        } else {
            let start_index: usize = line
                .iter()
                .position(|byte| *byte != b'\t' && *byte != 0x0b && *byte != 0x0c && *byte != b' ')
                .unwrap_or(0);

            let end_index: usize = line
                .iter()
                .rposition(|byte| *byte != b'\t' && *byte != 0x0b && *byte != 0x0c && *byte != b' ')
                .unwrap_or(line.len());

            &line[start_index..=end_index]
        }
    }

    /// Formats the line in ASCII lower case.
    #[inline(always)]
    pub fn to_ascii_lowercase(line: &[u8]) -> Vec<u8> {
        line.iter()
            .map(|byte| {
                if byte.is_ascii_uppercase() {
                    *byte + 32
                } else {
                    *byte
                }
            })
            .collect::<Vec<u8>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data() -> Vec<u8> {
        return vec![
            0x23, 0x20, 0x44, 0x69, 0x73, 0x6b, 0x20, 0x44, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70,
            0x74, 0x6f, 0x72, 0x46, 0x69, 0x6c, 0x65, 0x0a, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f,
            0x6e, 0x3d, 0x31, 0x0a, 0x43, 0x49, 0x44, 0x3d, 0x34, 0x63, 0x30, 0x36, 0x39, 0x33,
            0x32, 0x32, 0x0a, 0x70, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x43, 0x49, 0x44, 0x3d, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x0a, 0x63, 0x72, 0x65, 0x61, 0x74, 0x65,
            0x54, 0x79, 0x70, 0x65, 0x3d, 0x22, 0x6d, 0x6f, 0x6e, 0x6f, 0x6c, 0x69, 0x74, 0x68,
            0x69, 0x63, 0x53, 0x70, 0x61, 0x72, 0x73, 0x65, 0x22, 0x0a, 0x0a, 0x23, 0x20, 0x45,
            0x78, 0x74, 0x65, 0x6e, 0x74, 0x20, 0x64, 0x65, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74,
            0x69, 0x6f, 0x6e, 0x0a, 0x52, 0x57, 0x20, 0x38, 0x31, 0x39, 0x32, 0x20, 0x53, 0x50,
            0x41, 0x52, 0x53, 0x45, 0x20, 0x22, 0x65, 0x78, 0x74, 0x32, 0x2e, 0x76, 0x6d, 0x64,
            0x6b, 0x22, 0x0a, 0x0a, 0x23, 0x20, 0x54, 0x68, 0x65, 0x20, 0x44, 0x69, 0x73, 0x6b,
            0x20, 0x44, 0x61, 0x74, 0x61, 0x20, 0x42, 0x61, 0x73, 0x65, 0x0a, 0x23, 0x44, 0x44,
            0x42, 0x0a, 0x0a, 0x64, 0x64, 0x62, 0x2e, 0x76, 0x69, 0x72, 0x74, 0x75, 0x61, 0x6c,
            0x48, 0x57, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x20, 0x3d, 0x20, 0x22, 0x34,
            0x22, 0x0a, 0x64, 0x64, 0x62, 0x2e, 0x67, 0x65, 0x6f, 0x6d, 0x65, 0x74, 0x72, 0x79,
            0x2e, 0x63, 0x79, 0x6c, 0x69, 0x6e, 0x64, 0x65, 0x72, 0x73, 0x20, 0x3d, 0x20, 0x22,
            0x38, 0x22, 0x0a, 0x64, 0x64, 0x62, 0x2e, 0x67, 0x65, 0x6f, 0x6d, 0x65, 0x74, 0x72,
            0x79, 0x2e, 0x68, 0x65, 0x61, 0x64, 0x73, 0x20, 0x3d, 0x20, 0x22, 0x31, 0x36, 0x22,
            0x0a, 0x64, 0x64, 0x62, 0x2e, 0x67, 0x65, 0x6f, 0x6d, 0x65, 0x74, 0x72, 0x79, 0x2e,
            0x73, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x73, 0x20, 0x3d, 0x20, 0x22, 0x36, 0x33, 0x22,
            0x0a, 0x64, 0x64, 0x62, 0x2e, 0x61, 0x64, 0x61, 0x70, 0x74, 0x65, 0x72, 0x54, 0x79,
            0x70, 0x65, 0x20, 0x3d, 0x20, 0x22, 0x69, 0x64, 0x65, 0x22, 0x0a, 0x64, 0x64, 0x62,
            0x2e, 0x74, 0x6f, 0x6f, 0x6c, 0x73, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x20,
            0x3d, 0x20, 0x22, 0x32, 0x31, 0x34, 0x37, 0x34, 0x38, 0x33, 0x36, 0x34, 0x37, 0x22,
            0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
    }

    #[test]
    fn test_next_line() {
        let test_data: Vec<u8> = get_test_data();

        let mut test_struct = VmdkDescriptorStorage::new(&test_data);

        assert_eq!(test_struct.next_line(), Some(&test_data[0..21]));
        assert_eq!(test_struct.next_line(), Some(&test_data[22..31]));
    }

    // TODO: add tests for parse_content_identifier_value
    // TODO: add tests for parse_encoding_value
    // TODO: add tests for parse_extent
    // TODO: add tests for parse_file_name
    // TODO: add tests for parse_integer_value
    // TODO: add tests for parse_key_value_pair
    // TODO: add tests for trim
    // TODO: add tests for to_ascii_lowercase
}
