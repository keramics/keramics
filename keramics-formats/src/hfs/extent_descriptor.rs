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
use super::extent_descriptor_extended::HfsExtendedExtentDescriptor;
use super::extent_descriptor_standard::HfsStandardExtentDescriptor;

#[derive(Clone, Debug, PartialEq)]
/// Hierarchical File System (HFS) extent descriptor.
pub struct HfsExtentDescriptor {
    /// Block number.
    pub block_number: u32,

    /// Number of blocks.
    pub number_of_blocks: u32,
}

impl HfsExtentDescriptor {
    /// Creates a new extent descriptor.
    pub fn new() -> Self {
        Self {
            block_number: 0,
            number_of_blocks: 0,
        }
    }

    /// Reads the extent descriptor from a buffer.
    pub fn read_data(&mut self, format: &HfsFormat, data: &[u8]) -> Result<(), ErrorTrace> {
        match format {
            HfsFormat::Hfs => {
                HfsStandardExtentDescriptor::read_data(self, data)?;
            }
            HfsFormat::HfsPlus | HfsFormat::HfsX => {
                HfsExtendedExtentDescriptor::read_data(self, data)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_hfs() -> Vec<u8> {
        return vec![0x00, 0xf2, 0x00, 0x14];
    }

    fn get_test_data_hfsplus() -> Vec<u8> {
        return vec![0x00, 0x00, 0x00, 0xf2, 0x00, 0x00, 0x00, 0x14];
    }

    #[test]
    fn test_read_data_hfs() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfs();

        let mut test_struct = HfsExtentDescriptor::new();
        test_struct.read_data(&HfsFormat::Hfs, &test_data)?;

        assert_eq!(test_struct.block_number, 242);
        assert_eq!(test_struct.number_of_blocks, 20);

        Ok(())
    }

    #[test]
    fn test_read_data_hfsplus() -> Result<(), ErrorTrace> {
        let test_data: Vec<u8> = get_test_data_hfsplus();

        let mut test_struct = HfsExtentDescriptor::new();
        test_struct.read_data(&HfsFormat::HfsPlus, &test_data)?;

        assert_eq!(test_struct.block_number, 242);
        assert_eq!(test_struct.number_of_blocks, 20);

        Ok(())
    }
}
