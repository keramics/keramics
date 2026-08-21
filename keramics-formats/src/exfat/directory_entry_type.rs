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

/// Extensible File Allocation Table (exFAT) directory entry type.
#[derive(Debug, PartialEq)]
pub enum ExFatDirectoryEntryType {
    /// Allocation bitmap.
    AllocationBitmap,

    /// Case folding mappings.
    CaseFoldingMappings,

    /// Data stream.
    DataStream,

    /// File entry.
    FileEntry,

    /// File (entry) name.
    FileName,

    /// Terminator entry.
    Terminator,

    /// TexFAT padding.
    TextFatPadding,

    /// Unknown entry.
    Unknown,

    /// Volume identifier.
    VolumeIdentifier,

    /// Volume label.
    VolumeLabel,
}

impl ExFatDirectoryEntryType {
    /// Reads the directory entry type from a buffer.
    pub fn read_data(data: &[u8]) -> ExFatDirectoryEntryType {
        match &data[0] {
            0x81 => ExFatDirectoryEntryType::AllocationBitmap,
            0x82 => ExFatDirectoryEntryType::CaseFoldingMappings,
            0x83 => ExFatDirectoryEntryType::VolumeLabel,
            0x85 => ExFatDirectoryEntryType::FileEntry,
            0xa0 => ExFatDirectoryEntryType::VolumeIdentifier,
            0xa1 => ExFatDirectoryEntryType::TextFatPadding,
            0xc0 => ExFatDirectoryEntryType::DataStream,
            0xc1 => ExFatDirectoryEntryType::FileName,
            _ => {
                if data[0..32] == [0; 32] {
                    ExFatDirectoryEntryType::Terminator
                } else {
                    ExFatDirectoryEntryType::Unknown
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_data() {
        let test_data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::Terminator);

        let test_data: Vec<u8> = vec![
            0x81, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::AllocationBitmap);

        let test_data: Vec<u8> = vec![
            0x82, 0x00, 0x00, 0x00, 0x0d, 0xd3, 0x19, 0xe6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0xcc, 0x16, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::CaseFoldingMappings);

        let test_data: Vec<u8> = vec![
            0x83, 0x0a, 0x65, 0x00, 0x78, 0x00, 0x66, 0x00, 0x61, 0x00, 0x74, 0x00, 0x5f, 0x00,
            0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::VolumeLabel);

        let test_data: Vec<u8> = vec![
            0x85, 0x02, 0xeb, 0x8b, 0x20, 0x00, 0x00, 0x00, 0xcd, 0x62, 0x15, 0x5d, 0xcd, 0x62,
            0x15, 0x5d, 0xcd, 0x62, 0x15, 0x5d, 0x25, 0x25, 0x80, 0x80, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::FileEntry);

        let test_data: Vec<u8> = vec![
            0xc0, 0x01, 0x00, 0x09, 0x6b, 0x8e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::DataStream);

        let test_data: Vec<u8> = vec![
            0xc1, 0x00, 0x65, 0x00, 0x6d, 0x00, 0x70, 0x00, 0x74, 0x00, 0x79, 0x00, 0x66, 0x00,
            0x69, 0x00, 0x6c, 0x00, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let entry_type: ExFatDirectoryEntryType = ExFatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, ExFatDirectoryEntryType::FileName);
    }
}
