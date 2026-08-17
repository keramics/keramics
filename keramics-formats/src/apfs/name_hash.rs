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
use keramics_types::Utf16String;

/// Generates a reversed CRC-32 lookup table for a specific polynomial.
const fn generate_crc32_table(polynomial: u32) -> [u32; 256] {
    let mut table: [u32; 256] = [0; 256];
    let mut table_index: usize = 0;

    while table_index < 256 {
        let mut checksum: u32 = table_index as u32;
        let mut bit_index: usize = 0;

        while bit_index < 8 {
            if checksum & 1 != 0 {
                checksum = polynomial ^ (checksum >> 1);
            } else {
                checksum >>= 1;
            }
            bit_index += 1;
        }
        table[table_index] = checksum;

        table_index += 1;
    }
    table
}

const APFS_NAME_HASH_CRC32_TABLE: [u32; 256] = generate_crc32_table(0x82f63b78);

/// Apple File System (APFS) name hash.
pub struct ApfsNameHash {}

impl ApfsNameHash {
    /// Calculates a name hash.
    pub fn calculate(name: &Utf16String) -> Result<u32, ErrorTrace> {
        let mut checksum: u32 = 0xffffffff;

        let code_points: Vec<u32> = match name.decode() {
            Ok(code_points) => code_points,
            Err(mut error) => {
                keramics_core::error_trace_add_frame!(error, "Unable to decode UTF-16 string");
                return Err(error);
            }
        };
        for code_point in code_points.iter() {
            if *code_point == 0 {
                break;
            }
            // TODO: handle NFD

            let table_index: u32 = (checksum ^ (*code_point & 0x000000ff)) & 0x000000ff;
            checksum = APFS_NAME_HASH_CRC32_TABLE[table_index as usize] ^ (checksum >> 8);

            let table_index: u32 = (checksum ^ ((*code_point >> 8) & 0x000000ff)) & 0x000000ff;
            checksum = APFS_NAME_HASH_CRC32_TABLE[table_index as usize] ^ (checksum >> 8);

            let table_index: u32 = (checksum ^ ((*code_point >> 16) & 0x000000ff)) & 0x000000ff;
            checksum = APFS_NAME_HASH_CRC32_TABLE[table_index as usize] ^ (checksum >> 8);

            let table_index: u32 = (checksum ^ ((*code_point >> 24) & 0x000000ff)) & 0x000000ff;
            checksum = APFS_NAME_HASH_CRC32_TABLE[table_index as usize] ^ (checksum >> 8);
        }
        Ok(checksum & 0x003fffff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() -> Result<(), ErrorTrace> {
        let name: Utf16String = Utf16String::from("TeSt");
        let name_hash: u32 = ApfsNameHash::calculate(&name)?;
        assert_eq!(name_hash, 0x0000996a);

        Ok(())
    }
}
