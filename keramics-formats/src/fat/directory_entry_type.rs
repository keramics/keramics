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

/// File Allocation Table (FAT) directory entry type.
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

    // TODO: add tests
}
