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

use super::enums::HfsFormat;
use super::extents_overflow_key_extended::HfsExtendedExtentsOverflowKey;
use super::extents_overflow_key_standard::HfsStandardExtentsOverflowKey;

/// Hierarchical File System (HFS) extents overflow key.
pub struct HfsExtentsOverflowKey {
    /// Size.
    pub size: usize,

    /// Fork type.
    pub fork_type: u8,

    /// Identifier (CNID).
    pub identifier: u32,

    /// Block number.
    pub block_number: u32,
}

impl HfsExtentsOverflowKey {
    /// Creates a new extents overflow key.
    pub fn new() -> Self {
        Self {
            size: 0,
            fork_type: 0,
            identifier: 0,
            block_number: 0,
        }
    }

    /// Reads the extents overflow key for debugging.
    pub fn debug_read_data(format: &HfsFormat, data: &[u8]) -> String {
        match format {
            HfsFormat::Hfs => HfsStandardExtentsOverflowKey::debug_read_data(data),
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedExtentsOverflowKey::debug_read_data(data)
            }
        }
    }

    /// Reads the extents overflow key from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardExtentsOverflowKey::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedExtentsOverflowKey::read_data(self, data)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![0x07, 0xff, 0x00, 0x00, 0x00, 0x01, 0x00, 0x03];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![
            0x00, 0x0a, 0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03,
        ];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsExtentsOverflowKey::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.size, 8);
        assert_eq!(test_struct.fork_type, 0xff);
        assert_eq!(test_struct.identifier, 1);
        assert_eq!(test_struct.block_number, 3);

        Ok(())
    }

    #[test]
    fn test_read_data_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsExtentsOverflowKey::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.size, 12);
        assert_eq!(test_struct.fork_type, 0xff);
        assert_eq!(test_struct.identifier, 1);
        assert_eq!(test_struct.block_number, 3);

        Ok(())
    }
}
