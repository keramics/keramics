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

/// File Allocation Table (FAT) directory entry type.
#[derive(Debug, PartialEq)]
pub enum FatDirectoryEntryType {
    /// VFAT long name entry.
    LongName,

    /// Short name entry.
    ShortName,

    /// Terminator entry.
    Terminator,

    /// Unallocated entry.
    Unallocated,
}

impl FatDirectoryEntryType {
    /// Reads the directory entry type from a buffer.
    pub fn read_data(data: &[u8]) -> FatDirectoryEntryType {
        if data[0] == 0xe5 {
            FatDirectoryEntryType::Unallocated
        } else if data[11..13] == [0x0f, 0x00]
            && data[26..28] == [0x00, 0x00]
            && ((data[0] >= 0x01 && data[0] <= 0x13) || (data[0] >= 0x41 && data[0] <= 0x54))
        {
            FatDirectoryEntryType::LongName
        } else if data[0..32] == [0; 32] {
            FatDirectoryEntryType::Terminator
        } else {
            FatDirectoryEntryType::ShortName
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_data() {
        let test_data: Vec<u8> = vec![
            0xe5, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10, 0x00, 0x7d,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let entry_type: FatDirectoryEntryType = FatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, FatDirectoryEntryType::Unallocated);

        let test_data: Vec<u8> = vec![
            0x41, 0x74, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00, 0x64, 0x00, 0x0f, 0x00, 0x81,
            0x69, 0x00, 0x72, 0x00, 0x31, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff,
        ];

        let entry_type: FatDirectoryEntryType = FatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, FatDirectoryEntryType::LongName);

        let test_data: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let entry_type: FatDirectoryEntryType = FatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, FatDirectoryEntryType::Terminator);

        let test_data: Vec<u8> = vec![
            0x54, 0x45, 0x53, 0x54, 0x44, 0x49, 0x52, 0x31, 0x20, 0x20, 0x20, 0x10, 0x00, 0x7d,
            0x8f, 0x95, 0x53, 0x5b, 0x53, 0x5b, 0x00, 0x00, 0x8f, 0x95, 0x53, 0x5b, 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let entry_type: FatDirectoryEntryType = FatDirectoryEntryType::read_data(&test_data);
        assert_eq!(entry_type, FatDirectoryEntryType::ShortName);
    }
}
